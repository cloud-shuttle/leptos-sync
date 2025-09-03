# Leptos Local-First Library Design Specification

## Overview

**Library Name:** `leptos-sync`  
**Version:** 0.1.0  
**Target:** Leptos 0.8.x, Rust 1.75+, WASM, and native Rust  
**License:** MIT/Apache-2.0  
**Last Updated:** September 3rd, 2025

A comprehensive local-first library for Leptos that provides seamless offline functionality, real-time synchronization, and collaborative features while maintaining type safety and minimal bundle size.

## Core Architecture

### 1. Storage Layer Abstraction

```rust
// Unified storage trait that works across all storage backends
pub trait LocalStorage: Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync;
    
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), Self::Error>;
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
    async fn keys(&self) -> Result<Vec<String>, Self::Error>;
    async fn clear(&self) -> Result<(), Self::Error>;
}

// Storage implementations
pub struct OpfsStorage { /* Origin Private File System - Chrome 108+ */ }
pub struct IndexedDbStorage { /* Via web-sys IndexedDB - Universal support */ }
pub struct LocalStorageBackend { /* Browser localStorage - Universal support */ }
pub struct MemoryStorage { /* In-memory for testing */ }
pub struct HybridStorage { /* Automatic tiering */ }

impl HybridStorage {
    pub fn new() -> Self {
        // Automatically selects best available storage:
        // 1. OPFS (if available) - fastest, most reliable, Chrome 108+
        // 2. IndexedDB - good performance, wide support, all modern browsers
        // 3. LocalStorage - fallback for simple data, universal support
    }
}
```

### 2. CRDT Integration

```rust
// Trait for CRDT-enabled types
pub trait Mergeable: Clone + Serialize + DeserializeOwned + Send + Sync + PartialEq {
    fn merge(&mut self, other: &Self);
    fn conflicts(&self, other: &Self) -> Vec<Conflict>;
}

// Derive macro for automatic CRDT implementation
#[derive(LocalFirst, Serialize, Deserialize, Clone)]
#[local_first(crdt = "lww")] // last-write-wins
pub struct TodoItem {
    #[local_first(id)]
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    #[local_first(version)]
    pub version: u64,
    #[local_first(timestamp)]
    pub modified: f64,
}

// For complex CRDTs, integrate with actively maintained libraries
pub enum CrdtBackend {
    Yjs(yrs::Doc),           // For text collaboration - yrs 0.9+
    Automerge(automerge::Document), // For structured data - automerge 0.4+
    Loro(loro::LoroDoc),     // For high-performance scenarios - loro 0.3+
    Custom(Box<dyn Mergeable>),
}
```

### 3. Reactive Sync State

```rust
use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub enum SyncStatus {
    Offline,
    Syncing { progress: f32 },
    Synced,
    Error(String),
}

#[derive(Clone)]
pub struct SyncState {
    status: RwSignal<SyncStatus>,
    pending_changes: RwSignal<usize>,
    last_sync: RwSignal<Option<SystemTime>>,
    peers: RwSignal<Vec<PeerId>>,
}

// Reactive sync manager
pub struct SyncManager<T: Mergeable> {
    state: SyncState,
    storage: Arc<dyn LocalStorage>,
    transport: SyncTransport,
    phantom: PhantomData<T>,
}
```

### 4. Network Transport Layer

```rust
pub enum SyncTransport {
    WebSocket(WebSocketTransport),
    WebRTC(WebRtcTransport),
    Hybrid(HybridTransport),
    ServerFunction(ServerFnTransport),
}

// Leverage Leptos 0.8's WebSocket server functions
#[server(protocol = Websocket<BincodeEncoding, BincodeEncoding>)]
pub async fn sync_stream<T: Mergeable>(
    collection: String,
    changes: BoxedStream<Vec<Change<T>>, ServerFnError>,
) -> Result<BoxedStream<Vec<Change<T>>, ServerFnError>, ServerFnError> {
    // Bidirectional sync stream
}

pub struct HybridTransport {
    websocket: Option<WebSocketTransport>,
    webrtc: Option<WebRtcTransport>,
    // Automatic fallback and upgrade logic
}
```

### 5. Collection API

```rust
// Main collection type for local-first data
pub struct LocalFirstCollection<T: Mergeable + 'static> {
    name: String,
    storage: Arc<dyn LocalStorage>,
    sync: SyncManager<T>,
    items: RwSignal<HashMap<String, T>>,
    query_cache: Arc<Mutex<QueryCache<T>>>,
}

impl<T: Mergeable + 'static> LocalFirstCollection<T> {
    // Core CRUD operations
    pub fn create(&self, item: T) -> Resource<Result<T, Error>>;
    pub fn read(&self, id: &str) -> Resource<Option<T>>;
    pub fn update(&self, id: &str, f: impl Fn(&mut T)) -> Resource<Result<T, Error>>;
    pub fn delete(&self, id: &str) -> Resource<Result<(), Error>>;
    
    // Query API with reactive updates
    pub fn query(&self) -> QueryBuilder<T>;
    pub fn watch(&self, id: &str) -> Signal<Option<T>>;
    pub fn watch_many(&self, ids: &[String]) -> Signal<Vec<T>>;
    
    // Sync operations
    pub fn sync(&self) -> impl Future<Output = Result<(), Error>>;
    pub fn sync_status(&self) -> Signal<SyncStatus>;
}
```

### 6. Query Builder with Offline Support

```rust
pub struct QueryBuilder<T> {
    collection: Arc<LocalFirstCollection<T>>,
    filters: Vec<Filter>,
    sort: Option<SortOrder>,
    limit: Option<usize>,
}

impl<T: Mergeable + 'static> QueryBuilder<T> {
    pub fn filter(mut self, field: &str, op: Op, value: Value) -> Self;
    pub fn sort_by(mut self, field: &str, order: Order) -> Self;
    pub fn limit(mut self, n: usize) -> Self;
    
    // Returns reactive signal that updates with changes
    pub fn watch(self) -> Signal<Vec<T>> {
        let collection = self.collection.clone();
        create_memo(move |_| {
            // Re-runs when collection items change
            self.execute_local()
        })
    }
    
    // One-time execution
    pub fn get(self) -> Resource<Vec<T>>;
}
```

### 7. Leptos Component Integration

```rust
// High-level components for common patterns
#[component]
pub fn LocalFirstProvider(
    children: Children,
    #[prop(optional)] config: Option<LocalFirstConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let context = create_rw_signal(LocalFirstContext::new(config));
    provide_context(context);
    children()
}

#[component]
pub fn SyncStatusIndicator() -> impl IntoView {
    let ctx = use_context::<LocalFirstContext>()
        .expect("LocalFirstProvider required");
    let status = ctx.sync_status();
    
    view! {
        <div class="sync-status">
            {move || match status.get() {
                SyncStatus::Offline => view! { <span>"Offline"</span> },
                SyncStatus::Syncing { progress } => view! {
                    <span>"Syncing "{progress}"%"</span>
                },
                SyncStatus::Synced => view! { <span>"Synced"</span> },
                SyncStatus::Error(e) => view! { <span class="error">{e}</span> },
            }}
        </div>
    }
}

#[component]
pub fn OfflineFirst<T, F, IV>(
    resource: Resource<T>,
    fallback: F,
    children: impl Fn(T) -> IV + 'static,
) -> impl IntoView 
where
    T: Clone + 'static,
    F: Fn() -> View + 'static,
    IV: IntoView,
{
    // Handles loading states with offline-cached data
}
```

### 8. Conflict Resolution UI

```rust
#[component]
pub fn ConflictResolver<T: Mergeable + Display>(
    conflicts: ReadSignal<Vec<Conflict<T>>>,
    on_resolve: impl Fn(ConflictResolution<T>) + 'static,
) -> impl IntoView {
    view! {
        <div class="conflict-resolver">
            <For
                each=move || conflicts.get()
                key=|c| c.id.clone()
                children=move |conflict| view! {
                    <ConflictItem conflict on_resolve />
                }
            />
        </div>
    }
}
```

### 9. Advanced Features

```rust
// Optimistic UI updates
pub struct OptimisticUpdate<T> {
    local_id: Uuid,
    operation: Operation<T>,
    rollback: Option<Box<dyn Fn()>>,
}

impl<T: Mergeable> LocalFirstCollection<T> {
    pub fn optimistic_update(&self, update: impl Fn(&mut T)) -> OptimisticHandle {
        // Apply immediately, rollback if sync fails
    }
}

// Partial replication (inspired by Electric SQL's shapes)
pub struct ReplicationShape {
    tables: Vec<String>,
    filter: Option<String>,
    depth: usize,
}

// End-to-end encryption
pub struct EncryptedCollection<T> {
    inner: LocalFirstCollection<EncryptedData<T>>,
    key_manager: KeyManager,
}

// Presence and awareness (for collaboration)
#[derive(Clone, Serialize, Deserialize)]
pub struct Presence {
    user_id: String,
    cursor: Option<CursorPosition>,
    selection: Option<Selection>,
    metadata: HashMap<String, Value>,
}
```

### 10. Testing Utilities

```rust
// Test helpers for local-first scenarios
pub struct LocalFirstTest {
    storage: MemoryStorage,
    network: MockNetwork,
    time: MockTime,
}

impl LocalFirstTest {
    pub fn new() -> Self;
    pub fn simulate_offline(&mut self);
    pub fn simulate_latency(&mut self, ms: u64);
    pub fn simulate_concurrent_edit(&mut self, edits: Vec<Edit>);
    pub fn assert_eventually_consistent(&self);
}

// Property-based testing for CRDTs
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn crdt_merge_associative(a: TodoItem, b: TodoItem, c: TodoItem) {
            // (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
        }
    }
}
```

## Example Usage

```rust
use leptos::*;
use leptos_sync::*;

#[derive(LocalFirst, Serialize, Deserialize, Clone, Debug)]
#[local_first(crdt = "lww")]
struct Task {
    #[local_first(id)]
    id: Uuid,
    title: String,
    completed: bool,
    #[local_first(version)]
    version: u64,
}

#[component]
fn TodoApp() -> impl IntoView {
    // Initialize local-first collection
    let tasks = create_local_first_collection::<Task>("tasks")
        .with_sync(SyncTransport::hybrid())
        .with_storage(HybridStorage::new())
        .build();
    
    // Create reactive query
    let incomplete_tasks = tasks
        .query()
        .filter("completed", Op::Eq, false.into())
        .sort_by("created_at", Order::Desc)
        .watch();
    
    // Handle new task creation with optimistic updates
    let add_task = move |title: String| {
        tasks.optimistic_create(Task {
            id: Uuid::new_v4(),
            title,
            completed: false,
            version: 0,
        });
    };
    
    view! {
        <LocalFirstProvider>
            <div class="todo-app">
                <SyncStatusIndicator />
                <TaskInput on_submit=add_task />
                <TaskList tasks=incomplete_tasks />
                <ConflictResolver />
            </div>
        </LocalFirstProvider>
    }
}
```

## Performance Targets

- **Bundle Size**: < 50KB gzipped (core library)
- **Initial Sync**: < 100ms for 1000 items
- **CRDT Merge**: < 1ms for typical operations
- **Query Performance**: < 10ms for 10,000 items
- **Memory Usage**: < 10MB for 10,000 items

## Migration Path

```rust
// Easy migration from existing Leptos apps
// Before:
let (tasks, set_tasks) = create_signal(vec![]);

// After:
let tasks = create_local_first_collection::<Task>("tasks").build();
```

## Configuration

```toml
# Cargo.toml - Updated for 2025 ecosystem
[dependencies]
leptos = "0.8"  # Latest stable Leptos
leptos-ws = "0.8"  # WebSocket support
web-sys = "0.3"    # Latest web-sys
wasm-bindgen = "0.2"  # Latest wasm-bindgen
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }

# CRDT libraries - actively maintained as of 2025
yrs = "0.9"  # Yjs Rust bindings
automerge = "0.4"  # Automerge Rust
loro = "0.3"  # Loro CRDT library

[features]
default = ["hydrate", "hybrid-storage"]
yjs = ["yrs"]
automerge = ["automerge"]
encryption = ["age", "argon2"]
websocket = ["leptos-ws"]
webrtc = ["webrtc-rs"]
hybrid-storage = ["opfs", "indexeddb", "localstorage"]
```

## Browser Compatibility (September 2025)

### Full Support (All Features)
- **Chrome**: 108+ (OPFS, IndexedDB, WebRTC, WebSocket)
- **Firefox**: 110+ (IndexedDB, WebRTC, WebSocket)
- **Safari**: 16+ (IndexedDB, WebSocket)
- **Edge**: 108+ (OPFS, IndexedDB, WebRTC, WebSocket)

### Partial Support (Core Features)
- **Chrome**: 90+ (IndexedDB, WebSocket)
- **Firefox**: 90+ (IndexedDB, WebSocket)
- **Safari**: 14+ (IndexedDB, WebSocket)
- **Edge**: 90+ (IndexedDB, WebSocket)

### Fallback Support (Basic Features)
- **All Modern Browsers**: LocalStorage fallback
- **Legacy Browsers**: Memory-only mode with sync disabled

## Deployment Considerations

1. **Islands Architecture**: Components using local-first data automatically become islands
2. **Service Worker**: Auto-generated for offline support
3. **CDN Compatible**: Sync endpoints can be deployed separately
4. **Edge Functions**: Sync logic runs at edge for low latency
5. **Progressive Enhancement**: Graceful degradation for older browsers

This design provides a production-ready, type-safe, and performant foundation for building local-first applications with Leptos 0.8.x, combining the best patterns from the ecosystem with Rust's unique advantages.
