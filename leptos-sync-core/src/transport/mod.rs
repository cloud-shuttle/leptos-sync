//! Transport layer for synchronization

use crate::storage::StorageError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

pub mod websocket;
pub mod memory;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Not connected")]
    NotConnected,
}

/// Transport trait for synchronization
pub trait SyncTransport: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// Send data to remote peers
    fn send(&self, data: &[u8]) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
    
    /// Receive data from remote peers
    fn receive(&self) -> impl std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send;
    
    /// Check if transport is connected
    fn is_connected(&self) -> bool;
}

/// In-memory transport for testing
pub struct InMemoryTransport {
    connected: bool,
    message_queue: Arc<RwLock<Vec<Vec<u8>>>>,
}

impl InMemoryTransport {
    pub fn new() -> Self {
        Self {
            connected: true,
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_connection_status(connected: bool) -> Self {
        Self {
            connected,
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl SyncTransport for InMemoryTransport {
    type Error = TransportError;

    async fn send(&self, data: &[u8]) -> Result<(), Self::Error> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        
        let mut queue = self.message_queue.write().await;
        queue.push(data.to_vec());
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<Vec<u8>>, Self::Error> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        
        let mut queue = self.message_queue.write().await;
        let messages = queue.drain(..).collect();
        Ok(messages)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Clone for InMemoryTransport {
    fn clone(&self) -> Self {
        Self {
            connected: self.connected,
            message_queue: self.message_queue.clone(),
        }
    }
}

/// WebSocket transport wrapper
pub struct WebSocketTransport {
    inner: websocket::WebSocketTransport,
}

impl WebSocketTransport {
    pub fn new(url: String) -> Self {
        Self {
            inner: websocket::WebSocketTransport::new(url),
        }
    }

    pub async fn connect(&self) -> Result<(), TransportError> {
        self.inner.connect().await.map_err(|e| TransportError::ConnectionFailed(e.to_string()))
    }

    pub async fn disconnect(&self) -> Result<(), TransportError> {
        self.inner.disconnect().await.map_err(|e| TransportError::ConnectionFailed(e.to_string()))
    }
}

impl SyncTransport for WebSocketTransport {
    type Error = TransportError;

    async fn send(&self, data: &[u8]) -> Result<(), Self::Error> {
        self.inner.send(data).await.map_err(|e| TransportError::SendFailed(e.to_string()))
    }

    async fn receive(&self) -> Result<Vec<Vec<u8>>, Self::Error> {
        self.inner.receive().await.map_err(|e| TransportError::ReceiveFailed(e.to_string()))
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
}

impl Clone for WebSocketTransport {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Hybrid transport that can use multiple backends
#[derive(Clone)]
pub enum HybridTransport {
    WebSocket(WebSocketTransport),
    InMemory(InMemoryTransport),
    Fallback {
        primary: WebSocketTransport,
        fallback: InMemoryTransport,
    },
}

impl HybridTransport {
    pub fn with_websocket(url: String) -> Self {
        Self::WebSocket(WebSocketTransport::new(url))
    }

    pub fn with_in_memory() -> Self {
        Self::InMemory(InMemoryTransport::new())
    }

    pub fn with_fallback(primary: WebSocketTransport, fallback: InMemoryTransport) -> Self {
        Self::Fallback { primary, fallback }
    }

    pub fn add_transport(&mut self, transport: HybridTransport) {
        // For now, just replace the current transport
        // In a more sophisticated implementation, you'd maintain a list
        *self = transport;
    }
}

impl SyncTransport for HybridTransport {
    type Error = TransportError;

    async fn send(&self, data: &[u8]) -> Result<(), Self::Error> {
        match self {
            HybridTransport::WebSocket(ws) => ws.send(data).await,
            HybridTransport::InMemory(mem) => mem.send(data).await,
            HybridTransport::Fallback { primary, fallback } => {
                // Try primary first, fall back to fallback
                match primary.send(data).await {
                    Ok(()) => Ok(()),
                    Err(_) => fallback.send(data).await,
                }
            }
        }
    }

    async fn receive(&self) -> Result<Vec<Vec<u8>>, Self::Error> {
        match self {
            HybridTransport::WebSocket(ws) => ws.receive().await,
            HybridTransport::InMemory(mem) => mem.receive().await,
            HybridTransport::Fallback { primary, fallback } => {
                // Try primary first, fall back to fallback
                match primary.receive().await {
                    Ok(messages) => Ok(messages),
                    Err(_) => fallback.receive().await,
                }
            }
        }
    }

    fn is_connected(&self) -> bool {
        match self {
            HybridTransport::WebSocket(ws) => ws.is_connected(),
            HybridTransport::InMemory(mem) => mem.is_connected(),
            HybridTransport::Fallback { primary, fallback } => {
                primary.is_connected() || fallback.is_connected()
            }
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub url: Option<String>,
    pub timeout: std::time::Duration,
    pub retry_attempts: u32,
    pub heartbeat_interval: std::time::Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            url: None,
            timeout: std::time::Duration::from_secs(30),
            retry_attempts: 3,
            heartbeat_interval: std::time::Duration::from_secs(30),
        }
    }
}

/// Transport factory
pub struct TransportFactory;

impl TransportFactory {
    pub fn websocket(url: String) -> WebSocketTransport {
        WebSocketTransport::new(url)
    }

    pub fn in_memory() -> InMemoryTransport {
        InMemoryTransport::new()
    }

    pub fn hybrid(primary_url: String) -> HybridTransport {
        let primary = WebSocketTransport::new(primary_url);
        let fallback = InMemoryTransport::new();
        HybridTransport::with_fallback(primary, fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_transport() {
        let transport = InMemoryTransport::new();
        
        // Test send
        let data = b"test message";
        assert!(transport.send(data).await.is_ok());
        
        // Test receive
        let received = transport.receive().await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], data);
    }

    #[tokio::test]
    async fn test_hybrid_transport_fallback() {
        let primary = WebSocketTransport::new("ws://invalid-url".to_string());
        let fallback = InMemoryTransport::new();
        let transport = HybridTransport::with_fallback(primary, fallback.clone());
        
        // Send message to fallback transport directly
        let data = b"test message";
        assert!(fallback.send(data).await.is_ok());
        
        let received = fallback.receive().await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], data);
    }
}
