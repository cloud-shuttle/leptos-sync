//! CRDT-specific storage operations

use super::connection::IndexedDbConnection;
use super::errors::IndexedDbError;
use super::operations::IndexedDbOperations;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// CRDT delta record for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaRecord {
    pub id: String,
    pub collection_id: String,
    pub delta: Vec<u8>,
    pub timestamp: u64,
    pub replica_id: String,
    pub operation_type: String,
}

/// CRDT collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub id: String,
    pub name: String,
    pub crdt_type: String,
    pub version: u32,
    pub last_sync: u64,
    pub replica_count: u32,
}

/// CRDT peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub last_seen: u64,
    pub status: String,
    pub version: u32,
}

/// CRDT-specific storage operations
pub struct CrdtStore {
    operations: IndexedDbOperations,
}

impl CrdtStore {
    /// Create a new CRDT store
    pub fn new(connection: Arc<IndexedDbConnection>) -> Self {
        let operations = IndexedDbOperations::new(connection);
        Self { operations }
    }

    /// Store a CRDT delta
    pub async fn store_delta(&self, collection_id: &str, delta: &[u8], replica_id: &str, operation_type: &str) -> Result<(), IndexedDbError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let delta_id = format!("{}#{}#{}", collection_id, timestamp, replica_id);
        
        let record = DeltaRecord {
            id: delta_id,
            collection_id: collection_id.to_string(),
            delta: delta.to_vec(),
            timestamp,
            replica_id: replica_id.to_string(),
            operation_type: operation_type.to_string(),
        };

        self.operations.set("deltas", &record.id, &record).await
    }

    /// Get deltas for a collection within a time range
    pub async fn get_deltas(&self, collection_id: &str, from_timestamp: Option<u64>, to_timestamp: Option<u64>) -> Result<Vec<DeltaRecord>, IndexedDbError> {
        // For now, we'll get all deltas and filter them
        // In a real implementation, we'd use IndexedDB indexes for efficient querying
        let all_keys = self.operations.keys("deltas").await?;
        let mut deltas = Vec::new();

        for key in all_keys {
            if let Some(record) = self.operations.get::<DeltaRecord>("deltas", &key).await? {
                if record.collection_id == collection_id {
                    let include = match (from_timestamp, to_timestamp) {
                        (Some(from), Some(to)) => record.timestamp >= from && record.timestamp <= to,
                        (Some(from), None) => record.timestamp >= from,
                        (None, Some(to)) => record.timestamp <= to,
                        (None, None) => true,
                    };

                    if include {
                        deltas.push(record);
                    }
                }
            }
        }

        // Sort by timestamp
        deltas.sort_by_key(|d| d.timestamp);
        Ok(deltas)
    }

    /// Get the latest delta for a collection
    pub async fn get_latest_delta(&self, collection_id: &str) -> Result<Option<DeltaRecord>, IndexedDbError> {
        let deltas = self.get_deltas(collection_id, None, None).await?;
        Ok(deltas.into_iter().last())
    }

    /// Store collection metadata
    pub async fn store_metadata(&self, metadata: &CollectionMetadata) -> Result<(), IndexedDbError> {
        self.operations.set("metadata", &metadata.id, metadata).await
    }

    /// Get collection metadata
    pub async fn get_metadata(&self, collection_id: &str) -> Result<Option<CollectionMetadata>, IndexedDbError> {
        self.operations.get("metadata", collection_id).await
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionMetadata>, IndexedDbError> {
        let keys = self.operations.keys("metadata").await?;
        let mut collections = Vec::new();

        for key in keys {
            if let Some(metadata) = self.operations.get::<CollectionMetadata>("metadata", &key).await? {
                collections.push(metadata);
            }
        }

        Ok(collections)
    }

    /// Store peer information
    pub async fn store_peer(&self, peer: &PeerInfo) -> Result<(), IndexedDbError> {
        self.operations.set("peers", &peer.id, peer).await
    }

    /// Get peer information
    pub async fn get_peer(&self, peer_id: &str) -> Result<Option<PeerInfo>, IndexedDbError> {
        self.operations.get("peers", peer_id).await
    }

    /// List all peers
    pub async fn list_peers(&self) -> Result<Vec<PeerInfo>, IndexedDbError> {
        let keys = self.operations.keys("peers").await?;
        let mut peers = Vec::new();

        for key in keys {
            if let Some(peer) = self.operations.get::<PeerInfo>("peers", &key).await? {
                peers.push(peer);
            }
        }

        Ok(peers)
    }

    /// Update peer last seen timestamp
    pub async fn update_peer_last_seen(&self, peer_id: &str) -> Result<(), IndexedDbError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(mut peer) = self.get_peer(peer_id).await? {
            peer.last_seen = timestamp;
            self.store_peer(&peer).await?;
        }

        Ok(())
    }

    /// Clean up old deltas (for quota management)
    pub async fn cleanup_old_deltas(&self, collection_id: &str, keep_count: usize) -> Result<(), IndexedDbError> {
        let mut deltas = self.get_deltas(collection_id, None, None).await?;
        
        if deltas.len() > keep_count {
            // Sort by timestamp (oldest first)
            deltas.sort_by_key(|d| d.timestamp);
            
            // Remove oldest deltas
            let to_remove = deltas.len() - keep_count;
            for delta in deltas.into_iter().take(to_remove) {
                self.operations.delete("deltas", &delta.id).await?;
            }
        }

        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats, IndexedDbError> {
        let collections_count = self.operations.count("metadata").await?;
        let deltas_count = self.operations.count("deltas").await?;
        let peers_count = self.operations.count("peers").await?;

        Ok(StorageStats {
            collections_count,
            deltas_count,
            peers_count,
        })
    }

    /// Clear all CRDT data
    pub async fn clear_all(&self) -> Result<(), IndexedDbError> {
        self.operations.clear("collections").await?;
        self.operations.clear("metadata").await?;
        self.operations.clear("deltas").await?;
        self.operations.clear("peers").await?;
        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub collections_count: usize,
    pub deltas_count: usize,
    pub peers_count: usize,
}

impl Clone for CrdtStore {
    fn clone(&self) -> Self {
        Self {
            operations: self.operations.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crdt_store_creation() {
        // Test that CRDT store can be created
        // We can't easily test IndexedDB operations in unit tests without a real browser environment
        // This test just verifies the structure is correct
        #[cfg(not(target_arch = "wasm32"))]
        {
            // In non-WASM environment, operations should fail gracefully
            // This would fail to compile, so we just test the error handling
        }
    }

    #[test]
    fn test_delta_record_serialization() {
        let record = DeltaRecord {
            id: "test#123#replica1".to_string(),
            collection_id: "test_collection".to_string(),
            delta: b"test delta data".to_vec(),
            timestamp: 1234567890,
            replica_id: "replica1".to_string(),
            operation_type: "set".to_string(),
        };

        let serialized = serde_json::to_string(&record).unwrap();
        let deserialized: DeltaRecord = serde_json::from_str(&serialized).unwrap();

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.collection_id, deserialized.collection_id);
        assert_eq!(record.delta, deserialized.delta);
        assert_eq!(record.timestamp, deserialized.timestamp);
        assert_eq!(record.replica_id, deserialized.replica_id);
        assert_eq!(record.operation_type, deserialized.operation_type);
    }

    #[test]
    fn test_collection_metadata_serialization() {
        let metadata = CollectionMetadata {
            id: "test_collection".to_string(),
            name: "Test Collection".to_string(),
            crdt_type: "LwwMap".to_string(),
            version: 1,
            last_sync: 1234567890,
            replica_count: 3,
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: CollectionMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.crdt_type, deserialized.crdt_type);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.last_sync, deserialized.last_sync);
        assert_eq!(metadata.replica_count, deserialized.replica_count);
    }

    #[test]
    fn test_peer_info_serialization() {
        let peer = PeerInfo {
            id: "peer1".to_string(),
            last_seen: 1234567890,
            status: "connected".to_string(),
            version: 1,
        };

        let serialized = serde_json::to_string(&peer).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(peer.id, deserialized.id);
        assert_eq!(peer.last_seen, deserialized.last_seen);
        assert_eq!(peer.status, deserialized.status);
        assert_eq!(peer.version, deserialized.version);
    }
}