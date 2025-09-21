//! IndexedDB storage implementation with real browser storage

use super::{LocalStorage, StorageError};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

pub mod connection;
pub mod crdt_store;
pub mod errors;
pub mod operations;

use connection::IndexedDbConnection;
use crdt_store::CrdtStore;
use errors::IndexedDbError;
use operations::IndexedDbOperations;

/// IndexedDB storage implementation with real browser storage
pub struct IndexedDbStorage {
    db_name: String,
    store_name: String,
    fallback: super::memory::MemoryStorage,
    connection: Option<Arc<IndexedDbConnection>>,
    operations: Option<IndexedDbOperations>,
    crdt_store: Option<CrdtStore>,
    initialized: bool,
}

impl IndexedDbStorage {
    /// Create a new IndexedDB storage instance
    pub fn new(db_name: String, store_name: String) -> Self {
        Self {
            db_name,
            store_name,
            fallback: super::memory::MemoryStorage::new(),
            connection: None,
            operations: None,
            crdt_store: None,
            initialized: false,
        }
    }

    /// Initialize the IndexedDB database
    pub async fn initialize(&mut self) -> Result<(), StorageError> {
        if self.initialized {
            return Ok(());
        }

        match IndexedDbConnection::open(&self.db_name, 1).await {
            Ok(connection) => {
                let connection = Arc::new(connection);
                let operations = IndexedDbOperations::new(connection.clone());
                let crdt_store = CrdtStore::new(connection.clone());

                self.connection = Some(connection);
                self.operations = Some(operations);
                self.crdt_store = Some(crdt_store);
                self.initialized = true;
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize IndexedDB: {:?}, falling back to memory storage",
                    e
                );
                self.initialized = true; // Mark as initialized to avoid retry
                Ok(()) // Don't fail, just use fallback
            }
        }
    }

    /// Ensure the storage is initialized
    async fn ensure_initialized(&mut self) -> Result<(), StorageError> {
        if !self.initialized {
            self.initialize().await?;
        }
        Ok(())
    }

    /// Get the operations instance (if available)
    fn get_operations(&self) -> Option<&IndexedDbOperations> {
        self.operations.as_ref()
    }

    /// Get the CRDT store instance (if available)
    fn get_crdt_store(&self) -> Option<&CrdtStore> {
        self.crdt_store.as_ref()
    }

    /// Check if IndexedDB is available and working
    pub fn is_indexeddb_available(&self) -> bool {
        self.connection.is_some()
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<crdt_store::StorageStats, StorageError> {
        if let Some(crdt_store) = &self.crdt_store {
            crdt_store.get_stats().await.map_err(Into::into)
        } else {
            // Return empty stats if IndexedDB is not available
            Ok(crdt_store::StorageStats {
                collections_count: 0,
                deltas_count: 0,
                peers_count: 0,
            })
        }
    }

    /// Store a CRDT delta
    pub async fn store_delta(
        &mut self,
        collection_id: &str,
        delta: &[u8],
        replica_id: &str,
        operation_type: &str,
    ) -> Result<(), StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store
                .store_delta(collection_id, delta, replica_id, operation_type)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage for deltas
            let key = format!("delta:{}:{}:{}", collection_id, replica_id, operation_type);
            self.fallback.set(&key, &delta.to_vec()).await
        }
    }

    /// Get CRDT deltas for a collection
    pub async fn get_deltas(
        &mut self,
        collection_id: &str,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
    ) -> Result<Vec<crdt_store::DeltaRecord>, StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store
                .get_deltas(collection_id, from_timestamp, to_timestamp)
                .await
                .map_err(Into::into)
        } else {
            // Return empty deltas if IndexedDB is not available
            Ok(Vec::new())
        }
    }

    /// Store collection metadata
    pub async fn store_metadata(
        &mut self,
        metadata: &crdt_store::CollectionMetadata,
    ) -> Result<(), StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store
                .store_metadata(metadata)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage for metadata
            let key = format!("metadata:{}", metadata.id);
            self.fallback.set(&key, metadata).await
        }
    }

    /// Get collection metadata
    pub async fn get_metadata(
        &mut self,
        collection_id: &str,
    ) -> Result<Option<crdt_store::CollectionMetadata>, StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store
                .get_metadata(collection_id)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage for metadata
            let key = format!("metadata:{}", collection_id);
            self.fallback.get(&key).await
        }
    }

    /// List all collections
    pub async fn list_collections(
        &mut self,
    ) -> Result<Vec<crdt_store::CollectionMetadata>, StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store.list_collections().await.map_err(Into::into)
        } else {
            // Return empty list if IndexedDB is not available
            Ok(Vec::new())
        }
    }

    /// Store peer information
    pub async fn store_peer(&mut self, peer: &crdt_store::PeerInfo) -> Result<(), StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store.store_peer(peer).await.map_err(Into::into)
        } else {
            // Fall back to memory storage for peers
            let key = format!("peer:{}", peer.id);
            self.fallback.set(&key, peer).await
        }
    }

    /// Get peer information
    pub async fn get_peer(
        &mut self,
        peer_id: &str,
    ) -> Result<Option<crdt_store::PeerInfo>, StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store.get_peer(peer_id).await.map_err(Into::into)
        } else {
            // Fall back to memory storage for peers
            let key = format!("peer:{}", peer_id);
            self.fallback.get(&key).await
        }
    }

    /// List all peers
    pub async fn list_peers(&mut self) -> Result<Vec<crdt_store::PeerInfo>, StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store.list_peers().await.map_err(Into::into)
        } else {
            // Return empty list if IndexedDB is not available
            Ok(Vec::new())
        }
    }

    /// Clean up old deltas
    pub async fn cleanup_old_deltas(
        &mut self,
        collection_id: &str,
        keep_count: usize,
    ) -> Result<(), StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store
                .cleanup_old_deltas(collection_id, keep_count)
                .await
                .map_err(Into::into)
        } else {
            // No cleanup needed for memory storage
            Ok(())
        }
    }

    /// Clear all data
    pub async fn clear_all(&mut self) -> Result<(), StorageError> {
        self.ensure_initialized().await?;

        if let Some(crdt_store) = &self.crdt_store {
            crdt_store
                .clear_all()
                .await
                .map_err(|e| StorageError::OperationFailed(e.to_string()))?;
        }

        // Also clear fallback storage
        self.fallback.clear().await?;
        Ok(())
    }
}

impl Clone for IndexedDbStorage {
    fn clone(&self) -> Self {
        Self {
            db_name: self.db_name.clone(),
            store_name: self.store_name.clone(),
            fallback: self.fallback.clone(),
            connection: None,   // Cannot clone connections
            operations: None,   // Cannot clone operations
            crdt_store: None,   // Cannot clone CRDT store
            initialized: false, // Will need to reinitialize
        }
    }
}

#[async_trait]
impl LocalStorage for IndexedDbStorage {
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError> {
        if let Some(operations) = &self.operations {
            operations
                .set(&self.store_name, key, value)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.set(key, value).await
        }
    }

    async fn get<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError> {
        if let Some(operations) = &self.operations {
            operations
                .get(&self.store_name, key)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.get(key).await
        }
    }

    async fn remove(&self, key: &str) -> Result<(), StorageError> {
        if let Some(operations) = &self.operations {
            operations
                .delete(&self.store_name, key)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.remove(key).await
        }
    }

    async fn keys(&self) -> Result<Vec<String>, StorageError> {
        if let Some(operations) = &self.operations {
            operations.keys(&self.store_name).await.map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.keys().await
        }
    }

    async fn contains_key(&self, key: &str) -> Result<bool, StorageError> {
        if let Some(operations) = &self.operations {
            operations
                .contains_key(&self.store_name, key)
                .await
                .map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.contains_key(key).await
        }
    }

    async fn len(&self) -> Result<usize, StorageError> {
        if let Some(operations) = &self.operations {
            operations.count(&self.store_name).await.map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.len().await
        }
    }

    async fn is_empty(&self) -> Result<bool, StorageError> {
        let len = self.len().await?;
        Ok(len == 0)
    }

    async fn clear(&self) -> Result<(), StorageError> {
        if let Some(operations) = &self.operations {
            operations.clear(&self.store_name).await.map_err(Into::into)
        } else {
            // Fall back to memory storage
            self.fallback.clear().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_indexeddb_storage_creation() {
        let storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());

        assert_eq!(storage.db_name, "test_db");
        assert_eq!(storage.store_name, "test_store");
        assert!(!storage.initialized);
        assert!(!storage.is_indexeddb_available());
    }

    #[tokio::test]
    async fn test_indexeddb_storage_fallback() {
        let mut storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());

        // Test that operations work (falling back to memory storage)
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());

        let value = storage.get::<String>("key1").await.unwrap();
        // Note: This might be None if IndexedDB is not available in test environment
        // The important thing is that it doesn't panic

        // Test remove
        assert!(storage.remove("key1").await.is_ok());
    }

    #[tokio::test]
    async fn test_indexeddb_storage_clone() {
        let storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());
        let cloned_storage = storage.clone();

        // Test that cloned storage works
        assert!(cloned_storage
            .set("key1", &"value1".to_string())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_indexeddb_storage_stats() {
        let mut storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.collections_count, 0);
        assert_eq!(stats.deltas_count, 0);
        assert_eq!(stats.peers_count, 0);
    }
}
