//! Core synchronization library for Leptos applications
//! 
//! This crate provides the foundation for local-first, offline-capable data synchronization
//! using CRDTs (Conflict-free Replicated Data Types).

pub mod collection;
pub mod crdt;
pub mod error;
pub mod query;
pub mod storage;
pub mod sync;
pub mod transport;
pub mod security;

#[cfg(test)]
mod wasm_tests;

// Re-export main types for convenience
pub use collection::LocalFirstCollection;
pub use crdt::{LwwMap, LwwRegister, Mergeable, ReplicaId};
pub use error::{CoreError, Result};
pub use storage::{LocalStorage, StorageError};
pub use sync::{SyncError, SyncState};
pub use transport::{SyncTransport, TransportError};

// Re-export common traits and types
pub use serde::{Deserialize, Serialize};
pub use async_trait::async_trait;

/// Features available in this crate
pub mod features {
    /// Enable encryption support
    pub const ENCRYPTION: &str = "encryption";
    
    /// Enable compression support
    pub const COMPRESSION: &str = "compression";
    
    /// Enable metrics collection
    pub const METRICS: &str = "metrics";
    
    /// Enable distributed tracing
    pub const TRACING: &str = "tracing";
}
