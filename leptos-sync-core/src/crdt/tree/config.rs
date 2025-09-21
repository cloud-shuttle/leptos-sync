//! Configuration types for tree CRDTs

use serde::{Deserialize, Serialize};

/// Strategy for handling tree conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeStrategy {
    /// Add-Wins: Nodes are never removed, only marked as deleted
    AddWins,
    /// Remove-Wins: Deleted nodes are completely removed
    RemoveWins,
}

/// Configuration for tree CRDTs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeConfig {
    /// Conflict resolution strategy
    pub strategy: TreeStrategy,
    /// Whether to preserve deleted nodes in metadata
    pub preserve_deleted: bool,
    /// Maximum depth of the tree
    pub max_depth: Option<usize>,
    /// Maximum number of children per node
    pub max_children: Option<usize>,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            strategy: TreeStrategy::AddWins,
            preserve_deleted: true,
            max_depth: None,
            max_children: None,
        }
    }
}
