//! Last-Write-Wins Register implementation

use super::{replica_id::ReplicaId, traits::{CRDT, Mergeable}};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register_creation() {
        let replica_id = ReplicaId::default();
        let register = LwwRegister::new("test_value", replica_id);
        
        assert_eq!(register.value(), &"test_value");
        assert_eq!(register.replica_id(), replica_id);
    }

    #[test]
    fn test_lww_register_update() {
        let replica_id = ReplicaId::default();
        let mut register = LwwRegister::new("old_value", replica_id);
        
        register.update("new_value", replica_id);
        assert_eq!(register.value(), &"new_value");
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
    fn test_lww_register_conflict_detection() {
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();
        
        // Create registers with same timestamp but different replica IDs
        let timestamp = chrono::Utc::now();
        let reg1 = LwwRegister::new("value1", replica_id1).with_timestamp(timestamp);
        let reg2 = LwwRegister::new("value2", replica_id2).with_timestamp(timestamp);
        
        assert!(reg1.has_conflict(&reg2));
    }

    #[test]
    fn test_lww_register_serialization() {
        let replica_id = ReplicaId::default();
        let register = LwwRegister::new("test", replica_id);
        
        let serialized = serde_json::to_string(&register).unwrap();
        let deserialized: LwwRegister<String> = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(register.value(), deserialized.value());
        assert_eq!(register.replica_id(), deserialized.replica_id());
    }
}
