//! In-memory storage implementation

use super::{LocalStorage, StorageError};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory storage using HashMap
pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MemoryStorage {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

#[async_trait]
impl LocalStorage for MemoryStorage {
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<(), StorageError> {
        let serialized = serde_json::to_vec(value)
            .map_err(|e| StorageError::Serialization(e))?;
        
        let mut data = self.data.write().await;
        data.insert(key.to_string(), serialized);
        Ok(())
    }

    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> Result<Option<T>, StorageError> {
        let data = self.data.read().await;
        
        match data.get(key) {
            Some(bytes) => {
                let value = serde_json::from_slice(bytes)
                    .map_err(|e| StorageError::Serialization(e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn remove(&self, key: &str) -> Result<(), StorageError> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>, StorageError> {
        let data = self.data.read().await;
        Ok(data.keys().cloned().collect())
    }

    async fn contains_key(&self, key: &str) -> Result<bool, StorageError> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn len(&self) -> Result<usize, StorageError> {
        let data = self.data.read().await;
        Ok(data.len())
    }

    async fn is_empty(&self) -> Result<bool, StorageError> {
        let data = self.data.read().await;
        Ok(data.is_empty())
    }

    async fn clear(&self) -> Result<(), StorageError> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_basic_operations() {
        let storage = MemoryStorage::new();

        // Test set and get
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());
        let value = storage.get::<String>("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Test get non-existent key
        let value = storage.get::<String>("nonexistent").await.unwrap();
        assert_eq!(value, None);

        // Test remove
        assert!(storage.remove("key1").await.is_ok());
        let value = storage.get::<String>("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_storage_keys_and_length() {
        let storage = MemoryStorage::new();

        // Initially empty
        assert_eq!(storage.len().await.unwrap(), 0);
        assert!(storage.is_empty().await.unwrap());

        // Add some data
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());
        assert!(storage.set("key2", &"value2".to_string()).await.is_ok());

        // Check length and keys
        assert_eq!(storage.len().await.unwrap(), 2);
        assert!(!storage.is_empty().await.unwrap());

        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage = MemoryStorage::new();

        // Add some data
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());
        assert!(storage.set("key2", &"value2".to_string()).await.is_ok());

        // Clear all data
        assert!(storage.clear().await.is_ok());

        // Check that data is cleared
        assert_eq!(storage.len().await.unwrap(), 0);
        assert!(storage.is_empty().await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_contains_key() {
        let storage = MemoryStorage::new();

        // Initially no keys
        assert!(!storage.contains_key("key1").await.unwrap());

        // Add a key
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());
        assert!(storage.contains_key("key1").await.unwrap());

        // Remove the key
        assert!(storage.remove("key1").await.is_ok());
        assert!(!storage.contains_key("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_clone() {
        let storage = MemoryStorage::new();
        let cloned_storage = storage.clone();

        // Add data to original
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());

        // Check that cloned storage has the same data
        let value = cloned_storage.get::<String>("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }
}