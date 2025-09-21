//! Core CRDT traits

use super::replica_id::ReplicaId;

/// Trait for types that can be merged with other instances
pub trait Mergeable: Clone + Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static + Sized;
    
    /// Merge this instance with another instance
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error>;
    
    /// Check if there's a conflict with another instance
    fn has_conflict(&self, other: &Self) -> bool;
}

/// Trait for CRDTs that have a replica ID
pub trait CRDT {
    fn replica_id(&self) -> &ReplicaId;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test implementation of the traits
    #[derive(Debug, Clone, PartialEq)]
    struct TestCrdt {
        replica_id: ReplicaId,
        value: String,
    }

    impl TestCrdt {
        fn new(replica_id: ReplicaId, value: String) -> Self {
            Self { replica_id, value }
        }
    }

    impl CRDT for TestCrdt {
        fn replica_id(&self) -> &ReplicaId {
            &self.replica_id
        }
    }

    impl Mergeable for TestCrdt {
        type Error = std::io::Error;

        fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
            // Simple merge: take the longer string
            if other.value.len() > self.value.len() {
                self.value = other.value.clone();
            }
            Ok(())
        }

        fn has_conflict(&self, other: &Self) -> bool {
            self.value != other.value
        }
    }

    #[test]
    fn test_crdt_trait() {
        let replica_id = ReplicaId::default();
        let crdt = TestCrdt::new(replica_id, "test".to_string());
        
        assert_eq!(crdt.replica_id(), &replica_id);
    }

    #[test]
    fn test_mergeable_trait() {
        let replica_id = ReplicaId::default();
        let mut crdt1 = TestCrdt::new(replica_id, "short".to_string());
        let crdt2 = TestCrdt::new(replica_id, "much longer string".to_string());
        
        // Test merge
        crdt1.merge(&crdt2).unwrap();
        assert_eq!(crdt1.value, "much longer string");
        
        // Test conflict detection
        let crdt3 = TestCrdt::new(replica_id, "different".to_string());
        assert!(crdt1.has_conflict(&crdt3));
        assert!(!crdt1.has_conflict(&crdt2));
    }
}
