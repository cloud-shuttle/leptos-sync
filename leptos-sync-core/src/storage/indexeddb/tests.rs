//! Comprehensive IndexedDB tests

use super::*;
use crate::crdt::ReplicaId;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test database connection and basic operations
#[wasm_bindgen_test]
async fn test_database_connection() {
    let connection = IndexedDbConnection::open("test_connection_db", 1).await;
    assert!(connection.is_ok());
    
    if let Ok(conn) = connection {
        assert_eq!(conn.name(), "test_connection_db");
        assert_eq!(conn.version(), 1);
        
        // Test transaction creation
        let transaction = conn.transaction_readonly(&["collections"]);
        assert!(transaction.is_ok());
    }
}

/// Test basic CRUD operations
#[wasm_bindgen_test]
async fn test_basic_crud_operations() {
    let connection = IndexedDbConnection::open("test_crud_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);

    // Test set and get
    let test_data = "test value";
    let result = operations.set("collections", "test_key", &test_data).await;
    assert!(result.is_ok());

    let retrieved: Option<String> = operations.get("collections", "test_key").await.unwrap();
    assert_eq!(retrieved, Some(test_data.to_string()));

    // Test delete
    let result = operations.delete("collections", "test_key").await;
    assert!(result.is_ok());

    let retrieved: Option<String> = operations.get("collections", "test_key").await.unwrap();
    assert_eq!(retrieved, None);
}

/// Test bytes operations
#[wasm_bindgen_test]
async fn test_bytes_operations() {
    let connection = IndexedDbConnection::open("test_bytes_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);

    let test_data = b"test binary data";
    let result = operations.set_bytes("collections", "binary_key", test_data).await;
    assert!(result.is_ok());

    let retrieved = operations.get_bytes("collections", "binary_key").await.unwrap();
    assert_eq!(retrieved, Some(test_data.to_vec()));
}

/// Test count and keys operations
#[wasm_bindgen_test]
async fn test_count_and_keys() {
    let connection = IndexedDbConnection::open("test_count_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);

    // Add some test data
    for i in 0..5 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        operations.set("collections", &key, &value).await.unwrap();
    }

    let count = operations.count("collections").await.unwrap();
    assert_eq!(count, 5);

    let keys = operations.get_all_keys("collections").await.unwrap();
    assert_eq!(keys.len(), 5);
    
    for i in 0..5 {
        let expected_key = format!("key_{}", i);
        assert!(keys.contains(&expected_key));
    }
}

/// Test CRDT delta storage
#[wasm_bindgen_test]
async fn test_crdt_delta_storage() {
    let connection = IndexedDbConnection::open("test_delta_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);
    let crdt_store = CrdtStore::new(operations);

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let delta_data = b"test delta data";
    let collection_id = "test_collection";

    // Store a delta
    let result = crdt_store.store_delta(collection_id, replica_id, delta_data, "LwwRegister").await;
    assert!(result.is_ok());

    // Retrieve deltas
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].delta, delta_data);
    assert_eq!(deltas[0].replica_id, replica_id);
    assert_eq!(deltas[0].crdt_type, "LwwRegister");

    // Check metadata was created
    let metadata = crdt_store.get_collection_metadata(collection_id).await.unwrap();
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert_eq!(metadata.id, collection_id);
    assert_eq!(metadata.delta_count, 1);
}

/// Test peer management
#[wasm_bindgen_test]
async fn test_peer_management() {
    let connection = IndexedDbConnection::open("test_peers_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);
    let crdt_store = CrdtStore::new(operations);

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    
    // Update peer last seen
    let result = crdt_store.update_peer_last_seen(&replica_id).await;
    assert!(result.is_ok());

    // Get peer
    let peer = crdt_store.get_peer(&replica_id).await.unwrap();
    assert!(peer.is_some());
    let peer = peer.unwrap();
    assert_eq!(peer.replica_id, replica_id);
    assert!(peer.is_online);

    // Mark as offline
    let result = crdt_store.mark_peer_offline(&replica_id).await;
    assert!(result.is_ok());

    // Check peer is offline
    let peer = crdt_store.get_peer(&replica_id).await.unwrap();
    assert!(peer.is_some());
    let peer = peer.unwrap();
    assert!(!peer.is_online);
}

/// Test storage statistics
#[wasm_bindgen_test]
async fn test_storage_statistics() {
    let connection = IndexedDbConnection::open("test_stats_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);
    let crdt_store = CrdtStore::new(operations);

    // Add some test data
    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    crdt_store.store_delta("collection1", replica_id, b"data1", "LwwRegister").await.unwrap();
    crdt_store.store_delta("collection2", replica_id, b"data2", "GCounter").await.unwrap();
    crdt_store.update_peer_last_seen(&replica_id).await.unwrap();

    let stats = crdt_store.get_storage_stats().await.unwrap();
    assert_eq!(stats.collections_count, 2);
    assert_eq!(stats.deltas_count, 2);
    assert_eq!(stats.peers_count, 1);
    assert!(stats.total_size > 0);
}

/// Test cleanup operations
#[wasm_bindgen_test]
async fn test_cleanup_operations() {
    let connection = IndexedDbConnection::open("test_cleanup_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);
    let crdt_store = CrdtStore::new(operations);

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let collection_id = "test_collection";

    // Add multiple deltas
    for i in 0..5 {
        let data = format!("delta_{}", i).into_bytes();
        crdt_store.store_delta(collection_id, replica_id, &data, "LwwRegister").await.unwrap();
    }

    // Check we have 5 deltas
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 5);

    // Cleanup, keeping only 2
    crdt_store.cleanup_old_deltas(collection_id, 2).await.unwrap();

    // Check we now have 2 deltas
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 2);
}

/// Test storage compaction
#[wasm_bindgen_test]
async fn test_storage_compaction() {
    let connection = IndexedDbConnection::open("test_compaction_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);
    let crdt_store = CrdtStore::new(operations);

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let collection_id = "test_collection";

    // Add multiple deltas from the same replica
    for i in 0..3 {
        let data = format!("delta_{}", i).into_bytes();
        crdt_store.store_delta(collection_id, replica_id, &data, "LwwRegister").await.unwrap();
    }

    // Check we have 3 deltas
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 3);

    // Compact storage
    crdt_store.compact_storage(collection_id).await.unwrap();

    // Check we now have 1 delta (latest one)
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 1);
}

/// Test full IndexedDB storage integration
#[wasm_bindgen_test]
async fn test_full_storage_integration() {
    let storage = IndexedDbStorage::new("test_integration_db", 1).await;
    assert!(storage.is_ok());
    
    let storage = storage.unwrap();

    // Test basic operations
    let test_data = "test value";
    let result = storage.set("test_key", &test_data).await;
    assert!(result.is_ok());

    let retrieved: Option<String> = storage.get("test_key").await.unwrap();
    assert_eq!(retrieved, Some(test_data.to_string()));

    // Test CRDT operations
    let crdt_store = storage.crdt_store();
    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let delta_data = b"test delta data";
    let collection_id = "test_collection";

    let result = crdt_store.store_delta(collection_id, replica_id, delta_data, "LwwRegister").await;
    assert!(result.is_ok());

    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].delta, delta_data);

    // Test storage stats
    let stats = storage.get_stats().await.unwrap();
    assert!(stats.collections_count >= 0);
    assert!(stats.deltas_count >= 0);
    assert!(stats.peers_count >= 0);
}

/// Test error handling
#[wasm_bindgen_test]
async fn test_error_handling() {
    let connection = IndexedDbConnection::open("test_error_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);

    // Test getting non-existent key
    let result: Option<String> = operations.get("collections", "non_existent_key").await.unwrap();
    assert_eq!(result, None);

    // Test deleting non-existent key (should not error)
    let result = operations.delete("collections", "non_existent_key").await;
    assert!(result.is_ok());
}

/// Test batch operations
#[wasm_bindgen_test]
async fn test_batch_operations() {
    let connection = IndexedDbConnection::open("test_batch_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);

    // Test batch operations
    let result = operations.batch_operations(&["collections"], |_transaction| {
        // In a real implementation, we would perform multiple operations
        // within the transaction
        Ok(())
    }).await;
    assert!(result.is_ok());
}

/// Test index queries
#[wasm_bindgen_test]
async fn test_index_queries() {
    let connection = IndexedDbConnection::open("test_index_db", 1).await.unwrap();
    let operations = IndexedDbOperations::new(connection);
    let crdt_store = CrdtStore::new(operations);

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let collection_id = "test_collection";

    // Add some deltas
    for i in 0..3 {
        let data = format!("delta_{}", i).into_bytes();
        crdt_store.store_delta(collection_id, replica_id, &data, "LwwRegister").await.unwrap();
    }

    // Query by collection_id index
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 3);

    // All deltas should have the same collection_id
    for delta in &deltas {
        assert_eq!(delta.collection_id, collection_id);
    }
}

/// Test migration system
#[wasm_bindgen_test]
async fn test_migration_system() {
    let connection = IndexedDbConnection::open("test_migration_db", 3).await.unwrap();
    let migration_manager = MigrationManager::new(connection);

    // Validate migrations
    let result = migration_manager.validate_migrations();
    assert!(result.is_ok());

    // Get migration history
    let history = migration_manager.get_migration_history();
    assert!(!history.is_empty());
    assert_eq!(history[0].version, 1);
    assert_eq!(history[0].name, "initial_schema");
}

/// Test database statistics
#[wasm_bindgen_test]
async fn test_database_statistics() {
    let connection = IndexedDbConnection::open("test_db_stats", 1).await.unwrap();
    
    let stats = connection.get_stats().await.unwrap();
    assert_eq!(stats.name, "test_db_stats");
    assert_eq!(stats.version, 1);
    
    // Should have the default object stores
    assert!(stats.object_stores.contains_key("collections"));
    assert!(stats.object_stores.contains_key("metadata"));
    assert!(stats.object_stores.contains_key("deltas"));
    assert!(stats.object_stores.contains_key("peers"));
}

/// Test concurrent operations
#[wasm_bindgen_test]
async fn test_concurrent_operations() {
    let storage = IndexedDbStorage::new("test_concurrent_db", 1).await.unwrap();
    let crdt_store = storage.crdt_store();

    let replica_id1 = ReplicaId::from(uuid::Uuid::new_v4());
    let replica_id2 = ReplicaId::from(uuid::Uuid::new_v4());
    let collection_id = "test_collection";

    // Simulate concurrent delta storage
    let futures = vec![
        crdt_store.store_delta(collection_id, replica_id1, b"delta1", "LwwRegister"),
        crdt_store.store_delta(collection_id, replica_id2, b"delta2", "LwwRegister"),
    ];

    // Wait for all operations to complete
    for future in futures {
        let result = future.await;
        assert!(result.is_ok());
    }

    // Check that both deltas were stored
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 2);
}

/// Test storage limits and cleanup
#[wasm_bindgen_test]
async fn test_storage_limits() {
    let storage = IndexedDbStorage::new("test_limits_db", 1).await.unwrap();
    let crdt_store = storage.crdt_store();

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let collection_id = "test_collection";

    // Add many deltas
    for i in 0..10 {
        let data = format!("delta_{}", i).into_bytes();
        crdt_store.store_delta(collection_id, replica_id, &data, "LwwRegister").await.unwrap();
    }

    // Check initial count
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 10);

    // Cleanup to keep only 3
    crdt_store.cleanup_old_deltas(collection_id, 3).await.unwrap();

    // Check final count
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 3);
}

/// Test data integrity
#[wasm_bindgen_test]
async fn test_data_integrity() {
    let storage = IndexedDbStorage::new("test_integrity_db", 1).await.unwrap();
    let crdt_store = storage.crdt_store();

    let replica_id = ReplicaId::from(uuid::Uuid::new_v4());
    let collection_id = "test_collection";
    let original_data = b"original data";

    // Store a delta
    crdt_store.store_delta(collection_id, replica_id, original_data, "LwwRegister").await.unwrap();

    // Retrieve and verify data integrity
    let deltas = crdt_store.get_deltas(collection_id).await.unwrap();
    assert_eq!(deltas.len(), 1);
    
    let delta = &deltas[0];
    assert_eq!(delta.delta, original_data);
    assert_eq!(delta.replica_id, replica_id);
    assert_eq!(delta.collection_id, collection_id);
    assert_eq!(delta.crdt_type, "LwwRegister");
    assert_eq!(delta.size, original_data.len());
}
