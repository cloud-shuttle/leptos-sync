//! WASM-specific tests for browser functionality
//! 
//! This module contains tests that specifically target WebAssembly functionality
//! and browser-specific features like IndexedDB, localStorage, and Web APIs.

#[cfg(target_arch = "wasm32")]
mod wasm_browser_tests {
    use super::*;
    use wasm_bindgen_test::*;
    use crate::storage::{indexeddb::IndexedDbStorage, memory::MemoryStorage, Storage, LocalStorage};
    use crate::crdt::{LwwRegister, ReplicaId, Mergeable};
    use crate::collection::LocalFirstCollection;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestDocument {
        id: String,
        title: String,
        content: String,
        version: u32,
    }

    impl Default for TestDocument {
        fn default() -> Self {
            Self {
                id: "default".to_string(),
                title: "Default Title".to_string(),
                content: "Default content".to_string(),
                version: 1,
            }
        }
    }

    // ============================================================================
    // IndexedDB Storage Tests
    // ============================================================================

    #[wasm_bindgen_test]
    async fn test_indexeddb_storage_basic_operations() {
        let storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());
        
        let test_data = TestDocument {
            id: "browser_test_1".to_string(),
            title: "Browser Test Document".to_string(),
            content: "This is a test document for browser testing".to_string(),
            version: 1,
        };

        // Test set operation
        storage.set("browser_key_1", &test_data).await.unwrap();
        
        // Test get operation
        let retrieved: Option<TestDocument> = storage.get("browser_key_1").await.unwrap();
        assert_eq!(retrieved, Some(test_data.clone()));

        // Test contains_key
        assert!(storage.contains_key("browser_key_1").await.unwrap());
        assert!(!storage.contains_key("nonexistent_key").await.unwrap());

        // Test keys
        let keys = storage.keys().await.unwrap();
        assert!(keys.contains(&"browser_key_1".to_string()));

        // Test len
        let len = storage.len().await.unwrap();
        assert!(len > 0);

        // Test remove
        storage.remove("browser_key_1").await.unwrap();
        let retrieved_after_remove: Option<TestDocument> = storage.get("browser_key_1").await.unwrap();
        assert_eq!(retrieved_after_remove, None);

        web_sys::console::log_1(&"IndexedDB basic operations test completed".into());
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_storage_persistence() {
        let storage1 = IndexedDbStorage::new("persistence_test".to_string(), "store1".to_string());
        let storage2 = IndexedDbStorage::new("persistence_test".to_string(), "store1".to_string());
        
        let test_data = TestDocument {
            id: "persist_test".to_string(),
            title: "Persistence Test".to_string(),
            content: "Testing data persistence across storage instances".to_string(),
            version: 42,
        };

        // Set with first storage instance
        storage1.set("persist_key", &test_data).await.unwrap();
        
        // Get with second storage instance (should work if persistent)
        let retrieved: Option<TestDocument> = storage2.get("persist_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));

        // Clean up
        storage1.remove("persist_key").await.unwrap();

        web_sys::console::log_1(&"IndexedDB persistence test completed".into());
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_storage_serialization() {
        let storage = IndexedDbStorage::new("serialization_test".to_string(), "store1".to_string());
        
        // Test complex data structures
        let complex_data = vec![
            TestDocument {
                id: "doc1".to_string(),
                title: "Document 1".to_string(),
                content: "Content 1".to_string(),
                version: 1,
            },
            TestDocument {
                id: "doc2".to_string(),
                title: "Document 2".to_string(),
                content: "Content 2".to_string(),
                version: 2,
            },
        ];

        storage.set("complex_data", &complex_data).await.unwrap();
        let retrieved: Option<Vec<TestDocument>> = storage.get("complex_data").await.unwrap();
        assert_eq!(retrieved, Some(complex_data));

        // Test with HashMap
        let mut map_data = HashMap::new();
        map_data.insert("key1".to_string(), "value1".to_string());
        map_data.insert("key2".to_string(), "value2".to_string());

        storage.set("map_data", &map_data).await.unwrap();
        let retrieved_map: Option<HashMap<String, String>> = storage.get("map_data").await.unwrap();
        assert_eq!(retrieved_map, Some(map_data));

        web_sys::console::log_1(&"IndexedDB serialization test completed".into());
    }

    // ============================================================================
    // CRDT Tests in WASM Environment
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_crdt_lww_register_wasm() {
        let replica1 = ReplicaId::default();
        let replica2 = ReplicaId::default();
        
        let mut reg1 = LwwRegister::new("initial".to_string(), replica1);
        let reg2 = LwwRegister::new("updated".to_string(), replica2);
        
        // Test merge in WASM environment
        reg1.merge(&reg2).unwrap();
        
        // Verify merge worked correctly
        assert_eq!(reg1.value(), "updated");
        
        web_sys::console::log_1(&"LwwRegister WASM test completed".into());
    }

    #[wasm_bindgen_test]
    fn test_crdt_lww_map_wasm() {
        let replica1 = ReplicaId::default();
        let replica2 = ReplicaId::default();
        
        let mut map1 = LwwMap::new(replica1);
        let mut map2 = LwwMap::new(replica2);
        
        // Set values in both maps
        map1.set("key1".to_string(), "value1".to_string());
        map2.set("key2".to_string(), "value2".to_string());
        map2.set("key1".to_string(), "value1_updated".to_string());
        
        // Merge maps
        map1.merge(&map2).unwrap();
        
        // Verify merge results
        assert_eq!(map1.get("key1"), Some("value1_updated".to_string()));
        assert_eq!(map1.get("key2"), Some("value2".to_string()));
        
        web_sys::console::log_1(&"LwwMap WASM test completed".into());
    }

    #[wasm_bindgen_test]
    fn test_crdt_gcounter_wasm() {
        let replica1 = ReplicaId::default();
        let replica2 = ReplicaId::default();
        
        let mut counter1 = GCounter::new(replica1);
        let mut counter2 = GCounter::new(replica2);
        
        // Increment counters
        counter1.increment();
        counter1.increment();
        counter2.increment();
        
        // Merge counters
        counter1.merge(&counter2).unwrap();
        
        // Verify merge results
        assert_eq!(counter1.value(), 3);
        
        web_sys::console::log_1(&"GCounter WASM test completed".into());
    }

    // ============================================================================
    // LocalFirstCollection Tests in WASM
    // ============================================================================

    #[wasm_bindgen_test]
    async fn test_local_first_collection_wasm() {
        let storage = Storage::memory();
        let collection = LocalFirstCollection::<TestDocument>::new("wasm_test_collection", Some(storage)).unwrap();
        
        // Test basic collection operations
        assert_eq!(collection.name(), "wasm_test_collection");
        
        let test_doc = TestDocument {
            id: "wasm_doc_1".to_string(),
            title: "WASM Test Document".to_string(),
            content: "Testing LocalFirstCollection in WASM".to_string(),
            version: 1,
        };
        
        // Test insert
        collection.insert("doc1", &test_doc).await.unwrap();
        
        // Test get
        let retrieved = collection.get("doc1").await.unwrap();
        assert_eq!(retrieved, Some(test_doc));
        
        // Test list
        let docs = collection.list().await.unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].0, "doc1");
        
        web_sys::console::log_1(&"LocalFirstCollection WASM test completed".into());
    }

    // ============================================================================
    // Browser API Integration Tests
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_browser_apis_availability() {
        // Test that essential browser APIs are available
        let window = web_sys::window().expect("Window should be available in browser");
        let document = window.document().expect("Document should be available");
        let navigator = window.navigator();
        
        // Test localStorage availability
        let local_storage = window.local_storage().ok().flatten();
        assert!(local_storage.is_some(), "localStorage should be available in browser");
        
        // Test console availability
        web_sys::console::log_1(&"Browser APIs test completed".into());
        
        // Test that we can create DOM elements (for future UI testing)
        let div = document.create_element("div").unwrap();
        div.set_text_content(Some("Test element"));
        assert_eq!(div.text_content(), Some("Test element".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_wasm_memory_management() {
        // Test that we can create and manage WASM objects without memory leaks
        let mut counters = Vec::new();
        
        for i in 0..100 {
            let replica = ReplicaId::default();
            let mut counter = GCounter::new(replica);
            counter.increment();
            counters.push(counter);
        }
        
        // Verify all counters work
        for (i, counter) in counters.iter().enumerate() {
            assert_eq!(counter.value(), 1);
        }
        
        // Test that we can drop the vector (memory cleanup)
        drop(counters);
        
        web_sys::console::log_1(&"WASM memory management test completed".into());
    }

    // ============================================================================
    // Performance Tests in WASM
    // ============================================================================

    #[wasm_bindgen_test]
    fn test_wasm_performance_crdt_operations() {
        let start_time = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();
        
        // Perform many CRDT operations
        let replica = ReplicaId::default();
        let mut counter = GCounter::new(replica);
        
        for _ in 0..1000 {
            counter.increment();
        }
        
        let end_time = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();
        
        let duration = end_time - start_time;
        
        // Verify operations completed
        assert_eq!(counter.value(), 1000);
        
        // Log performance (should be very fast in WASM)
        web_sys::console::log_1(&format!("CRDT operations completed in {}ms", duration).into());
        
        // Performance should be reasonable (less than 100ms for 1000 operations)
        assert!(duration < 100.0, "CRDT operations should be fast in WASM");
    }

    #[wasm_bindgen_test]
    async fn test_wasm_performance_storage_operations() {
        let storage = IndexedDbStorage::new("perf_test".to_string(), "store1".to_string());
        
        let start_time = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();
        
        // Perform many storage operations
        for i in 0..100 {
            let data = TestDocument {
                id: format!("perf_doc_{}", i),
                title: format!("Performance Test Document {}", i),
                content: format!("Content for document {}", i),
                version: i as u32,
            };
            
            storage.set(&format!("key_{}", i), &data).await.unwrap();
        }
        
        let end_time = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();
        
        let duration = end_time - start_time;
        
        // Verify operations completed
        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 100);
        
        // Log performance
        web_sys::console::log_1(&format!("Storage operations completed in {}ms", duration).into());
        
        // Performance should be reasonable (less than 1000ms for 100 operations)
        assert!(duration < 1000.0, "Storage operations should be reasonably fast");
    }

    // ============================================================================
    // Error Handling Tests in WASM
    // ============================================================================

    #[wasm_bindgen_test]
    async fn test_wasm_error_handling() {
        let storage = IndexedDbStorage::new("error_test".to_string(), "store1".to_string());
        
        // Test error handling for invalid operations
        let result: Result<Option<String>, _> = storage.get("nonexistent_key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
        
        // Test error handling for serialization
        let invalid_data = std::collections::HashMap::<String, std::collections::HashMap<String, String>>::new();
        let result = storage.set("test_key", &invalid_data).await;
        // This should work fine (empty HashMap is serializable)
        assert!(result.is_ok());
        
        web_sys::console::log_1(&"WASM error handling test completed".into());
    }
}

// ============================================================================
// Non-WASM Tests (for development and CI)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_tests {
    use super::*;
    use crate::storage::{indexeddb::IndexedDbStorage, memory::MemoryStorage, Storage, LocalStorage};
    use crate::crdt::{LwwRegister, ReplicaId, Mergeable};
    use crate::collection::LocalFirstCollection;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestDocument {
        id: String,
        title: String,
        content: String,
        version: u32,
    }

    impl Default for TestDocument {
        fn default() -> Self {
            Self {
                id: "default".to_string(),
                title: "Default Title".to_string(),
                content: "Default content".to_string(),
                version: 1,
            }
        }
    }

    impl Mergeable for TestDocument {
        type Error = std::io::Error;
        
        fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
            if other.version > self.version {
                *self = other.clone();
            }
            Ok(())
        }
        
        fn has_conflict(&self, other: &Self) -> bool {
            self.id == other.id && (self.title != other.title || self.content != other.content)
        }
    }

    #[tokio::test]
    async fn test_indexeddb_storage_fallback_non_wasm() {
        let storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());
        
        // In non-WASM environment, this should fall back to memory storage
        let test_data = TestDocument {
            id: "non_wasm_test".to_string(),
            title: "Non-WASM Test".to_string(),
            content: "Testing fallback behavior".to_string(),
            version: 1,
        };
        
        // These operations should work via fallback
        storage.set("test_key", &test_data).await.unwrap();
        let retrieved: Option<TestDocument> = storage.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));
    }

    #[tokio::test]
    async fn test_crdt_operations_non_wasm() {
        let replica1 = ReplicaId::default();
        let replica2 = ReplicaId::default();
        
        let mut reg1 = LwwRegister::new("initial".to_string(), replica1);
        let reg2 = LwwRegister::new("updated".to_string(), replica2);
        
        reg1.merge(&reg2).unwrap();
        assert_eq!(reg1.value(), "updated");
    }

    #[tokio::test]
    async fn test_collection_operations_non_wasm() {
        let storage = Storage::memory();
        let transport = crate::transport::memory::InMemoryTransport::new();
        let collection = LocalFirstCollection::<TestDocument, _>::new(storage, transport);
        
        let test_doc = TestDocument {
            id: "non_wasm_doc".to_string(),
            title: "Non-WASM Document".to_string(),
            content: "Testing in non-WASM environment".to_string(),
            version: 1,
        };
        
        collection.insert("doc1", &test_doc).await.unwrap();
        let retrieved = collection.get("doc1").await.unwrap();
        assert_eq!(retrieved, Some(test_doc));
    }
}
