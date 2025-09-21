//! IndexedDB-specific error types

use super::super::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexedDbError {
    #[error("IndexedDB not supported: {0}")]
    NotSupported(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Object store error: {0}")]
    ObjectStoreError(String),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("Quota exceeded")]
    QuotaExceeded,
    #[error("Database corrupted: {0}")]
    DatabaseCorrupted(String),
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
}

impl From<IndexedDbError> for StorageError {
    fn from(err: IndexedDbError) -> Self {
        match err {
            IndexedDbError::NotSupported(msg) => StorageError::Unsupported(msg),
            IndexedDbError::DatabaseError(msg) => StorageError::OperationFailed(msg),
            IndexedDbError::TransactionError(msg) => StorageError::OperationFailed(msg),
            IndexedDbError::ObjectStoreError(msg) => StorageError::OperationFailed(msg),
            IndexedDbError::RequestError(msg) => StorageError::OperationFailed(msg),
            IndexedDbError::SerializationError(msg) => StorageError::Serialization(
                serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, msg)),
            ),
            IndexedDbError::OperationFailed(msg) => StorageError::OperationFailed(msg),
            IndexedDbError::QuotaExceeded => {
                StorageError::OperationFailed("Storage quota exceeded".to_string())
            }
            IndexedDbError::DatabaseCorrupted(msg) => {
                StorageError::OperationFailed(format!("Database corrupted: {}", msg))
            }
            IndexedDbError::MigrationFailed(msg) => {
                StorageError::OperationFailed(format!("Migration failed: {}", msg))
            }
            IndexedDbError::VersionMismatch { expected, actual } => StorageError::OperationFailed(
                format!("Version mismatch: expected {}, got {}", expected, actual),
            ),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for IndexedDbError {
    fn from(js_value: wasm_bindgen::JsValue) -> Self {
        let error_msg = if let Some(error) = js_value.dyn_ref::<js_sys::Error>() {
            error.message()
        } else {
            "Unknown JavaScript error".to_string()
        };
        IndexedDbError::OperationFailed(error_msg)
    }
}
