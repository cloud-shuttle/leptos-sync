//! Error types for tree CRDT operations

use std::error::Error;

/// Custom error type for tree operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeError {
    message: String,
}

impl TreeError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeError: {}", self.message)
    }
}

impl Error for TreeError {}
