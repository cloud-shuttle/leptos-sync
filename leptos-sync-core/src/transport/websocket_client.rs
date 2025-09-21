//! WebSocket client transport implementation

use super::{SyncTransport, TransportError};
use super::message_protocol::{SyncMessage, MessageCodec};
use crate::crdt::ReplicaId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{timeout, interval};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketClientError {
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
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

impl From<WebSocketClientError> for TransportError {
    fn from(err: WebSocketClientError) -> Self {
        match err {
            WebSocketClientError::ConnectionFailed(msg) => TransportError::ConnectionFailed(msg),
            WebSocketClientError::SendFailed(msg) => TransportError::SendFailed(msg),
            WebSocketClientError::ReceiveFailed(msg) => TransportError::ReceiveFailed(msg),
            WebSocketClientError::SerializationFailed(msg) => TransportError::SerializationFailed(msg),
            WebSocketClientError::NotConnected => TransportError::NotConnected,
            WebSocketClientError::Timeout(msg) => TransportError::ConnectionFailed(msg),
            WebSocketClientError::WebSocketError(msg) => TransportError::ConnectionFailed(msg),
        }
    }
}

/// Configuration for WebSocket client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketClientConfig {
    pub url: String,
    pub reconnect_attempts: u32,
    pub heartbeat_interval: Duration,
    pub message_timeout: Duration,
    pub connection_timeout: Duration,
    pub retry_delay: Duration,
}

impl Default for WebSocketClientConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:3001/sync".to_string(),
            reconnect_attempts: 5,
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(10),
            connection_timeout: Duration::from_secs(10),
            retry_delay: Duration::from_millis(1000),
        }
    }
}

/// Connection state for WebSocket client
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

/// WebSocket client transport implementation
pub struct WebSocketClient {
    config: WebSocketClientConfig,
    replica_id: ReplicaId,
    connection_state: Arc<RwLock<ConnectionState>>,
    message_sender: mpsc::UnboundedSender<Vec<u8>>,
    message_receiver: Arc<RwLock<mpsc::UnboundedReceiver<Vec<u8>>>>,
    heartbeat_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: WebSocketClientConfig, replica_id: ReplicaId) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            config,
            replica_id,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            message_sender: tx,
            message_receiver: Arc::new(RwLock::new(rx)),
            heartbeat_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new client with default configuration
    pub fn with_url(url: String, replica_id: ReplicaId) -> Self {
        let config = WebSocketClientConfig {
            url,
            ..Default::default()
        };
        Self::new(config, replica_id)
    }

    /// Get the current connection state
    pub async fn connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }

    /// Get the replica ID
    pub fn replica_id(&self) -> ReplicaId {
        self.replica_id
    }

    /// Connect to the WebSocket server
    pub async fn connect(&self) -> Result<(), WebSocketClientError> {
        let mut state = self.connection_state.write().await;
        if *state == ConnectionState::Connected {
            return Ok(());
        }

        *state = ConnectionState::Connecting;
        drop(state);

        // Attempt to establish connection with retry logic
        for attempt in 0..self.config.reconnect_attempts {
            match self.attempt_connection().await {
                Ok(()) => {
                    let mut state = self.connection_state.write().await;
                    *state = ConnectionState::Connected;
                    
                    // Start heartbeat task
                    self.start_heartbeat().await;
                    
                    return Ok(());
                }
                Err(e) => {
                    if attempt < self.config.reconnect_attempts - 1 {
                        tracing::warn!(
                            "Connection attempt {} failed: {}. Retrying in {:?}...",
                            attempt + 1,
                            e,
                            self.config.retry_delay
                        );
                        
                        let mut state = self.connection_state.write().await;
                        *state = ConnectionState::Reconnecting;
                        drop(state);
                        
                        tokio::time::sleep(self.config.retry_delay).await;
                    } else {
                        let mut state = self.connection_state.write().await;
                        *state = ConnectionState::Failed;
                        return Err(e);
                    }
                }
            }
        }

        Err(WebSocketClientError::ConnectionFailed("Max retry attempts exceeded".to_string()))
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&self) -> Result<(), WebSocketClientError> {
        // Stop heartbeat task
        self.stop_heartbeat().await;
        
        let mut state = self.connection_state.write().await;
        *state = ConnectionState::Disconnected;
        
        // TODO: Implement actual WebSocket disconnection
        tracing::debug!("Disconnected from WebSocket server");
        Ok(())
    }

    /// Send a synchronization message
    pub async fn send_message(&self, message: SyncMessage) -> Result<(), WebSocketClientError> {
        if !self.is_connected().await {
            return Err(WebSocketClientError::NotConnected);
        }

        let serialized = MessageCodec::serialize(&message)
            .map_err(|e| WebSocketClientError::SerializationFailed(e.to_string()))?;

        self.send_raw(&serialized).await
    }

    /// Send raw bytes
    pub async fn send_raw(&self, data: &[u8]) -> Result<(), WebSocketClientError> {
        if !self.is_connected().await {
            return Err(WebSocketClientError::NotConnected);
        }

        // TODO: Implement actual WebSocket sending
        tracing::debug!("Would send {} bytes via WebSocket", data.len());
        Ok(())
    }

    /// Receive a synchronization message
    pub async fn receive_message(&self) -> Result<Option<SyncMessage>, WebSocketClientError> {
        let mut receiver = self.message_receiver.write().await;
        
        match timeout(self.config.message_timeout, receiver.recv()).await {
            Ok(Some(data)) => {
                let message = MessageCodec::deserialize(&data)
                    .map_err(|e| WebSocketClientError::SerializationFailed(e.to_string()))?;
                Ok(Some(message))
            }
            Ok(None) => Ok(None),
            Err(_) => Err(WebSocketClientError::Timeout("Message receive timeout".to_string())),
        }
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connection_state.read().await == ConnectionState::Connected
    }

    // Private methods

    async fn attempt_connection(&self) -> Result<(), WebSocketClientError> {
        // TODO: Implement actual WebSocket connection using leptos-ws-pro
        // For now, simulate connection
        tracing::debug!("Attempting to connect to {}", self.config.url);
        
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Simulate connection success
        Ok(())
    }

    async fn start_heartbeat(&self) {
        let config = self.config.clone();
        let replica_id = self.replica_id;
        let sender = self.message_sender.clone();
        let state = self.connection_state.clone();

        let heartbeat_task = tokio::spawn(async move {
            let mut interval = interval(config.heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                // Check if still connected
                if *state.read().await != ConnectionState::Connected {
                    break;
                }
                
                // Send heartbeat message
                let heartbeat = SyncMessage::Heartbeat {
                    replica_id,
                    timestamp: SystemTime::now(),
                };
                
                match MessageCodec::serialize(&heartbeat) {
                    Ok(data) => {
                        if sender.send(data).is_err() {
                            tracing::warn!("Failed to send heartbeat - connection may be lost");
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize heartbeat: {}", e);
                        break;
                    }
                }
            }
        });

        let mut task_handle = self.heartbeat_task.write().await;
        *task_handle = Some(heartbeat_task);
    }

    async fn stop_heartbeat(&self) {
        let mut task_handle = self.heartbeat_task.write().await;
        if let Some(task) = task_handle.take() {
            task.abort();
        }
    }
}

impl SyncTransport for WebSocketClient {
    type Error = TransportError;

    fn send<'a>(&'a self, data: &'a [u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            self.send_raw(data).await.map_err(|e| e.into())
        })
    }

    fn receive(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            match self.receive_message().await {
                Ok(Some(message)) => {
                    let data = MessageCodec::serialize(&message)
                        .map_err(|e| TransportError::SerializationFailed(e.to_string()))?;
                    Ok(vec![data])
                }
                Ok(None) => Ok(Vec::new()),
                Err(e) => Err(e.into()),
            }
        })
    }

    fn is_connected(&self) -> bool {
        // Note: This is a synchronous method, so we can't await
        // In a real implementation, we'd need to maintain a cached connection state
        true // For now, assume connected if the client exists
    }
}

impl Clone for WebSocketClient {
    fn clone(&self) -> Self {
        // Create a new client with the same configuration
        Self::new(self.config.clone(), self.replica_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::ReplicaId;

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let replica_id = create_test_replica_id();
        let config = WebSocketClientConfig::default();
        let client = WebSocketClient::new(config, replica_id);
        
        assert_eq!(client.replica_id(), replica_id);
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_websocket_client_with_url() {
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::with_url("ws://test.example.com".to_string(), replica_id);
        
        assert_eq!(client.config.url, "ws://test.example.com");
        assert_eq!(client.replica_id(), replica_id);
    }

    #[tokio::test]
    async fn test_connection_state_transitions() {
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(WebSocketClientConfig::default(), replica_id);
        
        // Initially disconnected
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
        
        // Attempt connection (should succeed in test environment)
        let result = client.connect().await;
        assert!(result.is_ok());
        assert_eq!(client.connection_state().await, ConnectionState::Connected);
        
        // Disconnect
        let result = client.disconnect().await;
        assert!(result.is_ok());
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_send_message() {
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(WebSocketClientConfig::default(), replica_id);
        
        // Connect first
        client.connect().await.unwrap();
        
        // Send a heartbeat message
        let message = SyncMessage::Heartbeat {
            replica_id: replica_id.clone(),
            timestamp: SystemTime::now(),
        };
        
        let result = client.send_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_transport_implementation() {
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(WebSocketClientConfig::default(), replica_id);
        
        // Test SyncTransport trait implementation
        let test_data = b"test data";
        let result = client.send(test_data).await;
        assert!(result.is_ok());
        
        // Test receive (should return empty in test environment)
        let result = client.receive().await;
        assert!(result.is_ok());
    }
}
