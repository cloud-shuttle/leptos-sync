//! Counter CRDT implementations

use super::{replica_id::ReplicaId, traits::{CRDT, Mergeable}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Counter that can be incremented/decremented
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    increments: HashMap<ReplicaId, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        Self {
            increments: HashMap::new(),
        }
    }

    pub fn increment(&mut self, replica_id: ReplicaId) {
        *self.increments.entry(replica_id).or_insert(0) += 1;
    }

    pub fn value(&self) -> u64 {
        self.increments.values().sum()
    }

    pub fn replica_value(&self, replica_id: ReplicaId) -> u64 {
        self.increments.get(&replica_id).copied().unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.increments.len()
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Mergeable for GCounter {
    type Error = std::io::Error;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        for (replica_id, increment) in &other.increments {
            let current = self.increments.entry(*replica_id).or_insert(0);
            *current = (*current).max(*increment);
        }
        Ok(())
    }
    
    fn has_conflict(&self, _other: &Self) -> bool {
        // G-Counters are conflict-free by design
        false
    }
}

impl CRDT for GCounter {
    fn replica_id(&self) -> &ReplicaId {
        // GCounter doesn't have a single replica ID, so we'll use a default
        // In practice, this might need to be handled differently
        static DEFAULT_REPLICA: std::sync::LazyLock<ReplicaId> = std::sync::LazyLock::new(|| ReplicaId::from(uuid::Uuid::nil()));
        &DEFAULT_REPLICA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcounter_creation() {
        let counter = GCounter::new();
        assert_eq!(counter.value(), 0);
        assert_eq!(counter.len(), 0);
    }

    #[test]
    fn test_gcounter_operations() {
        let mut counter = GCounter::new();
        let replica_id = ReplicaId::default();
        
        counter.increment(replica_id);
        counter.increment(replica_id);
        
        assert_eq!(counter.value(), 2);
        assert_eq!(counter.replica_value(replica_id), 2);
    }

    #[test]
    fn test_gcounter_merge() {
        let mut counter1 = GCounter::new();
        let mut counter2 = GCounter::new();
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();
        
        counter1.increment(replica_id1);
        counter1.increment(replica_id1);
        counter2.increment(replica_id2);
        counter2.increment(replica_id2);
        counter2.increment(replica_id2);
        
        counter1.merge(&counter2).unwrap();
        
        assert_eq!(counter1.value(), 5);
        assert_eq!(counter1.replica_value(replica_id1), 2);
        assert_eq!(counter1.replica_value(replica_id2), 3);
    }

    #[test]
    fn test_gcounter_no_conflicts() {
        let counter1 = GCounter::new();
        let counter2 = GCounter::new();
        
        // G-Counters should never have conflicts
        assert!(!counter1.has_conflict(&counter2));
    }

    #[test]
    fn test_gcounter_serialization() {
        let mut counter = GCounter::new();
        let replica_id = ReplicaId::default();
        
        counter.increment(replica_id);
        counter.increment(replica_id);
        
        let serialized = serde_json::to_string(&counter).unwrap();
        let deserialized: GCounter = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(counter.value(), deserialized.value());
        assert_eq!(counter.replica_value(replica_id), deserialized.replica_value(replica_id));
    }

    #[test]
    fn test_gcounter_maximum_merge() {
        let mut counter1 = GCounter::new();
        let mut counter2 = GCounter::new();
        let replica_id = ReplicaId::default();
        
        // Set different values for the same replica
        counter1.increments.insert(replica_id, 5);
        counter2.increments.insert(replica_id, 3);
        
        counter1.merge(&counter2).unwrap();
        
        // Should keep the maximum value
        assert_eq!(counter1.replica_value(replica_id), 5);
    }
}
