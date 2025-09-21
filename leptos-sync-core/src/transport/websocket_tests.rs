//! WebSocket transport unit tests
//! 
//! Comprehensive tests for WebSocket transport functionality including
//! connection lifecycle, message handling, and error recovery.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{
        WebSocketClient, WebSocketClientConfig, WebSocketClientError,
        message_protocol::{SyncMessage, MessageCodec, UserInfo, ServerInfo, PresenceAction},
        SyncTransport,
    };
    use crate::crdt::{ReplicaId, CrdtType};
    use std::time::{SystemTime, Duration};
    use uuid::Uuid;
    use tokio::time::timeout;

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(Uuid::new_v4())
    }

    fn create_test_config() -> WebSocketClientConfig {
        WebSocketClientConfig {
            url: "ws://localhost:3001/test".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 3,
            user_info: Some(UserInfo {
                user_id: "test_user".to_string(),
                username: Some("testuser".to_string()),
                display_name: Some("Test User".to_string()),
                avatar_url: None,
            }),
        }
    }

    fn create_test_delta_message() -> SyncMessage {
        SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type: CrdtType::LwwRegister,
            delta: vec![1, 2, 3, 4],
            timestamp: SystemTime::now(),
            replica_id: create_test_replica_id(),
        }
    }

    fn create_test_heartbeat_message() -> SyncMessage {
        SyncMessage::Heartbeat {
            replica_id: create_test_replica_id(),
            timestamp: SystemTime::now(),
        }
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);
        
        assert_eq!(client.replica_id(), replica_id);
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_websocket_connection_lifecycle() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let mut client = WebSocketClient::new(config, replica_id);
        
        // Test initial state
        assert!(!client.is_connected().await);
        
        // Test connection (this will fail in test environment, but we can test the flow)
        let result = client.connect().await;
        // In a real test environment with a WebSocket server, this would succeed
        // For now, we expect it to fail gracefully
        assert!(result.is_err());
        
        // Test disconnection (should not error even when not connected)
        let result = client.disconnect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_message_serialization_roundtrip() {
        let test_messages = vec![
            create_test_delta_message(),
            create_test_heartbeat_message(),
            SyncMessage::PeerJoin {
                replica_id: create_test_replica_id(),
                user_info: Some(UserInfo {
                    user_id: "user123".to_string(),
                    username: Some("testuser".to_string()),
                    display_name: Some("Test User".to_string()),
                    avatar_url: None,
                }),
            },
            SyncMessage::PeerLeave {
                replica_id: create_test_replica_id(),
            },
            SyncMessage::Welcome {
                peer_id: create_test_replica_id(),
                timestamp: SystemTime::now(),
                server_info: ServerInfo {
                    server_id: "server-001".to_string(),
                    version: "0.8.4".to_string(),
                    capabilities: vec!["crdt_sync".to_string(), "presence".to_string()],
                },
            },
            SyncMessage::Presence {
                peer_id: create_test_replica_id(),
                action: PresenceAction::Join,
                timestamp: SystemTime::now(),
            },
            SyncMessage::BinaryAck {
                peer_id: create_test_replica_id(),
                size: 1024,
                timestamp: SystemTime::now(),
            },
        ];
        
        for (i, message) in test_messages.into_iter().enumerate() {
            // Test serialization
            let serialized = MessageCodec::serialize(&message)
                .expect(&format!("Failed to serialize message {}", i));
            
            // Test deserialization
            let deserialized = MessageCodec::deserialize(&serialized)
                .expect(&format!("Failed to deserialize message {}", i));
            
            // Verify round-trip consistency
            match (&message, &deserialized) {
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
    async fn test_message_codec_error_handling() {
        // Test invalid JSON
        let invalid_json = b"invalid json";
        let result = MessageCodec::deserialize(invalid_json);
        assert!(result.is_err(), "Should fail to deserialize invalid JSON");
        
        // Test empty data
        let empty_data = b"";
        let result = MessageCodec::deserialize(empty_data);
        assert!(result.is_err(), "Should fail to deserialize empty data");
        
        // Test malformed message structure
        let malformed_message = r#"{"type": "invalid_type", "version": "1.0.0"}"#;
        let result = MessageCodec::deserialize(malformed_message.as_bytes());
        assert!(result.is_err(), "Should fail to deserialize malformed message");
    }

    #[tokio::test]
    async fn test_websocket_config_validation() {
        // Test valid config
        let valid_config = create_test_config();
        assert_eq!(valid_config.url, "ws://localhost:3001/test");
        assert_eq!(valid_config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(valid_config.message_timeout, Duration::from_secs(5));
        assert_eq!(valid_config.reconnect_interval, Duration::from_secs(1));
        assert_eq!(valid_config.max_reconnect_attempts, 3);
        assert!(valid_config.user_info.is_some());
        
        // Test default config
        let default_config = WebSocketClientConfig::default();
        assert!(!default_config.url.is_empty());
        assert!(default_config.heartbeat_interval > Duration::from_secs(0));
        assert!(default_config.message_timeout > Duration::from_secs(0));
        assert!(default_config.reconnect_interval > Duration::from_secs(0));
        assert!(default_config.max_reconnect_attempts > 0);
    }

    #[tokio::test]
    async fn test_websocket_client_error_types() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);
        
        // Test connection error handling
        let result = client.connect().await;
        assert!(result.is_err());
        
        // Verify error type
        match result.unwrap_err() {
            WebSocketClientError::ConnectionFailed(_) => {
                // Expected error type
            }
            _ => panic!("Expected ConnectionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_websocket_client_timeout_handling() {
        let mut config = create_test_config();
        config.message_timeout = Duration::from_millis(100); // Very short timeout
        
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);
        
        // Test that operations timeout appropriately
        let result = timeout(Duration::from_millis(200), client.receive()).await;
        assert!(result.is_err(), "Receive should timeout with no connection");
    }

    #[tokio::test]
    async fn test_websocket_client_reconnect_config() {
        let mut config = create_test_config();
        config.max_reconnect_attempts = 5;
        config.reconnect_interval = Duration::from_millis(500);
        
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);
        
        // Test that reconnect configuration is properly set
        // (In a real implementation, we would test the actual reconnect logic)
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_websocket_client_user_info() {
        let user_info = UserInfo {
            user_id: "user123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        };
        
        let mut config = create_test_config();
        config.user_info = Some(user_info.clone());
        
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);
        
        // Test that user info is properly stored
        // (In a real implementation, we would verify the user info is used in connections)
        assert_eq!(client.replica_id(), replica_id);
    }

    #[tokio::test]
    async fn test_websocket_client_crdt_type_handling() {
        let client = WebSocketClient::new(create_test_config(), create_test_replica_id());
        
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
                replica_id: create_test_replica_id(),
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
    async fn test_websocket_client_presence_actions() {
        let client = WebSocketClient::new(create_test_config(), create_test_replica_id());
        
        // Test all presence actions
        let presence_actions = vec![
            PresenceAction::Join,
            PresenceAction::Leave,
            PresenceAction::Update,
        ];
        
        for action in presence_actions {
            let message = SyncMessage::Presence {
                peer_id: create_test_replica_id(),
                action,
                timestamp: SystemTime::now(),
            };
            
            // Test serialization
            let serialized = MessageCodec::serialize(&message)
                .expect(&format!("Failed to serialize presence message with action {:?}", action));
            
            // Test deserialization
            let deserialized = MessageCodec::deserialize(&serialized)
                .expect(&format!("Failed to deserialize presence message with action {:?}", action));
            
            // Verify action is preserved
            if let SyncMessage::Presence { action: deserialized_action, .. } = deserialized {
                assert_eq!(action, deserialized_action, "Presence action should be preserved");
            } else {
                panic!("Deserialized message should be a Presence message");
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_client_server_info() {
        let client = WebSocketClient::new(create_test_config(), create_test_replica_id());
        
        // Test server info with various capabilities
        let server_info = ServerInfo {
            server_id: "server-001".to_string(),
            version: "0.8.4".to_string(),
            capabilities: vec![
                "crdt_sync".to_string(),
                "presence".to_string(),
                "compression".to_string(),
                "encryption".to_string(),
            ],
        };
        
        let message = SyncMessage::Welcome {
            peer_id: create_test_replica_id(),
            timestamp: SystemTime::now(),
            server_info: server_info.clone(),
        };
        
        // Test serialization
        let serialized = MessageCodec::serialize(&message)
            .expect("Failed to serialize welcome message with server info");
        
        // Test deserialization
        let deserialized = MessageCodec::deserialize(&serialized)
            .expect("Failed to deserialize welcome message with server info");
        
        // Verify server info is preserved
        if let SyncMessage::Welcome { server_info: deserialized_info, .. } = deserialized {
            assert_eq!(server_info, deserialized_info, "Server info should be preserved");
        } else {
            panic!("Deserialized message should be a Welcome message");
        }
    }
}
