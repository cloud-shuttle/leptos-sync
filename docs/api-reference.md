# API Reference for Leptos-Sync

## Overview

**Last Updated:** September 3rd, 2025  
**Version:** 0.1.0  
**Target:** Leptos 0.8.x, Rust 1.75+  
**License:** MIT/Apache-2.0

This document provides a comprehensive reference for all public APIs in Leptos-Sync, including types, functions, components, and usage examples.

## Core Types

### 1. **LocalFirstCollection**

The main collection type for local-first data management.

```rust
pub struct LocalFirstCollection<T: Mergeable + 'static> {
    name: String,
    storage: Arc<dyn LocalStorage>,
    sync_manager: Arc<SyncManager<T>>,
    items: RwSignal<HashMap<String, T>>,
    indexes: HashMap<String, Index<T>>,
    query_cache: Arc<Mutex<QueryCache<T>>>,
    event_bus: broadcast::Sender<CollectionEvent<T>>,
}

impl<T: Mergeable + 'static> LocalFirstCollection<T> {
    /// Creates a new collection with the specified name and configuration
    pub fn new(name: &str, config: Option<Config>) -> Result<Self, Error>
    
    /// Creates a collection with custom storage and transport
    pub fn new_with_storage(
        name: &str,
        storage: Arc<dyn LocalStorage>,
        transport: Arc<dyn SyncTransport>,
    ) -> Self
    
    /// Returns the collection name
    pub fn name(&self) -> &str
    
    /// Returns the number of items in the collection
    pub fn count(&self) -> usize
    
    /// Checks if the collection is empty
    pub fn is_empty(&self) -> bool
}
```

### 2. **Mergeable Trait**

Trait for types that can be merged using CRDT algorithms.

```rust
pub trait Mergeable: Clone + Serialize + DeserializeOwned + Send + Sync + PartialEq {
    /// Merge another instance into self
    fn merge(&mut self, other: &Self);
    
    /// Generate diff between two versions
    fn diff(&self, other: &Self) -> Vec<Change>;
    
    /// Apply a change to this instance
    fn apply_change(&mut self, change: &Change) -> Result<(), MergeError>;
    
    /// Check for conflicts between versions
    fn conflicts(&self, other: &Self) -> Vec<Conflict>;
    
    /// Version vector for causal ordering
    fn version_vector(&self) -> VersionVector;
    
    /// Timestamp for last-write-wins resolution
    fn timestamp(&self) -> f64;
}
```

## Storage API

### 1. **LocalStorage Trait**

Abstract storage interface for different storage backends.

```rust
pub trait LocalStorage: Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync;
    
    /// Get a value by key
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    
    /// Set a value by key
    async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Self::Error>;
    
    /// Delete a value by key
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
    
    /// Get all keys
    async fn keys(&self) -> Result<Vec<String>, Self::Error>;
    
    /// Clear all data
    async fn clear(&self) -> Result<(), Self::Error>;
}
```

### 2. **Storage Implementations**

#### **MemoryStorage**

In-memory storage for testing and development.

```rust
pub struct MemoryStorage {
    data: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MemoryStorage {
    /// Creates a new memory storage instance
    pub fn new() -> Self
    
    /// Creates memory storage with initial capacity
    pub fn with_capacity(capacity: usize) -> Self
}
```

#### **IndexedDbStorage**

Browser IndexedDB storage for persistent data.

```rust
pub struct IndexedDbStorage {
    db_name: String,
    store_name: String,
    connection: Arc<Mutex<Option<IdbDatabase>>>,
}

impl IndexedDbStorage {
    /// Creates a new IndexedDB storage instance
    pub async fn new(db_name: &str) -> Result<Self, Error>
    
    /// Creates storage with custom store name
    pub async fn new_with_store(db_name: &str, store_name: &str) -> Result<Self, Error>
}
```

## Collection API

### 1. **CRUD Operations**

#### **Create**

```rust
impl<T: Mergeable + 'static> LocalFirstCollection<T> {
    /// Creates a new item in the collection
    pub fn create(&self, item: T) -> Resource<Result<T, Error>> {
        let collection = self.clone();
        create_resource(
            move || (),
            move |_| {
                let collection = collection.clone();
                let item = item.clone();
                async move {
                    // 1. Generate unique ID
                    let id = generate_id();
                    let mut item_with_id = item;
                    item_with_id.set_id(id.clone());
                    
                    // 2. Optimistic update
                    collection.items.update(|items| {
                        items.insert(id.clone(), item_with_id.clone());
                    });
                    
                    // 3. Persist to storage
                    collection.storage.set(&id, &item_with_id).await?;
                    
                    // 4. Queue for sync
                    collection.sync_manager.queue_change(Change::Create {
                        id: id.clone(),
                        data: item_with_id.clone(),
                        timestamp: now(),
                    });
                    
                    Ok(item_with_id)
                }
            }
        )
    }
}
```

#### **Read**

```rust
impl<T: Mergeable + 'static> LocalFirstCollection<T> {
    /// Gets an item by ID
    pub fn get(&self, id: &str) -> Resource<Option<T>> {
        let collection = self.clone();
        let id = id.to_string();
        
        create_resource(
            move || id.clone(),
            move |id| {
                let collection = collection.clone();
                async move {
                    // Check local state first
                    if let Some(item) = collection.items.with_untracked(|items| {
                        items.get(&id).cloned()
                    }) {
                        return Ok(Some(item));
                    }
                    
                    // Fallback to storage
                    collection.storage.get(&id).await
                }
            }
        )
    }
    
    /// Returns a reactive signal for watching an item
    pub fn watch(&self, id: &str) -> Signal<Option<T>> {
        let collection = self.clone();
        let id = id.to_string();
        
        create_memo(move |_| {
            collection.items.with(|items| items.get(&id).cloned())
        }).into()
    }
}
```

## Query API

### 1. **QueryBuilder**

Build complex queries with method chaining.

```rust
pub struct QueryBuilder<T: Mergeable> {
    collection: Arc<LocalFirstCollection<T>>,
    filters: Vec<Filter>,
    sort: Vec<SortCriterion>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl<T: Mergeable + 'static> QueryBuilder<T> {
    /// Creates a new query builder
    pub fn new(collection: Arc<LocalFirstCollection<T>>) -> Self
    
    /// Adds a filter condition
    pub fn filter<V: Into<Value>>(mut self, field: &str, op: Op, value: V) -> Self {
        self.filters.push(Filter {
            field: field.to_string(),
            operator: op,
            value: value.into(),
        });
        self
    }
    
    /// Adds a sort criterion
    pub fn sort_by(mut self, field: &str, order: Order) -> Self {
        self.sort.push(SortCriterion {
            field: field.to_string(),
            order,
        });
        self
    }
    
    /// Sets the maximum number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Returns a reactive signal that updates with changes
    pub fn watch(self) -> Signal<Vec<T>> {
        let collection = self.collection.clone();
        let query = self.compile();
        
        create_memo(move |_| {
            collection.items.with(|items| query.execute(items))
        }).into()
    }
}
```

### 2. **Filter Operations**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Equal to
    Eq,
    /// Not equal to
    Ne,
    /// Less than
    Lt,
    /// Less than or equal to
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal to
    Ge,
    /// In array
    In,
    /// Not in array
    NotIn,
    /// String contains
    Contains,
    /// Regex match
    Matches,
    /// Field exists
    Exists,
    /// Field doesn't exist
    NotExists,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}
```

## Sync API

### 1. **SyncManager**

Manages synchronization between local and remote data.

```rust
pub struct SyncManager<T: Mergeable + 'static> {
    collection: Arc<LocalFirstCollection<T>>,
    transport: Arc<dyn SyncTransport>,
    state: RwSignal<SyncStatus>,
    change_queue: Arc<Mutex<VecDeque<PendingChange<T>>>>,
    conflict_resolver: Box<dyn ConflictResolver<T>>,
    retry_strategy: ExponentialBackoff,
}

impl<T: Mergeable + 'static> SyncManager<T> {
    /// Creates a new sync manager
    pub fn new(
        collection: Arc<LocalFirstCollection<T>>,
        transport: Arc<dyn SyncTransport>,
    ) -> Self
    
    /// Manually triggers a sync operation
    pub async fn sync(&self) -> Result<SyncResult<T>, SyncError> {
        let _guard = self.sync_mutex.lock().await;
        
        self.update_state(SyncStatus::Syncing {
            progress: 0.0,
            pending_changes: self.pending_change_count(),
            conflicts: 0
        });
        
        // Phase 1: Send local changes
        let local_changes = self.collect_pending_changes().await?;
        let sent_changes = self.transport.send(local_changes).await?;
        
        // Phase 2: Receive remote changes
        let remote_changes = self.transport.receive().await?;
        
        // Phase 3: Merge and resolve conflicts
        let conflicts = self.merge_remote_changes(remote_changes).await?;
        let resolved_conflicts = self.resolve_conflicts(conflicts).await?;
        
        // Phase 4: Persist final state
        self.persist_changes().await?;
        
        self.update_state(SyncStatus::Synced {
            last_sync: SystemTime::now()
        });
        
        Ok(SyncResult {
            sent_changes: sent_changes.len(),
            received_changes: remote_changes.len(),
            resolved_conflicts: resolved_conflicts.len(),
        })
    }
    
    /// Returns the current sync status
    pub fn sync_status(&self) -> Signal<SyncStatus> {
        self.state.into()
    }
}
```

## Component API

### 1. **LocalFirstProvider**

Context provider for dependency injection.

```rust
#[component]
pub fn LocalFirstProvider(
    children: Children,
    #[prop(optional)] config: Option<LocalFirstConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let context = create_rw_signal(LocalFirstContext::new(config));
    
    // Initialize global sync status tracking
    let sync_status = create_rw_signal(GlobalSyncStatus::Initializing);
    context.sync_status.set(sync_status.get_untracked());
    
    provide_context(context);
    
    // Global error boundary for sync errors
    ErrorBoundary::new(children)
        .fallback(|errors| view! {
            <div class="sync-error-boundary">
                <h3>"Sync Error"</h3>
                <p>{move || errors.get().first().map(|e| e.to_string()).unwrap_or_default()}</p>
            </div>
        })
}
```

### 2. **SyncStatusIndicator**

Displays current synchronization status.

```rust
#[component]
pub fn SyncStatusIndicator(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] show_details: Option<bool>,
) -> impl IntoView {
    let context = use_context::<LocalFirstContext>()
        .expect("LocalFirstProvider required");
    
    let status = context.sync_status;
    let show_details = show_details.unwrap_or(false);
    
    view! {
        <div class=move || format!("sync-status {}", class.unwrap_or(""))
             class:online=move || matches!(status.get(), GlobalSyncStatus::Synced)
             class:offline=move || matches!(status.get(), GlobalSyncStatus::Offline)
             class:syncing=move || matches!(status.get(), GlobalSyncStatus::Syncing { .. })
             class:error=move || matches!(status.get(), GlobalSyncStatus::Error { .. })>
            
            {move || match status.get() {
                GlobalSyncStatus::Initializing => {
                    view! { <span class="status-text">"Initializing..."</span> }
                },
                GlobalSyncStatus::Offline => {
                    view! { 
                        <span class="status-text">"Offline"</span>
                        {move || if show_details {
                            view! { <div class="status-details">"All changes saved locally"</div> }
                        } else {
                            view! { <div></div> }
                        }}
                    }
                },
                GlobalSyncStatus::Syncing { progress, pending } => {
                    view! {
                        <span class="status-text">"Syncing..."</span>
                        <div class="progress-container">
                            <div class="progress-bar">
                                <div class="progress-fill" 
                                     style:width=move || format!("{}%", progress)></div>
                            </div>
                            {move || if show_details {
                                view! { 
                                    <div class="status-details">
                                        {pending}" pending changes"
                                    </div> 
                                }
                            } else {
                                view! { <div></div> }
                            }}
                        </div>
                    }
                },
                GlobalSyncStatus::Synced => {
                    view! {
                        <span class="status-text">"Synced"</span>
                        {move || if show_details {
                            view! { 
                                <div class="status-details">
                                    "All changes synchronized"
                                </div> 
                            }
                        } else {
                            view! { <div></div> }
                        }}
                    }
                },
                GlobalSyncStatus::Error { message, recoverable } => {
                    view! {
                        <span class="status-text status-error">"Sync Error"</span>
                        {move || if show_details {
                            view! { 
                                <div class="status-details error-details">
                                    <div class="error-message">{message.clone()}</div>
                                    {move || if recoverable {
                                        view! { 
                                            <button class="retry-button" 
                                                    on:click=move |_| {
                                                        // Trigger retry logic
                                                    }>
                                                "Retry"
                                            </button> 
                                        }
                                    } else {
                                        view! { <div></div> }
                                    }}
                                </div> 
                            }
                        } else {
                            view! { <div></div> }
                        }}
                    }
                }
            }}
        </div>
    }
}
```

## Error Types

### 1. **Core Error Types**

```rust
#[derive(Debug, thiserror::Error)]
pub enum LeptosSyncError {
    /// Storage operation failed
    #[error("Storage operation failed: {0}")]
    Storage(#[from] StorageError),
    
    /// Network communication failed
    #[error("Network communication failed: {0}")]
    Network(#[from] NetworkError),
    
    /// CRDT merge conflict
    #[error("CRDT merge conflict: {0}")]
    MergeConflict(String),
    
    /// Serialization failed
    #[error("Serialization failed: {0}")]
    Serialization(#[from] SerializationError),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Item not found
    #[error("Item not found: {0}")]
    NotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}
```

## Configuration

### 1. **Configuration Types**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFirstConfig {
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Synchronization configuration
    pub sync: SyncConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Feature flags
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    
    /// Database connection string
    pub database_url: Option<String>,
    
    /// Redis connection string
    pub redis_url: Option<String>,
    
    /// Storage encryption settings
    pub encryption: EncryptionConfig,
    
    /// Data retention policy
    pub retention: RetentionPolicy,
}
```

### 2. **Configuration Loading**

```rust
impl LocalFirstConfig {
    /// Loads configuration from environment variables
    pub fn from_env() -> Result<Self, Error> {
        let storage = StorageConfig::from_env()?;
        let sync = SyncConfig::from_env()?;
        let security = SecurityConfig::from_env()?;
        let performance = PerformanceConfig::from_env()?;
        let features = FeatureFlags::from_env()?;
        
        Ok(Self {
            storage,
            sync,
            security,
            performance,
            features,
        })
    }
    
    /// Loads configuration from a file
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)?;
        let config: LocalFirstConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if let Err(e) = self.storage.validate() {
            errors.push(format!("Storage config error: {}", e));
        }
        
        if let Err(e) = self.sync.validate() {
            errors.push(format!("Sync config error: {}", e));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Conclusion

This comprehensive API reference provides all the information needed to use Leptos-Sync effectively. The API is designed to be intuitive, type-safe, and performant while maintaining the flexibility needed for complex local-first applications.

Key features of the API:
- ✅ **Type Safety**: Full Rust type safety with compile-time guarantees
- ✅ **Reactive**: Built on Leptos reactive primitives for automatic UI updates
- ✅ **Async**: Full async/await support for non-blocking operations
- ✅ **Extensible**: Plugin architecture for custom storage and transport backends
- ✅ **Error Handling**: Comprehensive error types with recovery strategies
- ✅ **Configuration**: Flexible configuration system with validation

This API reference serves as the foundation for building robust, offline-capable applications with real-time synchronization capabilities.
