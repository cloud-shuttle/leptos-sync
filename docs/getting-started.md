# Getting Started with Leptos-Sync

Welcome to Leptos-Sync! This guide will walk you through setting up and building your first collaborative application with real-time synchronization.

## 🚀 Quick Start

### Prerequisites

- **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
- **Node.js**: Install Node.js for build tools
- **Basic Rust Knowledge**: Familiarity with Rust syntax and concepts

### Installation

Add Leptos-Sync to your `Cargo.toml`:

```toml
[dependencies]
leptos-sync-core = "0.8.0"
leptos-sync-components = "0.8.0"
leptos = { version = "0.8", features = ["csr"] }
serde = { version = "1.0", features = ["derive"] }
```

## 📚 Core Concepts

### What is Leptos-Sync?

Leptos-Sync is a **local-first synchronization library** that enables:
- **Real-time collaboration** between multiple users
- **Offline-first functionality** with automatic sync when online
- **Conflict-free data** using CRDTs (Conflict-free Replicated Data Types)
- **Instant UI updates** with reactive data binding

### Key Components

1. **CRDTs**: Mathematical data structures that automatically resolve conflicts
2. **Storage**: Local data persistence (memory, IndexedDB, file system)
3. **Transport**: Data synchronization layer (WebSocket, HTTP, in-memory)
4. **Collections**: High-level data management with automatic sync

## 🏗️ Building Your First App

### Step 1: Basic CRDT Usage

Let's start with a simple counter that syncs across devices:

```rust
use leptos::*;
use leptos_sync_core::{
    crdt::GCounter,
    collection::LocalFirstCollection,
    storage::Storage,
    transport::memory::InMemoryTransport,
};

#[component]
fn SyncCounter() -> impl IntoView {
    let (counter, set_counter) = create_signal(0);
    
    // Initialize sync collection
    let storage = Storage::memory();
    let transport = InMemoryTransport::new();
    let collection = LocalFirstCollection::<GCounter, _>::new(storage, transport);
    
    let increment = move |_| {
        set_counter.update(|c| *c += 1);
    };
    
    view! {
        <div>
            <h1>"Counter: " {counter}</h1>
            <button on:click=increment>"Increment"</button>
        </div>
    }
}
```

### Step 2: Collaborative Text Editor

Now let's build something more complex - a collaborative text editor:

```rust
use leptos_sync_core::crdt::LwwRegister;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Document {
    content: String,
    last_modified: chrono::DateTime<chrono::Utc>,
}

#[component]
fn CollaborativeEditor() -> impl IntoView {
    let (content, set_content) = create_signal(String::new());
    let (document_id, _) = create_signal("doc-1".to_string());
    
    // Create CRDT wrapper for document
    let storage = Storage::memory();
    let transport = InMemoryTransport::new();
    let collection = LocalFirstCollection::<LwwRegister<Document>, _>::new(storage, transport);
    
    let update_content = move |ev| {
        let new_content = event_target_value(&ev);
        set_content.set(new_content.clone());
        
        // Update in sync collection
        let document = Document {
            content: new_content,
            last_modified: chrono::Utc::now(),
        };
        
        spawn_local(async move {
            let _ = collection.insert(&document_id.get(), &LwwRegister::new(document, ReplicaId::default())).await;
        });
    };
    
    view! {
        <div>
            <h2>"Collaborative Editor"</h2>
            <textarea
                prop:value=content
                on:input=update_content
                class="w-full h-64 p-4 border rounded"
                placeholder="Start typing..."
            />
        </div>
    }
}
```

### Step 3: Real-Time Todo App

Let's build the collaborative todo app from our examples:

```rust
use leptos_sync_core::{
    crdt::{LwwMap, LwwRegister},
    collection::LocalFirstCollection,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TodoItem {
    id: String,
    text: String,
    completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TodoCRDT {
    item: LwwRegister<TodoItem>,
    metadata: LwwMap<String, String>,
}

#[component]
fn TodoApp() -> impl IntoView {
    let (todos, set_todos) = create_signal(Vec::new());
    let (new_todo, set_new_todo) = create_signal(String::new());
    
    let add_todo = move |_| {
        let text = new_todo.get();
        if !text.trim().is_empty() {
            let item = TodoItem {
                id: uuid::Uuid::new_v4().to_string(),
                text,
                completed: false,
            };
            
            set_todos.update(|list| list.push(item));
            set_new_todo.set(String::new());
        }
    };
    
    view! {
        <div>
            <h1>"Collaborative Todo App"</h1>
            <div class="flex gap-2 mb-4">
                <input
                    type="text"
                    prop:value=new_todo
                    on:input=move |ev| set_new_todo.set(event_target_value(&ev))
                    placeholder="Add new todo..."
                    class="flex-1 px-3 py-2 border rounded"
                />
                <button on:click=add_todo class="px-4 py-2 bg-blue-500 text-white rounded">
                    "Add"
                </button>
            </div>
            
            <div class="space-y-2">
                {move || {
                    todos.get().into_iter().map(|todo| {
                        view! {
                            <div class="flex items-center gap-2 p-3 border rounded">
                                <input type="checkbox" checked={todo.completed} />
                                <span>{todo.text}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}
```

## 🔧 Advanced Features

### DevTools - Debugging and Monitoring

Leptos-Sync includes comprehensive DevTools for debugging and monitoring your collaborative applications:

```rust
use leptos_sync_core::{DevTools, DevToolsConfig, CrdtInspector};

// Create DevTools with monitoring enabled
let devtools_config = DevToolsConfig {
    enable_crdt_inspection: true,
    enable_sync_monitoring: true,
    enable_transport_monitoring: true,
    enable_performance_metrics: true,
    max_events: 1000,
};

let devtools = Arc::new(DevTools::new(devtools_config));
let inspector = CrdtInspector::new(devtools.clone());

// Monitor CRDT operations
devtools.record_crdt_operation(
    "user-data".to_string(),
    "update".to_string(),
    replica_id.clone()
).await;

// Inspect CRDT state
let inspection = inspector.inspect_crdt(&register, "user-data".to_string()).await;
println!("CRDT Memory Usage: {} bytes", inspection.memory_usage_bytes);

// Export debugging data
let debug_data = devtools.export_data().await?;
```

**DevTools Features:**
- **CRDT Monitoring**: Track operations, memory usage, and state changes
- **Sync Analytics**: Monitor success rates, performance, and conflicts
- **Transport Health**: Track connections, messages, and errors
- **Performance Metrics**: Memory usage, CPU, and throughput monitoring
- **Event Analysis**: Filter and analyze all recorded events
- **Data Export**: Complete debugging data for offline analysis

See the [DevTools Guide](devtools-guide.md) for comprehensive documentation.

### Enhanced CRDT Types

Leptos-Sync includes advanced CRDT types for complex data structures:

#### List CRDTs
```rust
use leptos_sync_core::crdt::list::AddWinsList;

let mut todo_list = AddWinsList::new();
let element_id = todo_list.add("Buy groceries".to_string(), replica_id.clone());
todo_list.update(element_id, "Buy organic groceries".to_string(), replica_id.clone());
todo_list.remove(element_id, replica_id.clone());

// Get all visible elements
let items: Vec<String> = todo_list.get_all().into_iter().collect();
```

#### Tree CRDTs
```rust
use leptos_sync_core::crdt::tree::AddWinsTree;

let mut file_tree = AddWinsTree::new();
let root_id = file_tree.add_root("Documents".to_string(), replica_id.clone());
let folder_id = file_tree.add_child(root_id, "Projects".to_string(), replica_id.clone());
let file_id = file_tree.add_child(folder_id, "README.md".to_string(), replica_id.clone());

// Move files between folders
file_tree.move_node(file_id, root_id, replica_id.clone());
```

#### Graph CRDTs
```rust
use leptos_sync_core::crdt::graph::AddWinsGraph;

let mut social_graph = AddWinsGraph::new();
let alice_id = social_graph.add_vertex("Alice".to_string(), replica_id.clone());
let bob_id = social_graph.add_vertex("Bob".to_string(), replica_id.clone());
let edge_id = social_graph.add_edge(alice_id, bob_id, replica_id.clone());

// Find shortest path
let path = social_graph.shortest_path(alice_id, bob_id);
```

### Custom CRDT Types

You can create your own CRDT types by implementing the `Mergeable` trait:

```rust
use leptos_sync_core::crdt::Mergeable;
use std::io;

#[derive(Debug, Clone, PartialEq)]
struct CustomData {
    value: String,
    timestamp: u64,
}

impl Mergeable for CustomData {
    type Error = io::Error;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Custom merge logic - take the newer timestamp
        if other.timestamp > self.timestamp {
            *self = other.clone();
        }
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        // Define when two instances conflict
        self.value != other.value && self.timestamp == other.timestamp
    }
}
```

### Persistent Storage

Use IndexedDB for browser-based persistence:

```rust
use leptos_sync_core::storage::Storage;

// In browser environment
let storage = Storage::indexeddb("my-app-db").await?;

// Fallback to memory if IndexedDB fails
let storage = Storage::indexeddb("my-app-db")
    .await
    .unwrap_or_else(|_| Storage::memory());
```

### Multi-Transport Support

Leptos-Sync supports dynamic transport switching with automatic fallbacks:

```rust
use leptos_sync_core::{MultiTransport, MultiTransportConfig, TransportType, TransportEnum};

// Configure multi-transport with fallbacks
let transport_config = MultiTransportConfig {
    primary: TransportType::WebSocket,
    fallbacks: vec![TransportType::Memory],
    auto_switch: true,
    timeout_ms: 5000,
};

let mut multi_transport = MultiTransport::new(transport_config);

// Register different transport types
let websocket_transport = TransportEnum::WebSocket(/* WebSocket config */);
let memory_transport = TransportEnum::InMemory(InMemoryTransport::new());

multi_transport.register_transport(TransportType::WebSocket, websocket_transport);
multi_transport.register_transport(TransportType::Memory, memory_transport);

// Transport automatically switches on failure
// WebSocket fails → automatically switches to Memory transport
```

### WebSocket Transport

For real-time synchronization over the network:

```rust
use leptos_sync_core::transport::websocket::WebSocketTransport;

let transport = WebSocketTransport::new("ws://localhost:8080/sync")
    .with_reconnect_config(ReconnectConfig::default())
    .build()
    .await?;
```

## 🧪 Testing Your App

### Running Examples

```bash
# Run the collaborative todo app
cd examples/collaborative_todo
cargo run

# Run the DevTools demo
cargo run --bin devtools-demo

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Testing CRDT Properties

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_crdt_commutativity(a: u64, b: u64) {
            let mut crdt1 = GCounter::new();
            let mut crdt2 = GCounter::new();
            
            crdt1.increment(ReplicaId::default());
            crdt2.increment(ReplicaId::default());
            
            let mut result1 = crdt1.clone();
            let mut result2 = crdt2.clone();
            
            result1.merge(&crdt2)?;
            result2.merge(&crdt1)?;
            
            assert_eq!(result1, result2);
        }
    }
}
```

## 🚀 Deployment

### Browser Deployment

1. **Build for WASM**:
```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/your_app.wasm --out-dir dist
```

2. **Serve with a web server**:
```bash
python3 -m http.server 8000
# or
npx serve dist
```

### Server Integration

For production use, you'll want to implement a proper sync server:

```rust
use leptos_sync_core::transport::websocket::WebSocketServer;

#[tokio::main]
async fn main() {
    let server = WebSocketServer::new("0.0.0.0:8080")
        .with_sync_engine(sync_engine)
        .build()
        .await?;
    
    server.run().await?;
}
```

### Step 4: Production WebSocket Transport (v0.8.0)

For production applications, use the new `leptos-ws-pro` integration:

```rust
use leptos_sync_core::{
    transport::leptos_ws_pro_transport::{LeptosWsProTransport, LeptosWsProConfig},
    transport::hybrid_transport_impl::HybridTransport,
};

#[component]
fn ProductionApp() -> impl IntoView {
    // Configure production WebSocket transport
    let config = LeptosWsProConfig {
        url: "wss://your-sync-server.com/ws".to_string(),
        max_reconnect_attempts: 5,
        retry_delay: Duration::from_secs(1),
        heartbeat_interval: Duration::from_secs(30),
    };
    
    // Create hybrid transport with fallback
    let primary_transport = LeptosWsProTransport::new(config);
    let fallback_transport = InMemoryTransport::new();
    let transport = HybridTransport::with_fallback(
        HybridTransport::LeptosWsPro(primary_transport),
        HybridTransport::InMemory(fallback_transport)
    );
    
    // Initialize collection with production transport
    let storage = Storage::indexed_db("my-app-db");
    let collection = LocalFirstCollection::<GCounter, _>::new(storage, transport);
    
    // Your app logic here...
    view! {
        <div>
            <h1>"Production-Ready Sync App"</h1>
            // Your UI components
        </div>
    }
}
```

**Key Benefits:**
- **Production-Ready**: Built on `leptos-ws-pro` for robust WebSocket communication
- **Automatic Fallback**: Falls back to in-memory transport if WebSocket fails
- **Error Recovery**: Circuit breaker pattern with automatic reconnection
- **Performance**: Optimized for real-time collaboration

## 📖 Next Steps

### Learn More

- **CRDT Fundamentals**: Understand the math behind conflict resolution
- **Advanced Patterns**: Explore complex synchronization scenarios
- **Performance Tuning**: Optimize your app with our benchmarking tools
- **Community**: Join discussions and share your projects

### Examples to Explore

- **Real-time Dashboard**: Build a collaborative analytics dashboard
- **Document Editor**: Create a Google Docs-like collaborative editor
- **Chat Application**: Build a real-time chat with message history
- **Game State Sync**: Synchronize game state across multiple players

### Resources

- **API Reference**: Complete API documentation
- **Examples Repository**: Working code examples
- **Benchmark Results**: Performance characteristics
- **Community Discord**: Get help and share ideas

## 🆘 Getting Help

### Common Issues

1. **WASM Build Errors**: Ensure you have `wasm-pack` installed
2. **Sync Conflicts**: Check your CRDT implementation
3. **Performance Issues**: Use our benchmarking tools to identify bottlenecks

### Support Channels

- **GitHub Issues**: Report bugs and request features
- **Discord Community**: Real-time help and discussion
- **Documentation**: Comprehensive guides and examples
- **Examples**: Working code you can run and modify

---

**Ready to build something amazing?** Start with the simple examples above, then gradually add complexity as you become comfortable with the concepts. Remember, Leptos-Sync handles the hard parts of synchronization - you focus on building great user experiences!

Happy coding! 🚀
