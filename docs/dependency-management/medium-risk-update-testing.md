# Medium-Risk Dependency Update Testing Plan

## Overview
This document outlines the testing strategy for medium-risk dependency updates in Phase 2 of the dependency modernization plan.

## Updated Dependencies

### 1. leptos-ws-pro: 0.10.0 → 0.11.0
**Risk Level**: Medium
**Impact**: WebSocket functionality, configuration changes
**Breaking Changes**: Minor - Configuration API changes

### 2. sqlx: 0.7 → 0.8
**Risk Level**: Medium  
**Impact**: Database operations, query macros
**Breaking Changes**: Query API changes, connection pool updates

### 3. redis: 0.23 → 0.26
**Risk Level**: Medium
**Impact**: Redis client operations, connection handling
**Breaking Changes**: Client API improvements, connection pool changes

## Testing Strategy

### Phase 1: Compilation Testing
```bash
# Test compilation with updated dependencies
cargo check --workspace
cargo build --workspace
```

### Phase 2: Unit Testing
```bash
# Run all unit tests
cargo test --workspace --lib
cargo test --workspace --bins
```

### Phase 3: Integration Testing
```bash
# Run integration tests
cargo test --test integration
cargo test --test contracts
```

### Phase 4: WebSocket Testing
```bash
# Test WebSocket functionality
cargo test --package leptos-sync-core --test websocket_integration_tests
```

### Phase 5: Database Testing
```bash
# Test database functionality (if applicable)
cargo test --package leptos-sync-core --test database_tests
```

### Phase 6: Redis Testing
```bash
# Test Redis functionality (if applicable)
cargo test --package leptos-sync-core --test redis_tests
```

### Phase 7: Browser/WASM Testing
```bash
# Test WASM compatibility
wasm-pack test --chrome --headless
wasm-pack test --firefox --headless
```

### Phase 8: Performance Testing
```bash
# Run performance benchmarks
cargo bench --bench sync_performance
```

## Specific Test Cases

### WebSocket Testing (leptos-ws-pro 0.11.0)

#### 1. Connection Lifecycle
```rust
#[tokio::test]
async fn test_websocket_connection_lifecycle_v0_11() {
    let config = WebSocketClientConfig::new("ws://localhost:3001/test");
    let mut client = WebSocketClient::new(config, ReplicaId::new());
    
    // Test connection
    assert!(!client.is_connected().await);
    client.connect().await.unwrap();
    assert!(client.is_connected().await);
    
    // Test disconnection
    client.disconnect().await.unwrap();
    assert!(!client.is_connected().await);
}
```

#### 2. Message Handling
```rust
#[tokio::test]
async fn test_websocket_message_handling_v0_11() {
    let config = WebSocketClientConfig::new("ws://localhost:3001/test");
    let mut client = WebSocketClient::new(config, ReplicaId::new());
    client.connect().await.unwrap();
    
    // Test message sending
    let message = SyncMessage::Heartbeat {
        replica_id: ReplicaId::new(),
        timestamp: SystemTime::now(),
    };
    
    client.send_message(message.clone()).await.unwrap();
    
    // Test message receiving
    let received = client.receive_message().await.unwrap();
    assert_eq!(received, message);
}
```

#### 3. Configuration Changes
```rust
#[tokio::test]
async fn test_websocket_config_v0_11() {
    let config = WebSocketClientConfig::new("ws://localhost:3001/test")
        .with_heartbeat_interval(Duration::from_secs(30))
        .with_reconnect_attempts(5)
        .with_message_timeout(Duration::from_secs(10));
    
    let client = WebSocketClient::new(config, ReplicaId::new());
    
    // Verify configuration is applied
    assert_eq!(client.config().heartbeat_interval, Duration::from_secs(30));
    assert_eq!(client.config().max_reconnect_attempts, 5);
    assert_eq!(client.config().message_timeout, Duration::from_secs(10));
}
```

### Database Testing (sqlx 0.8)

#### 1. Query Macros
```rust
#[tokio::test]
async fn test_sqlx_query_macros_v0_8() {
    // Test basic query macro
    let result = sqlx::query!("SELECT 1 as test")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(result.test, 1);
}
```

#### 2. Connection Pool
```rust
#[tokio::test]
async fn test_sqlx_connection_pool_v0_8() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap();
    
    // Test pool configuration
    assert!(pool.size() > 0);
    assert!(pool.idle_connections() >= 0);
}
```

#### 3. Migration Support
```rust
#[tokio::test]
async fn test_sqlx_migrations_v0_8() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap();
    
    // Test migration runner
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
}
```

### Redis Testing (redis 0.26)

#### 1. Client Connection
```rust
#[tokio::test]
async fn test_redis_client_connection_v0_26() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = client.get_async_connection().await.unwrap();
    
    // Test basic operations
    let _: () = conn.set("test_key", "test_value").await.unwrap();
    let value: String = conn.get("test_key").await.unwrap();
    assert_eq!(value, "test_value");
}
```

#### 2. Connection Pool
```rust
#[tokio::test]
async fn test_redis_connection_pool_v0_26() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = redis::aio::ConnectionManager::new(client).await.unwrap();
    
    // Test pool operations
    let _: () = pool.set("pool_key", "pool_value").await.unwrap();
    let value: String = pool.get("pool_key").await.unwrap();
    assert_eq!(value, "pool_value");
}
```

#### 3. Pub/Sub Operations
```rust
#[tokio::test]
async fn test_redis_pubsub_v0_26() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = client.get_async_connection().await.unwrap();
    
    // Test pub/sub
    let _: () = conn.publish("test_channel", "test_message").await.unwrap();
    
    let mut pubsub = conn.into_pubsub();
    pubsub.subscribe("test_channel").await.unwrap();
    
    let msg = pubsub.get_message().await.unwrap();
    assert_eq!(msg.get_payload::<String>().unwrap(), "test_message");
}
```

## Error Handling Testing

### 1. WebSocket Error Handling
```rust
#[tokio::test]
async fn test_websocket_error_handling_v0_11() {
    let config = WebSocketClientConfig::new("ws://invalid-url");
    let mut client = WebSocketClient::new(config, ReplicaId::new());
    
    // Test connection failure
    let result = client.connect().await;
    assert!(result.is_err());
    
    // Test message sending on disconnected client
    let message = SyncMessage::Heartbeat {
        replica_id: ReplicaId::new(),
        timestamp: SystemTime::now(),
    };
    
    let result = client.send_message(message).await;
    assert!(result.is_err());
}
```

### 2. Database Error Handling
```rust
#[tokio::test]
async fn test_sqlx_error_handling_v0_8() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap();
    
    // Test invalid query
    let result = sqlx::query!("SELECT * FROM non_existent_table")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_err());
}
```

### 3. Redis Error Handling
```rust
#[tokio::test]
async fn test_redis_error_handling_v0_26() {
    let client = redis::Client::open("redis://invalid-host/").unwrap();
    let result = client.get_async_connection().await;
    
    assert!(result.is_err());
}
```

## Performance Testing

### 1. WebSocket Performance
```rust
#[tokio::test]
async fn test_websocket_performance_v0_11() {
    let config = WebSocketClientConfig::new("ws://localhost:3001/test");
    let mut client = WebSocketClient::new(config, ReplicaId::new());
    client.connect().await.unwrap();
    
    let start = std::time::Instant::now();
    
    // Send 1000 messages
    for i in 0..1000 {
        let message = SyncMessage::Heartbeat {
            replica_id: ReplicaId::new(),
            timestamp: SystemTime::now(),
        };
        client.send_message(message).await.unwrap();
    }
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(5)); // Should complete in under 5 seconds
}
```

### 2. Database Performance
```rust
#[tokio::test]
async fn test_sqlx_performance_v0_8() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap();
    
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
```

### 3. Redis Performance
```rust
#[tokio::test]
async fn test_redis_performance_v0_26() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut conn = client.get_async_connection().await.unwrap();
    
    let start = std::time::Instant::now();
    
    // Execute 1000 operations
    for i in 0..1000 {
        let _: () = conn.set(format!("key_{}", i), format!("value_{}", i)).await.unwrap();
    }
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(3)); // Should complete in under 3 seconds
}
```

## Rollback Testing

### 1. WebSocket Rollback
```rust
#[tokio::test]
async fn test_websocket_rollback() {
    // Test rollback to previous version
    // This would involve reverting Cargo.toml and testing functionality
    // In practice, this would be done manually or with git operations
}
```

### 2. Database Rollback
```rust
#[tokio::test]
async fn test_sqlx_rollback() {
    // Test rollback to SQLX 0.7
    // Verify that existing code still works with previous version
}
```

### 3. Redis Rollback
```rust
#[tokio::test]
async fn test_redis_rollback() {
    // Test rollback to Redis 0.23
    // Verify that existing code still works with previous version
}
```

## Test Execution Plan

### 1. Pre-Update Testing
- Run full test suite with current versions
- Document current performance benchmarks
- Verify all functionality works as expected

### 2. Update Testing
- Update dependencies one at a time
- Run tests after each update
- Document any issues or failures

### 3. Post-Update Testing
- Run comprehensive test suite
- Compare performance benchmarks
- Verify all functionality works correctly

### 4. Integration Testing
- Test with real WebSocket servers
- Test with real database instances
- Test with real Redis instances

### 5. Performance Testing
- Run performance benchmarks
- Compare with previous versions
- Identify any performance regressions

## Success Criteria

### 1. Compilation Success
- All packages compile without errors
- No deprecation warnings
- All features work correctly

### 2. Test Success
- All unit tests pass
- All integration tests pass
- All browser/WASM tests pass

### 3. Performance Success
- No significant performance regression
- Memory usage remains stable
- Response times remain acceptable

### 4. Functionality Success
- WebSocket connections work correctly
- Database operations work correctly
- Redis operations work correctly
- All existing features work as expected

## Rollback Plan

### 1. Automatic Rollback
- If any test fails, automatically rollback to previous versions
- Restore Cargo.toml and Cargo.lock to previous state
- Verify rollback was successful

### 2. Manual Rollback
- If automatic rollback fails, manual rollback procedures
- Document the rollback process
- Verify all functionality after rollback

### 3. Emergency Rollback
- If critical issues are discovered, emergency rollback procedures
- Immediate rollback to known good state
- Investigation and resolution of issues

## Conclusion

This testing plan ensures that medium-risk dependency updates are thoroughly tested before deployment. The comprehensive testing strategy covers compilation, functionality, performance, and error handling, with clear success criteria and rollback procedures.
