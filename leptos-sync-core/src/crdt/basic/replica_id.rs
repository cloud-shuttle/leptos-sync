//! Replica ID type and utilities

use serde::{Deserialize, Serialize};
use std::fmt;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replica_id_creation() {
        let replica_id = ReplicaId::default();
        assert_ne!(replica_id.0, Uuid::nil());
    }

    #[test]
    fn test_replica_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let replica_id = ReplicaId::from(uuid);
        assert_eq!(replica_id.0, uuid);
    }

    #[test]
    fn test_replica_id_serialization() {
        let replica_id = ReplicaId::default();
        let serialized = serde_json::to_string(&replica_id).unwrap();
        let deserialized: ReplicaId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(replica_id, deserialized);
    }

    #[test]
    fn test_replica_id_display() {
        let replica_id = ReplicaId::default();
        let display_str = format!("{}", replica_id);
        assert!(!display_str.is_empty());
    }

    #[test]
    fn test_replica_id_ordering() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        
        let replica_id1 = ReplicaId::from(uuid1);
        let replica_id2 = ReplicaId::from(uuid2);
        
        // Test that ordering is consistent
        let result1 = replica_id1 < replica_id2;
        let result2 = replica_id2 > replica_id1;
        assert_eq!(result1, result2);
    }
}
