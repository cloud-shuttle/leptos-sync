//! IndexedDB connection and schema management

use super::errors::IndexedDbError;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{
    window, IdbDatabase, IdbFactory, IdbObjectStore, IdbOpenDbRequest, IdbRequest, IdbTransaction,
    IdbTransactionMode, IdbVersionChangeEvent,
};

/// IndexedDB connection wrapper
#[derive(Debug)]
pub struct IndexedDbConnection {
    #[cfg(target_arch = "wasm32")]
    db: IdbDatabase,
    version: u32,
    db_name: String,
}

impl IndexedDbConnection {
    /// Open an IndexedDB database with version management
    #[cfg(target_arch = "wasm32")]
    pub async fn open(name: &str, version: u32) -> Result<Self, IndexedDbError> {
        let window = window()
            .ok_or_else(|| IndexedDbError::NotSupported("No window available".to_string()))?;

        let idb_factory = window
            .indexed_db()
            .map_err(|_| IndexedDbError::NotSupported("IndexedDB not available".to_string()))?;

        let open_request = idb_factory
            .open_with_u32(name, version)
            .map_err(|_| IndexedDbError::DatabaseError("Failed to open database".to_string()))?;

        // Set up the upgrade handler
        let db_name = name.to_string();
        let on_upgrade_needed = Closure::wrap(Box::new(move |event: &IdbVersionChangeEvent| {
            if let Some(db) = event
                .target()
                .and_then(|t| t.dyn_ref::<IdbOpenDbRequest>())
                .and_then(|r| r.result().ok())
            {
                Self::create_object_stores(&db, event.old_version(), event.new_version())
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to create object stores: {:?}", e);
                    });
            }
        }) as Box<dyn FnMut(&IdbVersionChangeEvent)>);

        open_request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        on_upgrade_needed.forget();

        // Wait for the database to open
        let result = JsFuture::from(open_request)
            .await
            .map_err(|_| IndexedDbError::DatabaseError("Failed to open database".to_string()))?;

        let database = result
            .dyn_into::<IdbDatabase>()
            .map_err(|_| IndexedDbError::DatabaseError("Invalid database result".to_string()))?;

        Ok(Self {
            db: database,
            version,
            db_name,
        })
    }

    /// Open an IndexedDB database (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn open(_name: &str, _version: u32) -> Result<Self, IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Create object stores and indexes based on version
    #[cfg(target_arch = "wasm32")]
    fn create_object_stores(
        db: &IdbDatabase,
        old_version: Option<u32>,
        new_version: Option<u32>,
    ) -> Result<(), IndexedDbError> {
        match (old_version, new_version) {
            (None, Some(1)) => Self::create_v1_schema(db),
            (Some(1), Some(2)) => Self::migrate_v1_to_v2(db),
            (Some(old), Some(new)) if old < new => {
                // Handle other migration paths
                Self::migrate_schema(db, old, new)
            }
            _ => Ok(()),
        }
    }

    /// Create object stores (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    fn create_object_stores(
        _db: &(),
        _old_version: Option<u32>,
        _new_version: Option<u32>,
    ) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Create version 1 schema
    #[cfg(target_arch = "wasm32")]
    fn create_v1_schema(db: &IdbDatabase) -> Result<(), IndexedDbError> {
        // Create collections object store
        let collections_store = db.create_object_store("collections").map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to create collections store".to_string())
        })?;

        // Create metadata object store
        let metadata_store = db.create_object_store("metadata").map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to create metadata store".to_string())
        })?;

        // Create deltas object store
        let deltas_store = db.create_object_store("deltas").map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to create deltas store".to_string())
        })?;

        // Create peers object store
        let peers_store = db.create_object_store("peers").map_err(|_| {
            IndexedDbError::ObjectStoreError("Failed to create peers store".to_string())
        })?;

        // Add indexes for efficient querying
        deltas_store
            .create_index("collection_id", "collection_id")
            .map_err(|_| {
                IndexedDbError::ObjectStoreError("Failed to create collection_id index".to_string())
            })?;

        deltas_store
            .create_index("timestamp", "timestamp")
            .map_err(|_| {
                IndexedDbError::ObjectStoreError("Failed to create timestamp index".to_string())
            })?;

        Ok(())
    }

    /// Create version 1 schema (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    fn create_v1_schema(_db: &()) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Migrate from version 1 to version 2
    #[cfg(target_arch = "wasm32")]
    fn migrate_v1_to_v2(db: &IdbDatabase) -> Result<(), IndexedDbError> {
        // Add new object stores or indexes for version 2
        // For now, just create the basic schema
        Self::create_v1_schema(db)?;

        // Add version 2 specific changes here
        // Example: Add new indexes or object stores

        Ok(())
    }

    /// Migrate from version 1 to version 2 (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    fn migrate_v1_to_v2(_db: &()) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Handle general schema migrations
    #[cfg(target_arch = "wasm32")]
    fn migrate_schema(
        db: &IdbDatabase,
        from_version: u32,
        to_version: u32,
    ) -> Result<(), IndexedDbError> {
        tracing::info!(
            "Migrating database from version {} to {}",
            from_version,
            to_version
        );

        // For now, just recreate the schema
        // In a real implementation, you'd have specific migration logic for each version
        Self::create_v1_schema(db)?;

        Ok(())
    }

    /// Handle general schema migrations (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    fn migrate_schema(
        _db: &(),
        _from_version: u32,
        _to_version: u32,
    ) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Get the database instance
    #[cfg(target_arch = "wasm32")]
    pub fn database(&self) -> &IdbDatabase {
        &self.db
    }

    /// Get the database instance (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn database(&self) -> Result<&(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Get the database version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get the database name
    pub fn name(&self) -> &str {
        &self.db_name
    }

    /// Create a read-write transaction
    #[cfg(target_arch = "wasm32")]
    pub fn transaction(&self, store_names: &[&str]) -> Result<IdbTransaction, IndexedDbError> {
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(store_names, IdbTransactionMode::Readwrite)
            .map_err(|_| {
                IndexedDbError::TransactionError("Failed to create transaction".to_string())
            })?;
        Ok(transaction)
    }

    /// Create a read-write transaction (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn transaction(&self, _store_names: &[&str]) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Create a read-only transaction
    #[cfg(target_arch = "wasm32")]
    pub fn readonly_transaction(
        &self,
        store_names: &[&str],
    ) -> Result<IdbTransaction, IndexedDbError> {
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(store_names, IdbTransactionMode::Readonly)
            .map_err(|_| {
                IndexedDbError::TransactionError(
                    "Failed to create readonly transaction".to_string(),
                )
            })?;
        Ok(transaction)
    }

    /// Create a read-only transaction (non-WASM - always fails)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn readonly_transaction(&self, _store_names: &[&str]) -> Result<(), IndexedDbError> {
        Err(IndexedDbError::NotSupported(
            "IndexedDB not available in non-WASM environment".to_string(),
        ))
    }

    /// Close the database connection
    #[cfg(target_arch = "wasm32")]
    pub fn close(&self) {
        self.db.close();
    }

    /// Close the database connection (non-WASM - no-op)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn close(&self) {
        // No-op in non-WASM environments
    }
}

impl Clone for IndexedDbConnection {
    fn clone(&self) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            db: self.db.clone(),
            version: self.version,
            db_name: self.db_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_creation() {
        // Test that connection creation works (or fails gracefully in non-WASM)
        let result = IndexedDbConnection::open("test_db", 1).await;

        #[cfg(target_arch = "wasm32")]
        {
            // In WASM environment, this should work if IndexedDB is available
            // We can't easily test this in unit tests, so we just verify it doesn't panic
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // In non-WASM environment, this should fail gracefully
            assert!(result.is_err());
            match result.unwrap_err() {
                IndexedDbError::NotSupported(_) => {}
                _ => panic!("Expected NotSupported error"),
            }
        }
    }
}
