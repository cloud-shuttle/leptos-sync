//! Enhanced synchronization engine for real-time sync

use crate::{
    crdt::{Mergeable, ReplicaId},
    storage::Storage,
    transport::{SyncTransport, TransportError},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncEngineError {
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("CRDT error: {0}")]
    CrdtError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Sync operation failed: {0}")]
    SyncFailed(String),
    #[error("Conflict resolution failed: {0}")]
    ConflictResolution(String),
}

/// Enhanced synchronization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncState {
    /// Not synchronized
    NotSynced,
    /// Currently synchronizing
    Syncing,
    /// Synchronized
    Synced,
    /// Synchronization failed
    Failed(String),
    /// Resolving conflicts
    ResolvingConflicts,
    /// Offline mode
    Offline,
}

/// Enhanced synchronization message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage<T> {
    /// Sync request with data
    Sync { key: String, data: T, replica_id: ReplicaId, timestamp: chrono::DateTime<chrono::Utc> },
    /// Acknowledgment of sync
    Ack { key: String, replica_id: ReplicaId },
    /// Peer presence announcement
    Presence { replica_id: ReplicaId, timestamp: chrono::DateTime<chrono::Utc> },
    /// Conflict resolution request
    Conflict { key: String, data: T, replica_id: ReplicaId, timestamp: chrono::DateTime<chrono::Utc> },
    /// Heartbeat to keep connection alive
    Heartbeat { replica_id: ReplicaId, timestamp: chrono::DateTime<chrono::Utc> },
}

/// Enhanced synchronization manager
pub struct SyncEngine<Tr> 
where 
    Tr: SyncTransport + Clone,
{
    replica_id: ReplicaId,
    state: Arc<RwLock<SyncState>>,
    peers: Arc<RwLock<HashMap<ReplicaId, PeerInfo>>>,
    storage: Storage,
    transport: Tr,
    sync_queue: Arc<RwLock<Vec<SyncMessage<Vec<u8>>>>>,
    conflict_resolver: Arc<RwLock<Option<DefaultConflictResolver>>>,
}

/// Information about a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub replica_id: ReplicaId,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub is_online: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub sync_status: PeerSyncStatus,
}

/// Peer synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerSyncStatus {
    /// Never synced
    Never,
    /// Last sync was successful
    Success { timestamp: chrono::DateTime<chrono::Utc> },
    /// Last sync failed
    Failed { timestamp: chrono::DateTime<chrono::Utc>, error: String },
    /// Currently syncing
    Syncing { started: chrono::DateTime<chrono::Utc> },
}

/// Default conflict resolver using Last-Write-Wins
pub struct DefaultConflictResolver;

impl DefaultConflictResolver {
    /// Resolve a conflict between two values
    pub fn resolve<T: Mergeable>(&self, local: &T, remote: &T) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        // For now, just merge them - in a real implementation you'd want more sophisticated logic
        let mut result = local.clone();
        result.merge(remote).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(result)
    }
}

impl<Tr> SyncEngine<Tr>
where
    Tr: SyncTransport + Clone + 'static,
{
    pub fn new(storage: Storage, transport: Tr) -> Self {
        Self {
            replica_id: ReplicaId::default(),
            state: Arc::new(RwLock::new(SyncState::NotSynced)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            storage,
            transport,
            sync_queue: Arc::new(RwLock::new(Vec::new())),
            conflict_resolver: Arc::new(RwLock::new(Some(DefaultConflictResolver))),
        }
    }

    pub fn with_replica_id(storage: Storage, transport: Tr, replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            state: Arc::new(RwLock::new(SyncState::NotSynced)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            storage,
            transport,
            sync_queue: Arc::new(RwLock::new(Vec::new())),
            conflict_resolver: Arc::new(RwLock::new(Some(DefaultConflictResolver))),
        }
    }

    pub async fn state(&self) -> SyncState {
        self.state.read().await.clone()
    }

    pub fn replica_id(&self) -> ReplicaId {
        self.replica_id
    }

    pub async fn is_online(&self) -> bool {
        self.transport.is_connected()
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Start the synchronization process
    pub async fn start_sync(&mut self) -> Result<(), SyncEngineError> {
        let mut state = self.state.write().await;
        *state = SyncState::Syncing;

        // Try to connect to transport
        if !self.transport.is_connected() {
            // For WebSocket, we'd try to connect here
            tracing::info!("Transport not connected, attempting to connect...");
        }

        // Announce presence to peers
        self.announce_presence().await?;

        // Start background sync loop
        self.start_background_sync().await;

        Ok(())
    }

    /// Stop the synchronization process
    pub async fn stop_sync(&mut self) -> Result<(), SyncEngineError> {
        let mut state = self.state.write().await;
        *state = SyncState::NotSynced;

        // Disconnect from transport if needed
        if self.transport.is_connected() {
            tracing::info!("Stopping sync, disconnecting from transport...");
        }

        Ok(())
    }

    /// Sync a CRDT value with peers
    pub async fn sync<V>(&mut self, key: &str, value: &V) -> Result<(), SyncEngineError>
    where
        V: Mergeable + Serialize + Send + Sync + Clone,
    {
        // Serialize the value
        let data = serde_json::to_vec(value)?;
        
        // Create sync message
        let message = SyncMessage::Sync {
            key: key.to_string(),
            data,
            replica_id: self.replica_id,
            timestamp: chrono::Utc::now(),
        };

        // Add to sync queue
        {
            let mut queue = self.sync_queue.write().await;
            queue.push(SyncMessage::Sync {
                key: key.to_string(),
                data: serde_json::to_vec(value)?,
                replica_id: self.replica_id,
                timestamp: chrono::Utc::now(),
            });
        }

        // Send via transport
        let message_bytes = serde_json::to_vec(&message)?;
        self.transport.send(&message_bytes).await
            .map_err(|e| SyncEngineError::Transport(TransportError::SendFailed(e.to_string())))?;

        Ok(())
    }

    /// Process incoming messages
    pub async fn process_messages(&mut self) -> Result<(), SyncEngineError> {
        // Receive messages from transport
        let messages = self.transport.receive().await
            .map_err(|e| SyncEngineError::Transport(TransportError::ReceiveFailed(e.to_string())))?;
        
        for message_bytes in messages {
            let message: SyncMessage<Vec<u8>> = serde_json::from_slice(&message_bytes)?;
            
            match message {
                SyncMessage::Sync { key, data, replica_id, timestamp } => {
                    // Handle sync message
                    self.handle_sync_message(key, data, replica_id, timestamp).await?;
                }
                SyncMessage::Ack { key, replica_id } => {
                    // Handle acknowledgment
                    self.handle_ack_message(key, replica_id).await?;
                }
                SyncMessage::Presence { replica_id, timestamp } => {
                    // Handle presence update
                    self.handle_presence_message(replica_id, timestamp).await?;
                }
                SyncMessage::Conflict { key, data, replica_id, timestamp } => {
                    // Handle conflict resolution
                    self.handle_conflict_message(key, data, replica_id, timestamp).await?;
                }
                SyncMessage::Heartbeat { replica_id, timestamp } => {
                    // Handle heartbeat
                    self.handle_heartbeat_message(replica_id, timestamp).await?;
                }
            }
        }

        Ok(())
    }

    /// Announce presence to peers
    async fn announce_presence(&self) -> Result<(), SyncEngineError> {
        let message: SyncMessage<()> = SyncMessage::Presence {
            replica_id: self.replica_id,
            timestamp: chrono::Utc::now(),
        };

        let message_bytes = serde_json::to_vec(&message)?;
        self.transport.send(&message_bytes).await
            .map_err(|e| SyncEngineError::Transport(TransportError::SendFailed(e.to_string())))?;

        Ok(())
    }

    /// Send heartbeat to peers
    async fn send_heartbeat(&self) -> Result<(), SyncEngineError> {
        let message: SyncMessage<()> = SyncMessage::Heartbeat {
            replica_id: self.replica_id,
            timestamp: chrono::Utc::now(),
        };

        let message_bytes = serde_json::to_vec(&message)?;
        self.transport.send(&message_bytes).await
            .map_err(|e| SyncEngineError::Transport(TransportError::SendFailed(e.to_string())))?;

        Ok(())
    }

    /// Start background synchronization loop
    async fn start_background_sync(&self) {
        let transport = self.transport.clone();
        let replica_id = self.replica_id;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Send heartbeat
                let message: SyncMessage<()> = SyncMessage::Heartbeat {
                    replica_id,
                    timestamp: chrono::Utc::now(),
                };
                
                if let Ok(message_bytes) = serde_json::to_vec(&message) {
                    let _ = transport.send(&message_bytes).await;
                }
            }
        });
    }

    /// Handle sync message
    async fn handle_sync_message(&mut self, key: String, _data: Vec<u8>, replica_id: ReplicaId, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), SyncEngineError> {
        tracing::debug!("Received sync message for key {} from replica {}", key, replica_id);
        
        // For now, just acknowledge
        let ack: SyncMessage<()> = SyncMessage::Ack {
            key,
            replica_id,
        };
        
        let ack_bytes = serde_json::to_vec(&ack)?;
        self.transport.send(&ack_bytes).await
            .map_err(|e| SyncEngineError::Transport(TransportError::SendFailed(e.to_string())))?;

        Ok(())
    }

    /// Handle acknowledgment message
    async fn handle_ack_message(&mut self, _key: String, _replica_id: ReplicaId) -> Result<(), SyncEngineError> {
        // For now, just log
        tracing::debug!("Received ack message");
        Ok(())
    }

    /// Handle presence message
    async fn handle_presence_message(&mut self, replica_id: ReplicaId, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), SyncEngineError> {
        let mut peers = self.peers.write().await;
        
        let peer_info = PeerInfo {
            replica_id,
            last_seen: timestamp,
            is_online: true,
            last_sync: None,
            sync_status: PeerSyncStatus::Never,
        };
        
        peers.insert(replica_id, peer_info);
        
        tracing::debug!("Updated peer info for replica {}", replica_id);
        Ok(())
    }

    /// Handle conflict message
    async fn handle_conflict_message(&mut self, _key: String, _data: Vec<u8>, _replica_id: ReplicaId, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), SyncEngineError> {
        // For now, just log
        tracing::debug!("Received conflict message");
        Ok(())
    }

    /// Handle heartbeat message
    async fn handle_heartbeat_message(&mut self, replica_id: ReplicaId, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), SyncEngineError> {
        let mut peers = self.peers.write().await;
        
        if let Some(peer_info) = peers.get_mut(&replica_id) {
            peer_info.last_seen = timestamp;
            tracing::debug!("Updated heartbeat for replica {}", replica_id);
        }

        Ok(())
    }

    /// Get all peers
    pub async fn peers(&self) -> impl Iterator<Item = (ReplicaId, PeerInfo)> {
        let peers = self.peers.read().await;
        peers.clone().into_iter()
    }

    /// Check if there's a conflict between two values
    fn has_conflict<V: Mergeable>(&self, _local: &V, _remote: &V) -> bool {
        // For now, always return false - in a real implementation you'd check timestamps
        false
    }
}
