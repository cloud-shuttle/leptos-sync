//! Compatibility layer for migrating from existing WebSocket implementation to leptos-ws-pro
//! 
//! This module provides a bridge between the existing message protocol and leptos-ws-pro,
//! allowing for gradual migration without breaking existing functionality.

use super::{SyncTransport, TransportError};
use crate::transport::leptos_ws_pro_transport::{LeptosWsProTransport, LeptosWsProConfig, MessageProtocolAdapter};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompatibilityError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
}

/// Message types used in the existing leptos-sync protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    #[serde(rename = "sync")]
    Sync {
        peer_id: String,
        data: serde_json::Value,
        timestamp: String,
    },
    #[serde(rename = "presence")]
    Presence {
        peer_id: String,
        action: String,
        timestamp: String,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: String,
    },
    #[serde(rename = "welcome")]
    Welcome {
        peer_id: String,
        timestamp: String,
        server_info: ServerInfo,
    },
    #[serde(rename = "binary_ack")]
    BinaryAck {
        peer_id: String,
        size: usize,
        timestamp: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub max_connections: usize,
    pub heartbeat_interval: u64,
}

/// Compatibility wrapper that implements the existing SyncTransport trait
/// while using leptos-ws-pro under the hood
pub struct CompatibilityTransport {
    leptos_ws_pro: LeptosWsProTransport,
    adapter: MessageProtocolAdapter,
    message_buffer: std::collections::VecDeque<Vec<u8>>,
}

impl CompatibilityTransport {
    /// Create a new compatibility transport
    pub fn new(config: LeptosWsProConfig) -> Self {
        let leptos_ws_pro = LeptosWsProTransport::new(config);
        let adapter = MessageProtocolAdapter::new(leptos_ws_pro.clone());
        
        Self {
            leptos_ws_pro,
            adapter,
            message_buffer: std::collections::VecDeque::new(),
        }
    }

    /// Create with a simple URL
    pub fn with_url(url: String) -> Self {
        let config = LeptosWsProConfig {
            url,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Connect using leptos-ws-pro
    pub async fn connect(&self) -> Result<(), CompatibilityError> {
        self.leptos_ws_pro.connect().await
            .map_err(|e| CompatibilityError::Transport(e.into()))
    }

    /// Disconnect using leptos-ws-pro
    pub async fn disconnect(&self) -> Result<(), CompatibilityError> {
        self.leptos_ws_pro.disconnect().await
            .map_err(|e| CompatibilityError::Transport(e.into()))
    }

    /// Send a sync message using the existing protocol
    pub async fn send_sync(&self, peer_id: &str, data: serde_json::Value) -> Result<(), CompatibilityError> {
        self.adapter.send_sync_message(peer_id, data).await
            .map_err(|e| CompatibilityError::Transport(e.into()))
    }

    /// Send a presence message using the existing protocol
    pub async fn send_presence(&self, peer_id: &str, action: &str) -> Result<(), CompatibilityError> {
        self.adapter.send_presence_message(peer_id, action).await
            .map_err(|e| CompatibilityError::Transport(e.into()))
    }

    /// Send a heartbeat using the existing protocol
    pub async fn send_heartbeat(&self) -> Result<(), CompatibilityError> {
        self.adapter.send_heartbeat().await
            .map_err(|e| CompatibilityError::Transport(e.into()))
    }

    /// Receive and parse messages using the existing protocol
    pub async fn receive_messages(&self) -> Result<Vec<SyncMessage>, CompatibilityError> {
        let raw_messages = self.adapter.receive_messages().await
            .map_err(|e| CompatibilityError::Transport(e.into()))?;

        let mut parsed_messages = Vec::new();
        for raw_message in raw_messages {
            match serde_json::from_value::<SyncMessage>(raw_message) {
                Ok(parsed) => parsed_messages.push(parsed),
                Err(e) => {
                    tracing::warn!("Failed to parse sync message: {}", e);
                    continue;
                }
            }
        }

        Ok(parsed_messages)
    }

    /// Get the underlying leptos-ws-pro transport for advanced usage
    pub fn leptos_ws_pro_transport(&self) -> &LeptosWsProTransport {
        &self.leptos_ws_pro
    }

    /// Get the message protocol adapter
    pub fn message_adapter(&self) -> &MessageProtocolAdapter {
        &self.adapter
    }
}

impl SyncTransport for CompatibilityTransport {
    type Error = TransportError;

    fn send<'a>(&'a self, data: &'a [u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> {
        Box::pin(async move {
        // Try to parse as a sync message first
        if let Ok(message) = serde_json::from_slice::<serde_json::Value>(data) {
            if let Some(msg_type) = message.get("type").and_then(|t| t.as_str()) {
                match msg_type {
                    "sync" => {
                        if let (Some(peer_id), Some(data)) = (
                            message.get("peer_id").and_then(|p| p.as_str()),
                            message.get("data")
                        ) {
                            return self.send_sync(peer_id, data.clone()).await
                                .map_err(|e| match e {
                                    CompatibilityError::Transport(t) => t,
                                    _ => TransportError::SendFailed(e.to_string()),
                                });
                        }
                    }
                    "presence" => {
                        if let (Some(peer_id), Some(action)) = (
                            message.get("peer_id").and_then(|p| p.as_str()),
                            message.get("action").and_then(|a| a.as_str())
                        ) {
                            return self.send_presence(peer_id, action).await
                                .map_err(|e| match e {
                                    CompatibilityError::Transport(t) => t,
                                    _ => TransportError::SendFailed(e.to_string()),
                                });
                        }
                    }
                    "heartbeat" => {
                        return self.send_heartbeat().await
                            .map_err(|e| match e {
                                CompatibilityError::Transport(t) => t,
                                _ => TransportError::SendFailed(e.to_string()),
                            });
                    }
                    _ => {
                        // Unknown message type, send as raw data
                    }
                }
            }
        }

        // Fall back to raw data sending
        self.leptos_ws_pro.send(data).await
        })
    }

    fn receive(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>> {
        Box::pin(async move {
        // Get parsed messages and convert back to raw bytes
        let messages = self.receive_messages().await
            .map_err(|e| match e {
                CompatibilityError::Transport(t) => t,
                _ => TransportError::ReceiveFailed(e.to_string()),
            })?;

        let mut raw_messages = Vec::new();
        for message in messages {
            match serde_json::to_vec(&message) {
                Ok(raw) => raw_messages.push(raw),
                Err(e) => {
                    tracing::warn!("Failed to serialize message: {}", e);
                    continue;
                }
            }
        }

        Ok(raw_messages)
        })
    }

    fn is_connected(&self) -> bool {
        self.leptos_ws_pro.is_connected()
    }
}

impl Clone for CompatibilityTransport {
    fn clone(&self) -> Self {
        Self {
            leptos_ws_pro: self.leptos_ws_pro.clone(),
            adapter: MessageProtocolAdapter::new(self.leptos_ws_pro.clone()),
            message_buffer: std::collections::VecDeque::new(),
        }
    }
}

/// Migration helper for existing WebSocket transport
/// 
/// Note: This is a simplified version that doesn't use trait objects
/// due to the SyncTransport trait not being dyn compatible.
pub struct MigrationHelper {
    new_transport: CompatibilityTransport,
    migration_complete: bool,
}

impl MigrationHelper {
    /// Create a new migration helper
    pub fn new(config: LeptosWsProConfig) -> Self {
        Self {
            new_transport: CompatibilityTransport::new(config),
            migration_complete: false,
        }
    }

    /// Migrate to the new transport
    pub async fn migrate(&mut self) -> Result<(), CompatibilityError> {
        // Connect to new transport
        self.new_transport.connect().await?;
        
        // Mark migration as complete
        self.migration_complete = true;
        
        Ok(())
    }

    /// Check if migration is complete
    pub fn is_migration_complete(&self) -> bool {
        self.migration_complete
    }

    /// Get the new transport
    pub fn new_transport(&self) -> &CompatibilityTransport {
        &self.new_transport
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::InMemoryTransport;

    #[tokio::test]
    async fn test_compatibility_transport_creation() {
        let config = LeptosWsProConfig::default();
        let transport = CompatibilityTransport::new(config);
        
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_sync_message_parsing() {
        let config = LeptosWsProConfig::default();
        let transport = CompatibilityTransport::new(config);
        
        // Test sync message
        let sync_data = serde_json::json!({
            "changes": ["change1", "change2"],
            "client_id": "test_client"
        });
        
        let result = transport.send_sync("test_peer", sync_data).await;
        assert!(result.is_err()); // Expected when not connected
    }

    #[tokio::test]
    async fn test_presence_message_parsing() {
        let config = LeptosWsProConfig::default();
        let transport = CompatibilityTransport::new(config);
        
        let result = transport.send_presence("test_peer", "connected").await;
        assert!(result.is_err()); // Expected when not connected
    }

    #[tokio::test]
    async fn test_heartbeat_message_parsing() {
        let config = LeptosWsProConfig::default();
        let transport = CompatibilityTransport::new(config);
        
        let result = transport.send_heartbeat().await;
        assert!(result.is_err()); // Expected when not connected
    }

    #[tokio::test]
    async fn test_migration_helper() {
        let config = LeptosWsProConfig {
            url: "ws://invalid-url-that-does-not-exist:9999".to_string(),
            ..Default::default()
        };
        let mut helper = MigrationHelper::new(config);
        
        assert!(!helper.is_migration_complete());
        
        // Migration should fail without connection, but helper should be set up
        let result = helper.migrate().await;
        assert!(result.is_err()); // Expected when not connected
    }

    #[tokio::test]
    async fn test_sync_transport_trait_compliance() {
        let config = LeptosWsProConfig::default();
        let transport = CompatibilityTransport::new(config);
        
        // Should implement SyncTransport trait
        assert!(!transport.is_connected());
        
        // Should handle trait methods without panicking
        let data = b"trait compliance test";
        let send_result = transport.send(data).await;
        assert!(send_result.is_err()); // Expected when not connected
        
        let receive_result = transport.receive().await;
        assert!(receive_result.is_ok());
    }
}
