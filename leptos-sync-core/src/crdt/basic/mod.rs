//! Basic CRDT implementations
//!
//! This module provides fundamental CRDT types including:
//! - ReplicaId: Unique identifier for replicas
//! - LwwRegister: Last-Write-Wins Register
//! - LwwMap: Last-Write-Wins Map
//! - GCounter: Grow-only Counter

pub mod counter;
pub mod lww_map;
pub mod lww_register;
pub mod replica_id;
pub mod traits;

// Re-export main types for convenience
pub use counter::GCounter;
pub use lww_map::LwwMap;
pub use lww_register::LwwRegister;
pub use replica_id::ReplicaId;
pub use traits::{CRDT, Mergeable};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_basic_crdt_integration() {
        let replica_id = ReplicaId::default();

        // Test LwwRegister
        let mut register = LwwRegister::new("initial", replica_id);
        register.update("updated", replica_id);
        assert_eq!(register.value(), &"updated");

        // Test LwwMap
        let mut map = LwwMap::new();
        map.insert("key1", "value1", replica_id);
        assert_eq!(map.get(&"key1"), Some(&"value1"));

        // Test GCounter
        let mut counter = GCounter::new();
        counter.increment(replica_id);
        counter.increment(replica_id);
        assert_eq!(counter.value(), 2);
    }

    #[test]
    fn test_crdt_traits_implementation() {
        let replica_id = ReplicaId::default();

        // Test that all basic CRDT types implement the required traits
        let register: LwwRegister<String> = LwwRegister::new("test".to_string(), replica_id);
        let map: LwwMap<String, String> = LwwMap::new();
        let counter = GCounter::new();

        // This should compile if all types implement CRDT trait
        let _: &dyn CRDT = &register;
        let _: &dyn CRDT = &map;
        let _: &dyn CRDT = &counter;
    }

    #[test]
    fn test_mergeable_traits_implementation() {
        let replica_id = ReplicaId::default();

        // Test LwwRegister merge
        let mut reg1 = LwwRegister::new("value1", replica_id);
        let reg2 = LwwRegister::new("value2", replica_id);

        std::thread::sleep(std::time::Duration::from_millis(1));
        reg1.merge(&reg2).unwrap();
        assert_eq!(reg1.value(), &"value2");

        // Test LwwMap merge
        let mut map1 = LwwMap::new();
        let mut map2 = LwwMap::new();

        map1.insert("key1", "value1", replica_id);
        map2.insert("key2", "value2", replica_id);

        map1.merge(&map2).unwrap();
        assert_eq!(map1.len(), 2);

        // Test GCounter merge
        let mut counter1 = GCounter::new();
        let mut counter2 = GCounter::new();

        counter1.increment(replica_id);
        counter2.increment(replica_id);
        counter2.increment(replica_id);

        counter1.merge(&counter2).unwrap();
        assert_eq!(counter1.value(), 2);
    }

    #[test]
    fn test_replica_id_usage() {
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();

        // Test that different replicas are different
        assert_ne!(replica_id1, replica_id2);

        // Test serialization
        let serialized = serde_json::to_string(&replica_id1).unwrap();
        let deserialized: ReplicaId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(replica_id1, deserialized);
    }
}
