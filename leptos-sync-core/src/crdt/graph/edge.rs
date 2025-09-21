//! Graph edge operations and types

use super::vertex::{VertexId, GraphError};
use super::super::ReplicaId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a graph edge
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId {
    /// Unique identifier for the edge
    pub id: Uuid,
    /// Replica that created the edge
    pub replica: ReplicaId,
}

impl EdgeId {
    /// Create a new edge ID
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            id: Uuid::new_v4(),
            replica,
        }
    }

    /// Create an edge ID from existing UUID and replica
    pub fn from_parts(id: Uuid, replica: ReplicaId) -> Self {
        Self { id, replica }
    }
}

/// Metadata for a graph edge
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// When the edge was created
    pub created_at: u64,
    /// When the edge was last modified
    pub modified_at: u64,
    /// Whether the edge is marked as deleted
    pub deleted: bool,
    /// Replica that last modified the edge
    pub last_modified_by: ReplicaId,
}

impl EdgeMetadata {
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

/// A graph edge connecting two vertices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Unique identifier
    pub id: EdgeId,
    /// Source vertex ID
    pub source: VertexId,
    /// Target vertex ID
    pub target: VertexId,
    /// Optional edge weight
    pub weight: Option<f64>,
    /// Metadata
    pub metadata: EdgeMetadata,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: VertexId, target: VertexId, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: EdgeId::new(replica),
            source,
            target,
            weight: None,
            metadata: EdgeMetadata::new(replica, timestamp),
        }
    }

    /// Create a new edge with weight
    pub fn with_weight(source: VertexId, target: VertexId, weight: f64, replica: ReplicaId, timestamp: u64) -> Self {
        Self {
            id: EdgeId::new(replica),
            source,
            target,
            weight: Some(weight),
            metadata: EdgeMetadata::new(replica, timestamp),
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
    use super::*;
    use super::super::super::ReplicaId;
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[test]
    fn test_edge_id_creation() {
        let replica = create_replica(1);
        let edge_id = EdgeId::new(replica);
        
        assert_eq!(edge_id.replica, replica);
        assert_ne!(edge_id.id, Uuid::nil());
    }

    #[test]
    fn test_edge_creation() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let source = VertexId::new(replica);
        let target = VertexId::new(replica);
        let edge = Edge::new(source.clone(), target.clone(), replica, timestamp);
        
        assert_eq!(edge.source, source);
        assert_eq!(edge.target, target);
        assert_eq!(edge.weight, None);
        assert_eq!(edge.metadata.created_at, timestamp);
        assert_eq!(edge.metadata.deleted, false);
    }

    #[test]
    fn test_edge_with_weight() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let source = VertexId::new(replica);
        let target = VertexId::new(replica);
        let weight = 5.5;
        let edge = Edge::with_weight(source.clone(), target.clone(), weight, replica, timestamp);
        
        assert_eq!(edge.weight, Some(weight));
    }

    #[test]
    fn test_edge_metadata_operations() {
        let replica = create_replica(1);
        let timestamp = 1234567890;
        let mut metadata = EdgeMetadata::new(replica, timestamp);
        
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

