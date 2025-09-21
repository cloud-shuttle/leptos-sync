//! IndexedDB CRUD operations

use super::connection::IndexedDbConnection;
use super::errors::IndexedDbError;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{IdbObjectStore, IdbRequest, IdbRequestReadyState, IdbTransaction};

/// IndexedDB operations wrapper
pub struct IndexedDbOperations {
    connection: Arc<IndexedDbConnection>,
}

impl IndexedDbOperations {
    /// Create a new operations wrapper
    pub fn new(connection: Arc<IndexedDbConnection>) -> Self {
        Self { connection }
    }

    /// Get a value from the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn get<T: DeserializeOwned>(
        &self,
        store_name: &str,
        key: &str,
    ) -> Result<Option<T>, IndexedDbError> {
        let transaction = self.connection.readonly_transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let request = store.get(&JsValue::from_str(key)).map_err(|_| {
            IndexedDbError::RequestError("Failed to create get request".to_string())
        })?;

        let result = JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute get request".to_string())
        })?;

        if result.is_undefined() {
            Ok(None)
        } else {
            let data = js_sys::Uint8Array::new(&result);
            let bytes = data.to_vec();
            let value = serde_json::from_slice(&bytes)
                .map_err(|e| IndexedDbError::SerializationError(e.to_string()))?;
            Ok(Some(value))
        }
    }

    /// Get a value from the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get<T: DeserializeOwned>(
        &self,
        _store_name: &str,
        _key: &str,
    ) -> Result<Option<T>, IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Set a value in the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn set<T: Serialize>(
        &self,
        store_name: &str,
        key: &str,
        value: &T,
    ) -> Result<(), IndexedDbError> {
        let transaction = self.connection.transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let serialized = serde_json::to_vec(value)
            .map_err(|e| IndexedDbError::SerializationError(e.to_string()))?;

        let array = js_sys::Uint8Array::from(serialized.as_slice());
        let request = store
            .put_key_val(&JsValue::from_str(key), &array.into())
            .map_err(|_| {
                IndexedDbError::RequestError("Failed to create put request".to_string())
            })?;

        JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute put request".to_string())
        })?;

        Ok(())
    }

    /// Set a value in the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn set<T: Serialize>(
        &self,
        _store_name: &str,
        _key: &str,
        _value: &T,
    ) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Delete a value from the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn delete(&self, store_name: &str, key: &str) -> Result<(), IndexedDbError> {
        let transaction = self.connection.transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let request = store.delete(&JsValue::from_str(key)).map_err(|_| {
            IndexedDbError::RequestError("Failed to create delete request".to_string())
        })?;

        JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute delete request".to_string())
        })?;

        Ok(())
    }

    /// Delete a value from the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn delete(&self, _store_name: &str, _key: &str) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Get all keys from the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn keys(&self, store_name: &str) -> Result<Vec<String>, IndexedDbError> {
        let transaction = self.connection.readonly_transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let request = store.get_all_keys().map_err(|_| {
            IndexedDbError::RequestError("Failed to create get_all_keys request".to_string())
        })?;

        let result = JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute get_all_keys request".to_string())
        })?;

        let keys_array = js_sys::Array::from(&result);
        let mut keys = Vec::new();

        for i in 0..keys_array.length() {
            if let Some(key) = keys_array.get(i).as_string() {
                keys.push(key);
            }
        }

        Ok(keys)
    }

    /// Get all keys from the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn keys(&self, _store_name: &str) -> Result<Vec<String>, IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Check if a key exists in the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn contains_key(&self, store_name: &str, key: &str) -> Result<bool, IndexedDbError> {
        let transaction = self.connection.readonly_transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let request = store.count_with_key(&JsValue::from_str(key)).map_err(|_| {
            IndexedDbError::RequestError("Failed to create count request".to_string())
        })?;

        let result = JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute count request".to_string())
        })?;

        let count = result.as_f64().unwrap_or(0.0) as u32;
        Ok(count > 0)
    }

    /// Check if a key exists in the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn contains_key(
        &self,
        _store_name: &str,
        _key: &str,
    ) -> Result<bool, IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Clear all data from the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn clear(&self, store_name: &str) -> Result<(), IndexedDbError> {
        let transaction = self.connection.transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let request = store.clear().map_err(|_| {
            IndexedDbError::RequestError("Failed to create clear request".to_string())
        })?;

        JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute clear request".to_string())
        })?;

        Ok(())
    }

    /// Clear all data from the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn clear(&self, _store_name: &str) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Get the count of records in the specified object store
    #[cfg(target_arch = "wasm32")]
    pub async fn count(&self, store_name: &str) -> Result<usize, IndexedDbError> {
        let transaction = self.connection.readonly_transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        let request = store.count().map_err(|_| {
            IndexedDbError::RequestError("Failed to create count request".to_string())
        })?;

        let result = JsFuture::from(request).await.map_err(|_| {
            IndexedDbError::RequestError("Failed to execute count request".to_string())
        })?;

        let count = result.as_f64().unwrap_or(0.0) as usize;
        Ok(count)
    }

    /// Get the count of records in the specified object store (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn count(&self, _store_name: &str) -> Result<usize, IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Batch operations for performance
    #[cfg(target_arch = "wasm32")]
    pub async fn batch_set<T: Serialize>(
        &self,
        store_name: &str,
        items: &[(String, T)],
    ) -> Result<(), IndexedDbError> {
        if items.is_empty() {
            return Ok(());
        }

        let transaction = self.connection.transaction(&[store_name])?;
        let store = transaction.object_store(store_name).map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to get object store".to_string())
        })?;

        for (key, value) in items {
            let serialized = serde_json::to_vec(value)
                .map_err(|e| IndexedDbError::SerializationError(e.to_string()))?;

            let array = js_sys::Uint8Array::from(serialized.as_slice());
            let request = store
                .put_key_val(&JsValue::from_str(key), &array.into())
                .map_err(|_| {
                    IndexedDbError::RequestError("Failed to create put request".to_string())
                })?;

            JsFuture::from(request).await.map_err(|_| {
                IndexedDbError::RequestError("Failed to execute put request".to_string())
            })?;
        }

        Ok(())
    }

    /// Batch operations for performance (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn batch_set<T: Serialize>(
        &self,
        _store_name: &str,
        _items: &[(String, T)],
    ) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }
}

impl Clone for IndexedDbOperations {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_operations_creation() {
        // Test that operations wrapper can be created
        // We can't easily test IndexedDB operations in unit tests without a real browser environment
        // This test just verifies the structure is correct
        #[cfg(not(target_arch = "wasm32"))]
        {
            // In non-WASM environment, operations should fail gracefully
            let result = IndexedDbConnection::open("test_db", 1).await;
            assert!(result.is_err());
            // Test that error handling works correctly
        }
    }
}
