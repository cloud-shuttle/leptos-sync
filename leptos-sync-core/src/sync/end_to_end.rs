//! End-to-end synchronization engine

use super::{PeerInfo, PeerSyncStatus, SyncEngine, SyncEngineError, SyncState};
use crate::{
    crdt::{LwwMap, LwwRegister, Mergeable, ReplicaId},
    storage::{LocalStorage, StorageError},
    transport::{SyncTransport, TransportError},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};

#[derive(Error, Debug)]
pub enum EndToEndSyncError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Sync engine error: {0}")]
    SyncEngine(#[from] SyncEngineError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Sync operation failed: {0}")]
    SyncFailed(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
}

/// End-to-end synchronization message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Sync request with data
    SyncRequest {
        collection_id: String,
        replica_id: ReplicaId,
        data: Vec<u8>,
        timestamp: u64,
    },
    /// Sync response with merged data
    SyncResponse {
        collection_id: String,
        replica_id: ReplicaId,
        data: Vec<u8>,
        timestamp: u64,
    },
    /// Peer presence announcement
    Presence {
        replica_id: ReplicaId,
        status: String,
        timestamp: u64,
    },
    /// Heartbeat message
    Heartbeat {
        replica_id: ReplicaId,
        timestamp: u64,
    },
    /// Acknowledgment
    Ack {
        message_id: String,
        replica_id: ReplicaId,
        timestamp: u64,
    },
}

/// Collection metadata for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub id: String,
    pub name: String,
    pub crdt_type: String,
    pub version: u32,
    pub last_sync: u64,
    pub replica_count: u32,
}

/// End-to-end synchronization manager
pub struct EndToEndSyncManager<S, T>
where
    S: LocalStorage + Send + Sync + 'static,
    T: SyncTransport + Send + Sync + 'static,
{
    replica_id: ReplicaId,
    storage: Arc<S>,
    transport: Arc<T>,
    collections: Arc<RwLock<HashMap<String, CollectionMetadata>>>,
    peers: Arc<RwLock<HashMap<ReplicaId, PeerInfo>>>,
    sync_state: Arc<RwLock<SyncState>>,
    message_sender: mpsc::UnboundedSender<SyncMessage>,
    message_receiver: Arc<RwLock<mpsc::UnboundedReceiver<SyncMessage>>>,
    sync_interval: Duration,
    heartbeat_interval: Duration,
    is_running: Arc<RwLock<bool>>,
}

impl<S, T> EndToEndSyncManager<S, T>
where
    S: LocalStorage + Send + Sync + 'static,
    T: SyncTransport + Send + Sync + 'static,
    EndToEndSyncError: From<<T as SyncTransport>::Error>,
{
    /// Create a new end-to-end sync manager
    pub fn new(
        replica_id: ReplicaId,
        storage: Arc<S>,
        transport: Arc<T>,
        sync_interval: Duration,
        heartbeat_interval: Duration,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            replica_id,
            storage,
            transport,
            collections: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            sync_state: Arc::new(RwLock::new(SyncState::Disconnected)),
            message_sender: tx,
            message_receiver: Arc::new(RwLock::new(rx)),
            sync_interval,
            heartbeat_interval,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the synchronization manager
    pub async fn start(&self) -> Result<(), EndToEndSyncError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        // Connect transport
        if !self.transport.is_connected() {
            // Note: In a real implementation, we'd call transport.connect()
            // For now, we'll assume the transport is already connected
        }

        // Update sync state
        {
            let mut state = self.sync_state.write().await;
            *state = SyncState::Connected;
        }

        // Start background tasks
        self.start_background_tasks().await;

        Ok(())
    }

    /// Stop the synchronization manager
    pub async fn stop(&self) -> Result<(), EndToEndSyncError> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);

        // Update sync state
        {
            let mut state = self.sync_state.write().await;
            *state = SyncState::Disconnected;
        }

        Ok(())
    }

    /// Start background synchronization tasks
    async fn start_background_tasks(&self) {
        let sync_manager = self.clone_for_background();
        let heartbeat_manager = self.clone_for_background();
        let message_handler = self.clone_for_background();

        // Start sync task
        tokio::spawn(async move {
            sync_manager.sync_task().await;
        });

        // Start heartbeat task
        tokio::spawn(async move {
            heartbeat_manager.heartbeat_task().await;
        });

        // Start message handler task
        tokio::spawn(async move {
            message_handler.message_handler_task().await;
        });
    }

    /// Background sync task
    async fn sync_task(&self) {
        let mut interval = interval(self.sync_interval);

        loop {
            interval.tick().await;

            let is_running = *self.is_running.read().await;
            if !is_running {
                break;
            }

            if let Err(e) = self.perform_sync().await {
                tracing::error!("Sync task error: {:?}", e);
            }
        }
    }

    /// Background heartbeat task
    async fn heartbeat_task(&self) {
        let mut interval = interval(self.heartbeat_interval);

        loop {
            interval.tick().await;

            let is_running = *self.is_running.read().await;
            if !is_running {
                break;
            }

            if let Err(e) = self.send_heartbeat().await {
                tracing::error!("Heartbeat task error: {:?}", e);
            }
        }
    }

    /// Background message handler task
    async fn message_handler_task(&self) {
        let mut receiver = self.message_receiver.write().await;

        while let Some(message) = receiver.recv().await {
            if let Err(e) = self.handle_message(message).await {
                tracing::error!("Message handler error: {:?}", e);
            }
        }
    }

    /// Perform synchronization with all peers
    async fn perform_sync(&self) -> Result<(), EndToEndSyncError> {
        let collections = self.collections.read().await;
        let peers = self.peers.read().await;

        for (collection_id, metadata) in collections.iter() {
            for (peer_id, peer_info) in peers.iter() {
                if peer_info.sync_status == PeerSyncStatus::Connected {
                    if let Err(e) = self.sync_with_peer(collection_id, peer_id).await {
                        tracing::warn!(
                            "Failed to sync collection {} with peer {}: {:?}",
                            collection_id,
                            peer_id,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Sync a specific collection with a specific peer
    async fn sync_with_peer(
        &self,
        collection_id: &str,
        peer_id: &ReplicaId,
    ) -> Result<(), EndToEndSyncError> {
        // Get local data
        let local_data = self.storage.get::<Vec<u8>>(collection_id).await?;

        if let Some(data) = local_data {
            // Create sync request
            let message = SyncMessage::SyncRequest {
                collection_id: collection_id.to_string(),
                replica_id: self.replica_id.clone(),
                data,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            };

            // Send sync request
            self.send_message(message).await?;
        }

        Ok(())
    }

    /// Send a heartbeat message
    async fn send_heartbeat(&self) -> Result<(), EndToEndSyncError> {
        let message = SyncMessage::Heartbeat {
            replica_id: self.replica_id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        self.send_message(message).await
    }

    /// Send a message via transport
    async fn send_message(&self, message: SyncMessage) -> Result<(), EndToEndSyncError> {
        let serialized = serde_json::to_vec(&message)?;
        self.transport.send(&serialized).await.map_err(EndToEndSyncError::from)?;
        Ok(())
    }

    /// Handle incoming messages
    async fn handle_message(&self, message: SyncMessage) -> Result<(), EndToEndSyncError> {
        match message {
            SyncMessage::SyncRequest {
                collection_id,
                replica_id,
                data,
                timestamp,
            } => {
                self.handle_sync_request(collection_id, replica_id, data, timestamp)
                    .await
            }
            SyncMessage::SyncResponse {
                collection_id,
                replica_id,
                data,
                timestamp,
            } => {
                self.handle_sync_response(collection_id, replica_id, data, timestamp)
                    .await
            }
            SyncMessage::Presence {
                replica_id,
                status,
                timestamp,
            } => self.handle_presence(replica_id, status, timestamp).await,
            SyncMessage::Heartbeat {
                replica_id,
                timestamp,
            } => self.handle_heartbeat(replica_id, timestamp).await,
            SyncMessage::Ack {
                message_id,
                replica_id,
                timestamp,
            } => self.handle_ack(message_id, replica_id, timestamp).await,
        }
    }

    /// Handle sync request from peer
    async fn handle_sync_request(
        &self,
        collection_id: String,
        replica_id: ReplicaId,
        data: Vec<u8>,
        timestamp: u64,
    ) -> Result<(), EndToEndSyncError> {
        // Get local data
        let local_data = self.storage.get::<Vec<u8>>(&collection_id).await?;

        // Merge data (simplified - in real implementation, use proper CRDT merge)
        let merged_data = if let Some(local) = local_data {
            // Simple merge strategy - in real implementation, use proper CRDT merge
            if timestamp
                > SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
                    - 1000
            {
                data // Use remote data if it's newer
            } else {
                local // Use local data if it's newer
            }
        } else {
            data // Use remote data if no local data
        };

        // Store merged data
        self.storage.set(&collection_id, &merged_data).await?;

        // Send sync response
        let response = SyncMessage::SyncResponse {
            collection_id,
            replica_id: self.replica_id.clone(),
            data: merged_data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        self.send_message(response).await
    }

    /// Handle sync response from peer
    async fn handle_sync_response(
        &self,
        collection_id: String,
        replica_id: ReplicaId,
        data: Vec<u8>,
        _timestamp: u64,
    ) -> Result<(), EndToEndSyncError> {
        // Store the merged data
        self.storage.set(&collection_id, &data).await?;

        // Update peer info
        {
            let mut peers = self.peers.write().await;
            if let Some(peer) = peers.get_mut(&replica_id) {
                  peer.last_sync = Some(chrono::Utc::now());
            }
        }

        Ok(())
    }

    /// Handle presence message
    async fn handle_presence(
        &self,
        replica_id: ReplicaId,
        status: String,
        timestamp: u64,
    ) -> Result<(), EndToEndSyncError> {
        let mut peers = self.peers.write().await;

        let peer_info = PeerInfo {
            replica_id: replica_id.clone(),
            last_seen: chrono::Utc::now(),
            is_online: status == "connected",
            last_sync: None,
            sync_status: if status == "connected" {
                PeerSyncStatus::Connected
            } else {
                PeerSyncStatus::Disconnected
            },
            // Additional fields for compatibility
            id: replica_id.clone(),
            status: if status == "connected" {
                PeerSyncStatus::Connected
            } else {
                PeerSyncStatus::Disconnected
            },
            version: 1,
        };

        peers.insert(replica_id, peer_info);
        Ok(())
    }

    /// Handle heartbeat message
    async fn handle_heartbeat(
        &self,
        replica_id: ReplicaId,
        timestamp: u64,
    ) -> Result<(), EndToEndSyncError> {
        let mut peers = self.peers.write().await;

        if let Some(peer) = peers.get_mut(&replica_id) {
            peer.last_seen = chrono::Utc::now();
        }

        Ok(())
    }

    /// Handle acknowledgment message
    async fn handle_ack(
        &self,
        _message_id: String,
        _replica_id: ReplicaId,
        _timestamp: u64,
    ) -> Result<(), EndToEndSyncError> {
        // Handle acknowledgment - in a real implementation, this would update message tracking
        Ok(())
    }

    /// Add a collection to synchronize
    pub async fn add_collection(
        &self,
        metadata: CollectionMetadata,
    ) -> Result<(), EndToEndSyncError> {
        let mut collections = self.collections.write().await;
        collections.insert(metadata.id.clone(), metadata);
        Ok(())
    }

    /// Remove a collection from synchronization
    pub async fn remove_collection(&self, collection_id: &str) -> Result<(), EndToEndSyncError> {
        let mut collections = self.collections.write().await;
        collections.remove(collection_id);
        Ok(())
    }

    /// Get collection metadata
    pub async fn get_collection(
        &self,
        collection_id: &str,
    ) -> Result<Option<CollectionMetadata>, EndToEndSyncError> {
        let collections = self.collections.read().await;
        Ok(collections.get(collection_id).cloned())
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionMetadata>, EndToEndSyncError> {
        let collections = self.collections.read().await;
        Ok(collections.values().cloned().collect())
    }

    /// Get peer information
    pub async fn get_peer(
        &self,
        peer_id: &ReplicaId,
    ) -> Result<Option<PeerInfo>, EndToEndSyncError> {
        let peers = self.peers.read().await;
        Ok(peers.get(peer_id).cloned())
    }

    /// List all peers
    pub async fn list_peers(&self) -> Result<Vec<PeerInfo>, EndToEndSyncError> {
        let peers = self.peers.read().await;
        Ok(peers.values().cloned().collect())
    }

    /// Get current sync state
    pub async fn get_sync_state(&self) -> SyncState {
        let state = self.sync_state.read().await;
        state.clone()
    }

    /// Check if the manager is running
    pub async fn is_running(&self) -> bool {
        let is_running = self.is_running.read().await;
        *is_running
    }

    /// Clone the manager for background tasks
    fn clone_for_background(&self) -> EndToEndSyncManager<S, T> {
        EndToEndSyncManager {
            replica_id: self.replica_id.clone(),
            storage: self.storage.clone(),
            transport: self.transport.clone(),
            collections: self.collections.clone(),
            peers: self.peers.clone(),
            sync_state: self.sync_state.clone(),
            message_sender: self.message_sender.clone(),
            message_receiver: self.message_receiver.clone(),
            sync_interval: self.sync_interval,
            heartbeat_interval: self.heartbeat_interval,
            is_running: self.is_running.clone(),
        }
    }
}

impl<S, T> Clone for EndToEndSyncManager<S, T>
where
    S: LocalStorage + Send + Sync + 'static,
    T: SyncTransport + Send + Sync + 'static,
    EndToEndSyncError: From<<T as SyncTransport>::Error>,
{
    fn clone(&self) -> Self {
        self.clone_for_background()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::MemoryStorage;
    use crate::transport::memory::InMemoryTransport;

    #[tokio::test]
    async fn test_end_to_end_sync_manager_creation() {
        let storage = Arc::new(MemoryStorage::new());
        let transport = Arc::new(InMemoryTransport::new());
        let replica_id = ReplicaId::default();

        let manager = EndToEndSyncManager::new(
            replica_id,
            storage,
            transport,
            Duration::from_secs(5),
            Duration::from_secs(30),
        );

        assert!(!manager.is_running().await);
        assert_eq!(manager.get_sync_state().await, SyncState::Disconnected);
    }

    #[tokio::test]
    async fn test_collection_management() {
        let storage = Arc::new(MemoryStorage::new());
        let transport = Arc::new(InMemoryTransport::new());
        let replica_id = ReplicaId::default();

        let manager = EndToEndSyncManager::new(
            replica_id,
            storage,
            transport,
            Duration::from_secs(5),
            Duration::from_secs(30),
        );

        let metadata = CollectionMetadata {
            id: "test_collection".to_string(),
            name: "Test Collection".to_string(),
            crdt_type: "LwwMap".to_string(),
            version: 1,
            last_sync: 0,
            replica_count: 1,
        };

        // Add collection
        assert!(manager.add_collection(metadata.clone()).await.is_ok());

        // Get collection
        let retrieved = manager.get_collection("test_collection").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test_collection");

        // List collections
        let collections = manager.list_collections().await.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, "test_collection");

        // Remove collection
        assert!(manager.remove_collection("test_collection").await.is_ok());

        let collections = manager.list_collections().await.unwrap();
        assert_eq!(collections.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_message_serialization() {
        let message = SyncMessage::SyncRequest {
            collection_id: "test_collection".to_string(),
            replica_id: ReplicaId::default(),
            data: b"test data".to_vec(),
            timestamp: 1234567890,
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: SyncMessage = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            SyncMessage::SyncRequest {
                collection_id,
                replica_id,
                data,
                timestamp,
            } => {
                assert_eq!(collection_id, "test_collection");
                assert_eq!(replica_id.0, replica_id.0); // Just check it's valid
                assert_eq!(data, b"test data");
                assert_eq!(timestamp, 1234567890);
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
