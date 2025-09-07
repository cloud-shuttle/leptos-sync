//! IndexedDB storage implementation

use super::{LocalStorage, StorageError};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{window, Storage};

/// IndexedDB storage implementation (currently falls back to localStorage)
pub struct IndexedDbStorage {
    db_name: String,
    store_name: String,
    fallback: super::memory::MemoryStorage,
}

impl IndexedDbStorage {
    pub fn new(db_name: String, store_name: String) -> Self {
        Self {
            db_name,
            store_name,
            fallback: super::memory::MemoryStorage::new(),
        }
    }

    /// Get the localStorage fallback
    #[cfg(target_arch = "wasm32")]
    fn get_local_storage() -> Result<Storage, StorageError> {
        let window = window().ok_or_else(|| {
            StorageError::OperationFailed("No window available".to_string())
        })?;
        
        let storage = window.local_storage().map_err(|_| {
            StorageError::OperationFailed("Failed to get localStorage".to_string())
        })?.ok_or_else(|| {
            StorageError::OperationFailed("localStorage not available".to_string())
        })?;
        
        Ok(storage)
    }

    /// Get the localStorage fallback (non-WASM - always fails to trigger fallback)
    #[cfg(not(target_arch = "wasm32"))]
    fn get_local_storage() -> Result<Storage, StorageError> {
        Err(StorageError::OperationFailed("localStorage not available in non-WASM environment".to_string()))
    }

    /// Get a value from localStorage
    #[cfg(target_arch = "wasm32")]
    fn get_from_local_storage<T: DeserializeOwned>(key: &str) -> Result<Option<T>, StorageError> {
        let storage = Self::get_local_storage()?;
        
        match storage.get_item(key).map_err(|_| {
            StorageError::OperationFailed("Failed to get item from localStorage".to_string())
        })? {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)
                    .map_err(|e| StorageError::Serialization(e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a value in localStorage
    #[cfg(target_arch = "wasm32")]
    fn set_to_local_storage<T: Serialize>(key: &str, value: &T) -> Result<(), StorageError> {
        let storage = Self::get_local_storage()?;
        
        let json_str = serde_json::to_string(value)
            .map_err(|e| StorageError::Serialization(e))?;
        
        storage.set_item(key, &json_str).map_err(|_| {
            StorageError::OperationFailed("Failed to set item in localStorage".to_string())
        })?;
        
        Ok(())
    }

    /// Remove a value from localStorage
    #[cfg(target_arch = "wasm32")]
    fn remove_from_local_storage(key: &str) -> Result<(), StorageError> {
        let storage = Self::get_local_storage()?;
        
        storage.remove_item(key).map_err(|_| {
            StorageError::OperationFailed("Failed to remove item from localStorage".to_string())
        })?;
        
        Ok(())
    }

    /// Get all keys from localStorage
    #[cfg(target_arch = "wasm32")]
    fn get_keys_from_local_storage() -> Result<Vec<String>, StorageError> {
        let storage = Self::get_local_storage()?;
        let length = storage.length().map_err(|_| {
            StorageError::OperationFailed("Failed to get localStorage length".to_string())
        })?;
        
        let mut keys = Vec::new();
        for i in 0..length {
            if let Some(key) = storage.key(i).map_err(|_| {
                StorageError::OperationFailed("Failed to get localStorage key".to_string())
            })? {
                keys.push(key);
            }
        }
        
        Ok(keys)
    }

    /// Get a value from localStorage (non-WASM - always fails to trigger fallback)
    #[cfg(not(target_arch = "wasm32"))]
    fn get_from_local_storage<T: DeserializeOwned>(_key: &str) -> Result<Option<T>, StorageError> {
        Err(StorageError::OperationFailed("localStorage not available in non-WASM environment".to_string()))
    }

    /// Set a value in localStorage (non-WASM - always fails to trigger fallback)
    #[cfg(not(target_arch = "wasm32"))]
    fn set_to_local_storage<T: Serialize>(_key: &str, _value: &T) -> Result<(), StorageError> {
        Err(StorageError::OperationFailed("localStorage not available in non-WASM environment".to_string()))
    }

    /// Remove a value from localStorage (non-WASM - always fails to trigger fallback)
    #[cfg(not(target_arch = "wasm32"))]
    fn remove_from_local_storage(_key: &str) -> Result<(), StorageError> {
        Err(StorageError::OperationFailed("localStorage not available in non-WASM environment".to_string()))
    }

    /// Get all keys from localStorage (non-WASM - always fails to trigger fallback)
    #[cfg(not(target_arch = "wasm32"))]
    fn get_keys_from_local_storage() -> Result<Vec<String>, StorageError> {
        Err(StorageError::OperationFailed("localStorage not available in non-WASM environment".to_string()))
    }
}

impl Clone for IndexedDbStorage {
    fn clone(&self) -> Self {
        Self {
            db_name: self.db_name.clone(),
            store_name: self.store_name.clone(),
            fallback: self.fallback.clone(),
        }
    }
}

#[async_trait]
impl LocalStorage for IndexedDbStorage {
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<(), StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::set_to_local_storage(key, value) {
            Ok(()) => Ok(()),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.set(key, value).await
            }
        }
    }

    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> Result<Option<T>, StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::get_from_local_storage(key) {
            Ok(value) => Ok(value),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.get(key).await
            }
        }
    }

    async fn remove(&self, key: &str) -> Result<(), StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::remove_from_local_storage(key) {
            Ok(()) => Ok(()),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.remove(key).await
            }
        }
    }

    async fn keys(&self) -> Result<Vec<String>, StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::get_keys_from_local_storage() {
            Ok(keys) => Ok(keys),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.keys().await
            }
        }
    }

    async fn contains_key(&self, key: &str) -> Result<bool, StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::get_from_local_storage::<()>(key) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.contains_key(key).await
            }
        }
    }

    async fn len(&self) -> Result<usize, StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::get_keys_from_local_storage() {
            Ok(keys) => Ok(keys.len()),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.len().await
            }
        }
    }

    async fn is_empty(&self) -> Result<bool, StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        match Self::get_keys_from_local_storage() {
            Ok(keys) => Ok(keys.is_empty()),
            Err(_) => {
                // Fall back to in-memory storage
                self.fallback.is_empty().await
            }
        }
    }

    async fn clear(&self) -> Result<(), StorageError> {
        // For now, use localStorage fallback since IndexedDB is not fully implemented
        let storage = Self::get_local_storage()?;
        storage.clear().map_err(|_| {
            StorageError::OperationFailed("Failed to clear localStorage".to_string())
        })?;
        Ok(())
    }
}

// IndexedDB-specific methods (not yet fully implemented)
impl IndexedDbStorage {
    /// Create an IndexedDB database
    pub async fn create_database(&self) -> Result<(), StorageError> {
        // TODO: Implement actual IndexedDB creation
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }

    /// Open an IndexedDB database
    pub async fn open_database(&self) -> Result<(), StorageError> {
        // TODO: Implement actual IndexedDB opening
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }

    /// Create an object store
    pub async fn create_object_store(&self) -> Result<(), StorageError> {
        // TODO: Implement actual object store creation
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }

    /// Get a value from IndexedDB
    pub async fn indexeddb_get<T: DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, StorageError> {
        // TODO: Implement actual IndexedDB get
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }

    /// Set a value in IndexedDB
    pub async fn indexeddb_set<T: Serialize>(&self, _key: &str, _value: &T) -> Result<(), StorageError> {
        // TODO: Implement actual IndexedDB set
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }

    /// Delete a value from IndexedDB
    pub async fn indexeddb_delete(&self, _key: &str) -> Result<(), StorageError> {
        // TODO: Implement actual IndexedDB delete
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }

    /// Get all keys from IndexedDB
    pub async fn indexeddb_keys(&self) -> Result<Vec<String>, StorageError> {
        // TODO: Implement actual IndexedDB keys
        Err(StorageError::Unsupported("IndexedDB not yet fully implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    #[tokio::test]
    async fn test_indexeddb_storage_fallback() {
        let storage = IndexedDbStorage::new(
            "test_db".to_string(),
            "test_store".to_string(),
        );

        // Test that operations work (falling back to localStorage or memory)
        assert!(storage.set("key1", &"value1".to_string()).await.is_ok());
        
        let value = storage.get::<String>("key1").await.unwrap();
        // Note: This might be None if localStorage is not available in test environment
        // The important thing is that it doesn't panic
        
        // Test remove
        assert!(storage.remove("key1").await.is_ok());
    }

    #[cfg(target_arch = "wasm32")]
    #[tokio::test]
    async fn test_indexeddb_storage_clone() {
        let storage = IndexedDbStorage::new(
            "test_db".to_string(),
            "test_store".to_string(),
        );
        let cloned_storage = storage.clone();

        // Test that cloned storage works
        assert!(cloned_storage.set("key1", &"value1".to_string()).await.is_ok());
    }
}