//! Last-Write-Wins Map implementation

use super::{replica_id::ReplicaId, traits::{CRDT, Mergeable}, lww_register::LwwRegister};
use std::collections::HashMap;
use std::hash::Hash;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_map_creation() {
        let map: LwwMap<String, String> = LwwMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
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
    fn test_lww_map_merge() {
        let mut map1 = LwwMap::new();
        let mut map2 = LwwMap::new();
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();
        
        map1.insert("key1".to_string(), "value1".to_string(), replica_id1);
        map2.insert("key2".to_string(), "value2".to_string(), replica_id2);
        
        map1.merge(&map2).unwrap();
        
        assert_eq!(map1.len(), 2);
        assert_eq!(map1.get(&"key1".to_string()), Some(&"value1".to_string()));
        assert_eq!(map1.get(&"key2".to_string()), Some(&"value2".to_string()));
    }

    #[test]
    fn test_lww_map_iteration() {
        let mut map = LwwMap::new();
        let replica_id = ReplicaId::default();
        
        map.insert("key1".to_string(), "value1".to_string(), replica_id);
        map.insert("key2".to_string(), "value2".to_string(), replica_id);
        
        let mut keys: Vec<_> = map.keys().collect();
        keys.sort();
        assert_eq!(keys, vec![&"key1".to_string(), &"key2".to_string()]);
        
        let mut values: Vec<_> = map.values().collect();
        values.sort();
        assert_eq!(values, vec![&"value1".to_string(), &"value2".to_string()]);
    }

    #[test]
    fn test_lww_map_conflict_detection() {
        let mut map1 = LwwMap::new();
        let mut map2 = LwwMap::new();
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();
        
        // Create conflicting entries with same timestamp
        let timestamp = chrono::Utc::now();
        let mut reg1 = LwwRegister::new("value1", replica_id1).with_timestamp(timestamp);
        let reg2 = LwwRegister::new("value2", replica_id2).with_timestamp(timestamp);
        
        map1.data.insert("key1".to_string(), reg1);
        map2.data.insert("key1".to_string(), reg2);
        
        assert!(map1.has_conflict(&map2));
    }
}
