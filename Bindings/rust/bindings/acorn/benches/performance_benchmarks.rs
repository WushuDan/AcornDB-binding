use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use acorn::{AcornTree, AcornBatch, AcornQuery, AcornSubscription, Error};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct BenchmarkData {
    id: String,
    name: String,
    value: i32,
    timestamp: SystemTime,
    metadata: Vec<String>,
}

impl BenchmarkData {
    fn new(id: String) -> Self {
        Self {
            id,
            name: format!("Item {}", id),
            value: id.parse::<i32>().unwrap_or(0),
            timestamp: SystemTime::now(),
            metadata: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        }
    }
}

// Basic Operations Benchmarks
fn bench_stash_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("stash_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("single_stash", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            let data = BenchmarkData::new("test-item".to_string());
            let json = serde_json::to_string(&data).unwrap();
            
            b.iter(|| {
                black_box(tree.stash("test-key", &json).unwrap());
            });
        });
        
        group.bench_with_input(BenchmarkId::new("sequential_stash", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            b.iter(|| {
                for i in 0..*size {
                    let data = BenchmarkData::new(i.to_string());
                    let json = serde_json::to_string(&data).unwrap();
                    black_box(tree.stash(&format!("key-{}", i), &json).unwrap());
                }
            });
        });
    }
    
    group.finish();
}

fn bench_crack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crack_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("sequential_crack", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                for i in 0..*size {
                    black_box(tree.crack(&format!("key-{}", i)).unwrap());
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("random_crack", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                for i in 0..*size {
                    let key = format!("key-{}", i % *size);
                    black_box(tree.crack(&key).unwrap());
                }
            });
        });
    }
    
    group.finish();
}

fn bench_toss_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("toss_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("sequential_toss", size), size, |b, &size| {
            b.iter(|| {
                let tree = AcornTree::open_memory().unwrap();
                
                // Pre-populate tree
                for i in 0..*size {
                    let data = BenchmarkData::new(i.to_string());
                    let json = serde_json::to_string(&data).unwrap();
                    tree.stash(&format!("key-{}", i), &json).unwrap();
                }
                
                // Toss operations
                for i in 0..*size {
                    black_box(tree.toss(&format!("key-{}", i)).unwrap());
                }
            });
        });
    }
    
    group.finish();
}

// Batch Operations Benchmarks
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("batch_stash", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            b.iter(|| {
                let mut batch = AcornBatch::new(tree.clone()).unwrap();
                
                for i in 0..*size {
                    let data = BenchmarkData::new(i.to_string());
                    let json = serde_json::to_string(&data).unwrap();
                    batch.stash(&format!("key-{}", i), &json).unwrap();
                }
                
                black_box(batch.commit().unwrap());
            });
        });
        
        group.bench_with_input(BenchmarkId::new("batch_toss", size), size, |b, &size| {
            b.iter(|| {
                let tree = AcornTree::open_memory().unwrap();
                
                // Pre-populate tree
                for i in 0..*size {
                    let data = BenchmarkData::new(i.to_string());
                    let json = serde_json::to_string(&data).unwrap();
                    tree.stash(&format!("key-{}", i), &json).unwrap();
                }
                
                let mut batch = AcornBatch::new(tree).unwrap();
                
                for i in 0..*size {
                    batch.toss(&format!("key-{}", i)).unwrap();
                }
                
                black_box(batch.commit().unwrap());
            });
        });
    }
    
    group.finish();
}

// Query Operations Benchmarks
fn bench_query_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_operations");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::new("query_all", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                let query = AcornQuery::new(tree.clone()).unwrap();
                let results: Vec<String> = query.collect().unwrap();
                black_box(results);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("query_filtered", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                let query = AcornQuery::new(tree.clone()).unwrap();
                let results: Vec<String> = query
                    .where_condition(|json| {
                        let data: BenchmarkData = serde_json::from_str(json).unwrap();
                        data.value % 2 == 0
                    })
                    .collect()
                    .unwrap();
                black_box(results);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("query_ordered", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                let query = AcornQuery::new(tree.clone()).unwrap();
                let results: Vec<String> = query
                    .order_by(|json| {
                        let data: BenchmarkData = serde_json::from_str(json).unwrap();
                        data.value
                    })
                    .collect()
                    .unwrap();
                black_box(results);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("query_paginated", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                let query = AcornQuery::new(tree.clone()).unwrap();
                let results: Vec<String> = query
                    .skip(100)
                    .take(50)
                    .collect()
                    .unwrap();
                black_box(results);
            });
        });
    }
    
    group.finish();
}

// Iterator Operations Benchmarks
fn bench_iterator_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterator_operations");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::new("iterator_all", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                tree.stash(&format!("key-{}", i), &json).unwrap();
            }
            
            b.iter(|| {
                let iter = tree.iter().unwrap();
                let mut count = 0;
                for _ in iter {
                    count += 1;
                }
                black_box(count);
            });
        });
        
        group.bench_with_input(BenchmarkId::new("iterator_prefix", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            // Pre-populate tree with different prefixes
            for i in 0..*size {
                let data = BenchmarkData::new(i.to_string());
                let json = serde_json::to_string(&data).unwrap();
                let prefix = if i % 10 == 0 { "prefix-a" } else { "prefix-b" };
                tree.stash(&format!("{}-key-{}", prefix, i), &json).unwrap();
            }
            
            b.iter(|| {
                let iter = tree.iter_from("prefix-a").unwrap();
                let mut count = 0;
                for _ in iter {
                    count += 1;
                }
                black_box(count);
            });
        });
    }
    
    group.finish();
}

// Memory Usage Benchmarks
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::new("memory_growth", size), size, |b, &size| {
            b.iter(|| {
                let tree = AcornTree::open_memory().unwrap();
                
                for i in 0..*size {
                    let data = BenchmarkData::new(i.to_string());
                    let json = serde_json::to_string(&data).unwrap();
                    tree.stash(&format!("key-{}", i), &json).unwrap();
                }
                
                // Force memory allocation
                let _ = tree.iter().unwrap().collect::<Vec<_>>();
                black_box(tree);
            });
        });
    }
    
    group.finish();
}

// Serialization Benchmarks
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let data = BenchmarkData::new("test-item".to_string());
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&data).unwrap());
        });
    });
    
    let json = serde_json::to_string(&data).unwrap();
    
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<BenchmarkData>(&json).unwrap());
        });
    });
    
    group.finish();
}

// FFI Overhead Benchmarks
fn bench_ffi_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_overhead");
    
    group.bench_function("tree_creation", |b| {
        b.iter(|| {
            black_box(AcornTree::open_memory().unwrap());
        });
    });
    
    group.bench_function("batch_creation", |b| {
        let tree = AcornTree::open_memory().unwrap();
        b.iter(|| {
            black_box(AcornBatch::new(tree.clone()).unwrap());
        });
    });
    
    group.bench_function("query_creation", |b| {
        let tree = AcornTree::open_memory().unwrap();
        b.iter(|| {
            black_box(AcornQuery::new(tree.clone()).unwrap());
        });
    });
    
    group.finish();
}

// Mixed Workload Benchmarks
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    
    for size in [1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("mixed_operations", size), size, |b, &size| {
            let tree = AcornTree::open_memory().unwrap();
            
            b.iter(|| {
                // Mixed operations: 70% reads, 20% writes, 10% deletes
                for i in 0..*size {
                    match i % 10 {
                        0..=6 => {
                            // Read operation
                            let key = format!("key-{}", i % (*size / 2));
                            let _ = tree.crack(&key);
                        },
                        7..=8 => {
                            // Write operation
                            let data = BenchmarkData::new(i.to_string());
                            let json = serde_json::to_string(&data).unwrap();
                            tree.stash(&format!("key-{}", i), &json).unwrap();
                        },
                        9 => {
                            // Delete operation
                            let key = format!("key-{}", i % (*size / 2));
                            let _ = tree.toss(&key);
                        },
                        _ => {}
                    }
                }
            });
        });
    }
    
    group.finish();
}

// Concurrent Operations Benchmarks
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    
    group.bench_function("concurrent_reads", |b| {
        let tree = AcornTree::open_memory().unwrap();
        
        // Pre-populate tree
        for i in 0..1000 {
            let data = BenchmarkData::new(i.to_string());
            let json = serde_json::to_string(&data).unwrap();
            tree.stash(&format!("key-{}", i), &json).unwrap();
        }
        
        b.iter(|| {
            let handles: Vec<_> = (0..10).map(|_| {
                let tree_clone = tree.clone();
                std::thread::spawn(move || {
                    for i in 0..100 {
                        let key = format!("key-{}", i % 1000);
                        let _ = tree_clone.crack(&key);
                    }
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_stash_operations,
    bench_crack_operations,
    bench_toss_operations,
    bench_batch_operations,
    bench_query_operations,
    bench_iterator_operations,
    bench_memory_usage,
    bench_serialization,
    bench_ffi_overhead,
    bench_mixed_workload,
    bench_concurrent_operations
);

criterion_main!(benches);
