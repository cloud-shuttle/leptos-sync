//! Performance benchmarks for synchronization operations
//! 
//! Benchmarks that measure the performance of CRDT operations,
//! storage operations, and synchronization scenarios.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use leptos_sync_core::*;
use leptos_sync_core::crdt::{LwwRegister, LwwMap, GCounter, ReplicaId};
use leptos_sync_core::storage::Storage;
use std::time::SystemTime;
use uuid::Uuid;

/// Create a LWW Map with the specified number of entries
fn create_lww_map_with_entries(size: usize) -> LwwMap<String> {
    let mut map = LwwMap::new();
    let replica_id = ReplicaId::from(Uuid::new_v4());
    let timestamp = SystemTime::now();
    
    for i in 0..size {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        map.set(&key, value, timestamp, replica_id);
    }
    
    map
}

/// Create a GCounter with the specified number of increments
fn create_gcounter_with_increments(size: usize) -> GCounter {
    let mut counter = GCounter::new();
    
    for _ in 0..size {
        counter.increment().unwrap();
    }
    
    counter
}

/// Create a LWW Register with the specified value
fn create_lww_register_with_value(value: String) -> LwwRegister<String> {
    let replica_id = ReplicaId::from(Uuid::new_v4());
    let timestamp = SystemTime::now();
    
    LwwRegister::new(value, timestamp, replica_id)
}

/// Benchmark CRDT merge operations
fn bench_crdt_merge_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crdt_merge");
    
    // Benchmark LWW Map merge operations
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lww_map", size), size, |b, &size| {
            let map_a = create_lww_map_with_entries(size);
            let map_b = create_lww_map_with_entries(size / 2);
            
            b.iter(|| {
                let mut result = map_a.clone();
                result.merge(black_box(&map_b));
                result
            });
        });
    }
    
    // Benchmark GCounter merge operations
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("g_counter", size), size, |b, &size| {
            let counter_a = create_gcounter_with_increments(size);
            let counter_b = create_gcounter_with_increments(size / 2);
            
            b.iter(|| {
                let mut result = counter_a.clone();
                result.merge(black_box(&counter_b));
                result
            });
        });
    }
    
    // Benchmark LWW Register merge operations
    group.bench_function("lww_register", |b| {
        let register_a = create_lww_register_with_value("value_a".to_string());
        let register_b = create_lww_register_with_value("value_b".to_string());
        
        b.iter(|| {
            let mut result = register_a.clone();
            result.merge(black_box(&register_b));
            result
        });
    });
    
    group.finish();
}

/// Benchmark storage operations
fn bench_storage_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage");
    
    // Benchmark memory storage operations
    group.bench_function("memory_storage_set_get", |b| {
        let storage = Storage::memory();
        
        b.iter(|| {
            let key = format!("key_{}", fastrand::u32(..));
            let value = format!("value_{}", fastrand::u32(..));
            
            // Note: In a real benchmark, we would use async runtime
            // For now, we'll simulate the operation
            black_box((key, value));
        });
    });
    
    // Benchmark IndexedDB storage operations (simulated)
    group.bench_function("indexeddb_storage_set_get", |b| {
        b.iter(|| {
            let key = format!("key_{}", fastrand::u32(..));
            let value = format!("value_{}", fastrand::u32(..));
            
            // Simulate IndexedDB operations
            black_box((key, value));
        });
    });
    
    group.finish();
}

/// Benchmark serialization operations
fn bench_serialization_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    // Benchmark LWW Map serialization
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lww_map_serialize", size), size, |b, &size| {
            let map = create_lww_map_with_entries(size);
            
            b.iter(|| {
                let serialized = serde_json::to_string(black_box(&map)).unwrap();
                black_box(serialized);
            });
        });
    }
    
    // Benchmark LWW Map deserialization
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lww_map_deserialize", size), size, |b, &size| {
            let map = create_lww_map_with_entries(size);
            let serialized = serde_json::to_string(&map).unwrap();
            
            b.iter(|| {
                let deserialized: LwwMap<String> = serde_json::from_str(black_box(&serialized)).unwrap();
                black_box(deserialized);
            });
        });
    }
    
    // Benchmark GCounter serialization
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("g_counter_serialize", size), size, |b, &size| {
            let counter = create_gcounter_with_increments(size);
            
            b.iter(|| {
                let serialized = serde_json::to_string(black_box(&counter)).unwrap();
                black_box(serialized);
            });
        });
    }
    
    group.finish();
}

/// Benchmark CRDT creation operations
fn bench_crdt_creation_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crdt_creation");
    
    // Benchmark LWW Map creation
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lww_map_create", size), size, |b, &size| {
            b.iter(|| {
                let map = create_lww_map_with_entries(black_box(size));
                black_box(map);
            });
        });
    }
    
    // Benchmark GCounter creation
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("g_counter_create", size), size, |b, &size| {
            b.iter(|| {
                let counter = create_gcounter_with_increments(black_box(size));
                black_box(counter);
            });
        });
    }
    
    // Benchmark LWW Register creation
    group.bench_function("lww_register_create", |b| {
        b.iter(|| {
            let register = create_lww_register_with_value(black_box("test_value".to_string()));
            black_box(register);
        });
    });
    
    group.finish();
}

/// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark memory usage for different CRDT sizes
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lww_map_memory", size), size, |b, &size| {
            b.iter(|| {
                let map = create_lww_map_with_entries(black_box(size));
                let serialized = serde_json::to_string(&map).unwrap();
                black_box(serialized.len());
            });
        });
    }
    
    // Benchmark memory usage for GCounter
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("g_counter_memory", size), size, |b, &size| {
            b.iter(|| {
                let counter = create_gcounter_with_increments(black_box(size));
                let serialized = serde_json::to_string(&counter).unwrap();
                black_box(serialized.len());
            });
        });
    }
    
    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");
    
    // Benchmark concurrent LWW Map operations
    group.bench_function("lww_map_concurrent_merge", |b| {
        let maps = (0..10).map(|_| create_lww_map_with_entries(100)).collect::<Vec<_>>();
        
        b.iter(|| {
            let mut result = maps[0].clone();
            for map in &maps[1..] {
                result.merge(black_box(map));
            }
            black_box(result);
        });
    });
    
    // Benchmark concurrent GCounter operations
    group.bench_function("g_counter_concurrent_merge", |b| {
        let counters = (0..10).map(|_| create_gcounter_with_increments(100)).collect::<Vec<_>>();
        
        b.iter(|| {
            let mut result = counters[0].clone();
            for counter in &counters[1..] {
                result.merge(black_box(counter));
            }
            black_box(result);
        });
    });
    
    group.finish();
}

/// Benchmark error handling
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    // Benchmark error handling for invalid operations
    group.bench_function("g_counter_overflow", |b| {
        let mut counter = GCounter::new();
        
        b.iter(|| {
            // Try to increment beyond reasonable limits
            let result = counter.increment_by(black_box(u64::MAX));
            black_box(result);
        });
    });
    
    // Benchmark error handling for serialization
    group.bench_function("serialization_error", |b| {
        let map = create_lww_map_with_entries(1000);
        
        b.iter(|| {
            // Simulate serialization error handling
            let result = serde_json::to_string(black_box(&map));
            black_box(result);
        });
    });
    
    group.finish();
}

/// Benchmark real-world scenarios
fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");
    
    // Benchmark document editing scenario
    group.bench_function("document_editing", |b| {
        let mut map = LwwMap::new();
        let replica_id = ReplicaId::from(Uuid::new_v4());
        let timestamp = SystemTime::now();
        
        b.iter(|| {
            // Simulate document editing operations
            for i in 0..100 {
                let key = format!("char_{}", i);
                let value = format!("char_{}", fastrand::u8(..));
                map.set(&key, value, timestamp, replica_id);
            }
            black_box(&map);
        });
    });
    
    // Benchmark counter scenario
    group.bench_function("vote_counting", |b| {
        let mut counter = GCounter::new();
        
        b.iter(|| {
            // Simulate vote counting operations
            for _ in 0..1000 {
                counter.increment().unwrap();
            }
            black_box(&counter);
        });
    });
    
    // Benchmark configuration scenario
    group.bench_function("config_management", |b| {
        let mut map = LwwMap::new();
        let replica_id = ReplicaId::from(Uuid::new_v4());
        let timestamp = SystemTime::now();
        
        b.iter(|| {
            // Simulate configuration management
            let configs = vec![
                ("theme", "dark"),
                ("language", "en"),
                ("notifications", "enabled"),
                ("auto_save", "true"),
            ];
            
            for (key, value) in configs {
                map.set(key, value.to_string(), timestamp, replica_id);
            }
            black_box(&map);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_crdt_merge_operations,
    bench_storage_operations,
    bench_serialization_operations,
    bench_crdt_creation_operations,
    bench_memory_usage,
    bench_concurrent_operations,
    bench_error_handling,
    bench_real_world_scenarios
);

criterion_main!(benches);
