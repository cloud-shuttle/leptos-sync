# leptos-sync-core

Core synchronization library for Leptos applications, providing local-first data synchronization with advanced conflict resolution and real-time capabilities.

## Features

- **CRDT Implementation**: Conflict-free replicated data types (LWW, MV-Register, GCounter)
- **Advanced Conflict Resolution**: Multiple strategies with custom conflict handling
- **Real-time Synchronization**: Live updates with presence detection
- **Security Features**: Encryption, compression, and secure key derivation
- **Storage Abstraction**: Hybrid storage with automatic fallback
- **Transport Layer**: WebSocket, in-memory, and hybrid transport options

## Usage

```rust
use leptos_sync_core::{
    LocalFirstCollection, 
    HybridStorage, 
    HybridTransport,
    LwwRegister
};

// Create a collection with automatic sync
let storage = HybridStorage::new();
let transport = HybridTransport::new();
let collection = LocalFirstCollection::<TodoItem>::new(
    "todos".to_string(),
    storage,
    transport
);

// Use reactive queries
let todos = collection.query().watch();
```

## Documentation

See the [main project README](../../README.md) for comprehensive documentation and examples.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
