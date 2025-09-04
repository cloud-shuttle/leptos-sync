//! CRDT (Conflict-free Replicated Data Type) implementations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use uuid::Uuid;

/// Unique identifier for a replica
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReplicaId(#[serde(with = "uuid_serde")] pub Uuid);

mod uuid_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        uuid.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Default for ReplicaId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for ReplicaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ReplicaId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ReplicaId> for Uuid {
    fn from(replica_id: ReplicaId) -> Self {
        replica_id.0
    }
}

/// Trait for types that can be merged with other instances
pub trait Mergeable: Clone + Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Merge this instance with another instance
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error>;
    
    /// Check if there's a conflict with another instance
    fn has_conflict(&self, other: &Self) -> bool;
}

/// Trait for CRDTs that have a replica ID
pub trait CRDT {
    fn replica_id(&self) -> &ReplicaId;
}

/// Last-Write-Wins Register
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LwwRegister<T> {
    value: T,
    timestamp: chrono::DateTime<chrono::Utc>,
    replica_id: ReplicaId,
}

impl<T: Default> Default for LwwRegister<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            timestamp: chrono::Utc::now(),
            replica_id: ReplicaId::default(),
        }
    }
}

impl<T> LwwRegister<T> {
    pub fn new(value: T, replica_id: ReplicaId) -> Self {
        Self {
            value,
            timestamp: chrono::Utc::now(),
            replica_id,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    pub fn replica_id(&self) -> ReplicaId {
        self.replica_id
    }

    pub fn update(&mut self, value: T, replica_id: ReplicaId) {
        self.value = value;
        self.timestamp = chrono::Utc::now();
        self.replica_id = replica_id;
    }

    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

impl<T: Clone + PartialEq + Send + Sync> Mergeable for LwwRegister<T> {
    type Error = std::io::Error;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        if other.timestamp > self.timestamp || 
           (other.timestamp == self.timestamp && other.replica_id.0 > self.replica_id.0) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.replica_id = other.replica_id;
        }
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.replica_id != other.replica_id
    }
}

impl<T> CRDT for LwwRegister<T> {
    fn replica_id(&self) -> &ReplicaId {
        &self.replica_id
    }
}

/// Last-Write-Wins Map
#[derive(Debug, Clone)]
pub struct LwwMap<K, V> {
    data: HashMap<K, LwwRegister<V>>,
}

impl<K, V> LwwMap<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + PartialEq + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V, replica_id: ReplicaId) {
        let register = LwwRegister::new(value, replica_id);
        self.data.insert(key, register);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key).map(|register| register.value())
    }

    pub fn get_register(&self, key: &K) -> Option<&LwwRegister<V>> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key).map(|register| register.value().clone())
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.values().map(|register| register.value())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter().map(|(k, v)| (k, v.value()))
    }
}

impl<K, V> Default for LwwMap<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + PartialEq + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Mergeable for LwwMap<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + PartialEq + Send + Sync,
{
    type Error = std::io::Error;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        for (key, other_register) in &other.data {
            match self.data.get_mut(key) {
                Some(existing_register) => {
                    existing_register.merge(other_register)?;
                }
                None => {
                    self.data.insert(key.clone(), other_register.clone());
                }
            }
        }
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        for (key, other_register) in &other.data {
            if let Some(existing_register) = self.data.get(key) {
                if existing_register.has_conflict(other_register) {
                    return true;
                }
            }
        }
        false
    }
}

impl<K, V> CRDT for LwwMap<K, V> {
    fn replica_id(&self) -> &ReplicaId {
        // LwwMap doesn't have a single replica ID, so we'll use a default
        // In practice, this might need to be handled differently
        static DEFAULT_REPLICA: std::sync::LazyLock<ReplicaId> = std::sync::LazyLock::new(|| ReplicaId::from(uuid::Uuid::nil()));
        &DEFAULT_REPLICA
    }
}

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
    fn test_replica_id_serialization() {
        let replica_id = ReplicaId::default();
        let serialized = serde_json::to_string(&replica_id).unwrap();
        let deserialized: ReplicaId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(replica_id, deserialized);
    }

    #[test]
    fn test_lww_register_merge() {
        let mut reg1 = LwwRegister::new("value1", ReplicaId::default());
        let reg2 = LwwRegister::new("value2", ReplicaId::default());
        
        // Wait a bit to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        reg1.merge(&reg2).unwrap();
        assert_eq!(reg1.value(), &"value2");
    }

    #[test]
    fn test_lww_map_operations() {
        let mut map = LwwMap::new();
        let replica_id = ReplicaId::default();
        
        map.insert("key1".to_string(), "value1".to_string(), replica_id);
        assert_eq!(map.get(&"key1".to_string()), Some(&"value1".to_string()));
        assert_eq!(map.len(), 1);
        
        map.remove(&"key1".to_string());
        assert_eq!(map.len(), 0);
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate arbitrary ReplicaId for testing
    fn arb_replica_id() -> impl Strategy<Value = ReplicaId> {
        any::<[u8; 16]>().prop_map(|bytes| {
            let uuid = uuid::Uuid::from_bytes(bytes);
            ReplicaId::from(uuid)
        })
    }

    /// Generate arbitrary timestamp for testing
    fn arb_timestamp() -> impl Strategy<Value = chrono::DateTime<chrono::Utc>> {
        (0i64..1_000_000_000_000i64).prop_map(|seconds| {
            chrono::DateTime::from_timestamp(seconds, 0).unwrap_or_else(|| chrono::Utc::now())
        })
    }

    /// Generate arbitrary LwwRegister for testing
    fn arb_lww_register() -> impl Strategy<Value = LwwRegister<String>> {
        (any::<String>(), arb_replica_id(), arb_timestamp()).prop_map(|(value, replica_id, timestamp)| {
            LwwRegister {
                value,
                timestamp,
                replica_id,
            }
        })
    }

    /// Generate arbitrary LwwMap for testing
    fn arb_lww_map() -> impl Strategy<Value = LwwMap<String, String>> {
        prop::collection::hash_map(
            any::<String>(),
            arb_lww_register(),
            0..10
        ).prop_map(|data| {
            LwwMap { data }
        })
    }

    /// Generate arbitrary GCounter for testing
    fn arb_gcounter() -> impl Strategy<Value = GCounter> {
        prop::collection::hash_map(
            arb_replica_id(),
            0u64..100,
            0..5
        ).prop_map(|increments| {
            GCounter { increments }
        })
    }

    proptest! {
        #[test]
        fn test_lww_register_commutativity(
            reg1 in arb_lww_register(),
            reg2 in arb_lww_register(),
        ) {
            let mut merged1 = reg1.clone();
            let mut merged2 = reg2.clone();
            
            // Test commutativity: merge(a, b) should equal merge(b, a)
            merged1.merge(&reg2).unwrap();
            merged2.merge(&reg1).unwrap();
            
            // The final value should be the same regardless of merge order
            prop_assert_eq!(merged1.value, merged2.value);
        }

        #[test]
        fn test_lww_register_associativity(
            reg1 in arb_lww_register(),
            reg2 in arb_lww_register(),
            reg3 in arb_lww_register(),
        ) {
            let mut merged_left = reg1.clone();
            let mut temp = reg2.clone();
            temp.merge(&reg3).unwrap();
            merged_left.merge(&temp).unwrap();
            
            let mut merged_right = reg1.clone();
            merged_right.merge(&reg2).unwrap();
            merged_right.merge(&reg3).unwrap();
            
            // Test associativity: merge(merge(a, b), c) == merge(a, merge(b, c))
            prop_assert_eq!(merged_left.value, merged_right.value);
        }

        #[test]
        fn test_lww_register_idempotency(
            reg in arb_lww_register(),
        ) {
            let mut merged = reg.clone();
            merged.merge(&reg).unwrap();
            
            // Test idempotency: merge(a, a) == a
            prop_assert_eq!(merged.value, reg.value);
            prop_assert_eq!(merged.timestamp, reg.timestamp);
            prop_assert_eq!(merged.replica_id, reg.replica_id);
        }

        #[test]
        fn test_lww_register_convergence(
            regs in prop::collection::vec(arb_lww_register(), 2..10),
        ) {
            // Test convergence: multiple replicas should converge to the same state
            let mut final_state = regs[0].clone();
            
            for reg in &regs[1..] {
                final_state.merge(reg).unwrap();
            }
            
            // Now merge in reverse order
            let mut reverse_final_state = regs[regs.len() - 1].clone();
            for reg in regs.iter().rev().skip(1) {
                reverse_final_state.merge(reg).unwrap();
            }
            
            // Both should converge to the same value
            prop_assert_eq!(final_state.value, reverse_final_state.value);
        }

        #[test]
        fn test_lww_register_timestamp_ordering(
            value in any::<String>(),
            replica_id in arb_replica_id(),
            timestamp1 in arb_timestamp(),
            timestamp2 in arb_timestamp(),
        ) {
            let mut reg1 = LwwRegister {
                value: value.clone(),
                timestamp: timestamp1,
                replica_id,
            };
            
            let reg2 = LwwRegister {
                value: "different_value".to_string(),
                timestamp: timestamp2,
                replica_id,
            };
            
            reg1.merge(&reg2).unwrap();
            
            // The register should contain the value from the register with the later timestamp
            if timestamp2 > timestamp1 {
                prop_assert_eq!(reg1.value, "different_value");
            } else {
                prop_assert_eq!(reg1.value, value);
            }
        }

        #[test]
        fn test_lww_register_replica_id_tie_breaking(
            value1 in any::<String>(),
            value2 in any::<String>(),
            replica_id1 in arb_replica_id(),
            replica_id2 in arb_replica_id(),
            timestamp in arb_timestamp(),
        ) {
            // Skip if replica IDs are the same (no tie to break)
            prop_assume!(replica_id1 != replica_id2);
            
            let mut reg1 = LwwRegister {
                value: value1.clone(),
                timestamp,
                replica_id: replica_id1,
            };
            
            let reg2 = LwwRegister {
                value: value2.clone(),
                timestamp,
                replica_id: replica_id2,
            };
            
            reg1.merge(&reg2).unwrap();
            
            // When timestamps are equal, the register with the "larger" replica_id should win
            let winner = if replica_id2.0 > replica_id1.0 { value2 } else { value1 };
            prop_assert_eq!(reg1.value, winner);
        }
    }

    proptest! {
        #[test]
        fn test_lww_map_commutativity(
            map1 in arb_lww_map(),
            map2 in arb_lww_map(),
        ) {
            let mut merged1 = map1.clone();
            let mut merged2 = map2.clone();
            
            merged1.merge(&map2).unwrap();
            merged2.merge(&map1).unwrap();
            
            // Test commutativity: merge(a, b) should equal merge(b, a)
            prop_assert_eq!(merged1.data.len(), merged2.data.len());
            
            for key in merged1.data.keys() {
                prop_assert!(merged2.data.contains_key(key));
                let reg1 = &merged1.data[key];
                let reg2 = &merged2.data[key];
                prop_assert_eq!(&reg1.value, &reg2.value);
            }
        }

        #[test]
        fn test_lww_map_associativity(
            map1 in arb_lww_map(),
            map2 in arb_lww_map(),
            map3 in arb_lww_map(),
        ) {
            let mut merged_left = map1.clone();
            let mut temp = map2.clone();
            temp.merge(&map3).unwrap();
            merged_left.merge(&temp).unwrap();
            
            let mut merged_right = map1.clone();
            merged_right.merge(&map2).unwrap();
            merged_right.merge(&map3).unwrap();
            
            // Test associativity
            prop_assert_eq!(merged_left.data.len(), merged_right.data.len());
            
            for key in merged_left.data.keys() {
                prop_assert!(merged_right.data.contains_key(key));
                let reg1 = &merged_left.data[key];
                let reg2 = &merged_right.data[key];
                prop_assert_eq!(&reg1.value, &reg2.value);
            }
        }

        #[test]
        fn test_lww_map_idempotency(
            map in arb_lww_map(),
        ) {
            let mut merged = map.clone();
            merged.merge(&map).unwrap();
            
            // Test idempotency: merge(a, a) == a
            prop_assert_eq!(merged.data.len(), map.data.len());
            
            for (key, original_register) in &map.data {
                let merged_register = &merged.data[key];
                prop_assert_eq!(&merged_register.value, &original_register.value);
                prop_assert_eq!(merged_register.timestamp, original_register.timestamp);
                prop_assert_eq!(merged_register.replica_id, original_register.replica_id);
            }
        }

        #[test]
        fn test_lww_map_convergence(
            maps in prop::collection::vec(arb_lww_map(), 2..5),
        ) {
            // Test convergence: multiple maps should converge to the same state
            let mut final_state = maps[0].clone();
            
            for map in &maps[1..] {
                final_state.merge(map).unwrap();
            }
            
            // Now merge in reverse order
            let mut reverse_final_state = maps[maps.len() - 1].clone();
            for map in maps.iter().rev().skip(1) {
                reverse_final_state.merge(map).unwrap();
            }
            
            // Both should converge to the same state
            prop_assert_eq!(final_state.data.len(), reverse_final_state.data.len());
            
            for key in final_state.data.keys() {
                prop_assert!(reverse_final_state.data.contains_key(key));
                let reg1 = &final_state.data[key];
                let reg2 = &reverse_final_state.data[key];
                prop_assert_eq!(&reg1.value, &reg2.value);
            }
        }
    }

    proptest! {
        #[test]
        fn test_gcounter_commutativity(
            counter1 in arb_gcounter(),
            counter2 in arb_gcounter(),
        ) {
            let mut merged1 = counter1.clone();
            let mut merged2 = counter2.clone();
            
            merged1.merge(&counter2).unwrap();
            merged2.merge(&counter1).unwrap();
            
            // Test commutativity: merge(a, b) should equal merge(b, a)
            let value1 = merged1.value();
            let value2 = merged2.value();
            prop_assert_eq!(merged1.increments, merged2.increments);
            prop_assert_eq!(value1, value2);
        }

        #[test]
        fn test_gcounter_associativity(
            counter1 in arb_gcounter(),
            counter2 in arb_gcounter(),
            counter3 in arb_gcounter(),
        ) {
            let mut merged_left = counter1.clone();
            let mut temp = counter2.clone();
            temp.merge(&counter3).unwrap();
            merged_left.merge(&temp).unwrap();
            
            let mut merged_right = counter1.clone();
            merged_right.merge(&counter2).unwrap();
            merged_right.merge(&counter3).unwrap();
            
            // Test associativity
            let value_left = merged_left.value();
            let value_right = merged_right.value();
            prop_assert_eq!(merged_left.increments, merged_right.increments);
            prop_assert_eq!(value_left, value_right);
        }

        #[test]
        fn test_gcounter_idempotency(
            counter in arb_gcounter(),
        ) {
            let mut merged = counter.clone();
            merged.merge(&counter).unwrap();
            
            // Test idempotency: merge(a, a) == a
            let merged_value = merged.value();
            let original_value = counter.value();
            prop_assert_eq!(merged.increments, counter.increments);
            prop_assert_eq!(merged_value, original_value);
        }

        #[test]
        fn test_gcounter_convergence(
            counters in prop::collection::vec(arb_gcounter(), 2..5),
        ) {
            // Test convergence: multiple counters should converge to the same state
            let mut final_state = counters[0].clone();
            
            for counter in &counters[1..] {
                final_state.merge(counter).unwrap();
            }
            
            // Now merge in reverse order
            let mut reverse_final_state = counters[counters.len() - 1].clone();
            for counter in counters.iter().rev().skip(1) {
                reverse_final_state.merge(counter).unwrap();
            }
            
            // Both should converge to the same state
            let final_value = final_state.value();
            let reverse_value = reverse_final_state.value();
            prop_assert_eq!(final_state.increments, reverse_final_state.increments);
            prop_assert_eq!(final_value, reverse_value);
        }

        #[test]
        fn test_gcounter_monotonicity(
            replica_id in arb_replica_id(),
            initial_value in 0u64..50,
            increment_amount in 1u64..10,
        ) {
            let mut counter = GCounter::new();
            counter.increments.insert(replica_id, initial_value);
            
            let old_value = counter.value();
            
            // Increment the counter
            for _ in 0..increment_amount {
                counter.increment(replica_id);
            }
            
            let new_value = counter.value();
            
            // Test monotonicity: counter should only increase
            prop_assert!(new_value > old_value);
            prop_assert_eq!(new_value, old_value + increment_amount);
        }

        #[test]
        fn test_gcounter_maximum_merge(
            replica_id in arb_replica_id(),
            value1 in 0u64..100,
            value2 in 0u64..100,
        ) {
            let mut counter1 = GCounter::new();
            counter1.increments.insert(replica_id, value1);
            
            let mut counter2 = GCounter::new();
            counter2.increments.insert(replica_id, value2);
            
            counter1.merge(&counter2).unwrap();
            
            // After merge, should have the maximum of both values
            let expected_value = value1.max(value2);
            prop_assert_eq!(counter1.replica_value(replica_id), expected_value);
        }
    }

    proptest! {
        #[test]
        fn test_crdt_serialization_roundtrip(
            register in arb_lww_register(),
        ) {
            // Test that CRDTs can be serialized and deserialized correctly
            let serialized = serde_json::to_string(&register).unwrap();
            let deserialized: LwwRegister<String> = serde_json::from_str(&serialized).unwrap();
            
            prop_assert_eq!(register.value, deserialized.value);
            prop_assert_eq!(register.timestamp, deserialized.timestamp);
            prop_assert_eq!(register.replica_id, deserialized.replica_id);
        }

        #[test]
        fn test_crdt_merge_preserves_serialization(
            reg1 in arb_lww_register(),
            reg2 in arb_lww_register(),
        ) {
            let mut merged = reg1.clone();
            merged.merge(&reg2).unwrap();
            
            // Test that merged CRDT can still be serialized
            let serialized = serde_json::to_string(&merged).unwrap();
            let deserialized: LwwRegister<String> = serde_json::from_str(&serialized).unwrap();
            
            prop_assert_eq!(merged.value, deserialized.value);
            prop_assert_eq!(merged.timestamp, deserialized.timestamp);
            prop_assert_eq!(merged.replica_id, deserialized.replica_id);
        }
    }
}
