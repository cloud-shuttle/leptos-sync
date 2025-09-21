# Test Coverage Improvement - Critical Priority

## Overview
Achieve comprehensive test coverage across all implemented functionality and establish reliable testing infrastructure.

## Current State Analysis

### Existing Test Count
- **Found**: ~150 actual unit tests (not 736 as claimed in README)
- **Coverage**: Primarily basic CRDT operations and storage layer
- **Missing**: Network synchronization, error recovery, browser integration
- **Quality**: Good test quality where they exist, but major gaps

### Test Distribution
```
✅ Well Tested (>80% coverage):
- LwwRegister, LwwMap, GCounter basic operations
- Memory storage CRUD operations  
- Basic serialization/deserialization
- CRDT builder pattern

⚠️ Partially Tested (30-80% coverage):
- Advanced CRDTs (RGA, LSEQ, Tree, Graph) - basic tests only
- Collection management - happy path only
- Memory pool - configuration only

❌ No Tests (<30% coverage):
- WebSocket transport (only placeholder functions)
- IndexedDB storage (falls back to localStorage)
- Sync engine (background processes, conflict resolution)
- Multi-replica synchronization scenarios
- Error recovery and reliability features
- Security and authentication
- WASM browser integration
```

## Testing Strategy

### 1. Unit Test Coverage (Weeks 1-2)

#### Priority 1: Core Functionality
**Files**: Extend existing test files (keep < 300 lines each)

**Transport Layer Tests**
```rust
// leptos-sync-core/src/transport/websocket_tests.rs (< 250 lines)
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_websocket_connection_lifecycle() {
        let mut transport = WebSocketTransport::new(test_config());
        
        // Test connection
        assert!(!transport.is_connected());
        transport.connect().await.unwrap();
        assert!(transport.is_connected());
        
        // Test disconnection
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_message_send_receive() {
        let mut transport = setup_test_transport().await;
        let test_message = create_test_delta_message();
        
        transport.send(test_message.clone()).await.unwrap();
        let received = transport.receive().await.unwrap();
        
        assert_eq!(received, test_message);
    }

    #[tokio::test] 
    async fn test_connection_recovery() {
        let mut transport = setup_test_transport().await;
        
        // Simulate network failure
        transport.simulate_network_failure().await;
        assert!(!transport.is_connected());
        
        // Test automatic reconnection
        sleep(Duration::from_millis(100)).await;
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_message_queuing_during_disconnect() {
        let mut transport = setup_test_transport().await;
        transport.disconnect().await.unwrap();
        
        // Send messages while disconnected
        let messages = vec![
            create_test_delta_message(),
            create_test_heartbeat_message(),
        ];
        
        for msg in &messages {
            transport.send(msg.clone()).await.unwrap(); // Should queue
        }
        
        // Reconnect and verify messages are sent
        transport.connect().await.unwrap();
        
        // Verify queued messages were sent
        let server_received = get_server_received_messages().await;
        assert_eq!(server_received, messages);
    }
}
```

**Storage Layer Tests**
```rust
// leptos-sync-core/src/storage/indexeddb_tests.rs (< 300 lines)
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_indexeddb_crud_operations() {
        let storage = IndexedDbStorage::new("test_db").await.unwrap();
        
        // Test set/get
        let key = "test_key";
        let value = b"test_value";
        storage.set(key, value).await.unwrap();
        
        let retrieved = storage.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));
        
        // Test delete
        storage.delete(key).await.unwrap();
        let after_delete = storage.get(key).await.unwrap();
        assert_eq!(after_delete, None);
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_batch_operations() {
        let storage = IndexedDbStorage::new("test_batch_db").await.unwrap();
        
        let operations = vec![
            ("key1", b"value1".to_vec()),
            ("key2", b"value2".to_vec()),
            ("key3", b"value3".to_vec()),
        ];
        
        // Batch set
        storage.batch_set(operations.clone()).await.unwrap();
        
        // Verify all values
        for (key, expected_value) in operations {
            let value = storage.get(&key).await.unwrap().unwrap();
            assert_eq!(value, expected_value);
        }
    }

    #[wasm_bindgen_test]
    async fn test_storage_quota_exceeded() {
        let storage = IndexedDbStorage::new("quota_test_db").await.unwrap();
        
        // Try to store large amount of data
        let large_value = vec![0u8; 50 * 1024 * 1024]; // 50MB
        let result = storage.set("large_key", &large_value).await;
        
        match result {
            Err(StorageError::QuotaExceeded) => {
                // Verify cleanup was attempted
                assert!(storage.get_available_space().await.unwrap() > 0);
            },
            _ => panic!("Expected QuotaExceeded error"),
        }
    }
}
```

### 2. Integration Tests (Weeks 2-3)

#### Multi-Replica Synchronization Tests
```rust
// tests/integration/multi_replica_sync.rs (< 300 lines)
use leptos_sync_core::*;
use tokio::time::timeout;

#[tokio::test]
async fn test_two_replica_lww_sync() {
    let (replica_a, replica_b) = setup_two_replicas().await;
    
    // Make changes on replica A
    let collection_a = replica_a.collection::<String>("test").await;
    collection_a.set("key1", "value_from_a".to_string()).await.unwrap();
    
    // Make conflicting changes on replica B  
    let collection_b = replica_b.collection::<String>("test").await;
    collection_b.set("key1", "value_from_b".to_string()).await.unwrap();
    
    // Connect replicas and wait for sync
    connect_replicas(&replica_a, &replica_b).await;
    wait_for_sync(&[&replica_a, &replica_b], Duration::from_secs(5)).await;
    
    // Verify eventual consistency (LWW should win)
    let value_a = collection_a.get("key1").await.unwrap();
    let value_b = collection_b.get("key1").await.unwrap();
    assert_eq!(value_a, value_b);
    
    // Verify the winner is the later timestamp
    assert!(value_a.is_some());
}

#[tokio::test]
async fn test_three_replica_counter_sync() {
    let replicas = setup_n_replicas(3).await;
    
    // Each replica increments counter
    for (i, replica) in replicas.iter().enumerate() {
        let counter = replica.collection::<GCounter>("counter").await;
        for _ in 0..=i {
            counter.increment().await.unwrap();
        }
    }
    
    // Connect all replicas in mesh topology
    connect_all_replicas(&replicas).await;
    wait_for_sync(&replicas, Duration::from_secs(10)).await;
    
    // Verify all replicas have same total count
    let expected_total = 0 + 1 + 2; // 3 total increments
    for replica in &replicas {
        let counter = replica.collection::<GCounter>("counter").await;
        assert_eq!(counter.value().await.unwrap(), expected_total);
    }
}

#[tokio::test]
async fn test_offline_online_synchronization() {
    let (replica_a, replica_b) = setup_two_replicas().await;
    connect_replicas(&replica_a, &replica_b).await;
    
    // Make initial sync
    let collection_a = replica_a.collection::<String>("test").await;
    collection_a.set("initial", "value".to_string()).await.unwrap();
    wait_for_sync(&[&replica_a, &replica_b], Duration::from_secs(2)).await;
    
    // Disconnect replica B
    disconnect_replica(&replica_b).await;
    
    // Make changes while offline
    collection_a.set("offline_change", "a_value".to_string()).await.unwrap();
    
    let collection_b = replica_b.collection::<String>("test").await;
    collection_b.set("offline_change", "b_value".to_string()).await.unwrap();
    
    // Reconnect and verify sync
    reconnect_replica(&replica_b).await;
    wait_for_sync(&[&replica_a, &replica_b], Duration::from_secs(5)).await;
    
    // Verify consistency restored
    let value_a = collection_a.get("offline_change").await.unwrap();
    let value_b = collection_b.get("offline_change").await.unwrap();
    assert_eq!(value_a, value_b);
}
```

### 3. Property-Based Tests (Week 3)

#### CRDT Invariant Testing
```rust
// tests/property/crdt_properties.rs (< 250 lines)
use proptest::prelude::*;
use leptos_sync_core::crdt::*;

// Test CRDT commutativity: merge(a, b) == merge(b, a)
proptest! {
    #[test]
    fn test_lww_register_commutativity(
        value_a in any::<String>(),
        value_b in any::<String>(),
        timestamp_offset in 0u64..1000,
    ) {
        let replica_a = ReplicaId::new();
        let replica_b = ReplicaId::new();
        let base_time = SystemTime::now();
        
        let mut register_a = LwwRegister::new(value_a.clone(), base_time, replica_a);
        let mut register_b = LwwRegister::new(value_b.clone(), 
            base_time + Duration::from_millis(timestamp_offset), replica_b);
        
        // Test commutativity
        let mut result1 = register_a.clone();
        result1.merge(&register_b);
        
        let mut result2 = register_b.clone(); 
        result2.merge(&register_a);
        
        assert_eq!(result1.value(), result2.value());
    }

    #[test]
    fn test_g_counter_associativity(
        increments_a in prop::collection::vec(1u64..10, 1..10),
        increments_b in prop::collection::vec(1u64..10, 1..10),
        increments_c in prop::collection::vec(1u64..10, 1..10),
    ) {
        let mut counter_a = GCounter::new();
        let mut counter_b = GCounter::new(); 
        let mut counter_c = GCounter::new();
        
        // Apply increments
        for inc in increments_a { counter_a.increment_by(inc).unwrap(); }
        for inc in increments_b { counter_b.increment_by(inc).unwrap(); }
        for inc in increments_c { counter_c.increment_by(inc).unwrap(); }
        
        // Test associativity: (a ⊔ b) ⊔ c == a ⊔ (b ⊔ c)
        let mut result1 = counter_a.clone();
        result1.merge(&counter_b);
        result1.merge(&counter_c);
        
        let mut temp = counter_b.clone();
        temp.merge(&counter_c);
        let mut result2 = counter_a.clone();
        result2.merge(&temp);
        
        assert_eq!(result1.value(), result2.value());
    }
}
```

### 4. Browser/WASM Tests (Week 4)

#### WebAssembly Integration Tests
```rust
// tests/browser/wasm_integration.rs (< 200 lines)
use wasm_bindgen_test::*;
use leptos_sync_core::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_indexeddb_persistence_across_page_reload() {
    let collection_id = "persistence_test";
    
    // Create collection and add data
    let client = LeptosSyncClient::new().await.unwrap();
    let collection = client.collection::<String>(collection_id).await;
    collection.set("persistent_key", "test_value".to_string()).await.unwrap();
    
    // Simulate page reload by creating new client instance
    drop(client);
    let new_client = LeptosSyncClient::new().await.unwrap();
    let new_collection = new_client.collection::<String>(collection_id).await;
    
    // Verify data persists
    let value = new_collection.get("persistent_key").await.unwrap();
    assert_eq!(value, Some("test_value".to_string()));
}

#[wasm_bindgen_test]
async fn test_websocket_connection_in_browser() {
    let client = LeptosSyncClient::new().await.unwrap();
    
    // Connect to test WebSocket server
    client.connect("ws://localhost:3001/test").await.unwrap();
    
    // Verify connection is established
    assert!(client.is_connected().await);
    
    // Test sending message
    let collection = client.collection::<String>("browser_test").await;
    collection.set("browser_key", "browser_value".to_string()).await.unwrap();
    
    // Verify message was sent (check server state or wait for echo)
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Add assertion based on server response
}
```

### 5. Performance/Load Tests (Week 4)

#### Benchmark Suite
```rust
// benches/sync_performance.rs (< 250 lines)
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use leptos_sync_core::*;

fn bench_crdt_merge_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crdt_merge");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lww_map", size), size, |b, &size| {
            let mut map_a = create_lww_map_with_entries(size);
            let map_b = create_lww_map_with_entries(size);
            
            b.iter(|| {
                let mut result = map_a.clone();
                result.merge(black_box(&map_b));
                result
            });
        });
        
        group.bench_with_input(BenchmarkId::new("rga_list", size), size, |b, &size| {
            let mut list_a = create_rga_with_elements(size);
            let list_b = create_rga_with_elements(size / 2);
            
            b.iter(|| {
                let mut result = list_a.clone();
                result.merge(black_box(&list_b));
                result
            });
        });
    }
    
    group.finish();
}

fn bench_storage_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage");
    
    group.bench_function("memory_storage_set_get", |b| {
        let storage = MemoryStorage::new();
        b.iter(|| {
            let key = format!("key_{}", fastrand::u32(..));
            let value = vec![fastrand::u8(..), fastrand::u8(..)];
            
            storage.set(&key, &value).unwrap();
            let retrieved = storage.get(&key).unwrap();
            black_box(retrieved);
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_crdt_merge_operations, bench_storage_operations);
criterion_main!(benches);
```

## Testing Infrastructure

### CI/CD Pipeline Enhancement
```yaml
# .github/workflows/comprehensive-tests.yml
name: Comprehensive Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --lib
      - run: cargo test --workspace --doc

  integration-tests:
    runs-on: ubuntu-latest  
    services:
      websocket-server:
        image: node:18
        options: --health-cmd "curl -f http://localhost:3001/health" --health-interval 10s
    steps:
      - uses: actions/checkout@v4
      - name: Start WebSocket server
        run: |
          cd tests/test-server
          npm install
          npm start &
          sleep 5
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test integration --

  browser-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      - name: Run browser tests
        run: wasm-pack test --chrome --headless --firefox --headless

  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4  
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --test property -- --test-threads=1

  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --bench sync_performance

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out xml --output-dir coverage/
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          file: coverage/cobertura.xml
```

### Test Organization
```
tests/
├── unit/                    # Unit tests (< 300 lines each)
│   ├── crdt_tests.rs
│   ├── storage_tests.rs     
│   └── transport_tests.rs
├── integration/             # Integration tests (< 300 lines each)
│   ├── multi_replica_sync.rs
│   ├── offline_sync.rs
│   └── error_recovery.rs
├── property/               # Property-based tests (< 250 lines each)
│   ├── crdt_properties.rs
│   └── sync_properties.rs
├── browser/                # WASM browser tests (< 200 lines each)
│   ├── wasm_integration.rs
│   └── indexeddb_tests.rs
└── test-utils/            # Shared test utilities (< 200 lines each)
    ├── test_server.rs
    ├── mock_transport.rs
    └── test_data.rs
```

## Coverage Targets

### Minimum Acceptable Coverage
- **Unit Tests**: 80% line coverage for all implemented features
- **Integration Tests**: 100% of public API endpoints tested
- **Property Tests**: All CRDT invariants validated
- **Browser Tests**: Core functionality in WebAssembly environment

### Tracking and Reporting
- Automated coverage reports in CI/CD
- Coverage regression prevention (block PRs that decrease coverage)
- Per-module coverage tracking
- Public coverage badge in README

## Acceptance Criteria

### Quantitative Targets
- [ ] 80%+ line coverage across all implemented functionality
- [ ] 100+ integration test scenarios  
- [ ] 50+ property-based test cases
- [ ] 25+ browser/WASM test cases
- [ ] All tests complete in under 5 minutes in CI

### Qualitative Requirements
- [ ] All public APIs have corresponding tests
- [ ] Edge cases and error conditions covered
- [ ] Performance regressions detected by benchmarks
- [ ] Cross-browser compatibility verified
- [ ] All test files under 300 lines

### Infrastructure
- [ ] Reliable CI/CD pipeline with comprehensive test suite
- [ ] Coverage reporting integrated and visible
- [ ] Property-based testing for CRDT correctness
- [ ] Browser testing in actual WebAssembly environment

## Time Estimate: 4 weeks (can be parallelized with implementation)
## Dependencies: WebSocket transport (02), IndexedDB storage (03)
## Risk: Medium - requires significant test development effort
## Impact: Critical - enables confident deployment and maintenance
