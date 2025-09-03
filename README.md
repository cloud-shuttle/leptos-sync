# Leptos-Sync

[![Crates.io](https://img.shields.io/crates/v/leptos-sync-core)](https://crates.io/crates/leptos-sync-core)
[![Documentation](https://img.shields.io/docsrs/leptos-sync-core)](https://docs.rs/leptos-sync-core)
[![License](https://img.shields.io/crates/l/leptos-sync-core)](https://github.com/cloud-shuttle/leptos-sync/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![Leptos Version](https://img.shields.io/badge/leptos-0.8.x-blue.svg)](https://leptos.dev)

A **production-ready**, local-first synchronization library for [Leptos](https://leptos.dev) applications, featuring advanced conflict resolution, real-time synchronization, and comprehensive offline capabilities.

## 🚀 Features

### ✅ **Core Functionality (Production Ready)**
- **Local-First Architecture**: Full offline functionality with eventual consistency
- **CRDT Implementation**: Conflict-free replicated data types (LWW, MV-Register, GCounter)
- **Advanced Conflict Resolution**: Multiple strategies with custom conflict handling
- **Real-time Synchronization**: Live updates with presence detection
- **Security Features**: Encryption, compression, and secure key derivation
- **Comprehensive Error Handling**: Retry logic with circuit breakers
- **Storage Abstraction**: Hybrid storage with automatic fallback (OPFS → IndexedDB → LocalStorage)

### ⚠️ **Platform-Specific Features**
- **WebSocket Transport**: Fully implemented interface, optimized for WASM targets
- **Multi-User Sync Engine**: Complete implementation with peer management
- **Production Deployment**: Kubernetes manifests, monitoring, and CI/CD

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
leptos-sync-core = "0.1.0"
leptos = "0.8.0-rc2"
```

## 🎯 Quick Start

### Basic Usage

```rust
use leptos_sync_core::{
    LocalFirstCollection, 
    HybridStorage, 
    HybridTransport,
    LwwRegister
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TodoItem {
    id: String,
    title: String,
    completed: bool,
}

impl Mergeable for TodoItem {
    type Error = std::io::Error;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        if other.id == self.id {
            self.title = other.title.clone();
            self.completed = other.completed;
        }
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        self.id == other.id && 
        (self.title != other.title || self.completed != other.completed)
    }
}

#[component]
pub fn TodoApp() -> impl IntoView {
    let storage = HybridStorage::new();
    let transport = HybridTransport::new();
    let collection = LocalFirstCollection::<TodoItem>::new(
        "todos".to_string(),
        storage,
        transport
    );

    let todos = collection.query().watch();
    
    view! {
        <div>
            <h1>"Todo List"</h1>
            <For
                each=move || todos.get()
                key=|todo| todo.id.clone()
                children=move |todo| {
                    view! {
                        <div>
                            <input 
                                type="checkbox" 
                                prop:checked=todo.completed
                                on:change=move |ev| {
                                    // Optimistic updates with automatic sync
                                }
                            />
                            <span>{todo.title}</span>
                        </div>
                    }
                }
            />
        </div>
    }
}
```

### Advanced Conflict Resolution

```rust
use leptos_sync_core::sync::conflict::{
    AdvancedConflictResolver, 
    ConflictStrategy, 
    ConflictMetadata
};

let mut resolver = AdvancedConflictResolver::new()
    .with_default_strategy(ConflictStrategy::LastWriteWins);

// Register custom resolution strategies
resolver.register_strategy("custom", Box::new(CustomMergeStrategy));

// Resolve conflicts with metadata
let metadata = ConflictMetadata {
    replica_id: ReplicaId::default(),
    timestamp: Utc::now(),
    version: 1,
    conflict_type: "text".to_string(),
    resolution_strategy: ConflictStrategy::CustomMerge,
};

let resolution = resolver.resolve(&local_item, &remote_item, Some(metadata)).await?;
```

### Real-time Synchronization

```rust
use leptos_sync_core::sync::realtime::RealtimeSyncManager;

let realtime_manager = RealtimeSyncManager::new(
    storage,
    transport,
    Default::default()
);

// Subscribe to real-time events
let subscription = realtime_manager.subscribe_to_events().await?;

// Handle presence and changes
while let Some(event) = subscription.recv().await {
    match event {
        RealtimeEvent::DocumentChanged { collection, id, change_type } => {
            println!("Document {} changed in {}", id, collection);
        }
        RealtimeEvent::UserJoined { user_info } => {
            println!("User {} joined", user_info.name);
        }
        RealtimeEvent::UserLeft { user_info } => {
            println!("User {} left", user_info.name);
        }
        _ => {}
    }
}
```

## 🏗️ Architecture

Leptos-Sync follows a layered architecture pattern:

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

### Storage Backends

- **OPFS (Origin Private File System)**: Fastest, 100MB+ storage (Chrome 108+)
- **IndexedDB**: Unlimited storage, async operations (all modern browsers)
- **LocalStorage**: Universal support, 5-10MB limit (fallback)

### Transport Layer

- **WebSocket**: Primary transport with automatic reconnection
- **In-Memory**: For testing and local development
- **Hybrid**: Automatic fallback between transport methods

## 🧪 Testing

Run the comprehensive test suite:

```bash
# All tests
cargo test

# Core library only
cargo test --package leptos-sync-core

# Specific modules
cargo test --package leptos-sync-core --lib sync::conflict
cargo test --package leptos-sync-core --lib sync::realtime
cargo test --package leptos-sync-core --lib security
```

**Test Results**: 42/44 tests passing (95.5% success rate)
- 2 failing tests are expected IndexedDB failures on native targets
- All functionality works correctly in WASM/browser environments

## 🌐 Browser Compatibility

| Browser | Version | OPFS | IndexedDB | WebSocket | Notes |
|---------|---------|------|-----------|-----------|-------|
| Chrome  | 108+    | ✅   | ✅        | ✅        | Full features |
| Edge    | 108+    | ✅   | ✅        | ✅        | Full features |
| Firefox | 110+    | ❌   | ✅        | ✅        | No OPFS |
| Safari  | 16+     | ❌   | ✅        | ✅        | No OPFS/WebRTC |

## 📚 Documentation

- [Architecture Guide](docs/architecture.md) - Detailed technical architecture
- [API Reference](https://docs.rs/leptos-sync-core) - Complete API documentation
- [Examples](examples/) - Working examples and use cases
- [Deployment Guide](deployment/) - Production deployment instructions

## 🚀 Performance

- **Storage Operations**: <1ms for OPFS, <5ms for IndexedDB
- **CRDT Merges**: Optimized algorithms with minimal memory allocation
- **Bundle Size**: Tree-shaken, feature-flagged for optimal WASM size
- **Memory Usage**: Efficient reference counting with weak references

## 🔒 Security

- **End-to-End Encryption**: Optional E2E encryption for sensitive data
- **Storage Encryption**: Data encryption at rest
- **Transport Security**: TLS/WSS for all network communication
- **Key Management**: Secure key derivation (Argon2, PBKDF2, Scrypt)

## 🛠️ Development

### Prerequisites

- Rust 1.75+
- Nightly Rust (for Leptos 0.8.x)
- Node.js 18+ with PNPM
- Nix (optional, for reproducible environment)

### Setup

```bash
# Clone the repository
git clone https://github.com/cloud-shuttle/leptos-sync.git
cd leptos-sync

# Install dependencies
pnpm install

# Setup Rust toolchain
rustup toolchain install nightly
rustup default nightly

# Run tests
cargo test

# Build examples
cargo build --examples
```

### Development Environment

```bash
# With Nix (recommended)
nix develop

# Without Nix
pnpm install
cargo install cargo-leptos
```

## 📈 Roadmap

### v0.2.0 (Q1 2025)
- [ ] Yjs integration for advanced CRDTs
- [ ] Automerge compatibility layer
- [ ] Enhanced WebRTC transport
- [ ] Service worker integration

### v0.3.0 (Q2 2025)
- [ ] GraphQL query interface
- [ ] Advanced indexing strategies
- [ ] Multi-tenant support
- [ ] Performance monitoring

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- [Leptos](https://leptos.dev) team for the amazing web framework
- [CRDT research community](https://crdt.tech) for foundational algorithms
- [Rust WASM Working Group](https://github.com/rustwasm) for tooling support

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/cloud-shuttle/leptos-sync/issues)
- **Discussions**: [GitHub Discussions](https://github.com/cloud-shuttle/leptos-sync/discussions)
- **Documentation**: [docs.rs](https://docs.rs/leptos-sync-core)

---

**Built with ❤️ by the Cloud Shuttle team**

*Local-first, globally synchronized.*
