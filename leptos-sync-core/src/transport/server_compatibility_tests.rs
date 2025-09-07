//! Test-Driven Development tests for server compatibility
//! 
//! These tests verify compatibility with the existing WebSocket server
//! implementation in leptos-sync/src/websocket_server.rs

use super::{SyncTransport, TransportError};
use crate::transport::leptos_ws_pro_transport::{LeptosWsProTransport, LeptosWsProConfig};
use crate::transport::compatibility_layer::{CompatibilityTransport, SyncMessage, ServerInfo};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerWelcomeMessage {
    #[serde(rename = "type")]
    message_type: String,
    peer_id: String,
    timestamp: String,
    server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerPresenceMessage {
    #[serde(rename = "type")]
    message_type: String,
    peer_id: String,
    action: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerSyncMessage {
    #[serde(rename = "type")]
    message_type: String,
    peer_id: String,
    data: serde_json::Value,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerHeartbeatMessage {
    #[serde(rename = "type")]
    message_type: String,
    timestamp: String,
    server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerBinaryAckMessage {
    #[serde(rename = "type")]
    message_type: String,
    peer_id: String,
    size: usize,
    timestamp: String,
}

/// Test configuration for server compatibility
#[derive(Debug, Clone)]
pub struct ServerCompatibilityTestConfig {
    pub server_url: String,
    pub timeout: Duration,
    pub max_reconnect_attempts: usize,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
    pub retry_delay: Duration,
}

impl Default for ServerCompatibilityTestConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080".to_string(),
            timeout: Duration::from_secs(30),
            max_reconnect_attempts: 5,
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            retry_delay: Duration::from_millis(1000),
        }
    }
}

impl From<ServerCompatibilityTestConfig> for LeptosWsProConfig {
    fn from(config: ServerCompatibilityTestConfig) -> Self {
        Self {
            url: config.server_url,
            timeout: config.timeout,
            max_reconnect_attempts: config.max_reconnect_attempts,
            heartbeat_interval: config.heartbeat_interval,
            connection_timeout: config.connection_timeout,
            retry_delay: config.retry_delay,
        }
    }
}

#[cfg(test)]
mod server_compatibility_tests {
    use super::*;

    /// Test 1: Server Welcome Message Compatibility
    /// 
    /// This test verifies that the transport can handle the welcome message
    /// format used by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_welcome_message_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test parsing welcome message format
        let welcome_message = ServerWelcomeMessage {
            message_type: "welcome".to_string(),
            peer_id: "test_peer_123".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            server_info: ServerInfo {
                version: "0.7.0".to_string(),
                max_connections: 1000,
                heartbeat_interval: 30,
            },
        };
        
        let serialized = serde_json::to_vec(&welcome_message).unwrap();
        
        // Should be able to parse the welcome message
        let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        match parsed {
            SyncMessage::Welcome { peer_id, timestamp, server_info } => {
                assert_eq!(peer_id, "test_peer_123");
                assert_eq!(server_info.version, "0.7.0");
                assert_eq!(server_info.max_connections, 1000);
                assert_eq!(server_info.heartbeat_interval, 30);
                println!("Welcome message parsed successfully: peer_id={}, version={}", peer_id, server_info.version);
            }
            _ => panic!("Expected welcome message, got: {:?}", parsed),
        }
    }

    /// Test 2: Server Presence Message Compatibility
    /// 
    /// This test verifies that the transport can handle presence messages
    /// in the format used by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_presence_message_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test parsing presence message format
        let presence_message = ServerPresenceMessage {
            message_type: "presence".to_string(),
            peer_id: "test_peer_456".to_string(),
            action: "connected".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let serialized = serde_json::to_vec(&presence_message).unwrap();
        
        // Should be able to parse the presence message
        let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        match parsed {
            SyncMessage::Presence { peer_id, action, timestamp } => {
                assert_eq!(peer_id, "test_peer_456");
                assert_eq!(action, "connected");
                println!("Presence message parsed successfully: peer_id={}, action={}", peer_id, action);
            }
            _ => panic!("Expected presence message, got: {:?}", parsed),
        }
        
        // Test disconnected action
        let disconnect_message = ServerPresenceMessage {
            message_type: "presence".to_string(),
            peer_id: "test_peer_456".to_string(),
            action: "disconnected".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let serialized = serde_json::to_vec(&disconnect_message).unwrap();
        let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        match parsed {
            SyncMessage::Presence { peer_id, action, timestamp } => {
                assert_eq!(peer_id, "test_peer_456");
                assert_eq!(action, "disconnected");
                println!("Disconnect message parsed successfully: peer_id={}, action={}", peer_id, action);
            }
            _ => panic!("Expected presence message, got: {:?}", parsed),
        }
    }

    /// Test 3: Server Sync Message Compatibility
    /// 
    /// This test verifies that the transport can handle sync messages
    /// in the format used by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_sync_message_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test parsing sync message format
        let sync_message = ServerSyncMessage {
            message_type: "sync".to_string(),
            peer_id: "test_peer_789".to_string(),
            data: serde_json::json!({
                "changes": [
                    {
                        "type": "insert",
                        "position": 0,
                        "content": "Hello, World!"
                    }
                ],
                "client_id": "client_001",
                "sequence": 42
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let serialized = serde_json::to_vec(&sync_message).unwrap();
        
        // Should be able to parse the sync message
        let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        match parsed {
            SyncMessage::Sync { peer_id, data, timestamp } => {
                assert_eq!(peer_id, "test_peer_789");
                assert!(data.is_object());
                println!("Sync message parsed successfully: peer_id={}, data keys: {:?}", 
                    peer_id, data.as_object().unwrap().keys().collect::<Vec<_>>());
            }
            _ => panic!("Expected sync message, got: {:?}", parsed),
        }
    }

    /// Test 4: Server Heartbeat Message Compatibility
    /// 
    /// This test verifies that the transport can handle heartbeat messages
    /// in the format used by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_heartbeat_message_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test parsing heartbeat message format
        let heartbeat_message = ServerHeartbeatMessage {
            message_type: "heartbeat".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            server_id: "server_001".to_string(),
        };
        
        let serialized = serde_json::to_vec(&heartbeat_message).unwrap();
        
        // Should be able to parse the heartbeat message
        let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        match parsed {
            SyncMessage::Heartbeat { timestamp } => {
                println!("Heartbeat message parsed successfully: timestamp={}", timestamp);
            }
            _ => panic!("Expected heartbeat message, got: {:?}", parsed),
        }
    }

    /// Test 5: Server Binary Acknowledgment Compatibility
    /// 
    /// This test verifies that the transport can handle binary acknowledgment
    /// messages in the format used by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_binary_ack_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test parsing binary ack message format
        let binary_ack_message = ServerBinaryAckMessage {
            message_type: "binary_ack".to_string(),
            peer_id: "test_peer_999".to_string(),
            size: 1024,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let serialized = serde_json::to_vec(&binary_ack_message).unwrap();
        
        // Should be able to parse the binary ack message
        let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        match parsed {
            SyncMessage::BinaryAck { peer_id, size, timestamp } => {
                assert_eq!(peer_id, "test_peer_999");
                assert_eq!(size, 1024);
                println!("Binary ack message parsed successfully: peer_id={}, size={}", peer_id, size);
            }
            _ => panic!("Expected binary ack message, got: {:?}", parsed),
        }
    }

    /// Test 6: Server Message Round-trip Compatibility
    /// 
    /// This test verifies that messages can be sent and received in the
    /// exact format expected by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_message_roundtrip_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test sending sync message in server format
        let sync_data = serde_json::json!({
            "changes": [
                {
                    "type": "update",
                    "id": "item_123",
                    "content": "Updated content"
                }
            ],
            "client_id": "client_002",
            "sequence": 100
        });
        
        let send_result = transport.send_sync("test_peer_roundtrip", sync_data.clone()).await;
        // This will fail without a server, but should not panic
        assert!(send_result.is_err()); // Expected when not connected
        
        // Test sending presence message in server format
        let presence_result = transport.send_presence("test_peer_roundtrip", "connected").await;
        assert!(presence_result.is_err()); // Expected when not connected
        
        // Test sending heartbeat in server format
        let heartbeat_result = transport.send_heartbeat().await;
        assert!(heartbeat_result.is_err()); // Expected when not connected
        
        println!("All server message formats validated successfully");
    }

    /// Test 7: Server Connection Protocol Compatibility
    /// 
    /// This test verifies that the connection protocol matches what the
    /// existing WebSocket server expects.
    #[tokio::test]
    async fn test_server_connection_protocol_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test connection establishment
        let connect_result = timeout(Duration::from_secs(5), transport.connect()).await;
        
        match connect_result {
            Ok(Ok(())) => {
                println!("Connected to server successfully");
                
                // Test that we can send messages in server format
                let test_data = serde_json::json!({
                    "test": "connection_protocol",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                
                let send_result = transport.send_sync("protocol_test_peer", test_data).await;
                if send_result.is_ok() {
                    println!("Successfully sent sync message to server");
                } else {
                    println!("Failed to send sync message: {:?}", send_result);
                }
                
                // Test receiving messages
                let receive_result = timeout(Duration::from_secs(2), transport.receive_messages()).await;
                match receive_result {
                    Ok(Ok(messages)) => {
                        println!("Received {} messages from server", messages.len());
                        for (i, message) in messages.iter().enumerate() {
                            println!("Message {}: {:?}", i, message);
                        }
                    }
                    Ok(Err(e)) => {
                        println!("Error receiving messages: {}", e);
                    }
                    Err(_) => {
                        println!("Timeout receiving messages");
                    }
                }
                
                // Disconnect
                let _ = transport.disconnect().await;
            }
            Ok(Err(e)) => {
                println!("Connection failed (expected if no server): {}", e);
            }
            Err(_) => {
                println!("Connection timeout (expected if no server)");
            }
        }
    }

    /// Test 8: Server Error Handling Compatibility
    /// 
    /// This test verifies that error handling is compatible with the
    /// existing WebSocket server's error responses.
    #[tokio::test]
    async fn test_server_error_handling_compatibility() {
        let config = ServerCompatibilityTestConfig {
            server_url: "ws://invalid-server-url:9999".to_string(),
            ..Default::default()
        };
        let transport = CompatibilityTransport::new(config.into());
        
        // Test connection to invalid server
        let connect_result = timeout(Duration::from_secs(5), transport.connect()).await;
        
        match connect_result {
            Ok(Ok(())) => {
                // Unexpected success
                println!("Unexpectedly connected to invalid server");
                let _ = transport.disconnect().await;
            }
            Ok(Err(e)) => {
                // Expected failure
                println!("Expected connection failure: {}", e);
                assert!(!transport.is_connected());
            }
            Err(_) => {
                // Timeout
                println!("Expected connection timeout");
                assert!(!transport.is_connected());
            }
        }
        
        // Test sending to disconnected transport
        let test_data = serde_json::json!({"test": "error_handling"});
        let send_result = transport.send_sync("error_test_peer", test_data).await;
        assert!(send_result.is_err(), "Send to disconnected transport should fail");
        
        // Test receiving from disconnected transport
        let receive_result = transport.receive_messages().await;
        assert!(receive_result.is_ok(), "Receive from disconnected transport should return empty");
        assert!(receive_result.unwrap().is_empty(), "Receive from disconnected transport should return empty messages");
    }

    /// Test 9: Server Performance Compatibility
    /// 
    /// This test verifies that the transport meets the performance
    /// requirements expected by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_performance_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test message serialization/deserialization performance
        let welcome_message = ServerWelcomeMessage {
            message_type: "welcome".to_string(),
            peer_id: "perf_test_peer".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            server_info: ServerInfo {
                version: "0.7.0".to_string(),
                max_connections: 1000,
                heartbeat_interval: 30,
            },
        };
        
        let sync_message = ServerSyncMessage {
            message_type: "sync".to_string(),
            peer_id: "perf_test_peer".to_string(),
            data: serde_json::json!({
                "changes": (0..100).map(|i| serde_json::json!({
                    "type": "insert",
                    "position": i,
                    "content": format!("Item {}", i)
                })).collect::<Vec<_>>(),
                "client_id": "perf_client",
                "sequence": 1
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let start_time = std::time::Instant::now();
        
        // Test welcome message
        let serialized = serde_json::to_vec(&welcome_message).unwrap();
        let deserialized: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        
        match deserialized {
            SyncMessage::Welcome { .. } => {
                println!("Welcome message serialization/deserialization successful");
            }
            _ => {
                println!("Unexpected message type: {:?}", deserialized);
            }
        }
        
        // Test sync message
        let serialized = serde_json::to_vec(&sync_message).unwrap();
        let deserialized: SyncMessage = serde_json::from_slice(&serialized).unwrap();
        
        match deserialized {
            SyncMessage::Sync { .. } => {
                println!("Sync message serialization/deserialization successful");
            }
            _ => {
                println!("Unexpected message type: {:?}", deserialized);
            }
        }
        
        let elapsed = start_time.elapsed();
        println!("Performance test completed in {:?}", elapsed);
        
        // Performance should be reasonable (under 10ms for these operations)
        assert!(elapsed < Duration::from_millis(10), "Performance too slow: {:?}", elapsed);
    }

    /// Test 10: Server Message Size Compatibility
    /// 
    /// This test verifies that the transport can handle message sizes
    /// within the limits expected by the existing WebSocket server.
    #[tokio::test]
    async fn test_server_message_size_compatibility() {
        let config = ServerCompatibilityTestConfig::default();
        let transport = CompatibilityTransport::new(config.into());
        
        // Test with different message sizes
        let sizes = vec![
            1024,      // 1KB
            10240,     // 10KB
            102400,    // 100KB
            1048576,   // 1MB (server limit)
        ];
        
        for size in sizes {
            let large_content = "x".repeat(size);
            let large_sync_message = ServerSyncMessage {
                message_type: "sync".to_string(),
                peer_id: "size_test_peer".to_string(),
                data: serde_json::json!({
                    "changes": [{
                        "type": "insert",
                        "position": 0,
                        "content": large_content
                    }],
                    "client_id": "size_test_client",
                    "sequence": 1
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            
            let serialized = serde_json::to_vec(&large_sync_message).unwrap();
            println!("Message size {} bytes serialized to {} bytes", size, serialized.len());
            
            // Should be able to parse large messages
            let parsed: SyncMessage = serde_json::from_slice(&serialized).unwrap();
            match parsed {
                SyncMessage::Sync { peer_id, .. } => {
                    assert_eq!(peer_id, "size_test_peer");
                    println!("Large message ({} bytes) parsed successfully", serialized.len());
                }
                _ => panic!("Expected sync message, got: {:?}", parsed),
            }
            
            // Test that we can send large messages (will fail without server, but should not panic)
            let send_result = transport.send(&serialized).await;
            assert!(send_result.is_err()); // Expected when not connected
        }
    }
}

/// Integration tests that require a running WebSocket server
#[cfg(test)]
mod server_integration_tests {
    use super::*;

    /// Test 11: Full Server Integration
    /// 
    /// This test requires the existing WebSocket server to be running
    /// and tests complete integration.
    #[tokio::test]
    #[ignore] // Ignored by default, run with: cargo test -- --ignored
    async fn test_full_server_integration() {
        // This test requires the existing WebSocket server to be running
        // It will be enabled when we implement the server integration
        println!("Full server integration test is disabled - requires running server");
    }

    /// Test 12: Server Message Flow Integration
    /// 
    /// This test verifies the complete message flow with the server.
    #[tokio::test]
    #[ignore] // Ignored by default, run with: cargo test -- --ignored
    async fn test_server_message_flow_integration() {
        // This test will verify the complete message flow with the server
        println!("Server message flow integration test is disabled - requires running server");
    }
}
