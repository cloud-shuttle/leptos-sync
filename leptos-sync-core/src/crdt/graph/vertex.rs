//! Graph vertex operations and types

use super::super::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

/// Custom error type for graph operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphError {
    message: String,
}

impl GraphError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphError: {}", self.message)
    }
}

impl Error for GraphError {}

/// Unique identifier for a graph vertex
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId {
    /// Unique identifier for the vertex
    pub id: Uuid,
    /// Replica that created the vertex
    pub replica: ReplicaId,
}

impl VertexId {
    /// Create a new vertex ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create a vertex ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Metadata for a graph vertex
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexMetadata {
    /// When the vertex was created
    pub created_at: u64,
    /// When the vertex was last modified
    pub modified_at: u64,
    /// Whether the vertex is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the vertex
    pub last_modified_by: ReplicaId,
}

impl VertexMetadata {
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

/// A graph vertex with its metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vertex<T> {
    /// Unique identifier
    pub id: VertexId,
    /// The actual value
    pub value: T,
    /// Metadata
    pub metadata: VertexMetadata,
}

impl<T> Vertex<T> {
    /// Create a new vertex
    pub fn new(value: T, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: VertexId::new(replica),
            value,
            metadata: VertexMetadata::new(replica, timestamp),
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
}

#[cfg(test)]
mod tests {
    use super::super::super::ReplicaId;
    use super::*;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_vertex_id_creation() {
        let replica = create_replica(1);
        let vertex_id = VertexId::new(replica);

        assert_eq!(vertex_id.replica, replica);
        assert_ne!(vertex_id.id, Uuid::nil());
    }

    #[test]
    fn test_vertex_creation() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let vertex = Vertex::new("test_value", replica, timestamp);

        assert_eq!(vertex.value, "test_value");
        assert_eq!(vertex.metadata.created_at, timestamp);
        assert_eq!(vertex.metadata.modified_at, timestamp);
        assert_eq!(vertex.metadata.deleted, false);
        assert_eq!(vertex.metadata.last_modified_by, replica);
    }

    #[test]
    fn test_vertex_metadata_operations() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let mut metadata = VertexMetadata::new(replica, timestamp);

        // Test modification
        let new_timestamp = 1234567891;
        metadata.mark_modified(replica, new_timestamp);
        assert_eq!(metadata.modified_at, new_timestamp);

        // Test deletion
        let delete_timestamp = 1234567892;
        metadata.mark_deleted(replica, delete_timestamp);
        assert_eq!(metadata.deleted, true);
        assert_eq!(metadata.modified_at, delete_timestamp);
    }
}
