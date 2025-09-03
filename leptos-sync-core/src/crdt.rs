//! CRDT (Conflict-free Replicated Data Type) implementations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use uuid::Uuid;

/// Unique identifier for a replica
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        if other.timestamp > self.timestamp {
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
