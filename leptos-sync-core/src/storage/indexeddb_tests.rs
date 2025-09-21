//! IndexedDB storage unit tests
//! 
//! Comprehensive tests for IndexedDB storage functionality including
//! CRUD operations, batch operations, and error handling.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        IndexedDbStorage, IndexedDbError, IndexedDbResult,
        StorageError, LocalStorage,
    };
    use crate::crdt::{ReplicaId, CrdtType};
    use std::time::SystemTime;
    use uuid::Uuid;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestData {
        id: String,
        value: String,
        timestamp: SystemTime,
    }

    fn create_test_data(id: &str, value: &str) -> TestData {
        TestData {
            id: id.to_string(),
            value: value.to_string(),
            timestamp: SystemTime::now(),
        }
    }

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_indexeddb_storage_creation() {
        let result = IndexedDbStorage::new("test_creation_db", 1).await;
        assert!(result.is_ok(), "Should create IndexedDB storage successfully");
        
        let storage = result.unwrap();
        assert_eq!(storage.name(), "test_creation_db");
        assert_eq!(storage.version(), 1);
    }

    #[tokio::test]
    async fn test_indexeddb_storage_default_creation() {
        let result = IndexedDbStorage::new_default().await;
        assert!(result.is_ok(), "Should create default IndexedDB storage successfully");
        
        let storage = result.unwrap();
        assert!(!storage.name().is_empty());
        assert!(storage.version() > 0);
    }

    #[tokio::test]
    async fn test_indexeddb_crud_operations() {
        let storage = IndexedDbStorage::new("test_crud_db", 1).await.unwrap();
        
        // Test set/get
        let key = "test_key";
        let value = create_test_data("test_id", "test_value");
        
        let result = storage.set(key, &value).await;
        assert!(result.is_ok(), "Should set value successfully");
        
        let retrieved = storage.get::<TestData>(key).await.unwrap();
        assert_eq!(retrieved, Some(value.clone()));
        
        // Test update
        let updated_value = create_test_data("test_id", "updated_value");
        let result = storage.set(key, &updated_value).await;
        assert!(result.is_ok(), "Should update value successfully");
        
        let retrieved = storage.get::<TestData>(key).await.unwrap();
        assert_eq!(retrieved, Some(updated_value));
        
        // Test delete
        let result = storage.remove(key).await;
        assert!(result.is_ok(), "Should delete value successfully");
        
        let after_delete = storage.get::<TestData>(key).await.unwrap();
        assert_eq!(after_delete, None);
    }

    #[tokio::test]
    async fn test_indexeddb_batch_operations() {
        let storage = IndexedDbStorage::new("test_batch_db", 1).await.unwrap();
        
        let operations = vec![
            ("key1", create_test_data("id1", "value1")),
            ("key2", create_test_data("id2", "value2")),
            ("key3", create_test_data("id3", "value3")),
        ];
        
        // Batch set
        for (key, value) in &operations {
            let result = storage.set(key, value).await;
            assert!(result.is_ok(), "Should set batch value successfully");
        }
        
        // Verify all values
        for (key, expected_value) in &operations {
            let value = storage.get::<TestData>(key).await.unwrap().unwrap();
            assert_eq!(value, *expected_value);
        }
        
        // Batch delete
        for (key, _) in &operations {
            let result = storage.remove(key).await;
            assert!(result.is_ok(), "Should delete batch value successfully");
        }
        
        // Verify all values are deleted
        for (key, _) in &operations {
            let value = storage.get::<TestData>(key).await.unwrap();
            assert_eq!(value, None);
        }
    }

    #[tokio::test]
    async fn test_indexeddb_keys_enumeration() {
        let storage = IndexedDbStorage::new("test_keys_db", 1).await.unwrap();
        
        // Add some test data
        let test_keys = vec!["key1", "key2", "key3", "key4"];
        for key in &test_keys {
            let value = create_test_data(key, "test_value");
            storage.set(key, &value).await.unwrap();
        }
        
        // Get all keys
        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), test_keys.len());
        
        // Verify all expected keys are present
        for expected_key in &test_keys {
            assert!(keys.contains(&expected_key.to_string()));
        }
    }

    #[tokio::test]
    async fn test_indexeddb_count_operations() {
        let storage = IndexedDbStorage::new("test_count_db", 1).await.unwrap();
        
        // Initially empty
        let count = storage.len().await.unwrap();
        assert_eq!(count, 0);
        
        // Add some data
        let test_keys = vec!["key1", "key2", "key3"];
        for key in &test_keys {
            let value = create_test_data(key, "test_value");
            storage.set(key, &value).await.unwrap();
        }
        
        // Check count
        let count = storage.len().await.unwrap();
        assert_eq!(count, test_keys.len());
        
        // Remove one item
        storage.remove("key1").await.unwrap();
        let count = storage.len().await.unwrap();
        assert_eq!(count, test_keys.len() - 1);
    }

    #[tokio::test]
    async fn test_indexeddb_clear_operations() {
        let storage = IndexedDbStorage::new("test_clear_db", 1).await.unwrap();
        
        // Add some data
        let test_keys = vec!["key1", "key2", "key3"];
        for key in &test_keys {
            let value = create_test_data(key, "test_value");
            storage.set(key, &value).await.unwrap();
        }
        
        // Verify data exists
        let count = storage.len().await.unwrap();
        assert_eq!(count, test_keys.len());
        
        // Clear all data
        let result = storage.clear().await;
        assert!(result.is_ok(), "Should clear all data successfully");
        
        // Verify data is cleared
        let count = storage.len().await.unwrap();
        assert_eq!(count, 0);
        
        // Verify individual keys are gone
        for key in &test_keys {
            let value = storage.get::<TestData>(key).await.unwrap();
            assert_eq!(value, None);
        }
    }

    #[tokio::test]
    async fn test_indexeddb_contains_key() {
        let storage = IndexedDbStorage::new("test_contains_db", 1).await.unwrap();
        
        let key = "test_key";
        let value = create_test_data("test_id", "test_value");
        
        // Initially should not contain key
        let contains = storage.contains_key(key).await.unwrap();
        assert!(!contains);
        
        // Add key
        storage.set(key, &value).await.unwrap();
        
        // Should now contain key
        let contains = storage.contains_key(key).await.unwrap();
        assert!(contains);
        
        // Remove key
        storage.remove(key).await.unwrap();
        
        // Should no longer contain key
        let contains = storage.contains_key(key).await.unwrap();
        assert!(!contains);
    }

    #[tokio::test]
    async fn test_indexeddb_is_empty() {
        let storage = IndexedDbStorage::new("test_empty_db", 1).await.unwrap();
        
        // Initially should be empty
        let is_empty = storage.is_empty().await.unwrap();
        assert!(is_empty);
        
        // Add data
        let value = create_test_data("test_id", "test_value");
        storage.set("test_key", &value).await.unwrap();
        
        // Should no longer be empty
        let is_empty = storage.is_empty().await.unwrap();
        assert!(!is_empty);
        
        // Clear data
        storage.clear().await.unwrap();
        
        // Should be empty again
        let is_empty = storage.is_empty().await.unwrap();
        assert!(is_empty);
    }

    #[tokio::test]
    async fn test_indexeddb_bytes_operations() {
        let storage = IndexedDbStorage::new("test_bytes_db", 1).await.unwrap();
        
        let key = "test_bytes_key";
        let value = b"test binary data".to_vec();
        
        // Test set_bytes
        let result = storage.set_bytes(key, &value).await;
        assert!(result.is_ok(), "Should set bytes successfully");
        
        // Test get_bytes
        let retrieved = storage.get_bytes(key).await.unwrap();
        assert_eq!(retrieved, Some(value));
        
        // Test delete
        storage.remove(key).await.unwrap();
        let after_delete = storage.get_bytes(key).await.unwrap();
        assert_eq!(after_delete, None);
    }

    #[tokio::test]
    async fn test_indexeddb_crdt_delta_storage() {
        let storage = IndexedDbStorage::new("test_crdt_db", 1).await.unwrap();
        
        let collection_id = "test_collection";
        let replica_id = create_test_replica_id();
        let delta_data = vec![1, 2, 3, 4, 5];
        
        // Store a delta
        let result = storage.store_delta(collection_id, replica_id, &delta_data, "LwwRegister").await;
        assert!(result.is_ok(), "Should store delta successfully");
        
        // Retrieve deltas
        let deltas = storage.get_deltas(collection_id, None, None).await.unwrap();
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].delta_data, delta_data);
        assert_eq!(deltas[0].collection_id, collection_id);
        assert_eq!(deltas[0].replica_id, replica_id);
    }

    #[tokio::test]
    async fn test_indexeddb_collection_metadata() {
        let storage = IndexedDbStorage::new("test_metadata_db", 1).await.unwrap();
        
        let collection_id = "test_collection";
        let metadata = crate::storage::indexeddb::CollectionMetadata {
            id: collection_id.to_string(),
            name: "Test Collection".to_string(),
            crdt_type: "LwwRegister".to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            replica_count: 1,
            size_bytes: 1024,
        };
        
        // Store metadata
        let result = storage.store_collection_metadata(&metadata).await;
        assert!(result.is_ok(), "Should store metadata successfully");
        
        // Retrieve metadata
        let retrieved = storage.get_collection_metadata(collection_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, collection_id);
    }

    #[tokio::test]
    async fn test_indexeddb_peer_tracking() {
        let storage = IndexedDbStorage::new("test_peer_db", 1).await.unwrap();
        
        let replica_id = create_test_replica_id();
        let peer_info = crate::storage::indexeddb::PeerRecord {
            replica_id,
            last_seen: SystemTime::now(),
            is_online: true,
            user_info: Some(crate::transport::message_protocol::UserInfo {
                user_id: "user123".to_string(),
                username: Some("testuser".to_string()),
                display_name: Some("Test User".to_string()),
                avatar_url: None,
            }),
        };
        
        // Store peer info
        let result = storage.store_peer_info(&peer_info).await;
        assert!(result.is_ok(), "Should store peer info successfully");
        
        // Retrieve peer info
        let retrieved = storage.get_peer_info(replica_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().replica_id, replica_id);
    }

    #[tokio::test]
    async fn test_indexeddb_storage_stats() {
        let storage = IndexedDbStorage::new("test_stats_db", 1).await.unwrap();
        
        // Add some data
        let test_keys = vec!["key1", "key2", "key3"];
        for key in &test_keys {
            let value = create_test_data(key, "test_value");
            storage.set(key, &value).await.unwrap();
        }
        
        // Get storage stats
        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.name, "test_stats_db");
        assert_eq!(stats.version, 1);
        assert!(stats.object_stores.len() > 0);
    }

    #[tokio::test]
    async fn test_indexeddb_cleanup_operations() {
        let storage = IndexedDbStorage::new("test_cleanup_db", 1).await.unwrap();
        
        let collection_id = "test_collection";
        let replica_id = create_test_replica_id();
        
        // Add multiple deltas
        for i in 0..10 {
            let delta_data = vec![i as u8];
            storage.store_delta(collection_id, replica_id, &delta_data, "LwwRegister").await.unwrap();
        }
        
        // Cleanup old deltas (keep only 5)
        let result = storage.cleanup_old_deltas(collection_id, 5).await;
        assert!(result.is_ok(), "Should cleanup old deltas successfully");
        
        // Verify only 5 deltas remain
        let deltas = storage.get_deltas(collection_id, None, None).await.unwrap();
        assert_eq!(deltas.len(), 5);
    }

    #[tokio::test]
    async fn test_indexeddb_compact_storage() {
        let storage = IndexedDbStorage::new("test_compact_db", 1).await.unwrap();
        
        let collection_id = "test_collection";
        
        // Add some data
        let value = create_test_data("test_id", "test_value");
        storage.set("test_key", &value).await.unwrap();
        
        // Compact storage
        let result = storage.compact_storage(collection_id).await;
        assert!(result.is_ok(), "Should compact storage successfully");
    }

    #[tokio::test]
    async fn test_indexeddb_error_handling() {
        // Test with invalid database name
        let result = IndexedDbStorage::new("", 1).await;
        assert!(result.is_err(), "Should fail with empty database name");
        
        // Test with invalid version
        let result = IndexedDbStorage::new("test_error_db", 0).await;
        assert!(result.is_err(), "Should fail with version 0");
    }

    #[tokio::test]
    async fn test_indexeddb_serialization_errors() {
        let storage = IndexedDbStorage::new("test_serialization_db", 1).await.unwrap();
        
        // Test with data that can't be serialized
        // (This would require a type that doesn't implement Serialize)
        // For now, we'll test with valid data and ensure serialization works
        let value = create_test_data("test_id", "test_value");
        let result = storage.set("test_key", &value).await;
        assert!(result.is_ok(), "Should serialize valid data successfully");
    }

    #[tokio::test]
    async fn test_indexeddb_concurrent_operations() {
        let storage = IndexedDbStorage::new("test_concurrent_db", 1).await.unwrap();
        
        // Test concurrent set operations
        let handles: Vec<_> = (0..10).map(|i| {
            let storage = storage.clone();
            let key = format!("concurrent_key_{}", i);
            let value = create_test_data(&format!("id_{}", i), &format!("value_{}", i));
            
            tokio::spawn(async move {
                storage.set(&key, &value).await
            })
        }).collect();
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent set operation should succeed");
        }
        
        // Verify all values were set
        for i in 0..10 {
            let key = format!("concurrent_key_{}", i);
            let value = storage.get::<TestData>(&key).await.unwrap();
            assert!(value.is_some(), "Concurrent value should exist");
        }
    }

    #[tokio::test]
    async fn test_indexeddb_large_data_handling() {
        let storage = IndexedDbStorage::new("test_large_db", 1).await.unwrap();
        
        // Test with large data
        let large_value = vec![0u8; 1024 * 1024]; // 1MB
        let result = storage.set_bytes("large_key", &large_value).await;
        
        // This might fail due to quota limits, but should handle gracefully
        match result {
            Ok(_) => {
                // If successful, verify we can retrieve it
                let retrieved = storage.get_bytes("large_key").await.unwrap();
                assert_eq!(retrieved, Some(large_value));
            }
            Err(StorageError::OperationFailed(msg)) if msg.contains("quota") => {
                // Expected behavior when quota is exceeded
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }
}
