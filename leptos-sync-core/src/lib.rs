//! Core synchronization library for Leptos applications
//!
//! This crate provides the foundation for local-first, offline-capable data synchronization
//! using CRDTs (Conflict-free Replicated Data Types).

pub mod collection;
pub mod crdt;
pub mod devtools;
pub mod error;
pub mod memory_pool;
pub mod query;
pub mod reliability;
pub mod serialization;
pub mod storage;
pub mod sync;
pub mod transport;
pub mod validation;

// Re-export multi-transport functionality
pub use transport::multi_transport::{
    MultiTransport, MultiTransportConfig, TransportEnum, TransportType,
};

// Re-export devtools functionality
pub use devtools::{
    CrdtInspection, CrdtInspector, DevTools, DevToolsConfig, DevToolsEvent, DevToolsExport,
    PerformanceMetrics, SyncStats, TransportStats,
};

// Re-export reliability functionality
pub use reliability::{ReliabilityConfig, ReliabilityError, ReliabilityManager};

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
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};

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
