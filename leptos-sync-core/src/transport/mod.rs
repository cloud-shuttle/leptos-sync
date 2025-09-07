//! Transport layer for synchronization

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

pub mod websocket;
pub mod memory;
pub mod multi_transport;
pub mod leptos_ws_pro_transport;
pub mod compatibility_layer;
pub mod hybrid_transport_impl;

#[cfg(test)]
pub mod leptos_ws_pro_tests;

#[cfg(test)]
pub mod real_websocket_tests;

#[cfg(test)]
pub mod server_compatibility_tests;
#[cfg(test)]
pub mod hybrid_transport_tests;

#[cfg(test)]
pub mod enhanced_features_tests;

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

// From implementation is already in leptos_ws_pro_transport.rs

impl From<compatibility_layer::CompatibilityError> for TransportError {
    fn from(err: compatibility_layer::CompatibilityError) -> Self {
        match err {
            compatibility_layer::CompatibilityError::Transport(transport_err) => transport_err,
            compatibility_layer::CompatibilityError::Serialization(msg) => {
                TransportError::SerializationFailed(msg)
            }
            compatibility_layer::CompatibilityError::Protocol(msg) => {
                TransportError::ConnectionFailed(msg)
            }
        }
    }
}

/// Transport trait for synchronization
pub trait SyncTransport: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// Send data to remote peers
    fn send<'a>(&'a self, data: &'a [u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>>;
    
    /// Receive data from remote peers
    fn receive(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>>;
    
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

    fn send<'a>(&'a self, data: &'a [u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            if !self.connected {
                return Err(TransportError::NotConnected);
            }
            
            let mut queue = self.message_queue.write().await;
            queue.push(data.to_vec());
            Ok(())
        })
    }

    fn receive(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            if !self.connected {
                return Err(TransportError::NotConnected);
            }
            
            let mut queue = self.message_queue.write().await;
            let messages = queue.drain(..).collect();
            Ok(messages)
        })
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

    fn send<'a>(&'a self, data: &'a [u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            self.inner.send(data).await.map_err(|e| TransportError::SendFailed(e.to_string()))
        })
    }

    fn receive(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            self.inner.receive().await.map_err(|e| TransportError::ReceiveFailed(e.to_string()))
        })
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

// Re-export HybridTransport from the implementation module
pub use hybrid_transport_impl::HybridTransport;

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

    pub fn leptos_ws_pro(config: leptos_ws_pro_transport::LeptosWsProConfig) -> HybridTransport {
        HybridTransport::with_leptos_ws_pro(config)
    }

    pub fn compatibility(config: leptos_ws_pro_transport::LeptosWsProConfig) -> HybridTransport {
        HybridTransport::with_compatibility(config)
    }

    pub fn in_memory() -> InMemoryTransport {
        InMemoryTransport::new()
    }

    pub fn hybrid(primary_url: String) -> HybridTransport {
        let primary = HybridTransport::with_leptos_ws_pro(leptos_ws_pro_transport::LeptosWsProConfig {
            url: primary_url,
            ..Default::default()
        });
        let fallback = HybridTransport::with_in_memory();
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
        let primary = HybridTransport::with_websocket("ws://invalid-url".to_string());
        let fallback = HybridTransport::with_in_memory();
        let transport = HybridTransport::with_fallback(primary, fallback.clone());
        
        // Send message to fallback transport directly
        let data = b"test message";
        assert!(fallback.send(data).await.is_ok());
        
        let received = fallback.receive().await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], data);
    }
}
