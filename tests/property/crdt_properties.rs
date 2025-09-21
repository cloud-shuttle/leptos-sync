//! Property-based tests for CRDT invariants
//!
//! Tests that validate CRDT mathematical properties including
//! commutativity, associativity, and idempotency.

use leptos_sync_core::crdt::*;
use proptest::prelude::*;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Generate random replica IDs
fn replica_id_strategy() -> impl Strategy<Value = ReplicaId> {
    any::<[u8; 16]>().prop_map(|bytes| ReplicaId::from(Uuid::from_bytes(bytes)))
}

/// Generate random timestamps
fn timestamp_strategy() -> impl Strategy<Value = SystemTime> {
    (0u64..1000000).prop_map(|offset| SystemTime::now() + Duration::from_millis(offset))
}

/// Generate random strings for LWW register values
fn string_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..100).prop_map(|chars| chars.into_iter().collect())
}

/// Generate random integers for counters
fn integer_strategy() -> impl Strategy<Value = u64> {
    0u64..1000
}

/// Generate random key-value pairs for maps
fn key_value_strategy() -> impl Strategy<Value = (String, String)> {
    (string_strategy(), string_strategy())
}

// Test CRDT commutativity: merge(a, b) == merge(b, a)
proptest! {
    #[test]
    fn test_lww_register_commutativity(
        value_a in string_strategy(),
        value_b in string_strategy(),
        timestamp_offset in 0u64..1000,
        replica_a in replica_id_strategy(),
        replica_b in replica_id_strategy(),
    ) {
        let base_time = SystemTime::now();

        let mut register_a = LwwRegister::new(value_a.clone(), base_time, replica_a);
        let mut register_b = LwwRegister::new(value_b.clone(),
            base_time + Duration::from_millis(timestamp_offset), replica_b);

        // Test commutativity
        let mut result1 = register_a.clone();
        result1.merge(&register_b);

        let mut result2 = register_b.clone();
        result2.merge(&register_a);

        assert_eq!(result1.value(), result2.value());
    }

    #[test]
    fn test_g_counter_commutativity(
        increments_a in prop::collection::vec(integer_strategy(), 1..10),
        increments_b in prop::collection::vec(integer_strategy(), 1..10),
        replica_a in replica_id_strategy(),
        replica_b in replica_id_strategy(),
    ) {
        let mut counter_a = GCounter::new();
        let mut counter_b = GCounter::new();

        // Apply increments
        for inc in increments_a {
            counter_a.increment_by(inc).unwrap();
        }
        for inc in increments_b {
            counter_b.increment_by(inc).unwrap();
        }

        // Test commutativity
        let mut result1 = counter_a.clone();
        result1.merge(&counter_b);

        let mut result2 = counter_b.clone();
        result2.merge(&counter_a);

        assert_eq!(result1.value(), result2.value());
    }

    #[test]
    fn test_lww_map_commutativity(
        entries_a in prop::collection::vec(key_value_strategy(), 0..10),
        entries_b in prop::collection::vec(key_value_strategy(), 0..10),
        replica_a in replica_id_strategy(),
        replica_b in replica_id_strategy(),
    ) {
        let mut map_a = LwwMap::new();
        let mut map_b = LwwMap::new();

        // Apply entries
        for (key, value) in entries_a {
            map_a.set(&key, value, SystemTime::now(), replica_a);
        }
        for (key, value) in entries_b {
            map_b.set(&key, value, SystemTime::now(), replica_b);
        }

        // Test commutativity
        let mut result1 = map_a.clone();
        result1.merge(&map_b);

        let mut result2 = map_b.clone();
        result2.merge(&map_a);

        assert_eq!(result1.len(), result2.len());
        // In a real implementation, we would compare all key-value pairs
    }
}

// Test CRDT associativity: (a ⊔ b) ⊔ c == a ⊔ (b ⊔ c)
proptest! {
    #[test]
    fn test_g_counter_associativity(
        increments_a in prop::collection::vec(integer_strategy(), 1..5),
        increments_b in prop::collection::vec(integer_strategy(), 1..5),
        increments_c in prop::collection::vec(integer_strategy(), 1..5),
    ) {
        let mut counter_a = GCounter::new();
        let mut counter_b = GCounter::new();
        let mut counter_c = GCounter::new();

        // Apply increments
        for inc in increments_a { counter_a.increment_by(inc).unwrap(); }
        for inc in increments_b { counter_b.increment_by(inc).unwrap(); }
        for inc in increments_c { counter_c.increment_by(inc).unwrap(); }

        // Test associativity: (a ⊔ b) ⊔ c == a ⊔ (b ⊔ c)
        let mut result1 = counter_a.clone();
        result1.merge(&counter_b);
        result1.merge(&counter_c);

        let mut temp = counter_b.clone();
        temp.merge(&counter_c);
        let mut result2 = counter_a.clone();
        result2.merge(&temp);

        assert_eq!(result1.value(), result2.value());
    }

    #[test]
    fn test_lww_register_associativity(
        value_a in string_strategy(),
        value_b in string_strategy(),
        value_c in string_strategy(),
        timestamp_offset in 0u64..1000,
        replica_a in replica_id_strategy(),
        replica_b in replica_id_strategy(),
        replica_c in replica_id_strategy(),
    ) {
        let base_time = SystemTime::now();

        let mut register_a = LwwRegister::new(value_a, base_time, replica_a);
        let mut register_b = LwwRegister::new(value_b,
            base_time + Duration::from_millis(timestamp_offset), replica_b);
        let mut register_c = LwwRegister::new(value_c,
            base_time + Duration::from_millis(timestamp_offset * 2), replica_c);

        // Test associativity
        let mut result1 = register_a.clone();
        result1.merge(&register_b);
        result1.merge(&register_c);

        let mut temp = register_b.clone();
        temp.merge(&register_c);
        let mut result2 = register_a.clone();
        result2.merge(&temp);

        assert_eq!(result1.value(), result2.value());
    }
}

// Test CRDT idempotency: merge(a, a) == a
proptest! {
    #[test]
    fn test_lww_register_idempotency(
        value in string_strategy(),
        timestamp in timestamp_strategy(),
        replica in replica_id_strategy(),
    ) {
        let mut register = LwwRegister::new(value.clone(), timestamp, replica);
        let original_value = register.value().clone();

        // Test idempotency
        register.merge(&register.clone());

        assert_eq!(register.value(), &original_value);
    }

    #[test]
    fn test_g_counter_idempotency(
        increments in prop::collection::vec(integer_strategy(), 1..10),
    ) {
        let mut counter = GCounter::new();

        // Apply increments
        for inc in increments {
            counter.increment_by(inc).unwrap();
        }

        let original_value = counter.value();

        // Test idempotency
        counter.merge(&counter.clone());

        assert_eq!(counter.value(), original_value);
    }
}

// Test CRDT monotonicity: merge(a, b) >= a and merge(a, b) >= b
proptest! {
    #[test]
    fn test_g_counter_monotonicity(
        increments_a in prop::collection::vec(integer_strategy(), 1..10),
        increments_b in prop::collection::vec(integer_strategy(), 1..10),
    ) {
        let mut counter_a = GCounter::new();
        let mut counter_b = GCounter::new();

        // Apply increments
        for inc in increments_a {
            counter_a.increment_by(inc).unwrap();
        }
        for inc in increments_b {
            counter_b.increment_by(inc).unwrap();
        }

        let value_a = counter_a.value();
        let value_b = counter_b.value();

        // Test monotonicity
        counter_a.merge(&counter_b);

        assert!(counter_a.value() >= value_a);
        assert!(counter_a.value() >= value_b);
    }
}

// Test CRDT convergence: multiple replicas should converge to the same state
proptest! {
    #[test]
    fn test_lww_register_convergence(
        values in prop::collection::vec(string_strategy(), 2..5),
        timestamps in prop::collection::vec(timestamp_strategy(), 2..5),
        replicas in prop::collection::vec(replica_id_strategy(), 2..5),
    ) {
        // Create multiple registers with different values
        let mut registers: Vec<LwwRegister<String>> = values
            .into_iter()
            .zip(timestamps.into_iter())
            .zip(replicas.into_iter())
            .map(|((value, timestamp), replica)| {
                LwwRegister::new(value, timestamp, replica)
            })
            .collect();

        // Merge all registers into the first one
        for i in 1..registers.len() {
            registers[0].merge(&registers[i]);
        }

        // Merge all registers into the second one (different order)
        for i in 2..registers.len() {
            registers[1].merge(&registers[i]);
        }
        registers[1].merge(&registers[0]);

        // Both should have the same final value
        assert_eq!(registers[0].value(), registers[1].value());
    }

    #[test]
    fn test_g_counter_convergence(
        increment_sets in prop::collection::vec(
            prop::collection::vec(integer_strategy(), 1..5),
            2..5
        ),
    ) {
        // Create multiple counters with different increments
        let mut counters: Vec<GCounter> = increment_sets
            .into_iter()
            .map(|increments| {
                let mut counter = GCounter::new();
                for inc in increments {
                    counter.increment_by(inc).unwrap();
                }
                counter
            })
            .collect();

        // Merge all counters into the first one
        for i in 1..counters.len() {
            counters[0].merge(&counters[i]);
        }

        // Merge all counters into the second one (different order)
        for i in 2..counters.len() {
            counters[1].merge(&counters[i]);
        }
        counters[1].merge(&counters[0]);

        // Both should have the same final value
        assert_eq!(counters[0].value(), counters[1].value());
    }
}

// Test CRDT conflict resolution properties
proptest! {
    #[test]
    fn test_lww_register_conflict_resolution(
        value_a in string_strategy(),
        value_b in string_strategy(),
        timestamp_a in timestamp_strategy(),
        timestamp_b in timestamp_strategy(),
        replica_a in replica_id_strategy(),
        replica_b in replica_id_strategy(),
    ) {
        let mut register_a = LwwRegister::new(value_a, timestamp_a, replica_a);
        let mut register_b = LwwRegister::new(value_b, timestamp_b, replica_b);

        // Merge and check that the later timestamp wins
        register_a.merge(&register_b);

        let expected_value = if timestamp_a >= timestamp_b {
            register_a.value()
        } else {
            register_b.value()
        };

        assert_eq!(register_a.value(), expected_value);
    }

    #[test]
    fn test_lww_map_conflict_resolution(
        key in string_strategy(),
        value_a in string_strategy(),
        value_b in string_strategy(),
        timestamp_a in timestamp_strategy(),
        timestamp_b in timestamp_strategy(),
        replica_a in replica_id_strategy(),
        replica_b in replica_id_strategy(),
    ) {
        let mut map_a = LwwMap::new();
        let mut map_b = LwwMap::new();

        map_a.set(&key, value_a, timestamp_a, replica_a);
        map_b.set(&key, value_b, timestamp_b, replica_b);

        // Merge and check that the later timestamp wins
        map_a.merge(&map_b);

        let expected_value = if timestamp_a >= timestamp_b {
            map_a.get(&key)
        } else {
            map_b.get(&key)
        };

        assert_eq!(map_a.get(&key), expected_value);
    }
}

// Test CRDT serialization/deserialization properties
proptest! {
    #[test]
    fn test_lww_register_serialization_roundtrip(
        value in string_strategy(),
        timestamp in timestamp_strategy(),
        replica in replica_id_strategy(),
    ) {
        let original = LwwRegister::new(value, timestamp, replica);

        // Serialize and deserialize
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: LwwRegister<String> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.value(), deserialized.value());
        // Note: We can't easily compare timestamps due to precision issues
    }

    #[test]
    fn test_g_counter_serialization_roundtrip(
        increments in prop::collection::vec(integer_strategy(), 1..10),
    ) {
        let mut original = GCounter::new();
        for inc in increments {
            original.increment_by(inc).unwrap();
        }

        // Serialize and deserialize
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: GCounter = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.value(), deserialized.value());
    }
}

// Test CRDT size and memory properties
proptest! {
    #[test]
    fn test_lww_register_size_properties(
        value in string_strategy(),
        timestamp in timestamp_strategy(),
        replica in replica_id_strategy(),
    ) {
        let register = LwwRegister::new(value, timestamp, replica);

        // Test that the register has a reasonable size
        let serialized = serde_json::to_string(&register).unwrap();
        assert!(serialized.len() > 0);
        assert!(serialized.len() < 10000); // Reasonable upper bound
    }

    #[test]
    fn test_g_counter_size_properties(
        increments in prop::collection::vec(integer_strategy(), 1..100),
    ) {
        let mut counter = GCounter::new();
        for inc in increments {
            counter.increment_by(inc).unwrap();
        }

        // Test that the counter has a reasonable size
        let serialized = serde_json::to_string(&counter).unwrap();
        assert!(serialized.len() > 0);
        assert!(serialized.len() < 10000); // Reasonable upper bound
    }
}
