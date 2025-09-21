//! Storage layer for local-first applications

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub mod indexed;
pub mod indexeddb;
pub mod memory;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Key not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Storage operation failed: {0}")]
    OperationFailed(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// Trait for local storage implementations
#[async_trait]
pub trait LocalStorage: Send + Sync {
    /// Store a value with the given key
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError>;

    /// Retrieve a value by key
    async fn get<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError>;

    /// Remove a value by key
    async fn remove(&self, key: &str) -> Result<(), StorageError>;

    /// Get all keys
    async fn keys(&self) -> Result<Vec<String>, StorageError>;

    /// Check if a key exists
    async fn contains_key(&self, key: &str) -> Result<bool, StorageError> {
        let keys = self.keys().await?;
        Ok(keys.contains(&key.to_string()))
    }

    /// Get the number of stored items
    async fn len(&self) -> Result<usize, StorageError> {
        let keys = self.keys().await?;
        Ok(keys.len())
    }

    /// Check if storage is empty
    async fn is_empty(&self) -> Result<bool, StorageError> {
        self.len().await.map(|l| l == 0)
    }

    /// Clear all stored data
    async fn clear(&self) -> Result<(), StorageError> {
        let keys = self.keys().await?;
        for key in keys {
            self.remove(&key).await?;
        }
        Ok(())
    }
}

/// Storage enum that can hold different storage backends
#[derive(Clone)]
pub enum Storage {
    Memory(memory::MemoryStorage),
    IndexedDb(indexeddb::IndexedDbStorage),
}

impl Storage {
    pub fn memory() -> Self {
        Self::Memory(memory::MemoryStorage::new())
    }

    pub async fn indexeddb(db_name: &str, version: u32) -> Result<Self, StorageError> {
        let storage = indexeddb::IndexedDbStorage::new(db_name.to_string(), "default".to_string());
        Ok(Self::IndexedDb(storage))
    }

    pub async fn indexeddb_default() -> Result<Self, StorageError> {
        let storage =
            indexeddb::IndexedDbStorage::new("default_db".to_string(), "default".to_string());
        Ok(Self::IndexedDb(storage))
    }
}

#[async_trait]
impl LocalStorage for Storage {
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError> {
        match self {
            Storage::Memory(storage) => storage.set(key, value).await,
            Storage::IndexedDb(storage) => storage.set(key, value).await,
        }
    }

    async fn get<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError> {
        match self {
            Storage::Memory(storage) => storage.get(key).await,
            Storage::IndexedDb(storage) => storage.get(key).await,
        }
    }

    async fn remove(&self, key: &str) -> Result<(), StorageError> {
        match self {
            Storage::Memory(storage) => storage.remove(key).await,
            Storage::IndexedDb(storage) => storage.remove(key).await,
        }
    }

    async fn keys(&self) -> Result<Vec<String>, StorageError> {
        match self {
            Storage::Memory(storage) => storage.keys().await,
            Storage::IndexedDb(storage) => storage.keys().await,
        }
    }

    async fn contains_key(&self, key: &str) -> Result<bool, StorageError> {
        match self {
            Storage::Memory(storage) => storage.contains_key(key).await,
            Storage::IndexedDb(storage) => storage.contains_key(key).await,
        }
    }

    async fn len(&self) -> Result<usize, StorageError> {
        match self {
            Storage::Memory(storage) => storage.len().await,
            Storage::IndexedDb(storage) => storage.len().await,
        }
    }

    async fn is_empty(&self) -> Result<bool, StorageError> {
        match self {
            Storage::Memory(storage) => storage.is_empty().await,
            Storage::IndexedDb(storage) => storage.is_empty().await,
        }
    }

    async fn clear(&self) -> Result<(), StorageError> {
        match self {
            Storage::Memory(storage) => storage.clear().await,
            Storage::IndexedDb(storage) => storage.clear().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_enum() {
        let storage = Storage::memory();

        // Test basic operations
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());
        let value = storage.get::<String>("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Test remove
        assert!(storage.remove("key1").await.is_ok());
        let value = storage.get::<String>("key1").await.unwrap();
        assert_eq!(value, None);
    }
}
