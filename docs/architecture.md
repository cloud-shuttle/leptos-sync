# Architecture Specification for Leptos-Sync

## Overview

Leptos-Sync is a comprehensive local-first library architected for high performance, type safety, and seamless integration with the Leptos web framework. This document details the technical architecture, design patterns, and system interactions that enable robust offline-first applications.

**Last Updated:** September 3rd, 2025  
**Target Versions:** Leptos 0.8.x, Rust 1.75+, WASM target

## Architectural Principles

### 1. Local-First Architecture
- **Data Sovereignty**: User data lives primarily on device, with cloud as optional sync destination
- **Offline-First**: Full functionality without network connectivity
- **Eventually Consistent**: Changes converge across devices through CRDT algorithms
- **Immediate Responsiveness**: Zero-latency user interactions through optimistic updates

### 2. Layered Architecture Pattern

```
┌─────────────────────────────────────────────────────┐
│                Application Layer                     │ ← Leptos Components
├─────────────────────────────────────────────────────┤
│              Component Library                       │ ← UI Components & Hooks
├─────────────────────────────────────────────────────┤
│               Collection API                         │ ← CRUD Operations
├─────────────────────────────────────────────────────┤
│              Synchronization Engine                  │ ← Conflict Resolution
├─────────────────────────────────────────────────────┤
│              CRDT Implementation                     │ ← Mergeable Types  
├─────────────────────────────────────────────────────┤
│              Transport Abstraction                   │ ← Network Protocols
├─────────────────────────────────────────────────────┤
│              Storage Abstraction                     │ ← Persistence Layer
└─────────────────────────────────────────────────────┘
```

## Core System Components

### 1. Storage Abstraction Layer

#### Design Pattern: Strategy Pattern + Adapter Pattern
```rust
pub trait LocalStorage: Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync;
    
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Self::Error>;
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
    async fn keys(&self) -> Result<Vec<String>, Self::Error>;
    async fn clear(&self) -> Result<(), Self::Error>;
    
    // Batch operations for performance
    async fn get_many<T: DeserializeOwned>(&self, keys: &[&str]) -> Result<HashMap<String, T>, Self::Error>;
    async fn set_many<T: Serialize>(&self, items: &HashMap<&str, &T>) -> Result<(), Self::Error>;
}
```

#### Storage Backend Implementations

**HybridStorage** (Auto-selecting storage with fallback chain):
```rust
pub struct HybridStorage {
    primary: Arc<dyn LocalStorage>,
    fallback: Option<Arc<dyn LocalStorage>>,
    capabilities: StorageCapabilities,
}

impl HybridStorage {
    pub fn new() -> Self {
        let capabilities = detect_storage_capabilities();
        let primary = Self::select_optimal_storage(&capabilities);
        let fallback = Self::select_fallback_storage(&capabilities);
        
        Self { primary, fallback, capabilities }
    }
    
    fn select_optimal_storage(caps: &StorageCapabilities) -> Arc<dyn LocalStorage> {
        if caps.supports_opfs {
            Arc::new(OpfsStorage::new())
        } else if caps.supports_indexeddb {
            Arc::new(IndexedDbStorage::new("leptos-sync"))
        } else {
            Arc::new(LocalStorageBackend::new())
        }
    }
}

// Performance characteristics by backend (2025 standards)
pub struct StorageCapabilities {
    pub supports_opfs: bool,          // 100MB+, fastest, Chrome 108+
    pub supports_indexeddb: bool,     // Unlimited, async, all modern browsers
    pub supports_localstorage: bool,  // 5-10MB, sync, universal support
    pub estimated_quota: u64,
    pub is_persistent: bool,
}
```

**Performance Requirements** (2025 targets):
- OPFS: <1ms read/write for small objects (<1KB)
- IndexedDB: <5ms read/write for small objects
- LocalStorage: <2ms read/write (but limited size)

### 2. CRDT (Conflict-free Replicated Data Types) System

#### Design Pattern: Command Pattern + Observer Pattern

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

// Mathematical CRDT properties (automatically verified in tests)
// 1. Commutativity: A ⊔ B = B ⊔ A  
// 2. Associativity: (A ⊔ B) ⊔ C = A ⊔ (B ⊔ C)
// 3. Idempotence: A ⊔ A = A
```

#### CRDT Implementation Strategies

**Last-Write-Wins (LWW) - Simple Types**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LwwValue<T> {
    pub value: T,
    pub timestamp: f64,
    pub replica_id: ReplicaId,
    pub version: u64,
}

impl<T: Clone + PartialEq> Mergeable for LwwValue<T> {
    fn merge(&mut self, other: &Self) {
        if other.timestamp > self.timestamp || 
           (other.timestamp == self.timestamp && other.replica_id > self.replica_id) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.replica_id = other.replica_id;
            self.version = std::cmp::max(self.version, other.version) + 1;
        }
    }
}
```

**Multi-Value Register (MV-Register) - Concurrent Updates**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvRegister<T> {
    pub values: HashMap<ReplicaId, (T, VectorClock)>,
    pub version_vector: VersionVector,
}

impl<T: Clone + PartialEq> Mergeable for MvRegister<T> {
    fn merge(&mut self, other: &Self) {
        // Merge values based on vector clock comparison
        for (replica, (value, clock)) in &other.values {
            match self.values.get(replica) {
                Some((_, our_clock)) if clock.happens_after(our_clock) => {
                    self.values.insert(*replica, (value.clone(), clock.clone()));
                }
                None => {
                    self.values.insert(*replica, (value.clone(), clock.clone()));
                }
                _ => {} // Our version is newer or concurrent
            }
        }
        self.version_vector.merge(&other.version_vector);
    }
}
```

**Derive Macro for Automatic CRDT Implementation**:
```rust
#[derive(LocalFirst, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[local_first(crdt = "lww")]
pub struct TodoItem {
    #[local_first(id)]
    pub id: Uuid,
    
    #[local_first(crdt = "lww")]
    pub title: String,
    
    #[local_first(crdt = "lww")]  
    pub completed: bool,
    
    #[local_first(crdt = "mv")] // Multi-value for concurrent updates
    pub tags: Vec<String>,
    
    #[local_first(version)]
    pub version: u64,
    
    #[local_first(timestamp)]
    pub modified: f64,
    
    #[local_first(replica)]
    pub replica_id: ReplicaId,
}

// Generated implementation handles per-field CRDT logic
impl Mergeable for TodoItem {
    fn merge(&mut self, other: &Self) {
        // Generated code merges each field according to its CRDT strategy
        self.title.merge(&other.title);
        self.completed.merge(&other.completed);  
        self.tags.merge(&other.tags);
        // Update metadata
        self.version = std::cmp::max(self.version, other.version) + 1;
        self.modified = std::cmp::max(self.modified, other.modified);
    }
}
```

### 3. Synchronization Engine

#### Design Pattern: Publisher-Subscriber + State Machine

```rust
pub struct SyncManager<T: Mergeable + 'static> {
    collection: Arc<LocalFirstCollection<T>>,
    transport: Arc<dyn SyncTransport>,
    state: RwSignal<SyncState>,
    change_queue: Arc<Mutex<VecDeque<PendingChange<T>>>>,
    conflict_resolver: Box<dyn ConflictResolver<T>>,
    retry_strategy: ExponentialBackoff,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyncState {
    Disconnected,
    Connecting,
    Connected,
    Syncing { 
        progress: f32,
        pending_changes: usize,
        conflicts: usize 
    },
    Synced { last_sync: SystemTime },
    Error { 
        error: String, 
        retry_in: Duration 
    },
}

impl<T: Mergeable + 'static> SyncManager<T> {
    pub async fn sync(&self) -> Result<SyncResult<T>, SyncError> {
        let _guard = self.sync_mutex.lock().await; // Prevent concurrent syncs
        
        self.update_state(SyncState::Syncing { 
            progress: 0.0,
            pending_changes: self.pending_change_count(),
            conflicts: 0
        });
        
        // Phase 1: Send local changes
        let local_changes = self.collect_pending_changes().await?;
        let sent_changes = self.transport.send(local_changes).await?;
        self.update_state(SyncState::Syncing { progress: 0.3, pending_changes: 0, conflicts: 0 });
        
        // Phase 2: Receive remote changes  
        let remote_changes = self.transport.receive().await?;
        self.update_state(SyncState::Syncing { progress: 0.6, pending_changes: 0, conflicts: 0 });
        
        // Phase 3: Merge and resolve conflicts
        let conflicts = self.merge_remote_changes(remote_changes).await?;
        let resolved_conflicts = self.resolve_conflicts(conflicts).await?;
        self.update_state(SyncState::Syncing { progress: 0.9, pending_changes: 0, conflicts: resolved_conflicts.len() });
        
        // Phase 4: Persist final state
        self.persist_changes().await?;
        
        self.update_state(SyncState::Synced { last_sync: SystemTime::now() });
        
        Ok(SyncResult {
            sent_changes: sent_changes.len(),
            received_changes: remote_changes.len(),
            resolved_conflicts: resolved_conflicts.len(),
        })
    }
}
```

#### Conflict Resolution Strategies

```rust
pub trait ConflictResolver<T>: Send + Sync {
    async fn resolve(&self, conflict: Conflict<T>) -> Result<Resolution<T>, ConflictError>;
}

// Automatic resolution strategies
pub struct AutoResolver<T> {
    strategy: ResolutionStrategy,
    phantom: PhantomData<T>,
}

pub enum ResolutionStrategy {
    LastWriteWins,
    FirstWriteWins, 
    MergeValues,      // For lists, sets
    UserDecision,     // Present UI for resolution
    CustomFunction(Box<dyn Fn(&T, &T) -> T + Send + Sync>),
}

// Interactive resolution with UI components
pub struct InteractiveResolver<T> {
    pending_conflicts: RwSignal<Vec<Conflict<T>>>,
    resolution_channel: broadcast::Receiver<Resolution<T>>,
}
```

### 4. Transport Layer Architecture

#### Design Pattern: Strategy Pattern + Adapter Pattern

```rust
pub trait SyncTransport: Send + Sync + 'static {
    async fn connect(&self, endpoint: &str) -> Result<(), TransportError>;
    async fn disconnect(&self) -> Result<(), TransportError>;
    
    async fn send(&self, changes: Vec<Change>) -> Result<SendResult, TransportError>;
    async fn receive(&self) -> Result<Vec<Change>, TransportError>;
    
    fn connection_status(&self) -> Signal<ConnectionStatus>;
    fn supports_feature(&self, feature: TransportFeature) -> bool;
}

// Multi-transport with automatic fallback
pub struct HybridTransport {
    transports: Vec<Box<dyn SyncTransport>>,
    active_transport: AtomicUsize,
    fallback_strategy: FallbackStrategy,
}

impl HybridTransport {
    pub fn new() -> Self {
        let mut transports: Vec<Box<dyn SyncTransport>> = Vec::new();
        
        // Primary: WebRTC for direct peer-to-peer
        if supports_webrtc() {
            transports.push(Box::new(WebRtcTransport::new()));
        }
        
        // Secondary: WebSocket for server-mediated sync
        transports.push(Box::new(WebSocketTransport::new()));
        
        // Tertiary: HTTP polling fallback
        transports.push(Box::new(HttpPollingTransport::new()));
        
        Self {
            transports,
            active_transport: AtomicUsize::new(0),
            fallback_strategy: FallbackStrategy::Exponential,
        }
    }
}
```

#### WebSocket Transport (Primary)

```rust
pub struct WebSocketTransport {
    url: String,
    connection: RwSignal<Option<web_sys::WebSocket>>,
    message_queue: Arc<Mutex<VecDeque<Message>>>,
    reconnect_strategy: ExponentialBackoff,
    heartbeat_interval: Duration,
}

impl SyncTransport for WebSocketTransport {
    async fn send(&self, changes: Vec<Change>) -> Result<SendResult, TransportError> {
        let message = Message::SyncChanges { 
            changes,
            client_id: self.client_id(),
            sequence_number: self.next_sequence(),
        };
        
        let serialized = bincode::serialize(&message)
            .map_err(TransportError::Serialization)?;
            
        self.send_raw(serialized).await?;
        
        Ok(SendResult {
            sent_bytes: serialized.len(),
            sequence_number: message.sequence_number,
        })
    }
}

// Leptos 0.8 server function integration
#[server(protocol = Websocket<BincodeEncoding, BincodeEncoding>)]
pub async fn sync_stream(
    collection_name: String,
    client_changes: BoxedStream<Vec<Change<serde_json::Value>>, ServerFnError>,
) -> Result<BoxedStream<Vec<Change<serde_json::Value>>, ServerFnError>, ServerFnError> {
    let collection = get_or_create_collection(&collection_name).await?;
    
    let response_stream = client_changes.map(move |changes| {
        // Merge changes with server state
        let server_changes = collection.merge_changes(changes?).await?;
        Ok(server_changes)
    });
    
    Ok(Box::pin(response_stream))
}
```

#### WebRTC Transport (Peer-to-Peer)

```rust
pub struct WebRtcTransport {
    peer_connections: HashMap<PeerId, RTCPeerConnection>,
    signaling_channel: Box<dyn SignalingChannel>,
    local_peer_id: PeerId,
    mesh_topology: MeshTopology,
}

impl WebRtcTransport {
    // Direct peer-to-peer synchronization without central server
    pub async fn sync_with_peer(&self, peer_id: &PeerId) -> Result<(), TransportError> {
        let connection = self.get_or_create_peer_connection(peer_id).await?;
        
        // Create data channel for sync messages
        let data_channel = connection.create_data_channel(
            "leptos-sync", 
            Some(RTCDataChannelInit { ordered: Some(true), ..Default::default() })
        ).await?;
        
        // Exchange changes directly
        let our_changes = self.collect_changes_since_last_sync(peer_id).await?;
        data_channel.send(&bincode::serialize(&our_changes)?).await?;
        
        Ok(())
    }
}
```

### 5. Collection API & Query Engine

#### Design Pattern: Repository Pattern + Observer Pattern

```rust
pub struct LocalFirstCollection<T: Mergeable + 'static> {
    name: String,
    storage: Arc<dyn LocalStorage>,
    sync_manager: Arc<SyncManager<T>>,
    
    // Reactive state
    items: RwSignal<HashMap<String, T>>,
    indexes: HashMap<String, Index<T>>,
    query_cache: Arc<Mutex<QueryCache<T>>>,
    
    // Event system
    event_bus: broadcast::Sender<CollectionEvent<T>>,
    change_listeners: Vec<Box<dyn Fn(&CollectionEvent<T>) + Send + Sync>>,
}

impl<T: Mergeable + 'static> LocalFirstCollection<T> {
    // Core CRUD operations
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
                    
                    // 2. Optimistic update - add to local state immediately
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
                    
                    // 5. Emit event
                    let _ = collection.event_bus.send(CollectionEvent::Created {
                        id: id.clone(),
                        item: item_with_id.clone(),
                    });
                    
                    Ok(item_with_id)
                }
            }
        )
    }
    
    pub fn update<F>(&self, id: &str, update_fn: F) -> Resource<Result<T, Error>>
    where
        F: FnOnce(&mut T) + Send + Sync + 'static,
    {
        let collection = self.clone();
        let id = id.to_string();
        create_resource(
            move || (),
            move |_| {
                let collection = collection.clone();
                let id = id.clone();
                async move {
                    // 1. Get current item
                    let mut current = collection.items.with_untracked(|items| {
                        items.get(&id).cloned()
                    }).ok_or(Error::NotFound)?;
                    
                    // 2. Create optimistic version
                    let mut optimistic = current.clone();
                    update_fn(&mut optimistic);
                    optimistic.increment_version();
                    optimistic.update_timestamp();
                    
                    // 3. Optimistic update
                    collection.items.update(|items| {
                        items.insert(id.clone(), optimistic.clone());
                    });
                    
                    // 4. Persist and sync (similar to create)
                    collection.storage.set(&id, &optimistic).await?;
                    collection.sync_manager.queue_change(Change::Update {
                        id: id.clone(),
                        old_data: current,
                        new_data: optimistic.clone(),
                        timestamp: now(),
                    });
                    
                    Ok(optimistic)
                }
            }
        )
    }
}
```

#### Advanced Query System

```rust
pub struct QueryBuilder<T: Mergeable> {
    collection: Arc<LocalFirstCollection<T>>,
    filters: Vec<Filter>,
    sort: Vec<SortCriterion>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl<T: Mergeable + 'static> QueryBuilder<T> {
    pub fn filter<V: Into<Value>>(mut self, field: &str, op: Op, value: V) -> Self {
        self.filters.push(Filter {
            field: field.to_string(),
            operator: op,
            value: value.into(),
        });
        self
    }
    
    pub fn sort_by(mut self, field: &str, order: Order) -> Self {
        self.sort.push(SortCriterion {
            field: field.to_string(),
            order,
        });
        self
    }
    
    // Returns reactive signal that automatically updates
    pub fn watch(self) -> Signal<Vec<T>> {
        let collection = self.collection.clone();
        let query = self.compile();
        
        create_memo(move |_| {
            // Automatically re-runs when collection.items changes
            collection.items.with(|items| {
                query.execute(items)
            })
        }).into()
    }
    
    // One-time execution with Resource for loading states
    pub fn get(self) -> Resource<Vec<T>> {
        let collection = self.collection.clone();
        let query = self.compile();
        
        create_resource(
            move || (),
            move |_| {
                let collection = collection.clone();
                let query = query.clone();
                async move {
                    // Check query cache first
                    if let Some(cached) = collection.query_cache.lock().await.get(&query.hash()) {
                        return Ok(cached.clone());
                    }
                    
                    // Execute query
                    let results = collection.items.with_untracked(|items| {
                        query.execute(items)
                    });
                    
                    // Cache results
                    collection.query_cache.lock().await.insert(query.hash(), results.clone());
                    
                    Ok(results)
                }
            }
        )
    }
}

// Compiled query for performance
pub struct CompiledQuery<T> {
    filters: Vec<CompiledFilter>,
    sort: Vec<SortCriterion>,
    limit: Option<usize>,
    hash: u64,
}

impl<T: Mergeable> CompiledQuery<T> {
    fn execute(&self, items: &HashMap<String, T>) -> Vec<T> {
        let mut results: Vec<T> = items
            .values()
            .filter(|item| self.matches_filters(item))
            .cloned()
            .collect();
        
        // Sort results
        if !self.sort.is_empty() {
            results.sort_by(|a, b| self.compare_items(a, b));
        }
        
        // Apply limit
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }
        
        results
    }
}
```

### 6. Component Library Architecture

#### Design Pattern: Composition Pattern + Provider Pattern

```rust
// Context provider for dependency injection
#[derive(Clone)]
pub struct LocalFirstContext {
    pub config: LocalFirstConfig,
    pub collections: HashMap<String, Box<dyn Any + Send + Sync>>,
    pub sync_status: RwSignal<GlobalSyncStatus>,
    pub error_handler: Arc<dyn ErrorHandler + Send + Sync>,
}

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

// High-level collection hook
pub fn use_local_first_collection<T: Mergeable + Clone + 'static>(
    name: &str,
) -> LocalFirstCollection<T> {
    let context = use_context::<LocalFirstContext>()
        .expect("LocalFirstProvider required");
        
    // Get or create collection
    context.collections
        .entry(name.to_string())
        .or_insert_with(|| {
            let storage = HybridStorage::new();
            let transport = HybridTransport::new();
            Box::new(LocalFirstCollection::new(name, storage, transport))
        })
        .downcast_ref::<LocalFirstCollection<T>>()
        .unwrap()
        .clone()
}

// Reactive sync status component
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

## Performance Architecture

### 1. Memory Management
- **Weak References**: Prevent circular references between collections and sync managers
- **Reference Counting**: Smart pointers (Arc/Rc) for shared ownership
- **Memory Pools**: Reuse allocations for frequently created objects
- **Lazy Loading**: Load data only when accessed

### 2. Computational Optimization
- **Incremental Updates**: Only recompute changed portions of queries
- **Memoization**: Cache expensive computations (query results, CRDT merges)
- **Batching**: Group operations to reduce overhead
- **Background Processing**: Non-blocking sync and conflict resolution

### 3. Bundle Size Optimization
- **Feature Flags**: Optional functionality behind feature gates
- **Tree Shaking**: Only include used components in final bundle
- **Code Splitting**: Separate chunks for advanced features
- **WASM Optimization**: Optimal WASM binary size

```toml
# Cargo.toml feature flags - Updated for 2025
[features]
default = ["hybrid-storage", "websocket"]
full = ["yjs", "automerge", "encryption", "webrtc", "opfs"]

# Optional features with current versions
yjs = ["yrs"]
automerge = ["automerge"]
encryption = ["age", "argon2"] 
webrtc = ["web-sys/RtcPeerConnection"]
opfs = ["web-sys/StorageManager"]
```

## Security Architecture

### 1. Data Protection
- **End-to-End Encryption**: Optional E2E encryption for sensitive data
- **Storage Encryption**: Encrypt data at rest in browser storage
- **Transport Security**: All network communication over TLS/WSS
- **Key Management**: Secure key derivation and storage

### 2. Access Control
- **Collection-Level Permissions**: Control access to different collections
- **Field-Level Security**: Encrypt specific fields within documents
- **User Authentication**: Integrate with existing auth systems
- **Audit Logging**: Track all data access and modifications

## Error Handling Architecture

### 1. Error Categories
```rust
#[derive(Debug, thiserror::Error)]
pub enum LeptosSyncError {
    #[error("Storage operation failed: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Network communication failed: {0}")]
    Network(#[from] NetworkError),
    
    #[error("CRDT merge conflict: {0}")]
    MergeConflict(String),
    
    #[error("Serialization failed: {0}")]
    Serialization(#[from] SerializationError),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}
```

### 2. Recovery Strategies
- **Exponential Backoff**: Automatic retry with increasing delays
- **Circuit Breaker**: Prevent cascading failures
- **Graceful Degradation**: Continue functioning with reduced capability
- **User Notification**: Clear feedback on error states and recovery options

## Deployment Architecture

### 1. Client-Side Deployment
- **Progressive Enhancement**: Works without server components
- **Service Worker**: Optional offline-first service worker
- **CDN Distribution**: Optimized for global content delivery
- **Browser Compatibility**: Graceful fallbacks for older browsers

### 2. Server-Side Integration  
- **Leptos SSR**: Full server-side rendering support
- **Edge Functions**: Deploy sync logic at CDN edge
- **Database Integration**: Connect to existing databases
- **Horizontal Scaling**: Support for multi-instance deployments

## Browser Compatibility Matrix (September 2025)

### Full Feature Support
| Browser | Version | OPFS | IndexedDB | WebRTC | WebSocket | Notes |
|---------|---------|------|-----------|--------|-----------|-------|
| Chrome  | 108+    | ✅   | ✅        | ✅     | ✅        | OPFS support |
| Edge    | 108+    | ✅   | ✅        | ✅     | ✅        | Chromium-based |
| Firefox | 110+    | ❌   | ✅        | ✅     | ✅        | No OPFS yet |
| Safari  | 16+     | ❌   | ✅        | ❌     | ✅        | No WebRTC/OPFS |

### Progressive Enhancement
- **Modern Browsers**: Full feature set with optimal performance
- **Legacy Browsers**: Core functionality with storage fallbacks
- **Mobile Browsers**: Touch-optimized UI with offline support

This architecture provides a robust foundation for building sophisticated local-first applications while maintaining the performance, type safety, and developer experience that Rust and Leptos provide.
