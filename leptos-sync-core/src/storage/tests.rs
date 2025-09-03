//! Storage layer tests

use super::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: String,
    value: i32,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_operations() {
        let storage = MemoryStorage::new();
        
        let test_data = TestData {
            id: "test1".to_string(),
            value: 42,
            message: "Hello World".to_string(),
        };

        // Test set and get
        storage.set("test_key", &test_data).await.unwrap();
        let retrieved: Option<TestData> = storage.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data.clone()));

        // Test keys
        let keys = storage.keys().await.unwrap();
        assert!(keys.contains(&"test_key".to_string()));

        // Test delete
        storage.delete("test_key").await.unwrap();
        let retrieved: Option<TestData> = storage.get("test_key").await.unwrap();
        assert_eq!(retrieved, None);

        // Test clear
        storage.set("key1", &test_data).await.unwrap();
        storage.set("key2", &test_data).await.unwrap();
        storage.clear().await.unwrap();
        let keys = storage.keys().await.unwrap();
        assert!(keys.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_indexed_db_storage_fallback() {
        // IndexedDbStorage should fall back to memory when not in browser
        let storage = IndexedDbStorage::with_fallback("test_db".to_string(), "test_store".to_string());
        
        let test_data = TestData {
            id: "test1".to_string(),
            value: 42,
            message: "Hello World".to_string(),
        };

        // Should work with fallback to memory
        storage.set("test_key", &test_data).await.unwrap();
        let retrieved: Option<TestData> = storage.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));
    }

    #[tokio::test] 
    async fn test_storage_error_handling() {
        let storage = MemoryStorage::new();
        
        // Test getting non-existent key
        let result: Result<Option<TestData>, _> = storage.get("nonexistent").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

        // Test delete non-existent key (should not error)
        let result = storage.delete("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_storage_serialization() {
        let storage = MemoryStorage::new();
        
        // Test different data types
        storage.set("string", &"hello".to_string()).await.unwrap();
        storage.set("number", &42i32).await.unwrap();
        storage.set("bool", &true).await.unwrap();
        
        let s: Option<String> = storage.get("string").await.unwrap();
        let n: Option<i32> = storage.get("number").await.unwrap();
        let b: Option<bool> = storage.get("bool").await.unwrap();
        
        assert_eq!(s, Some("hello".to_string()));
        assert_eq!(n, Some(42));
        assert_eq!(b, Some(true));
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_indexed_db_storage_in_browser() {
        let storage = IndexedDbStorage::new("test_db".to_string(), "test_store".to_string());
        
        let test_data = TestData {
            id: "browser_test".to_string(),
            value: 123,
            message: "Browser test".to_string(),
        };

        // Test basic operations
        storage.set("browser_key", &test_data).await.unwrap();
        let retrieved: Option<TestData> = storage.get("browser_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));

        // Test persistence (this would require manual verification)
        // In a real browser, data should persist across page refreshes
        web_sys::console::log_1(&"Storage test completed in browser".into());
    }

    #[wasm_bindgen_test]
    async fn test_storage_persistence() {
        let storage1 = IndexedDbStorage::new("persistence_test".to_string(), "store1".to_string());
        let storage2 = IndexedDbStorage::new("persistence_test".to_string(), "store1".to_string());
        
        let test_data = TestData {
            id: "persist_test".to_string(),
            value: 456,
            message: "Persistence test".to_string(),
        };

        // Set with first storage instance
        storage1.set("persist_key", &test_data).await.unwrap();
        
        // Get with second storage instance (should work if persistent)
        let retrieved: Option<TestData> = storage2.get("persist_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));

        web_sys::console::log_1(&"Persistence test completed".into());
    }
}