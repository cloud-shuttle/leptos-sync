//! WebSocket message protocol for CRDT synchronization

use crate::crdt::ReplicaId;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// CRDT types that can be synchronized
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrdtType {
    LwwRegister,
    LwwMap,
    GCounter,
    RGA,
    LSEQ,
    YjsTree,
    DAG,
    Graph,
    Tree,
}

/// User information for peer identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// WebSocket synchronization message protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// CRDT delta message
    Delta {
        collection_id: String,
        crdt_type: CrdtType,
        delta: Vec<u8>,
        timestamp: SystemTime,
        replica_id: ReplicaId,
    },
    /// Heartbeat message to maintain connection
    Heartbeat {
        replica_id: ReplicaId,
        timestamp: SystemTime,
    },
    /// Peer joining the session
    PeerJoin {
        replica_id: ReplicaId,
        user_info: Option<UserInfo>,
    },
    /// Peer leaving the session
    PeerLeave { replica_id: ReplicaId },
    /// Welcome message from server
    Welcome {
        peer_id: ReplicaId,
        timestamp: SystemTime,
        server_info: Option<ServerInfo>,
    },
    /// Presence update
    Presence {
        peer_id: ReplicaId,
        action: PresenceAction,
        timestamp: SystemTime,
    },
    /// Binary data acknowledgment
    BinaryAck {
        peer_id: ReplicaId,
        size: usize,
        timestamp: SystemTime,
    },
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub max_connections: Option<usize>,
    pub features: Vec<String>,
}

/// Presence action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceAction {
    Join,
    Leave,
    Update,
}

/// Message wrapper with protocol versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWrapper {
    pub version: u32,
    pub message: SyncMessage,
    pub message_id: Option<String>,
}

impl MessageWrapper {
    pub const PROTOCOL_VERSION: u32 = 1;

    pub fn new(message: SyncMessage) -> Self {
        Self {
            version: Self::PROTOCOL_VERSION,
            message,
            message_id: None,
        }
    }

    pub fn with_id(message: SyncMessage, message_id: String) -> Self {
        Self {
            version: Self::PROTOCOL_VERSION,
            message,
            message_id: Some(message_id),
        }
    }
}

/// Message serialization/deserialization utilities
pub struct MessageCodec;

impl MessageCodec {
    /// Serialize a message to JSON bytes
    pub fn serialize(message: &SyncMessage) -> Result<Vec<u8>, serde_json::Error> {
        let wrapper = MessageWrapper::new(message.clone());
        serde_json::to_vec(&wrapper)
    }

    /// Deserialize JSON bytes to a message
    pub fn deserialize(data: &[u8]) -> Result<SyncMessage, serde_json::Error> {
        let wrapper: MessageWrapper = serde_json::from_slice(data)?;

        // Check protocol version compatibility
        if wrapper.version > MessageWrapper::PROTOCOL_VERSION {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Unsupported protocol version: {} (supported: {})",
                    wrapper.version,
                    MessageWrapper::PROTOCOL_VERSION
                ),
            )));
        }

        Ok(wrapper.message)
    }

    /// Serialize a message with compression (if enabled)
    pub fn serialize_compressed(
        message: &SyncMessage,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json_data = Self::serialize(message)?;

        // For now, just return JSON data
        // TODO: Add compression support when compression feature is enabled
        Ok(json_data)
    }

    /// Deserialize compressed message data
    pub fn deserialize_compressed(data: &[u8]) -> Result<SyncMessage, Box<dyn std::error::Error>> {
        // For now, just deserialize as JSON
        // TODO: Add decompression support when compression feature is enabled
        Self::deserialize(data).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::ReplicaId;
    use std::time::UNIX_EPOCH;

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(uuid::Uuid::new_v4())
    }

    #[test]
    fn test_message_serialization() {
        let replica_id = create_test_replica_id();
        let message = SyncMessage::Heartbeat {
            replica_id: replica_id.clone(),
            timestamp: UNIX_EPOCH,
        };

        let serialized = MessageCodec::serialize(&message).unwrap();
        let deserialized = MessageCodec::deserialize(&serialized).unwrap();

        match (message, deserialized) {
            (
                SyncMessage::Heartbeat {
                    replica_id: id1,
                    timestamp: t1,
                },
                SyncMessage::Heartbeat {
                    replica_id: id2,
                    timestamp: t2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(t1, t2);
            }
            _ => panic!("Message types don't match"),
        }
    }

    #[test]
    fn test_delta_message() {
        let replica_id = create_test_replica_id();
        let delta_data = b"test delta data".to_vec();

        let message = SyncMessage::Delta {
            collection_id: "test_collection".to_string(),
            crdt_type: CrdtType::LwwRegister,
            delta: delta_data.clone(),
            timestamp: UNIX_EPOCH,
            replica_id: replica_id.clone(),
        };

        let serialized = MessageCodec::serialize(&message).unwrap();
        let deserialized = MessageCodec::deserialize(&serialized).unwrap();

        match deserialized {
            SyncMessage::Delta {
                collection_id,
                crdt_type,
                delta,
                timestamp,
                replica_id: id,
            } => {
                assert_eq!(collection_id, "test_collection");
                assert_eq!(crdt_type, CrdtType::LwwRegister);
                assert_eq!(delta, delta_data);
                assert_eq!(timestamp, UNIX_EPOCH);
                assert_eq!(id, replica_id);
            }
            _ => panic!("Expected Delta message"),
        }
    }

    #[test]
    fn test_peer_join_message() {
        let replica_id = create_test_replica_id();
        let user_info = UserInfo {
            user_id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };

        let message = SyncMessage::PeerJoin {
            replica_id: replica_id.clone(),
            user_info: Some(user_info.clone()),
        };

        let serialized = MessageCodec::serialize(&message).unwrap();
        let deserialized = MessageCodec::deserialize(&serialized).unwrap();

        match deserialized {
            SyncMessage::PeerJoin {
                replica_id: id,
                user_info: info,
            } => {
                assert_eq!(id, replica_id);
                assert_eq!(info, Some(user_info));
            }
            _ => panic!("Expected PeerJoin message"),
        }
    }

    #[test]
    fn test_message_wrapper() {
        let replica_id = create_test_replica_id();
        let message = SyncMessage::Heartbeat {
            replica_id,
            timestamp: UNIX_EPOCH,
        };

        let wrapper = MessageWrapper::new(message.clone());
        assert_eq!(wrapper.version, MessageWrapper::PROTOCOL_VERSION);
        assert_eq!(wrapper.message_id, None);

        let wrapper_with_id = MessageWrapper::with_id(message, "msg123".to_string());
        assert_eq!(wrapper_with_id.version, MessageWrapper::PROTOCOL_VERSION);
        assert_eq!(wrapper_with_id.message_id, Some("msg123".to_string()));
    }

    #[test]
    fn test_compressed_serialization() {
        let replica_id = create_test_replica_id();
        let message = SyncMessage::Heartbeat {
            replica_id,
            timestamp: UNIX_EPOCH,
        };

        // Test that compressed serialization works (currently just JSON)
        let compressed = MessageCodec::serialize_compressed(&message).unwrap();
        let decompressed = MessageCodec::deserialize_compressed(&compressed).unwrap();

        match (message, decompressed) {
            (
                SyncMessage::Heartbeat {
                    replica_id: id1,
                    timestamp: t1,
                },
                SyncMessage::Heartbeat {
                    replica_id: id2,
                    timestamp: t2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(t1, t2);
            }
            _ => panic!("Message types don't match"),
        }
    }
}
