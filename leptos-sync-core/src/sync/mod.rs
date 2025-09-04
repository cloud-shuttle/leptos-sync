//! Synchronization engine implementation

pub mod engine;
pub mod conflict;
pub mod realtime;

use crate::{
    crdt::{Mergeable, ReplicaId},
    storage::{LocalStorage, StorageError},
    transport::{SyncTransport, TransportError},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub use engine::{SyncEngine, SyncEngineError, SyncState, PeerInfo, PeerSyncStatus, DefaultConflictResolver};

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("CRDT error: {0}")]
    CrdtError(#[from] std::io::Error),
    #[error("Sync operation failed: {0}")]
    SyncFailed(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("GDPR error: {0}")]
    GDPRError(String),
}

/// Legacy synchronization message types (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage<T> {
    /// Sync request with data
    Sync { key: String, data: T },
    /// Acknowledgment of sync
    Ack { key: String },
    /// Peer presence announcement
    Presence { replica_id: ReplicaId },
}

/// Legacy synchronization manager (for backward compatibility)
pub struct SyncManager<S, T> 
where 
    S: LocalStorage,
    T: SyncTransport,
{
    replica_id: ReplicaId,
    state: SyncState,
    peers: HashMap<ReplicaId, PeerInfo>,
    storage: S,
    transport: T,
}

impl<S, T> SyncManager<S, T>
where
    S: LocalStorage,
    T: SyncTransport,
{
    pub fn new(storage: S, transport: T) -> Self {
        Self {
            replica_id: ReplicaId::default(),
            state: SyncState::NotSynced,
            peers: HashMap::new(),
            storage,
            transport,
        }
    }

    pub fn with_replica_id(storage: S, transport: T, replica_id: ReplicaId) -> Self {
        Self {
            replica_id,
            state: SyncState::NotSynced,
            peers: HashMap::new(),
            storage,
            transport,
        }
    }

    pub fn state(&self) -> &SyncState {
        &self.state
    }

    pub fn replica_id(&self) -> ReplicaId {
        self.replica_id
    }

    pub fn is_online(&self) -> bool {
        self.transport.is_connected()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Sync a CRDT value with peers
    pub async fn sync<V>(&mut self, key: &str, value: &V) -> Result<(), SyncError>
    where
        V: Mergeable + Serialize + Send + Sync + Clone,
        V::Error: Into<SyncError>,
    {
        // Store locally first
        self.storage.set(key, value).await
            .map_err(|e| SyncError::SyncFailed(format!("Storage error: {}", e)))?;

        // Announce to peers if connected
        if self.transport.is_connected() {
            let message = SyncMessage::Sync {
                key: key.to_string(),
                data: value.clone(),
            };
            let serialized = serde_json::to_vec(&message)?;
            self.transport.send(&serialized).await
                .map_err(|e| SyncError::SyncFailed(format!("Transport error: {}", e)))?;
        }

        Ok(())
    }

    /// Process incoming sync messages
    pub async fn process_messages<V>(&mut self) -> Result<Vec<(String, V)>, SyncError>
    where
        V: Mergeable + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync,
        V::Error: Into<SyncError>,
    {
        let mut updates = Vec::new();

        // Check for incoming messages
        let messages = self.transport.receive().await
            .map_err(|e| SyncError::SyncFailed(format!("Transport error: {}", e)))?;
        
        for message_bytes in messages {
            match serde_json::from_slice::<SyncMessage<V>>(&message_bytes) {
                Ok(SyncMessage::Sync { key, data }) => {
                    // Try to merge with existing data
                    match self.storage.get::<V>(&key).await
                        .map_err(|e| SyncError::SyncFailed(format!("Storage error: {}", e)))? {
                        Some(mut existing) => {
                            existing.merge(&data).map_err(Into::into)?;
                            self.storage.set(&key, &existing).await
                                .map_err(|e| SyncError::SyncFailed(format!("Storage error: {}", e)))?;
                            updates.push((key, existing));
                        }
                        None => {
                            // No existing data, store as-is
                            self.storage.set(&key, &data).await
                                .map_err(|e| SyncError::SyncFailed(format!("Storage error: {}", e)))?;
                            updates.push((key, data));
                        }
                    }
                }
                Ok(SyncMessage::Ack { key: _ }) => {
                    // Handle acknowledgment
                    tracing::debug!("Received sync acknowledgment");
                }
                Ok(SyncMessage::Presence { replica_id }) => {
                    // Update peer info
                    let peer_info = PeerInfo {
                        replica_id,
                        last_seen: chrono::Utc::now(),
                        is_online: true,
                        last_sync: None,
                        sync_status: PeerSyncStatus::Never,
                    };
                    self.peers.insert(replica_id, peer_info);
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize sync message: {}", e);
                }
            }
        }

        Ok(updates)
    }

    /// Announce presence to peers
    pub async fn announce_presence(&mut self) -> Result<(), SyncError> {
        if !self.transport.is_connected() {
            return Ok(());
        }

        let message = SyncMessage::<()>::Presence {
            replica_id: self.replica_id,
        };
        let serialized = serde_json::to_vec(&message)?;
        self.transport.send(&serialized).await
            .map_err(|e| SyncError::SyncFailed(format!("Transport error: {}", e)))?;

        Ok(())
    }

    /// Get all peers
    pub fn peers(&self) -> impl Iterator<Item = (&ReplicaId, &PeerInfo)> {
        self.peers.iter()
    }
}
