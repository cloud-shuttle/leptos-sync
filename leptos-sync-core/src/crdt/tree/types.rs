//! Core types for tree CRDTs

use super::super::ReplicaId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a tree node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    /// Unique identifier for the node
    pub id: Uuid,
    /// Replica that created the node
    pub replica: ReplicaId,
}

impl NodeId {
    /// Create a new node ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create a node ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Metadata for a tree node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// When the node was created
    pub created_at: u64,
    /// When the node was last modified
    pub modified_at: u64,
    /// Whether the node is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the node
    pub last_modified_by: ReplicaId,
}

impl NodeMetadata {
    /// Create new metadata
    pub fn new(replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            created_at: timestamp,
            modified_at: timestamp,
            deleted: false,
            last_modified_by: replica,
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.modified_at = timestamp;
        self.last_modified_by = replica;
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.deleted = true;
        self.mark_modified(replica, timestamp);
    }
}

/// A tree node with its metadata and relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode<T> {
    /// Unique identifier
    pub id: NodeId,
    /// The actual value
    pub value: T,
    /// Metadata
    pub metadata: NodeMetadata,
    /// Parent node ID (None for root)
    pub parent: Option<NodeId>,
    /// Child node IDs
    pub children: Vec<NodeId>,
}

impl<T> TreeNode<T> {
    /// Create a new tree node
    pub fn new(value: T, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: NodeId::new(replica),
            value,
            metadata: NodeMetadata::new(replica, timestamp),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create a child node
    pub fn new_child(value: T, replica: ReplicaId, timestamp: u64, parent: NodeId) -> Self {
        Self {
            id: NodeId::new(replica),
            value,
            metadata: NodeMetadata::new(replica, timestamp),
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Mark as modified
    pub fn mark_modified(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_modified(replica, timestamp);
    }

    /// Mark as deleted
    pub fn mark_deleted(&mut self, replica: ReplicaId, timestamp: u64) {
        self.metadata.mark_deleted(replica, timestamp);
    }

    /// Add a child
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Remove a child
    pub fn remove_child(&mut self, child_id: &NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }
}
