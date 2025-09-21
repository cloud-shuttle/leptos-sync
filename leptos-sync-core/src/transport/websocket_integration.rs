//! WebSocket integration with sync engine

use super::{CrdtType, SyncMessage, WebSocketClient, WebSocketClientConfig};
use crate::crdt::{Mergeable, ReplicaId};
use crate::storage::Storage;
use crate::sync::{SyncEngine, SyncEngineError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Error, Debug)]
pub enum WebSocketIntegrationError {
    #[error("Sync engine error: {0}")]
    SyncEngine(#[from] SyncEngineError),
    #[error("Transport error: {0}")]
    Transport(#[from] super::TransportError),
    #[error("WebSocket client error: {0}")]
    WebSocketClient(#[from] super::WebSocketClientError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Integration error: {0}")]
    Integration(String),
}

/// Configuration for WebSocket integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketIntegrationConfig {
    pub client_config: WebSocketClientConfig,
    pub sync_interval: Duration,
    pub delta_batch_size: usize,
    pub enable_compression: bool,
    pub enable_heartbeat: bool,
}

impl Default for WebSocketIntegrationConfig {
    fn default() -> Self {
        Self {
            client_config: WebSocketClientConfig::default(),
            sync_interval: Duration::from_millis(100),
            delta_batch_size: 10,
            enable_compression: false,
            enable_heartbeat: true,
        }
    }
}

/// WebSocket-integrated sync engine
pub struct WebSocketSyncEngine {
    sync_engine: Arc<SyncEngine<WebSocketClient>>,
    websocket_client: Arc<WebSocketClient>,
    config: WebSocketIntegrationConfig,
    is_running: Arc<RwLock<bool>>,
    sync_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebSocketSyncEngine {
    /// Create a new WebSocket sync engine
    pub fn new(
        storage: Storage,
        config: WebSocketIntegrationConfig,
        replica_id: ReplicaId,
    ) -> Self {
        let websocket_client = Arc::new(WebSocketClient::new(
            config.client_config.clone(),
            replica_id,
        ));

        let sync_engine = Arc::new(SyncEngine::new(storage, (*websocket_client).clone()));

        Self {
            sync_engine,
            websocket_client,
            config,
            is_running: Arc::new(RwLock::new(false)),
            sync_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the WebSocket sync engine
    pub async fn start(&self) -> Result<(), WebSocketIntegrationError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }

        // Connect WebSocket client
        self.websocket_client.connect().await?;

        // Start sync task
        self.start_sync_task().await;

        *is_running = true;
        tracing::info!("WebSocket sync engine started");
        Ok(())
    }

    /// Stop the WebSocket sync engine
    pub async fn stop(&self) -> Result<(), WebSocketIntegrationError> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        // Stop sync task
        self.stop_sync_task().await;

        // Disconnect WebSocket client
        self.websocket_client.disconnect().await?;

        *is_running = false;
        tracing::info!("WebSocket sync engine stopped");
        Ok(())
    }

    /// Send a CRDT delta to peers
    pub async fn send_delta(
        &self,
        collection_id: String,
        crdt_type: CrdtType,
        delta: Vec<u8>,
    ) -> Result<(), WebSocketIntegrationError> {
        let message = SyncMessage::Delta {
            collection_id,
            crdt_type,
            delta,
            timestamp: SystemTime::now(),
            replica_id: self.websocket_client.replica_id(),
        };

        self.websocket_client.send_message(message).await?;
        Ok(())
    }

    /// Get the underlying sync engine
    pub fn sync_engine(&self) -> &Arc<SyncEngine<WebSocketClient>> {
        &self.sync_engine
    }

    /// Get the WebSocket client
    pub fn websocket_client(&self) -> &Arc<WebSocketClient> {
        &self.websocket_client
    }

    /// Check if the engine is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get connection status
    pub async fn is_connected(&self) -> bool {
        self.websocket_client.is_connected().await
    }

    // Private methods

    async fn start_sync_task(&self) {
        let sync_engine = self.sync_engine.clone();
        let websocket_client = self.websocket_client.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();

        let sync_task = tokio::spawn(async move {
            let mut interval = interval(config.sync_interval);
            let mut delta_buffer = Vec::new();

            loop {
                interval.tick().await;

                // Check if still running
                if !*is_running.read().await {
                    break;
                }

                // Process incoming messages
                if let Ok(Some(message)) = websocket_client.receive_message().await {
                    if let Err(e) = Self::handle_incoming_message(&sync_engine, message).await {
                        tracing::error!("Error handling incoming message: {}", e);
                    }
                }

                // Collect local deltas
                // TODO: Implement delta collection from sync engine
                // For now, just simulate
                if delta_buffer.len() >= config.delta_batch_size {
                    Self::send_delta_batch(&websocket_client, &mut delta_buffer).await;
                }
            }
        });

        let mut task_handle = self.sync_task.write().await;
        *task_handle = Some(sync_task);
    }

    async fn stop_sync_task(&self) {
        let mut task_handle = self.sync_task.write().await;
        if let Some(task) = task_handle.take() {
            task.abort();
        }
    }

    async fn handle_incoming_message(
        sync_engine: &Arc<SyncEngine<WebSocketClient>>,
        message: SyncMessage,
    ) -> Result<(), WebSocketIntegrationError> {
        match message {
            SyncMessage::Delta {
                collection_id,
                crdt_type,
                delta,
                replica_id,
                ..
            } => {
                // Apply delta to local CRDT
                tracing::debug!(
                    "Received delta for collection {} from replica {:?}",
                    collection_id,
                    replica_id
                );

                // TODO: Apply delta to the appropriate CRDT in the sync engine
                // This would involve deserializing the delta and merging it
            }
            SyncMessage::Heartbeat { replica_id, .. } => {
                tracing::debug!("Received heartbeat from replica {:?}", replica_id);
                // Update peer info in sync engine
            }
            SyncMessage::PeerJoin {
                replica_id,
                user_info,
            } => {
                tracing::info!(
                    "Peer joined: {:?} with user info: {:?}",
                    replica_id,
                    user_info
                );
                // Add peer to sync engine
            }
            SyncMessage::PeerLeave { replica_id } => {
                tracing::info!("Peer left: {:?}", replica_id);
                // Remove peer from sync engine
            }
            _ => {
                tracing::debug!("Received message: {:?}", message);
            }
        }

        Ok(())
    }

    async fn send_delta_batch(
        websocket_client: &Arc<WebSocketClient>,
        delta_buffer: &mut Vec<(String, CrdtType, Vec<u8>)>,
    ) {
        for (collection_id, crdt_type, delta) in delta_buffer.drain(..) {
            let message = SyncMessage::Delta {
                collection_id,
                crdt_type,
                delta,
                timestamp: SystemTime::now(),
                replica_id: websocket_client.replica_id(),
            };

            if let Err(e) = websocket_client.send_message(message).await {
                tracing::error!("Failed to send delta: {}", e);
            }
        }
    }
}

/// Builder for WebSocket sync engine
pub struct WebSocketSyncEngineBuilder {
    config: WebSocketIntegrationConfig,
    replica_id: Option<ReplicaId>,
}

impl WebSocketSyncEngineBuilder {
    pub fn new() -> Self {
        Self {
            config: WebSocketIntegrationConfig::default(),
            replica_id: None,
        }
    }

    pub fn with_config(mut self, config: WebSocketIntegrationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_replica_id(mut self, replica_id: ReplicaId) -> Self {
        self.replica_id = Some(replica_id);
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.config.client_config.url = url;
        self
    }

    pub fn build(self, storage: Storage) -> WebSocketSyncEngine {
        let replica_id = self
            .replica_id
            .unwrap_or_else(|| ReplicaId::from(uuid::Uuid::new_v4()));
        WebSocketSyncEngine::new(storage, self.config, replica_id)
    }
}

impl Default for WebSocketSyncEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::ReplicaId;
    use crate::storage::memory::MemoryStorage;

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_websocket_sync_engine_creation() {
        let storage = MemoryStorage::new();
        let config = WebSocketIntegrationConfig::default();
        let replica_id = create_test_replica_id();

        let engine =
            WebSocketSyncEngine::new(crate::storage::Storage::Memory(storage), config, replica_id);

        assert_eq!(engine.websocket_client().replica_id(), replica_id);
        assert!(!engine.is_running().await);
    }

    #[tokio::test]
    async fn test_websocket_sync_engine_builder() {
        let storage = MemoryStorage::new();
        let replica_id = create_test_replica_id();

        let engine = WebSocketSyncEngineBuilder::new()
            .with_replica_id(replica_id)
            .with_url("ws://test.example.com".to_string())
            .build(crate::storage::Storage::Memory(storage));

        assert_eq!(engine.websocket_client().replica_id(), replica_id);
        assert_eq!(engine.config.client_config.url, "ws://test.example.com");
    }

    #[tokio::test]
    async fn test_send_delta() {
        let storage = MemoryStorage::new();
        let config = WebSocketIntegrationConfig::default();
        let replica_id = create_test_replica_id();

        let engine =
            WebSocketSyncEngine::new(crate::storage::Storage::Memory(storage), config, replica_id);

        let delta_data = b"test delta".to_vec();
        let result = engine
            .send_delta(
                "test_collection".to_string(),
                CrdtType::LwwRegister,
                delta_data,
            )
            .await;

        // Should succeed even without connection in test environment
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_stop_cycle() {
        let storage = MemoryStorage::new();
        let config = WebSocketIntegrationConfig::default();
        let replica_id = create_test_replica_id();

        let engine =
            WebSocketSyncEngine::new(crate::storage::Storage::Memory(storage), config, replica_id);

        // Initially not running
        assert!(!engine.is_running().await);

        // Start engine
        let result = engine.start().await;
        assert!(result.is_ok());
        assert!(engine.is_running().await);

        // Stop engine
        let result = engine.stop().await;
        assert!(result.is_ok());
        assert!(!engine.is_running().await);
    }
}
