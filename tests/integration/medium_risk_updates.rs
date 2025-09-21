//! Integration tests for medium-risk dependency updates
//!
//! Tests that validate functionality after updating medium-risk dependencies
//! including leptos-ws-pro, sqlx, and redis.

use leptos_sync_core::crdt::{CrdtType, ReplicaId};
use leptos_sync_core::transport::{
    message_protocol::{PresenceAction, ServerInfo, SyncMessage, UserInfo},
    SyncTransport, WebSocketClient, WebSocketClientConfig,
};
use leptos_sync_core::*;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;
use uuid::Uuid;

/// Test WebSocket functionality with leptos-ws-pro 0.11.0
mod websocket_tests {
    use super::*;

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

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(Uuid::new_v4())
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
    async fn test_websocket_connection_lifecycle_v0_11() {
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
    async fn test_websocket_message_handling_v0_11() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);

        // Test message serialization (this should work even without connection)
        let message = create_test_delta_message();

        // Test that we can create and serialize messages
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized: SyncMessage = serde_json::from_str(&serialized).unwrap();
        match (&message, &deserialized) {
            (
                SyncMessage::Delta {
                    collection_id: id1,
                    crdt_type: type1,
                    delta: delta1,
                    replica_id: rid1,
                    ..
                },
                SyncMessage::Delta {
                    collection_id: id2,
                    crdt_type: type2,
                    delta: delta2,
                    replica_id: rid2,
                    ..
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(type1, type2);
                assert_eq!(delta1, delta2);
                assert_eq!(rid1, rid2);
            }
            _ => panic!("Message types don't match"),
        }
    }

    #[tokio::test]
    async fn test_websocket_config_v0_11() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);

        // Test that configuration is properly stored
        assert_eq!(client.replica_id(), replica_id);

        // Test that we can create different configurations
        let config2 = WebSocketClientConfig {
            url: "ws://localhost:3002/test".to_string(),
            heartbeat_interval: Duration::from_secs(60),
            message_timeout: Duration::from_secs(10),
            reconnect_interval: Duration::from_secs(2),
            max_reconnect_attempts: 5,
            user_info: None,
        };

        let client2 = WebSocketClient::new(config2, replica_id);
        assert_eq!(client2.replica_id(), replica_id);
    }

    #[tokio::test]
    async fn test_websocket_error_handling_v0_11() {
        let config = WebSocketClientConfig {
            url: "ws://invalid-url".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 3,
            user_info: None,
        };

        let replica_id = create_test_replica_id();
        let mut client = WebSocketClient::new(config, replica_id);

        // Test connection failure
        let result = client.connect().await;
        assert!(result.is_err());

        // Test message sending on disconnected client
        let message = create_test_heartbeat_message();
        let result = client.send_message(message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_websocket_performance_v0_11() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);

        let start = std::time::Instant::now();

        // Test message creation performance
        let mut messages = Vec::new();
        for i in 0..1000 {
            let message = SyncMessage::Heartbeat {
                replica_id: create_test_replica_id(),
                timestamp: SystemTime::now(),
            };
            messages.push(message);
        }

        let duration = start.elapsed();
        assert!(duration < Duration::from_secs(1)); // Should complete in under 1 second

        // Test serialization performance
        let start = std::time::Instant::now();
        for message in &messages {
            let _ = serde_json::to_string(message).unwrap();
        }

        let duration = start.elapsed();
        assert!(duration < Duration::from_secs(1)); // Should complete in under 1 second
    }
}

/// Test database functionality with sqlx 0.8
#[cfg(feature = "sqlx")]
mod database_tests {
    use super::*;
    use sqlx::{Row, SqlitePool};

    async fn create_test_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_sqlx_query_macros_v0_8() {
        let pool = create_test_pool().await;

        // Test basic query macro
        let result = sqlx::query!("SELECT 1 as test")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.test, 1);
    }

    #[tokio::test]
    async fn test_sqlx_connection_pool_v0_8() {
        let pool = create_test_pool().await;

        // Test pool configuration
        assert!(pool.size() > 0);
        assert!(pool.idle_connections() >= 0);

        // Test multiple connections
        let conn1 = pool.acquire().await.unwrap();
        let conn2 = pool.acquire().await.unwrap();

        drop(conn1);
        drop(conn2);
    }

    #[tokio::test]
    async fn test_sqlx_error_handling_v0_8() {
        let pool = create_test_pool().await;

        // Test invalid query
        let result = sqlx::query!("SELECT * FROM non_existent_table")
            .fetch_one(&pool)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sqlx_performance_v0_8() {
        let pool = create_test_pool().await;

        let start = std::time::Instant::now();

        // Execute 1000 queries
        for i in 0..1000 {
            let _: i32 = sqlx::query_scalar!("SELECT ?")
                .bind(i)
                .fetch_one(&pool)
                .await
                .unwrap();
        }

        let duration = start.elapsed();
        assert!(duration < Duration::from_secs(2)); // Should complete in under 2 seconds
    }
}

/// Test Redis functionality with redis 0.26
#[cfg(feature = "redis")]
mod redis_tests {
    use super::*;
    use redis::{Client, Commands};

    async fn create_test_client() -> Client {
        Client::open("redis://127.0.0.1/").unwrap()
    }

    #[tokio::test]
    async fn test_redis_client_connection_v0_26() {
        let client = create_test_client().await;
        let mut conn = client.get_async_connection().await.unwrap();

        // Test basic operations
        let _: () = conn.set("test_key", "test_value").await.unwrap();
        let value: String = conn.get("test_key").await.unwrap();
        assert_eq!(value, "test_value");

        // Clean up
        let _: () = conn.del("test_key").await.unwrap();
    }

    #[tokio::test]
    async fn test_redis_connection_pool_v0_26() {
        let client = create_test_client().await;
        let pool = redis::aio::ConnectionManager::new(client).await.unwrap();

        // Test pool operations
        let _: () = pool.set("pool_key", "pool_value").await.unwrap();
        let value: String = pool.get("pool_key").await.unwrap();
        assert_eq!(value, "pool_value");

        // Clean up
        let _: () = pool.del("pool_key").await.unwrap();
    }

    #[tokio::test]
    async fn test_redis_error_handling_v0_26() {
        let client = Client::open("redis://invalid-host/").unwrap();
        let result = client.get_async_connection().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_redis_performance_v0_26() {
        let client = create_test_client().await;
        let mut conn = client.get_async_connection().await.unwrap();

        let start = std::time::Instant::now();

        // Execute 1000 operations
        for i in 0..1000 {
            let _: () = conn
                .set(format!("key_{}", i), format!("value_{}", i))
                .await
                .unwrap();
        }

        let duration = start.elapsed();
        assert!(duration < Duration::from_secs(3)); // Should complete in under 3 seconds

        // Clean up
        for i in 0..1000 {
            let _: () = conn.del(format!("key_{}", i)).await.unwrap();
        }
    }
}

/// Test integration between updated dependencies
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_database_integration() {
        // Test that WebSocket and database functionality work together
        // This would involve setting up a WebSocket server that uses a database
        // For now, we'll test that both can be used in the same application

        let config = WebSocketClientConfig {
            url: "ws://localhost:3001/test".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 3,
            user_info: None,
        };

        let replica_id = ReplicaId::from(Uuid::new_v4());
        let client = WebSocketClient::new(config, replica_id);

        // Test that we can create both WebSocket client and database pool
        #[cfg(feature = "sqlx")]
        {
            let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

            // Both should work together
            assert_eq!(client.replica_id(), replica_id);
            assert!(pool.size() > 0);
        }
    }

    #[tokio::test]
    async fn test_websocket_redis_integration() {
        // Test that WebSocket and Redis functionality work together
        let config = WebSocketClientConfig {
            url: "ws://localhost:3001/test".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 3,
            user_info: None,
        };

        let replica_id = ReplicaId::from(Uuid::new_v4());
        let client = WebSocketClient::new(config, replica_id);

        // Test that we can create both WebSocket client and Redis client
        #[cfg(feature = "redis")]
        {
            let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();

            // Both should work together
            assert_eq!(client.replica_id(), replica_id);
            assert!(redis_client.get_connection().is_ok());
        }
    }

    #[tokio::test]
    async fn test_database_redis_integration() {
        // Test that database and Redis functionality work together
        #[cfg(all(feature = "sqlx", feature = "redis"))]
        {
            let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

            let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();

            // Both should work together
            assert!(pool.size() > 0);
            assert!(redis_client.get_connection().is_ok());
        }
    }
}

/// Test rollback scenarios
mod rollback_tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_rollback_compatibility() {
        // Test that WebSocket functionality is compatible with rollback
        // This would involve testing with previous versions
        // For now, we'll test that current functionality works

        let config = WebSocketClientConfig {
            url: "ws://localhost:3001/test".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 3,
            user_info: None,
        };

        let replica_id = ReplicaId::from(Uuid::new_v4());
        let client = WebSocketClient::new(config, replica_id);

        // Test basic functionality
        assert_eq!(client.replica_id(), replica_id);
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_database_rollback_compatibility() {
        // Test that database functionality is compatible with rollback
        #[cfg(feature = "sqlx")]
        {
            let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

            // Test basic functionality
            assert!(pool.size() > 0);

            let result = sqlx::query!("SELECT 1 as test")
                .fetch_one(&pool)
                .await
                .unwrap();

            assert_eq!(result.test, 1);
        }
    }

    #[tokio::test]
    async fn test_redis_rollback_compatibility() {
        // Test that Redis functionality is compatible with rollback
        #[cfg(feature = "redis")]
        {
            let client = redis::Client::open("redis://127.0.0.1/").unwrap();

            // Test basic functionality
            assert!(client.get_connection().is_ok());
        }
    }
}
