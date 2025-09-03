//! Leptos-Sync Core Library
//! 
//! This library provides local-first, offline-capable data synchronization
//! for Leptos applications using CRDTs and conflict resolution.

pub mod storage;
pub mod crdt;
pub mod sync;
pub mod error;
pub mod collection;
pub mod query;
pub mod transport;

pub use collection::LocalFirstCollection;
pub use error::{Error, Result};

/// Re-export common dependencies for convenience
pub use serde;
pub use serde_json;
pub use uuid;

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
