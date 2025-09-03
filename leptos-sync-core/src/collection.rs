//! Local-first collection with synchronization capabilities

use crate::{
    crdt::{Mergeable, ReplicaId},
    storage::{LocalStorage, Storage, StorageError},
    sync::{SyncEngine, SyncState},
    transport::{SyncTransport, TransportError},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::marker::PhantomData;
use tokio::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Sync error: {0}")]
    Sync(#[from] crate::sync::SyncEngineError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Local-first collection that can synchronize with remote peers
pub struct LocalFirstCollection<T, Tr>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + Mergeable + Default,
    Tr: SyncTransport + Clone + 'static,
{
    storage: Storage,
    sync_engine: Arc<RwLock<SyncEngine<Tr>>>,
    auto_sync: bool,
    _phantom: PhantomData<T>,
}

/// Builder for LocalFirstCollection
pub struct CollectionBuilder<Tr>
where
    Tr: SyncTransport + Clone + 'static,
{
    storage: Storage,
    transport: Tr,
    auto_sync: bool,
    replica_id: Option<ReplicaId>,
}

impl<Tr> CollectionBuilder<Tr>
where
    Tr: SyncTransport + Clone + 'static,
{
    pub fn new(storage: Storage, transport: Tr) -> Self {
        Self {
            storage,
            transport,
            auto_sync: false,
            replica_id: None,
        }
    }

    pub fn with_auto_sync(mut self, enabled: bool) -> Self {
        self.auto_sync = enabled;
        self
    }

    pub fn with_replica_id(mut self, replica_id: ReplicaId) -> Self {
        self.replica_id = Some(replica_id);
        self
    }

    pub fn build<T>(self) -> LocalFirstCollection<T, Tr>
    where
        T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + Mergeable + Default,
    {
        let sync_engine = if let Some(replica_id) = self.replica_id {
            SyncEngine::with_replica_id(self.storage.clone(), self.transport.clone(), replica_id)
        } else {
            SyncEngine::new(self.storage.clone(), self.transport.clone())
        };

        LocalFirstCollection::<T, Tr> {
            storage: self.storage,
            sync_engine: Arc::new(RwLock::new(sync_engine)),
            auto_sync: self.auto_sync,
            _phantom: PhantomData,
        }
    }
}

impl<T, Tr> LocalFirstCollection<T, Tr>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + Mergeable + Default,
    Tr: SyncTransport + Clone + 'static,
{
    /// Create a new collection
    pub fn new(storage: Storage, transport: Tr) -> Self {
        let sync_engine = SyncEngine::new(storage.clone(), transport);
        
        Self {
            storage,
            sync_engine: Arc::new(RwLock::new(sync_engine)),
            auto_sync: false,
            _phantom: PhantomData,
        }
    }

    /// Create a collection with a specific replica ID
    pub fn with_replica_id(storage: Storage, transport: Tr, replica_id: ReplicaId) -> Self {
        let sync_engine = SyncEngine::with_replica_id(storage.clone(), transport, replica_id);
        
        Self {
            storage,
            sync_engine: Arc::new(RwLock::new(sync_engine)),
            auto_sync: false,
            _phantom: PhantomData,
        }
    }

    /// Get the replica ID for this collection
    pub fn replica_id(&self) -> ReplicaId {
        // This is a simplified version - in a real implementation you'd get it from the sync engine
        ReplicaId::default()
    }

    /// Insert or update an item
    pub async fn insert(&self, key: &str, value: &T) -> Result<(), CollectionError> {
        // Store locally first
        self.storage.set(key, value).await?;

        // Sync if auto-sync is enabled
        if self.auto_sync {
            let mut engine = self.sync_engine.write().await;
            engine.sync(key, value).await?;
        }

        Ok(())
    }

    /// Get an item by key
    pub async fn get(&self, key: &str) -> Result<Option<T>, CollectionError> {
        self.storage.get(key).await.map_err(Into::into)
    }

    /// Remove an item
    pub async fn remove(&self, key: &str) -> Result<(), CollectionError> {
        self.storage.remove(key).await.map_err(Into::into)
    }

    /// Get all keys
    pub async fn keys(&self) -> Result<Vec<String>, CollectionError> {
        self.storage.keys().await.map_err(Into::into)
    }

    /// Get all values
    pub async fn values(&self) -> Result<Vec<T>, CollectionError> {
        let keys = self.storage.keys().await.map_err(|e| CollectionError::Storage(e))?;
        let mut values = Vec::new();
        
        for key in keys {
            if let Some(value) = self.get(&key).await? {
                values.push(value);
            }
        }
        
        Ok(values)
    }

    /// Check if a key exists
    pub async fn contains_key(&self, key: &str) -> Result<bool, CollectionError> {
        self.storage.contains_key(key).await.map_err(Into::into)
    }

    /// Get the number of items
    pub async fn len(&self) -> Result<usize, CollectionError> {
        self.storage.len().await.map_err(Into::into)
    }

    /// Check if the collection is empty
    pub async fn is_empty(&self) -> Result<bool, CollectionError> {
        self.storage.is_empty().await.map_err(Into::into)
    }

    /// Start synchronization
    pub async fn start_sync(&self) -> Result<(), CollectionError> {
        let mut engine = self.sync_engine.write().await;
        engine.start_sync().await.map_err(Into::into)
    }

    /// Stop synchronization
    pub async fn stop_sync(&self) -> Result<(), CollectionError> {
        let mut engine = self.sync_engine.write().await;
        engine.stop_sync().await.map_err(Into::into)
    }

    /// Get synchronization state
    pub async fn sync_state(&self) -> Result<SyncState, CollectionError> {
        let engine = self.sync_engine.read().await;
        Ok(engine.state().await)
    }

    /// Check if online
    pub async fn is_online(&self) -> Result<bool, CollectionError> {
        let engine = self.sync_engine.read().await;
        Ok(engine.is_online().await)
    }

    /// Get peer count
    pub async fn peer_count(&self) -> Result<usize, CollectionError> {
        let engine = self.sync_engine.read().await;
        Ok(engine.peer_count().await)
    }

    /// Set auto-sync mode
    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.auto_sync = enabled;
    }

    /// Force synchronization
    pub async fn force_sync(&self) -> Result<(), CollectionError> {
        let mut engine = self.sync_engine.write().await;
        
        // Process any pending messages
        engine.process_messages().await.map_err(|e| CollectionError::Sync(e))?;
        
        Ok(())
    }

    /// Get all peers
    pub async fn peers(&self) -> Result<impl Iterator<Item = (ReplicaId, crate::sync::PeerInfo)>, CollectionError> {
        let engine = self.sync_engine.read().await;
        Ok(engine.peers().await)
    }

    /// Get sync information
    pub async fn sync_info(&self) -> Result<SyncInfo, CollectionError> {
        let engine = self.sync_engine.read().await;
        
        Ok(SyncInfo {
            sync_state: engine.state().await,
            peer_count: engine.peer_count().await,
            is_online: engine.is_online().await,
        })
    }
}

/// Synchronization information
#[derive(Debug, Clone)]
pub struct SyncInfo {
    pub sync_state: SyncState,
    pub peer_count: usize,
    pub is_online: bool,
}

/// Iterator over collection items
pub struct CollectionIterator<T, Tr>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + Mergeable + Default,
    Tr: SyncTransport + Clone + 'static,
{
    collection: Arc<LocalFirstCollection<T, Tr>>,
    keys: Vec<String>,
    current_index: usize,
    _phantom: PhantomData<T>,
}

impl<T, Tr> CollectionIterator<T, Tr>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + Mergeable + Default,
    Tr: SyncTransport + Clone + 'static,
{
    pub fn new(collection: Arc<LocalFirstCollection<T, Tr>>) -> Self {
        Self {
            collection,
            keys: Vec::new(),
            current_index: 0,
            _phantom: PhantomData,
        }
    }

    pub async fn load_keys(&mut self) -> Result<(), CollectionError> {
        self.keys = self.collection.storage.keys().await.map_err(|e| CollectionError::Storage(e))?;
        Ok(())
    }
}

impl<T, Tr> Iterator for CollectionIterator<T, Tr>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + Mergeable + Default,
    Tr: SyncTransport + Clone + 'static,
{
    type Item = (String, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.keys.len() {
            return None;
        }

        let key = self.keys[self.current_index].clone();
        self.current_index += 1;

        // Note: This is a simplified implementation
        // In a real implementation, you'd want to handle the async nature properly
        Some((key, T::default())) // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::transport::InMemoryTransport;
    use crate::crdt::{LwwRegister, ReplicaId};

    #[tokio::test]
    async fn test_collection_basic_operations() {
        let storage = Storage::memory();
        let transport = InMemoryTransport::new();
        let collection = LocalFirstCollection::<LwwRegister<String>, _>::new(storage, transport);

        // Test insert and get
        let value1 = LwwRegister::new("value1".to_string(), ReplicaId::default());
        assert!(collection.insert("key1", &value1).await.is_ok());
        let value = collection.get("key1").await.unwrap();
        assert_eq!(value, Some(value1));

        // Test remove
        assert!(collection.remove("key1").await.is_ok());
        let value = collection.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_collection_builder() {
        let storage = Storage::memory();
        let transport = InMemoryTransport::new();
        
        let collection = CollectionBuilder::new(storage, transport)
            .with_auto_sync(true)
            .build::<LwwRegister<String>>();

        assert!(collection.auto_sync);
    }
}
