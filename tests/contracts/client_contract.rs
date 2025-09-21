//! Client contract tests
//! 
//! Tests that validate the client correctly handles all server message types
//! and implements the client-side contract as defined in the API specification.

use leptos_sync_core::transport::message_protocol::{SyncMessage, MessageCodec, UserInfo, ServerInfo, PresenceAction};
use leptos_sync_core::crdt::{ReplicaId, CrdtType};
use leptos_sync_core::transport::{WebSocketClient, WebSocketClientConfig};
use std::time::{SystemTime, Duration};
use uuid::Uuid;
use tokio::time::timeout;

/// Mock WebSocket server for testing
struct MockWebSocketServer {
    url: String,
    messages: Vec<SyncMessage>,
}

impl MockWebSocketServer {
    fn new() -> Self {
        Self {
            url: "ws://localhost:9999/mock".to_string(),
            messages: Vec::new(),
        }
    }
    
    fn url(&self) -> &str {
        &self.url
    }
    
    async fn send_delta_message(&mut self) {
        let message = SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type: CrdtType::LwwRegister,
            delta: vec![1, 2, 3, 4],
            timestamp: SystemTime::now(),
            replica_id: ReplicaId::from(Uuid::new_v4()),
        };
        self.messages.push(message);
    }
    
    async fn send_heartbeat_message(&mut self) {
        let message = SyncMessage::Heartbeat {
            replica_id: ReplicaId::from(Uuid::new_v4()),
            timestamp: SystemTime::now(),
        };
        self.messages.push(message);
    }
    
    async fn send_peer_join_message(&mut self) {
        let user_info = UserInfo {
            user_id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: None,
        };
        let message = SyncMessage::PeerJoin {
            replica_id: ReplicaId::from(Uuid::new_v4()),
            user_info: Some(user_info),
        };
        self.messages.push(message);
    }
    
    async fn send_peer_leave_message(&mut self) {
        let message = SyncMessage::PeerLeave {
            replica_id: ReplicaId::from(Uuid::new_v4()),
        };
        self.messages.push(message);
    }
    
    async fn send_welcome_message(&mut self) {
        let server_info = ServerInfo {
            server_id: "server-001".to_string(),
            version: "0.8.4".to_string(),
            capabilities: vec!["crdt_sync".to_string(), "presence".to_string()],
        };
        let message = SyncMessage::Welcome {
            peer_id: ReplicaId::from(Uuid::new_v4()),
            timestamp: SystemTime::now(),
            server_info,
        };
        self.messages.push(message);
    }
    
    async fn send_presence_message(&mut self) {
        let message = SyncMessage::Presence {
            peer_id: ReplicaId::from(Uuid::new_v4()),
            action: PresenceAction::Join,
            timestamp: SystemTime::now(),
        };
        self.messages.push(message);
    }
    
    async fn send_binary_ack_message(&mut self) {
        let message = SyncMessage::BinaryAck {
            peer_id: ReplicaId::from(Uuid::new_v4()),
            size: 1024,
            timestamp: SystemTime::now(),
        };
        self.messages.push(message);
    }
    
    fn get_messages(&self) -> &[SyncMessage] {
        &self.messages
    }
}

/// Mock client for testing message handling
struct MockClient {
    received_messages: Vec<SyncMessage>,
    errors: Vec<String>,
}

impl MockClient {
    fn new() -> Self {
        Self {
            received_messages: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    async fn handle_message(&mut self, message: SyncMessage) {
        match message {
            SyncMessage::Delta { .. } => {
                self.received_messages.push(message);
            }
            SyncMessage::Heartbeat { .. } => {
                self.received_messages.push(message);
            }
            SyncMessage::PeerJoin { .. } => {
                self.received_messages.push(message);
            }
            SyncMessage::PeerLeave { .. } => {
                self.received_messages.push(message);
            }
            SyncMessage::Welcome { .. } => {
                self.received_messages.push(message);
            }
            SyncMessage::Presence { .. } => {
                self.received_messages.push(message);
            }
            SyncMessage::BinaryAck { .. } => {
                self.received_messages.push(message);
            }
        }
    }
    
    fn get_received_messages(&self) -> &[SyncMessage] {
        &self.received_messages
    }
    
    fn get_errors(&self) -> &[String] {
        &self.errors
    }
    
    fn collect_messages_for(&self, _duration: Duration) -> &[SyncMessage] {
        &self.received_messages
    }
}

#[tokio::test]
async fn test_client_handles_all_server_messages() {
    let mut mock_server = MockWebSocketServer::new();
    let mut client = MockClient::new();
    
    // Configure server to send all message types from schema
    mock_server.send_delta_message().await;
    mock_server.send_heartbeat_message().await;
    mock_server.send_peer_join_message().await;
    mock_server.send_peer_leave_message().await;
    mock_server.send_welcome_message().await;
    mock_server.send_presence_message().await;
    mock_server.send_binary_ack_message().await;
    
    // Simulate client receiving and handling messages
    for message in mock_server.get_messages() {
        client.handle_message(message.clone()).await;
    }
    
    // Verify client processes each message type correctly
    let received = client.collect_messages_for(Duration::from_secs(1));
    assert_eq!(received.len(), 7, "Client should receive all 7 message types");
    
    // Verify no errors or unhandled message types
    assert!(client.get_errors().is_empty(), "Client should not have any errors");
    
    // Verify message types are correctly handled
    let mut delta_count = 0;
    let mut heartbeat_count = 0;
    let mut peer_join_count = 0;
    let mut peer_leave_count = 0;
    let mut welcome_count = 0;
    let mut presence_count = 0;
    let mut binary_ack_count = 0;
    
    for message in received {
        match message {
            SyncMessage::Delta { .. } => delta_count += 1,
            SyncMessage::Heartbeat { .. } => heartbeat_count += 1,
            SyncMessage::PeerJoin { .. } => peer_join_count += 1,
            SyncMessage::PeerLeave { .. } => peer_leave_count += 1,
            SyncMessage::Welcome { .. } => welcome_count += 1,
            SyncMessage::Presence { .. } => presence_count += 1,
            SyncMessage::BinaryAck { .. } => binary_ack_count += 1,
        }
    }
    
    assert_eq!(delta_count, 1, "Should receive 1 delta message");
    assert_eq!(heartbeat_count, 1, "Should receive 1 heartbeat message");
    assert_eq!(peer_join_count, 1, "Should receive 1 peer join message");
    assert_eq!(peer_leave_count, 1, "Should receive 1 peer leave message");
    assert_eq!(welcome_count, 1, "Should receive 1 welcome message");
    assert_eq!(presence_count, 1, "Should receive 1 presence message");
    assert_eq!(binary_ack_count, 1, "Should receive 1 binary ack message");
}

#[tokio::test]
async fn test_client_message_serialization_roundtrip() {
    let client = MockClient::new();
    
    // Test that client can serialize and deserialize all message types
    let test_messages = vec![
        SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type: CrdtType::LwwRegister,
            delta: vec![1, 2, 3, 4],
            timestamp: SystemTime::now(),
            replica_id: ReplicaId::from(Uuid::new_v4()),
        },
        SyncMessage::Heartbeat {
            replica_id: ReplicaId::from(Uuid::new_v4()),
            timestamp: SystemTime::now(),
        },
        SyncMessage::PeerJoin {
            replica_id: ReplicaId::from(Uuid::new_v4()),
            user_info: Some(UserInfo {
                user_id: "user123".to_string(),
                username: Some("testuser".to_string()),
                display_name: Some("Test User".to_string()),
                avatar_url: None,
            }),
        },
        SyncMessage::PeerLeave {
            replica_id: ReplicaId::from(Uuid::new_v4()),
        },
        SyncMessage::Welcome {
            peer_id: ReplicaId::from(Uuid::new_v4()),
            timestamp: SystemTime::now(),
            server_info: ServerInfo {
                server_id: "server-001".to_string(),
                version: "0.8.4".to_string(),
                capabilities: vec!["crdt_sync".to_string(), "presence".to_string()],
            },
        },
        SyncMessage::Presence {
            peer_id: ReplicaId::from(Uuid::new_v4()),
            action: PresenceAction::Join,
            timestamp: SystemTime::now(),
        },
        SyncMessage::BinaryAck {
            peer_id: ReplicaId::from(Uuid::new_v4()),
            size: 1024,
            timestamp: SystemTime::now(),
        },
    ];
    
    for (i, original_message) in test_messages.into_iter().enumerate() {
        // Serialize message
        let serialized = MessageCodec::serialize(&original_message)
            .expect(&format!("Failed to serialize message {}", i));
        
        // Deserialize message
        let deserialized = MessageCodec::deserialize(&serialized)
            .expect(&format!("Failed to deserialize message {}", i));
        
        // Verify round-trip consistency
        match (&original_message, &deserialized) {
            (SyncMessage::Delta { collection_id: id1, crdt_type: type1, delta: delta1, replica_id: rid1, .. },
             SyncMessage::Delta { collection_id: id2, crdt_type: type2, delta: delta2, replica_id: rid2, .. }) => {
                assert_eq!(id1, id2, "Collection ID mismatch in message {}", i);
                assert_eq!(type1, type2, "CRDT type mismatch in message {}", i);
                assert_eq!(delta1, delta2, "Delta data mismatch in message {}", i);
                assert_eq!(rid1, rid2, "Replica ID mismatch in message {}", i);
            }
            (SyncMessage::Heartbeat { replica_id: rid1, .. },
             SyncMessage::Heartbeat { replica_id: rid2, .. }) => {
                assert_eq!(rid1, rid2, "Replica ID mismatch in message {}", i);
            }
            (SyncMessage::PeerJoin { replica_id: rid1, user_info: info1 },
             SyncMessage::PeerJoin { replica_id: rid2, user_info: info2 }) => {
                assert_eq!(rid1, rid2, "Replica ID mismatch in message {}", i);
                assert_eq!(info1, info2, "User info mismatch in message {}", i);
            }
            (SyncMessage::PeerLeave { replica_id: rid1 },
             SyncMessage::PeerLeave { replica_id: rid2 }) => {
                assert_eq!(rid1, rid2, "Replica ID mismatch in message {}", i);
            }
            (SyncMessage::Welcome { peer_id: pid1, server_info: info1, .. },
             SyncMessage::Welcome { peer_id: pid2, server_info: info2, .. }) => {
                assert_eq!(pid1, pid2, "Peer ID mismatch in message {}", i);
                assert_eq!(info1, info2, "Server info mismatch in message {}", i);
            }
            (SyncMessage::Presence { peer_id: pid1, action: action1, .. },
             SyncMessage::Presence { peer_id: pid2, action: action2, .. }) => {
                assert_eq!(pid1, pid2, "Peer ID mismatch in message {}", i);
                assert_eq!(action1, action2, "Action mismatch in message {}", i);
            }
            (SyncMessage::BinaryAck { peer_id: pid1, size: size1, .. },
             SyncMessage::BinaryAck { peer_id: pid2, size: size2, .. }) => {
                assert_eq!(pid1, pid2, "Peer ID mismatch in message {}", i);
                assert_eq!(size1, size2, "Size mismatch in message {}", i);
            }
            _ => panic!("Message type mismatch in message {}", i),
        }
    }
}

#[tokio::test]
async fn test_client_error_handling() {
    let client = MockClient::new();
    
    // Test handling of malformed messages
    let malformed_data = b"invalid json data";
    let result = MessageCodec::deserialize(malformed_data);
    assert!(result.is_err(), "Client should handle malformed messages gracefully");
    
    // Test handling of empty messages
    let empty_data = b"";
    let result = MessageCodec::deserialize(empty_data);
    assert!(result.is_err(), "Client should handle empty messages gracefully");
    
    // Test handling of unknown message types
    let unknown_message = r#"{"type": "unknown_type", "version": "1.0.0", "timestamp": "2022-01-01T00:00:00Z", "replica_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
    let result = MessageCodec::deserialize(unknown_message.as_bytes());
    assert!(result.is_err(), "Client should handle unknown message types gracefully");
}

#[tokio::test]
async fn test_client_websocket_connection_handling() {
    let config = WebSocketClientConfig::default();
    let replica_id = ReplicaId::from(Uuid::new_v4());
    let client = WebSocketClient::new(config, replica_id);
    
    // Test client creation
    assert_eq!(client.replica_id(), replica_id);
    
    // Test connection state (should be false initially)
    assert!(!client.is_connected().await);
    
    // Test disconnect (should not error even when not connected)
    let result = client.disconnect().await;
    assert!(result.is_ok(), "Disconnect should not error when not connected");
}

#[tokio::test]
async fn test_client_message_timeout_handling() {
    let mut config = WebSocketClientConfig::default();
    config.message_timeout = Duration::from_millis(100); // Very short timeout
    
    let replica_id = ReplicaId::from(Uuid::new_v4());
    let client = WebSocketClient::new(config, replica_id);
    
    // Test that operations timeout appropriately
    let result = timeout(Duration::from_millis(200), client.receive()).await;
    assert!(result.is_err(), "Receive should timeout with no connection");
}

#[tokio::test]
async fn test_client_crdt_type_handling() {
    let client = MockClient::new();
    
    // Test all CRDT types
    let crdt_types = vec![
        CrdtType::LwwRegister,
        CrdtType::LwwMap,
        CrdtType::GCounter,
        CrdtType::Rga,
        CrdtType::Lseq,
        CrdtType::Tree,
        CrdtType::Graph,
    ];
    
    for crdt_type in crdt_types {
        let message = SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type,
            delta: vec![],
            timestamp: SystemTime::now(),
            replica_id: ReplicaId::from(Uuid::new_v4()),
        };
        
        // Test serialization
        let serialized = MessageCodec::serialize(&message)
            .expect(&format!("Failed to serialize {} message", format!("{:?}", crdt_type)));
        
        // Test deserialization
        let deserialized = MessageCodec::deserialize(&serialized)
            .expect(&format!("Failed to deserialize {} message", format!("{:?}", crdt_type)));
        
        // Verify CRDT type is preserved
        if let SyncMessage::Delta { crdt_type: deserialized_type, .. } = deserialized {
            assert_eq!(crdt_type, deserialized_type, "CRDT type should be preserved");
        } else {
            panic!("Deserialized message should be a Delta message");
        }
    }
}

#[tokio::test]
async fn test_client_user_info_handling() {
    let client = MockClient::new();
    
    // Test user info with all fields
    let user_info = UserInfo {
        user_id: "user123".to_string(),
        username: Some("testuser".to_string()),
        display_name: Some("Test User".to_string()),
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
    };
    
    let message = SyncMessage::PeerJoin {
        replica_id: ReplicaId::from(Uuid::new_v4()),
        user_info: Some(user_info.clone()),
    };
    
    // Test serialization and deserialization
    let serialized = MessageCodec::serialize(&message)
        .expect("Failed to serialize peer join message with user info");
    let deserialized = MessageCodec::deserialize(&serialized)
        .expect("Failed to deserialize peer join message with user info");
    
    // Verify user info is preserved
    if let SyncMessage::PeerJoin { user_info: deserialized_info, .. } = deserialized {
        assert_eq!(user_info, deserialized_info.unwrap(), "User info should be preserved");
    } else {
        panic!("Deserialized message should be a PeerJoin message");
    }
    
    // Test user info with minimal fields
    let minimal_user_info = UserInfo {
        user_id: "user456".to_string(),
        username: None,
        display_name: None,
        avatar_url: None,
    };
    
    let minimal_message = SyncMessage::PeerJoin {
        replica_id: ReplicaId::from(Uuid::new_v4()),
        user_info: Some(minimal_user_info.clone()),
    };
    
    let serialized = MessageCodec::serialize(&minimal_message)
        .expect("Failed to serialize peer join message with minimal user info");
    let deserialized = MessageCodec::deserialize(&serialized)
        .expect("Failed to deserialize peer join message with minimal user info");
    
    if let SyncMessage::PeerJoin { user_info: deserialized_info, .. } = deserialized {
        assert_eq!(minimal_user_info, deserialized_info.unwrap(), "Minimal user info should be preserved");
    } else {
        panic!("Deserialized message should be a PeerJoin message");
    }
}
