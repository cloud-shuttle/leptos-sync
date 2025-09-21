//! Common types and error handling for advanced CRDTs

use super::super::ReplicaId;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Error types for advanced CRDT operations
#[derive(Debug, Clone, PartialEq)]
pub enum AdvancedCrdtError {
    /// Invalid position in sequence
    InvalidPosition(String),
    /// Element not found
    ElementNotFound(String),
    /// Invalid parent-child relationship
    InvalidRelationship(String),
    /// Cycle detected in DAG
    CycleDetected(String),
    /// Invalid operation for CRDT type
    InvalidOperation(String),
    /// Merge operation failed
    MergeError(String),
}

impl fmt::Display for AdvancedCrdtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdvancedCrdtError::InvalidPosition(msg) => write!(f, "Invalid position: {}", msg),
            AdvancedCrdtError::ElementNotFound(msg) => write!(f, "Element not found: {}", msg),
            AdvancedCrdtError::InvalidRelationship(msg) => write!(f, "Invalid relationship: {}", msg),
            AdvancedCrdtError::CycleDetected(msg) => write!(f, "Cycle detected: {}", msg),
            AdvancedCrdtError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            AdvancedCrdtError::MergeError(msg) => write!(f, "Merge error: {}", msg),
        }
    }
}

impl std::error::Error for AdvancedCrdtError {}

/// Position identifier for RGA and LSEQ
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PositionId {
    /// Replica ID that created this position
    pub replica_id: ReplicaId,
    /// Logical timestamp
    pub timestamp: u64,
    /// Additional disambiguation value
    pub disambiguation: u64,
}

impl PositionId {
    /// Create a new position ID
    pub fn new(replica_id: ReplicaId, timestamp: u64, disambiguation: u64) -> Self {
        Self {
            replica_id,
            timestamp,
            disambiguation,
        }
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
    fn test_position_id_creation() {
        let replica = create_replica(1);
        let position = PositionId::new(replica, 12345, 67890);
        
        assert_eq!(position.replica_id, replica);
        assert_eq!(position.timestamp, 12345);
        assert_eq!(position.disambiguation, 67890);
    }

    #[test]
    fn test_position_id_ordering() {
        let replica1 = create_replica(1);
        let replica2 = create_replica(2);
        
        let pos1 = PositionId::new(replica1, 100, 10);
        let pos2 = PositionId::new(replica1, 200, 10);
        let pos3 = PositionId::new(replica2, 100, 10);
        
        assert!(pos1 < pos2);
        assert!(pos1 < pos3);
        assert!(pos2 > pos3);
    }

    #[test]
    fn test_advanced_crdt_error_display() {
        let error = AdvancedCrdtError::InvalidPosition("test".to_string());
        assert_eq!(format!("{}", error), "Invalid position: test");
        
        let error = AdvancedCrdtError::ElementNotFound("missing".to_string());
        assert_eq!(format!("{}", error), "Element not found: missing");
    }
}
