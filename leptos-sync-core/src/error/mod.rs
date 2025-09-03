//! Error types and handling for the core library

use thiserror::Error;

pub mod retry;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("CRDT error: {0}")]
    Crdt(String),
    #[error("Sync error: {0}")]
    Sync(String),
    #[error("Security error: {0}")]
    Security(String),
    #[error("Retry error: {0}")]
    Retry(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
