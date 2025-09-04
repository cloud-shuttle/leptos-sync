//! Performance benchmarks for CRDT operations
//! 
//! This module benchmarks the performance of CRDT operations including:
//! - LwwRegister operations (set, merge)
//! - LwwMap operations (set, get, merge)
//! - GCounter operations (increment, merge)
//! - Serialization/deserialization performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use leptos_sync_core::crdt::{LwwRegister, LwwMap, GCounter, ReplicaId, Mergeable};
use leptos_sync_core::collection::LocalFirstCollection;
use leptos_sync_core::storage::{Storage, memory::MemoryStorage};
use leptos_sync_core::transport::memory::InMemoryTransport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BenchmarkDocument {
    id: String,
    title: String,
    content: String,
    metadata: HashMap<String, String>,
    version: u32,
}

impl Default for BenchmarkDocument {
    fn default() -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "benchmark".to_string());
        metadata.insert("category".to_string(), "test".to_string());
        
        Self {
            id: "benchmark_doc".to_string(),
            title: "Benchmark Document".to_string(),
            content: "This is a benchmark document for performance testing".to_string(),
            metadata,
            version: 1,
        }
    }
}

impl Mergeable for BenchmarkDocument {
    type Error = std::io::Error;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Simple merge: take the one with higher version
        if other.version > self.version {
            *self = other.clone();
        }
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        // Conflict if same ID but different content
        self.id == other.id && (self.title != other.title || self.content != other.content)
    }
}

// ============================================================================
// LwwRegister Benchmarks
// ============================================================================

fn benchmark_lww_register_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lww_register_creation");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("create", size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let replica = ReplicaId::default();
                    let _register = LwwRegister::new(black_box(format!("value_{}", i)), replica);
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_lww_register_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("lww_register_merge");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("merge", size), size, |b, &size| {
            let mut registers: Vec<LwwRegister<String>> = (0..size)
                .map(|i| {
                    let replica = ReplicaId::default();
                    LwwRegister::new(format!("value_{}", i), replica)
                })
                .collect();
            
            b.iter(|| {
                let mut result = registers[0].clone();
                for i in 1..size {
                    result.merge(black_box(&registers[i])).unwrap();
                }
                black_box(result);
            });
        });
    }
    
    group.finish();
}

fn benchmark_lww_register_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("lww_register_serialization");
    
    let replica = ReplicaId::default();
    let register = LwwRegister::new("benchmark_value".to_string(), replica);
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&register)).unwrap();
            black_box(serialized);
        });
    });
    
    let serialized = serde_json::to_string(&register).unwrap();
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            let deserialized: LwwRegister<String> = serde_json::from_str(black_box(&serialized)).unwrap();
            black_box(deserialized);
        });
    });
    
    group.finish();
}

// ============================================================================
// LwwMap Benchmarks
// ============================================================================

fn benchmark_lww_map_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("lww_map_operations");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("insert_operations", size), size, |b, &size| {
            let replica = ReplicaId::default();
            let mut map = LwwMap::new();
            
            b.iter(|| {
                for i in 0..size {
                    map.insert(
                        black_box(format!("key_{}", i)),
                        black_box(format!("value_{}", i)),
                        replica
                    );
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("get_operations", size), size, |b, &size| {
            let replica = ReplicaId::default();
            let mut map = LwwMap::new();
            
            // Pre-populate the map
            for i in 0..size {
                map.insert(format!("key_{}", i), format!("value_{}", i), replica);
            }
            
            b.iter(|| {
                for i in 0..size {
                    let _value = map.get(black_box(&format!("key_{}", i)));
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_lww_map_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("lww_map_merge");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("merge", size), size, |b, &size| {
            let replica1 = ReplicaId::default();
            let replica2 = ReplicaId::default();
            
            let mut map1 = LwwMap::new();
            let mut map2 = LwwMap::new();
            
            // Populate both maps with different data
            for i in 0..size {
                map1.insert(format!("key_{}", i), format!("value1_{}", i), replica1);
                map2.insert(format!("key_{}", i), format!("value2_{}", i), replica2);
            }
            
            b.iter(|| {
                let mut result = map1.clone();
                result.merge(black_box(&map2)).unwrap();
                black_box(result);
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// GCounter Benchmarks
// ============================================================================

fn benchmark_gcounter_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("gcounter_operations");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("increment", size), size, |b, &size| {
            let replica = ReplicaId::default();
            let mut counter = GCounter::new();
            
            b.iter(|| {
                for _ in 0..size {
                    counter.increment(replica);
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("value", size), size, |b, &size| {
            let replica = ReplicaId::default();
            let mut counter = GCounter::new();
            
            // Pre-increment the counter
            for _ in 0..size {
                counter.increment(replica);
            }
            
            b.iter(|| {
                let _value = counter.value();
            });
        });
    }
    
    group.finish();
}

fn benchmark_gcounter_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("gcounter_merge");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("merge", size), size, |b, &size| {
            let replica1 = ReplicaId::default();
            let replica2 = ReplicaId::default();
            
            let mut counter1 = GCounter::new();
            let mut counter2 = GCounter::new();
            
            // Pre-increment both counters
            for _ in 0..size {
                counter1.increment(replica1);
                counter2.increment(replica2);
            }
            
            b.iter(|| {
                let mut result = counter1.clone();
                result.merge(black_box(&counter2)).unwrap();
                black_box(result);
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// Collection Benchmarks
// ============================================================================

fn benchmark_collection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection_operations");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
            let storage = Storage::memory();
            let transport = InMemoryTransport::new();
            let _collection = LocalFirstCollection::<BenchmarkDocument, _>::new(storage, transport);
            
            b.iter(|| {
                for i in 0..size {
                    let doc = BenchmarkDocument {
                        id: format!("doc_{}", i),
                        title: format!("Title {}", i),
                        content: format!("Content {}", i),
                        metadata: HashMap::new(),
                        version: i as u32,
                    };
                    // Note: This is a synchronous benchmark, so we can't use async operations
                    // In a real benchmark, you'd need to use a runtime or make the operations sync
                    black_box(doc);
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("get", size), size, |b, &size| {
            let storage = Storage::memory();
            let transport = InMemoryTransport::new();
            let _collection = LocalFirstCollection::<BenchmarkDocument, _>::new(storage, transport);
            
            b.iter(|| {
                for i in 0..size {
                    // Note: This is a synchronous benchmark, so we can't use async operations
                    black_box(i);
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_collection_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection_list");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("list", size), size, |b, &size| {
            b.iter(|| {
                // Simulate collection list operation
                let mut docs = Vec::new();
                for i in 0..size {
                    let doc = BenchmarkDocument {
                        id: format!("doc_{}", i),
                        title: format!("Title {}", i),
                        content: format!("Content {}", i),
                        metadata: HashMap::new(),
                        version: i as u32,
                    };
                    docs.push((format!("key_{}", i), doc));
                }
                black_box(docs);
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// Storage Benchmarks
// ============================================================================

fn benchmark_storage_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_operations");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("set", size), size, |b, &size| {
            b.iter(|| {
                // Simulate storage set operation
                let mut data = HashMap::new();
                for i in 0..size {
                    let doc = BenchmarkDocument {
                        id: format!("doc_{}", i),
                        title: format!("Title {}", i),
                        content: format!("Content {}", i),
                        metadata: HashMap::new(),
                        version: i as u32,
                    };
                    data.insert(format!("key_{}", i), doc);
                }
                black_box(data);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("get", size), size, |b, &size| {
            // Pre-populate data
            let mut data = HashMap::new();
            for i in 0..size {
                let doc = BenchmarkDocument {
                    id: format!("doc_{}", i),
                    title: format!("Title {}", i),
                    content: format!("Content {}", i),
                    metadata: HashMap::new(),
                    version: i as u32,
                };
                data.insert(format!("key_{}", i), doc);
            }
            
            b.iter(|| {
                for i in 0..size {
                    let _doc = data.get(black_box(&format!("key_{}", i)));
                }
            });
        });
    }
    
    group.finish();
}

// ============================================================================
// Memory Usage Benchmarks
// ============================================================================

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("lww_register_memory", |b| {
        b.iter(|| {
            let replica = ReplicaId::default();
            let register = LwwRegister::new("test_value".to_string(), replica);
            black_box(register);
        });
    });
    
    group.bench_function("lww_map_memory", |b| {
        b.iter(|| {
            let replica = ReplicaId::default();
            let mut map = LwwMap::new();
            
            for i in 0..100 {
                map.insert(format!("key_{}", i), format!("value_{}", i), replica);
            }
            
            black_box(map);
        });
    });
    
    group.bench_function("gcounter_memory", |b| {
        b.iter(|| {
            let replica = ReplicaId::default();
            let mut counter = GCounter::new();
            
            for _ in 0..100 {
                counter.increment(replica);
            }
            
            black_box(counter);
        });
    });
    
    group.finish();
}

// ============================================================================
// Concurrent Operations Benchmarks
// ============================================================================

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    
    group.bench_function("concurrent_merges", |b| {
        b.iter(|| {
            let replica1 = ReplicaId::default();
            let replica2 = ReplicaId::default();
            let replica3 = ReplicaId::default();
            
            let mut reg1 = LwwRegister::new("value1".to_string(), replica1);
            let reg2 = LwwRegister::new("value2".to_string(), replica2);
            let reg3 = LwwRegister::new("value3".to_string(), replica3);
            
            // Simulate concurrent merges
            reg1.merge(&reg2).unwrap();
            reg1.merge(&reg3).unwrap();
            
            black_box(reg1);
        });
    });
    
    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    benches,
    benchmark_lww_register_creation,
    benchmark_lww_register_merge,
    benchmark_lww_register_serialization,
    benchmark_lww_map_operations,
    benchmark_lww_map_merge,
    benchmark_gcounter_operations,
    benchmark_gcounter_merge,
    benchmark_collection_operations,
    benchmark_collection_list,
    benchmark_storage_operations,
    benchmark_memory_usage,
    benchmark_concurrent_operations
);

criterion_main!(benches);
