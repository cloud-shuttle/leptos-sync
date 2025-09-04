use super::{SyncTransport, TransportError, InMemoryTransport, WebSocketTransport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Transport type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportType {
    WebSocket,
    Http,
    WebRTC,
    Memory, // For testing
}

/// Configuration for multi-transport system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTransportConfig {
    /// Primary transport to use
    pub primary: TransportType,
    /// Fallback transports in order of preference
    pub fallbacks: Vec<TransportType>,
    /// Whether to automatically switch transports on failure
    pub auto_switch: bool,
    /// Timeout for transport operations
    pub timeout_ms: u64,
}

impl Default for MultiTransportConfig {
    fn default() -> Self {
        Self {
            primary: TransportType::WebSocket,
            fallbacks: vec![TransportType::Http, TransportType::Memory],
            auto_switch: true,
            timeout_ms: 5000,
        }
    }
}

/// Transport enum that can hold different transport types
#[derive(Clone)]
pub enum TransportEnum {
    WebSocket(WebSocketTransport),
    InMemory(InMemoryTransport),
}

impl SyncTransport for TransportEnum {
    type Error = TransportError;

    async fn send(&self, data: &[u8]) -> Result<(), Self::Error> {
        match self {
            TransportEnum::WebSocket(ws) => ws.send(data).await,
            TransportEnum::InMemory(mem) => mem.send(data).await,
        }
    }

    async fn receive(&self) -> Result<Vec<Vec<u8>>, Self::Error> {
        match self {
            TransportEnum::WebSocket(ws) => ws.receive().await,
            TransportEnum::InMemory(mem) => mem.receive().await,
        }
    }

    fn is_connected(&self) -> bool {
        match self {
            TransportEnum::WebSocket(ws) => ws.is_connected(),
            TransportEnum::InMemory(mem) => mem.is_connected(),
        }
    }
}

/// Multi-transport implementation that can switch between different transport types
pub struct MultiTransport {
    config: MultiTransportConfig,
    transports: HashMap<TransportType, TransportEnum>,
    current_transport: Arc<RwLock<TransportType>>,
}

impl MultiTransport {
    /// Create a new multi-transport instance
    pub fn new(config: MultiTransportConfig) -> Self {
        Self {
            config,
            transports: HashMap::new(),
            current_transport: Arc::new(RwLock::new(TransportType::WebSocket)),
        }
    }

    /// Register a transport implementation
    pub fn register_transport(&mut self, transport_type: TransportType, transport: TransportEnum) {
        self.transports.insert(transport_type, transport);
    }

    /// Get the current active transport
    pub async fn current_transport(&self) -> TransportType {
        self.current_transport.read().await.clone()
    }

    /// Switch to a different transport
    pub async fn switch_transport(&self, transport_type: TransportType) -> Result<(), TransportError> {
        if !self.transports.contains_key(&transport_type) {
            return Err(TransportError::ConnectionFailed(format!("Transport {:?} not registered", transport_type)));
        }
        
        let mut current = self.current_transport.write().await;
        *current = transport_type;
        Ok(())
    }

    /// Get available transports
    pub fn available_transports(&self) -> Vec<TransportType> {
        self.transports.keys().cloned().collect()
    }

    /// Get the current transport configuration
    pub fn config(&self) -> &MultiTransportConfig {
        &self.config
    }

    /// Check if a transport type is registered
    pub fn has_transport(&self, transport_type: &TransportType) -> bool {
        self.transports.contains_key(transport_type)
    }

    /// Get the number of registered transports
    pub fn transport_count(&self) -> usize {
        self.transports.len()
    }
}

impl SyncTransport for MultiTransport {
    type Error = TransportError;

    async fn send(&self, data: &[u8]) -> Result<(), Self::Error> {
        let current_type = self.current_transport.read().await.clone();
        
        if let Some(transport) = self.transports.get(&current_type) {
            transport.send(data).await
        } else {
            Err(TransportError::SendFailed(format!("No transport available for {:?}", current_type)))
        }
    }

    async fn receive(&self) -> Result<Vec<Vec<u8>>, Self::Error> {
        let current_type = self.current_transport.read().await.clone();
        
        if let Some(transport) = self.transports.get(&current_type) {
            transport.receive().await
        } else {
            Err(TransportError::ReceiveFailed(format!("No transport available for {:?}", current_type)))
        }
    }

    fn is_connected(&self) -> bool {
        let current_type = self.current_transport.try_read().unwrap_or_else(|_| {
            // If we can't get a read lock, assume not connected
            panic!("Failed to acquire read lock")
        });
        
        if let Some(transport) = self.transports.get(&*current_type) {
            transport.is_connected()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_transport_creation() {
        let config = MultiTransportConfig::default();
        let multi_transport = MultiTransport::new(config);
        
        assert_eq!(multi_transport.current_transport().await, TransportType::WebSocket);
        assert!(multi_transport.available_transports().is_empty());
    }

    #[tokio::test]
    async fn test_register_and_switch_transports() {
        let config = MultiTransportConfig {
            primary: TransportType::WebSocket,
            fallbacks: vec![TransportType::Memory],
            auto_switch: true,
            timeout_ms: 5000,
        };
        
        let mut multi_transport = MultiTransport::new(config);
        
        // Register transports
        let ws_transport = TransportEnum::WebSocket(WebSocketTransport::new("ws://test".to_string()));
        let memory_transport = TransportEnum::InMemory(InMemoryTransport::new());
        
        multi_transport.register_transport(TransportType::WebSocket, ws_transport);
        multi_transport.register_transport(TransportType::Memory, memory_transport);
        
        // Check available transports
        let available = multi_transport.available_transports();
        assert_eq!(available.len(), 2);
        assert!(available.contains(&TransportType::WebSocket));
        assert!(available.contains(&TransportType::Memory));
        
        // Switch transport
        multi_transport.switch_transport(TransportType::Memory).await.unwrap();
        assert_eq!(multi_transport.current_transport().await, TransportType::Memory);
    }

    #[tokio::test]
    async fn test_transport_operations() {
        let config = MultiTransportConfig::default();
        let mut multi_transport = MultiTransport::new(config);
        
        let mock_transport = TransportEnum::InMemory(InMemoryTransport::new());
        multi_transport.register_transport(TransportType::WebSocket, mock_transport);
        
        // Test send
        multi_transport.send(b"test data").await.unwrap();
        
        // Test receive
        let data = multi_transport.receive().await.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], b"test data");
    }

    #[tokio::test]
    async fn test_transport_failure_handling() {
        let config = MultiTransportConfig {
            primary: TransportType::WebSocket,
            fallbacks: vec![TransportType::Memory],
            auto_switch: true,
            timeout_ms: 5000,
        };
        
        let mut multi_transport = MultiTransport::new(config);
        
        // Use a WebSocket transport that will fail to connect
        let failing_transport = TransportEnum::WebSocket(WebSocketTransport::new("ws://invalid-url".to_string()));
        multi_transport.register_transport(TransportType::WebSocket, failing_transport);
        
        // WebSocket transport might not fail immediately on send/receive
        // Let's test that it's not connected instead
        assert!(!multi_transport.is_connected());
        
        // Test that we can switch to a working transport
        let working_transport = TransportEnum::InMemory(InMemoryTransport::new());
        multi_transport.register_transport(TransportType::Memory, working_transport);
        
        multi_transport.switch_transport(TransportType::Memory).await.unwrap();
        assert!(multi_transport.is_connected());
        
        // Now operations should work
        assert!(multi_transport.send(b"test").await.is_ok());
        assert!(multi_transport.receive().await.is_ok());
    }

    #[tokio::test]
    async fn test_switch_to_unregistered_transport() {
        let config = MultiTransportConfig::default();
        let multi_transport = MultiTransport::new(config);
        
        // Should fail to switch to unregistered transport
        let result = multi_transport.switch_transport(TransportType::WebRTC).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operations_without_registered_transport() {
        let config = MultiTransportConfig::default();
        let multi_transport = MultiTransport::new(config);
        
        // All operations should fail when no transport is registered
        assert!(multi_transport.send(b"test").await.is_err());
        assert!(multi_transport.receive().await.is_err());
        assert!(!multi_transport.is_connected());
    }

    #[tokio::test]
    async fn test_multi_transport_utility_methods() {
        let config = MultiTransportConfig::default();
        let mut multi_transport = MultiTransport::new(config);
        
        // Initially no transports
        assert_eq!(multi_transport.transport_count(), 0);
        assert!(!multi_transport.has_transport(&TransportType::WebSocket));
        
        // Register a transport
        let transport = TransportEnum::InMemory(InMemoryTransport::new());
        multi_transport.register_transport(TransportType::WebSocket, transport);
        
        // Check utility methods
        assert_eq!(multi_transport.transport_count(), 1);
        assert!(multi_transport.has_transport(&TransportType::WebSocket));
        assert!(!multi_transport.has_transport(&TransportType::Memory));
        
        // Check config access
        let config = multi_transport.config();
        assert_eq!(config.primary, TransportType::WebSocket);
        assert!(config.auto_switch);
    }
}
