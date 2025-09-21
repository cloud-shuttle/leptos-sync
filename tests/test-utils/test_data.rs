//! Test data utilities
//! 
//! Provides test data generators and utilities for comprehensive testing.

use leptos_sync_core::crdt::{LwwRegister, LwwMap, GCounter, ReplicaId};
use leptos_sync_core::transport::message_protocol::{SyncMessage, UserInfo, ServerInfo, PresenceAction};
use std::time::SystemTime;
use uuid::Uuid;

/// Generate random test data
pub struct TestDataGenerator {
    replica_id: ReplicaId,
}

impl TestDataGenerator {
    /// Create a new test data generator
    pub fn new() -> Self {
        Self {
            replica_id: ReplicaId::from(Uuid::new_v4()),
        }
    }
    
    /// Create a generator with a specific replica ID
    pub fn with_replica_id(replica_id: ReplicaId) -> Self {
        Self { replica_id }
    }
    
    /// Generate a random string
    pub fn random_string(&self, length: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.replica_id.hash(&mut hasher);
        length.hash(&mut hasher);
        
        let hash = hasher.finish();
        format!("test_string_{}_{}", hash, length)
    }
    
    /// Generate a random integer
    pub fn random_int(&self, max: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.replica_id.hash(&mut hasher);
        max.hash(&mut hasher);
        
        let hash = hasher.finish();
        hash % max
    }
    
    /// Generate a random timestamp
    pub fn random_timestamp(&self) -> SystemTime {
        let offset = self.random_int(1000000);
        SystemTime::now() + std::time::Duration::from_millis(offset)
    }
    
    /// Generate a random replica ID
    pub fn random_replica_id(&self) -> ReplicaId {
        ReplicaId::from(Uuid::new_v4())
    }
    
    /// Generate a random user info
    pub fn random_user_info(&self) -> UserInfo {
        UserInfo {
            user_id: format!("user_{}", self.random_int(1000)),
            username: Some(format!("username_{}", self.random_int(1000))),
            display_name: Some(format!("Display Name {}", self.random_int(1000))),
            avatar_url: Some(format!("https://example.com/avatar_{}.jpg", self.random_int(1000))),
        }
    }
    
    /// Generate a random server info
    pub fn random_server_info(&self) -> ServerInfo {
        ServerInfo {
            server_id: format!("server_{}", self.random_int(100)),
            version: "0.8.4".to_string(),
            capabilities: vec![
                "crdt_sync".to_string(),
                "presence".to_string(),
                "compression".to_string(),
            ],
        }
    }
    
    /// Generate a random LWW register
    pub fn random_lww_register(&self) -> LwwRegister<String> {
        let value = self.random_string(20);
        let timestamp = self.random_timestamp();
        let replica_id = self.random_replica_id();
        
        LwwRegister::new(value, timestamp, replica_id)
    }
    
    /// Generate a random LWW map
    pub fn random_lww_map(&self, size: usize) -> LwwMap<String> {
        let mut map = LwwMap::new();
        let replica_id = self.random_replica_id();
        let timestamp = self.random_timestamp();
        
        for i in 0..size {
            let key = format!("key_{}", i);
            let value = self.random_string(10);
            map.set(&key, value, timestamp, replica_id);
        }
        
        map
    }
    
    /// Generate a random GCounter
    pub fn random_gcounter(&self, max_increments: u64) -> GCounter {
        let mut counter = GCounter::new();
        let increments = self.random_int(max_increments) + 1;
        
        for _ in 0..increments {
            counter.increment().unwrap();
        }
        
        counter
    }
    
    /// Generate a random delta message
    pub fn random_delta_message(&self) -> SyncMessage {
        SyncMessage::Delta {
            collection_id: format!("collection_{}", self.random_int(100)),
            crdt_type: leptos_sync_core::crdt::CrdtType::LwwRegister,
            delta: vec![self.random_int(255) as u8; 10],
            timestamp: self.random_timestamp(),
            replica_id: self.random_replica_id(),
        }
    }
    
    /// Generate a random heartbeat message
    pub fn random_heartbeat_message(&self) -> SyncMessage {
        SyncMessage::Heartbeat {
            replica_id: self.random_replica_id(),
            timestamp: self.random_timestamp(),
        }
    }
    
    /// Generate a random peer join message
    pub fn random_peer_join_message(&self) -> SyncMessage {
        SyncMessage::PeerJoin {
            replica_id: self.random_replica_id(),
            user_info: Some(self.random_user_info()),
        }
    }
    
    /// Generate a random peer leave message
    pub fn random_peer_leave_message(&self) -> SyncMessage {
        SyncMessage::PeerLeave {
            replica_id: self.random_replica_id(),
        }
    }
    
    /// Generate a random welcome message
    pub fn random_welcome_message(&self) -> SyncMessage {
        SyncMessage::Welcome {
            peer_id: self.random_replica_id(),
            timestamp: self.random_timestamp(),
            server_info: self.random_server_info(),
        }
    }
    
    /// Generate a random presence message
    pub fn random_presence_message(&self) -> SyncMessage {
        let actions = vec![PresenceAction::Join, PresenceAction::Leave, PresenceAction::Update];
        let action = actions[self.random_int(3) as usize];
        
        SyncMessage::Presence {
            peer_id: self.random_replica_id(),
            action,
            timestamp: self.random_timestamp(),
        }
    }
    
    /// Generate a random binary ack message
    pub fn random_binary_ack_message(&self) -> SyncMessage {
        SyncMessage::BinaryAck {
            peer_id: self.random_replica_id(),
            size: self.random_int(10000) as usize,
            timestamp: self.random_timestamp(),
        }
    }
    
    /// Generate a random sync message of any type
    pub fn random_sync_message(&self) -> SyncMessage {
        let message_types = vec![
            "delta", "heartbeat", "peer_join", "peer_leave", 
            "welcome", "presence", "binary_ack"
        ];
        let message_type = message_types[self.random_int(7) as usize];
        
        match message_type {
            "delta" => self.random_delta_message(),
            "heartbeat" => self.random_heartbeat_message(),
            "peer_join" => self.random_peer_join_message(),
            "peer_leave" => self.random_peer_leave_message(),
            "welcome" => self.random_welcome_message(),
            "presence" => self.random_presence_message(),
            "binary_ack" => self.random_binary_ack_message(),
            _ => self.random_heartbeat_message(),
        }
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data sets for different scenarios
pub struct TestDataSets;

impl TestDataSets {
    /// Generate a small dataset for quick tests
    pub fn small() -> TestDataGenerator {
        TestDataGenerator::new()
    }
    
    /// Generate a medium dataset for standard tests
    pub fn medium() -> TestDataGenerator {
        TestDataGenerator::new()
    }
    
    /// Generate a large dataset for stress tests
    pub fn large() -> TestDataGenerator {
        TestDataGenerator::new()
    }
    
    /// Generate a dataset with specific replica ID
    pub fn with_replica_id(replica_id: ReplicaId) -> TestDataGenerator {
        TestDataGenerator::with_replica_id(replica_id)
    }
}

/// Test data validation utilities
pub struct TestDataValidator;

impl TestDataValidator {
    /// Validate that a LWW register has valid data
    pub fn validate_lww_register(register: &LwwRegister<String>) -> bool {
        !register.value().is_empty()
    }
    
    /// Validate that a LWW map has valid data
    pub fn validate_lww_map(map: &LwwMap<String>) -> bool {
        map.len() > 0
    }
    
    /// Validate that a GCounter has valid data
    pub fn validate_gcounter(counter: &GCounter) -> bool {
        counter.value() >= 0
    }
    
    /// Validate that a sync message has valid data
    pub fn validate_sync_message(message: &SyncMessage) -> bool {
        match message {
            SyncMessage::Delta { collection_id, delta, .. } => {
                !collection_id.is_empty() && !delta.is_empty()
            }
            SyncMessage::Heartbeat { .. } => true,
            SyncMessage::PeerJoin { user_info, .. } => {
                user_info.as_ref().map_or(true, |info| !info.user_id.is_empty())
            }
            SyncMessage::PeerLeave { .. } => true,
            SyncMessage::Welcome { server_info, .. } => {
                !server_info.server_id.is_empty() && !server_info.version.is_empty()
            }
            SyncMessage::Presence { .. } => true,
            SyncMessage::BinaryAck { size, .. } => *size > 0,
        }
    }
}

/// Test data comparison utilities
pub struct TestDataComparator;

impl TestDataComparator {
    /// Compare two LWW registers for equality
    pub fn compare_lww_registers(a: &LwwRegister<String>, b: &LwwRegister<String>) -> bool {
        a.value() == b.value()
    }
    
    /// Compare two LWW maps for equality
    pub fn compare_lww_maps(a: &LwwMap<String>, b: &LwwMap<String>) -> bool {
        a.len() == b.len()
    }
    
    /// Compare two GCounters for equality
    pub fn compare_gcounters(a: &GCounter, b: &GCounter) -> bool {
        a.value() == b.value()
    }
    
    /// Compare two sync messages for equality
    pub fn compare_sync_messages(a: &SyncMessage, b: &SyncMessage) -> bool {
        match (a, b) {
            (SyncMessage::Delta { collection_id: id1, delta: delta1, .. },
             SyncMessage::Delta { collection_id: id2, delta: delta2, .. }) => {
                id1 == id2 && delta1 == delta2
            }
            (SyncMessage::Heartbeat { replica_id: id1, .. },
             SyncMessage::Heartbeat { replica_id: id2, .. }) => {
                id1 == id2
            }
            (SyncMessage::PeerJoin { replica_id: id1, user_info: info1 },
             SyncMessage::PeerJoin { replica_id: id2, user_info: info2 }) => {
                id1 == id2 && info1 == info2
            }
            (SyncMessage::PeerLeave { replica_id: id1 },
             SyncMessage::PeerLeave { replica_id: id2 }) => {
                id1 == id2
            }
            (SyncMessage::Welcome { peer_id: id1, server_info: info1, .. },
             SyncMessage::Welcome { peer_id: id2, server_info: info2, .. }) => {
                id1 == id2 && info1 == info2
            }
            (SyncMessage::Presence { peer_id: id1, action: action1, .. },
             SyncMessage::Presence { peer_id: id2, action: action2, .. }) => {
                id1 == id2 && action1 == action2
            }
            (SyncMessage::BinaryAck { peer_id: id1, size: size1, .. },
             SyncMessage::BinaryAck { peer_id: id2, size: size2, .. }) => {
                id1 == id2 && size1 == size2
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_generator_creation() {
        let generator = TestDataGenerator::new();
        assert!(!generator.replica_id.to_string().is_empty());
    }

    #[test]
    fn test_random_string_generation() {
        let generator = TestDataGenerator::new();
        let string1 = generator.random_string(10);
        let string2 = generator.random_string(10);
        
        assert_eq!(string1.len(), 10);
        assert_eq!(string2.len(), 10);
        assert_ne!(string1, string2);
    }

    #[test]
    fn test_random_int_generation() {
        let generator = TestDataGenerator::new();
        let int1 = generator.random_int(100);
        let int2 = generator.random_int(100);
        
        assert!(int1 < 100);
        assert!(int2 < 100);
    }

    #[test]
    fn test_random_lww_register_generation() {
        let generator = TestDataGenerator::new();
        let register = generator.random_lww_register();
        
        assert!(TestDataValidator::validate_lww_register(&register));
    }

    #[test]
    fn test_random_lww_map_generation() {
        let generator = TestDataGenerator::new();
        let map = generator.random_lww_map(10);
        
        assert!(TestDataValidator::validate_lww_map(&map));
        assert_eq!(map.len(), 10);
    }

    #[test]
    fn test_random_gcounter_generation() {
        let generator = TestDataGenerator::new();
        let counter = generator.random_gcounter(100);
        
        assert!(TestDataValidator::validate_gcounter(&counter));
    }

    #[test]
    fn test_random_sync_message_generation() {
        let generator = TestDataGenerator::new();
        let message = generator.random_sync_message();
        
        assert!(TestDataValidator::validate_sync_message(&message));
    }

    #[test]
    fn test_data_validation() {
        let generator = TestDataGenerator::new();
        
        let register = generator.random_lww_register();
        assert!(TestDataValidator::validate_lww_register(&register));
        
        let map = generator.random_lww_map(5);
        assert!(TestDataValidator::validate_lww_map(&map));
        
        let counter = generator.random_gcounter(50);
        assert!(TestDataValidator::validate_gcounter(&counter));
        
        let message = generator.random_sync_message();
        assert!(TestDataValidator::validate_sync_message(&message));
    }

    #[test]
    fn test_data_comparison() {
        let generator = TestDataGenerator::new();
        
        let register1 = generator.random_lww_register();
        let register2 = generator.random_lww_register();
        
        // They should be different
        assert!(!TestDataComparator::compare_lww_registers(&register1, &register2));
        
        // But a register should equal itself
        assert!(TestDataComparator::compare_lww_registers(&register1, &register1));
    }
}
