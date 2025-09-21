# IndexedDB Storage Implementation - High Priority

## Overview
Implement actual IndexedDB storage for persistent local data instead of falling back to localStorage.

## Current State
- `IndexedDbStorage` exists but all methods return "not yet fully implemented" 
- Falls back to localStorage which has ~5MB limits
- No schema management or migration support
- Missing transaction support for atomic operations

## Design Requirements

### Core Features
- Real IndexedDB operations (not localStorage fallback)
- Schema versioning and migrations
- Transaction support for atomic CRDT operations
- Efficient querying with proper indexing
- Batch operations for performance
- Error handling for quota exceeded scenarios

### Database Schema
```rust
pub struct CrdtDatabase {
    pub version: u32,
    pub collections: ObjectStore,  // Primary data
    pub metadata: ObjectStore,     // Collection metadata  
    pub deltas: ObjectStore,      // CRDT delta log
    pub peers: ObjectStore,       // Known peer information
}
```

## Implementation Plan

### Phase 1: Core IndexedDB Wrapper (Week 1)
**File**: `leptos-sync-core/src/storage/indexeddb/connection.rs` (< 300 lines)

Database connection and schema:
- Database opening with version management
- Object store creation and configuration  
- Index management for efficient queries
- Transaction management (read/write)

### Phase 2: CRUD Operations (Week 1)
**File**: `leptos-sync-core/src/storage/indexeddb/operations.rs` (< 250 lines)

Basic storage operations:
- Get/Set/Delete individual records
- Batch operations for performance
- Key enumeration and range queries
- Cursor-based iteration for large datasets

### Phase 3: CRDT Integration (Week 1)
**File**: `leptos-sync-core/src/storage/indexeddb/crdt_store.rs` (< 200 lines)

CRDT-specific storage:
- Delta log persistence and querying
- Efficient merge operation storage
- Tombstone record management
- Conflict resolution data persistence

### Phase 4: Migration System (Week 1)
**File**: `leptos-sync-core/src/storage/indexeddb/migrations.rs` (< 200 lines)

Schema evolution:
- Version detection and upgrade paths
- Data migration between schema versions
- Rollback support for failed migrations
- Migration validation and testing

## Breaking Down Current File

### Current Issues
- `indexeddb.rs` is 315 lines with placeholder implementations
- Mixes connection logic with operation implementations
- No proper error handling or recovery

### Proposed Structure
```
storage/indexeddb/
├── connection.rs      (< 300 lines) - DB connection & schema
├── operations.rs      (< 250 lines) - CRUD operations  
├── crdt_store.rs      (< 200 lines) - CRDT-specific logic
├── migrations.rs      (< 200 lines) - Schema migrations
├── errors.rs          (< 150 lines) - Error types
├── transactions.rs    (< 150 lines) - Transaction helpers
└── mod.rs            (< 100 lines) - Public API
```

## Technical Implementation

### IndexedDB Connection
```rust
pub struct IndexedDbConnection {
    db: IdbDatabase,
    version: u32,
}

impl IndexedDbConnection {
    pub async fn open(name: &str, version: u32) -> Result<Self, StorageError> {
        let mut db_req = IdbDatabase::open_u32(name, version)?;
        
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| {
            let db = evt.db();
            Self::create_object_stores(&db, evt.old_version(), evt.new_version())?;
            Ok(())
        }));
        
        let db = JsFuture::from(db_req).await?;
        
        Ok(Self {
            db: db.into(),
            version,
        })
    }
    
    fn create_object_stores(
        db: &IdbDatabase, 
        old_version: Option<u32>, 
        new_version: Option<u32>
    ) -> Result<(), StorageError> {
        // Create object stores and indexes based on version
        match (old_version, new_version) {
            (None, Some(1)) => self.create_v1_schema(db),
            (Some(1), Some(2)) => self.migrate_v1_to_v2(db),
            // ... other migration paths
        }
    }
}
```

### Storage Operations
```rust
impl Storage for IndexedDbStorage {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let transaction = self.db.transaction_on_one_with_mode(
            "collections", 
            IdbTransactionMode::Readonly
        )?;
        
        let store = transaction.object_store("collections")?;
        let request = store.get(&JsValue::from_str(key))?;
        let result = JsFuture::from(request).await?;
        
        if result.is_undefined() {
            Ok(None)
        } else {
            let data = js_sys::Uint8Array::new(&result);
            Ok(Some(data.to_vec()))
        }
    }
    
    async fn set(&self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        let transaction = self.db.transaction_on_one_with_mode(
            "collections",
            IdbTransactionMode::Readwrite
        )?;
        
        let store = transaction.object_store("collections")?;
        let array = js_sys::Uint8Array::from(value);
        let request = store.put_key_val(&JsValue::from_str(key), &array.into())?;
        
        JsFuture::from(request).await?;
        Ok(())
    }
    
    // ... other operations
}
```

### CRDT Delta Storage
```rust
pub struct DeltaStore {
    db: Arc<IndexedDbConnection>,
}

impl DeltaStore {
    pub async fn append_delta(
        &self, 
        collection_id: &str, 
        delta: &CrdtDelta
    ) -> Result<(), StorageError> {
        let transaction = self.db.transaction_on_one_with_mode(
            "deltas",
            IdbTransactionMode::Readwrite
        )?;
        
        let store = transaction.object_store("deltas")?;
        let record = DeltaRecord {
            id: format!("{}#{}", collection_id, delta.timestamp),
            collection_id: collection_id.to_string(),
            delta: serde_json::to_vec(delta)?,
            timestamp: delta.timestamp,
        };
        
        let request = store.add(&serde_wasm_bindgen::to_value(&record)?)?;
        JsFuture::from(request).await?;
        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests
- Database connection and schema creation
- CRUD operations with various data types
- Transaction rollback scenarios
- Migration between schema versions
- Error handling (quota exceeded, corruption)

### Integration Tests  
- End-to-end CRDT persistence and retrieval
- Multi-collection data operations
- Performance with large datasets
- Browser compatibility testing

### Browser Tests (WASM)
- IndexedDB availability detection
- Quota management and cleanup
- Cross-tab synchronization
- Storage eviction scenarios

## Error Handling

### Quota Management
```rust
pub enum StorageError {
    QuotaExceeded,
    NotSupported,
    DatabaseCorrupted,
    TransactionFailed(String),
    SerializationError(String),
}

impl IndexedDbStorage {
    async fn handle_quota_exceeded(&self) -> Result<(), StorageError> {
        // Implement cleanup strategies:
        // 1. Remove old delta records
        // 2. Compact CRDT state
        // 3. Notify user about storage limits
        todo!()
    }
}
```

## Performance Considerations
- Use batch operations for multiple records
- Implement cursor-based pagination for large queries
- Add proper indexing for common query patterns
- Cache frequently accessed data in memory
- Implement background cleanup of old deltas

## Acceptance Criteria
- [ ] All IndexedDB operations work without localStorage fallback
- [ ] Schema migrations work correctly between versions
- [ ] CRDT deltas persist and can be queried efficiently  
- [ ] Transactions ensure data consistency
- [ ] Quota exceeded scenarios handled gracefully
- [ ] All files under 300 lines
- [ ] Cross-browser compatibility (Chrome, Firefox, Safari)

## Time Estimate: 1-2 weeks  
## Dependencies: Compilation fixes (01)
## Risk: Medium - browser API complexity and compatibility
