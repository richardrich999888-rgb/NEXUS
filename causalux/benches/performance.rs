// CAUSALUX v2.0 - Performance Benchmarks
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use causalux_v2::{CausaluxRuntime, RuntimeConfig};

// ============================================================================
// DOCUMENT OPERATIONS BENCHMARKS
// ============================================================================

fn bench_document_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_insert");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = CausaluxRuntime::new(RuntimeConfig::default());
            runtime.create_document("doc1".to_string(), "Benchmark Doc".to_string()).unwrap();
            
            b.iter(|| {
                for i in 0..size {
                    runtime.insert_text("doc1", i, "A").unwrap();
                }
            });
        });
    }
    
    group.finish();
}

fn bench_document_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_delete");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let runtime = CausaluxRuntime::new(RuntimeConfig::default());
                    runtime.create_document("doc1".to_string(), "Benchmark Doc".to_string()).unwrap();
                    
                    // Insert text first
                    for i in 0..size {
                        runtime.insert_text("doc1", i, "A").unwrap();
                    }
                    runtime
                },
                |runtime| {
                    // Delete all text
                    for _ in 0..size {
                        runtime.delete_text("doc1", 0, 1).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput
            );
        });
    }
    
    group.finish();
}

// ============================================================================
// COUNTER BENCHMARKS
// ============================================================================

fn bench_counter_increment(c: &mut Criterion) {
    let mut group = c.benchmark_group("counter_increment");
    
    for ops in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(ops), ops, |b, &ops| {
            let runtime = CausaluxRuntime::new(RuntimeConfig::default());
            
            b.iter(|| {
                for _ in 0..ops {
                    runtime.increment_counter("counter1", black_box(1)).unwrap();
                }
            });
        });
    }
    
    group.finish();
}

fn bench_counter_concurrent(c: &mut Criterion) {
    c.bench_function("counter_concurrent_merge", |b| {
        b.iter_batched(
            || {
                let node1 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node1".to_string(),
                    ..Default::default()
                });
                let node2 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node2".to_string(),
                    ..Default::default()
                });
                
                // Both increment offline
                for _ in 0..1000 {
                    node1.increment_counter("counter", 1).unwrap();
                    node2.increment_counter("counter", 1).unwrap();
                }
                
                (node1, node2)
            },
            |(node1, node2)| {
                // Sync and merge
                node1.sync_with_node(&node2).unwrap();
            },
            criterion::BatchSize::SmallInput
        );
    });
}

// ============================================================================
// SET OPERATIONS BENCHMARKS
// ============================================================================

fn bench_set_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_add");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let runtime = CausaluxRuntime::new(RuntimeConfig::default());
            
            b.iter(|| {
                for i in 0..size {
                    runtime.add_to_set("set1", format!("item_{}", i)).unwrap();
                }
            });
        });
    }
    
    group.finish();
}

fn bench_set_remove(c: &mut Criterion) {
    c.bench_function("set_remove_1000", |b| {
        b.iter_batched(
            || {
                let runtime = CausaluxRuntime::new(RuntimeConfig::default());
                
                // Add 1000 items
                for i in 0..1000 {
                    runtime.add_to_set("set1", format!("item_{}", i)).unwrap();
                }
                
                runtime
            },
            |runtime| {
                // Remove all items
                for i in 0..1000 {
                    runtime.remove_from_set("set1", &format!("item_{}", i)).unwrap();
                }
            },
            criterion::BatchSize::SmallInput
        );
    });
}

// ============================================================================
// SYNC BENCHMARKS
// ============================================================================

fn bench_sync_small_partition(c: &mut Criterion) {
    c.bench_function("sync_100_ops", |b| {
        b.iter_batched(
            || {
                let node1 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node1".to_string(),
                    ..Default::default()
                });
                let node2 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node2".to_string(),
                    ..Default::default()
                });
                
                // Node1 creates document and edits
                node1.create_document("doc1".to_string(), "Test".to_string()).unwrap();
                for i in 0..100 {
                    node1.insert_text("doc1", i, "A").unwrap();
                }
                
                (node1, node2)
            },
            |(node1, node2)| {
                node1.sync_with_node(&node2).unwrap();
            },
            criterion::BatchSize::SmallInput
        );
    });
}

fn bench_sync_large_partition(c: &mut Criterion) {
    c.bench_function("sync_1000_ops", |b| {
        b.iter_batched(
            || {
                let node1 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node1".to_string(),
                    ..Default::default()
                });
                let node2 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node2".to_string(),
                    ..Default::default()
                });
                
                // Node1 performs many operations
                node1.create_document("doc1".to_string(), "Test".to_string()).unwrap();
                for i in 0..1000 {
                    node1.insert_text("doc1", i, "A").unwrap();
                }
                
                (node1, node2)
            },
            |(node1, node2)| {
                node1.sync_with_node(&node2).unwrap();
            },
            criterion::BatchSize::SmallInput
        );
    });
}

fn bench_sync_bidirectional(c: &mut Criterion) {
    c.bench_function("sync_bidirectional", |b| {
        b.iter_batched(
            || {
                let node1 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node1".to_string(),
                    ..Default::default()
                });
                let node2 = CausaluxRuntime::new(RuntimeConfig {
                    node_id: "node2".to_string(),
                    ..Default::default()
                });
                
                // Both nodes create different documents
                node1.create_document("doc1".to_string(), "Node1 Doc".to_string()).unwrap();
                node2.create_document("doc2".to_string(), "Node2 Doc".to_string()).unwrap();
                
                for i in 0..100 {
                    node1.insert_text("doc1", i, "A").unwrap();
                    node2.insert_text("doc2", i, "B").unwrap();
                }
                
                (node1, node2)
            },
            |(node1, node2)| {
                node1.sync_with_node(&node2).unwrap();
                node2.sync_with_node(&node1).unwrap();
            },
            criterion::BatchSize::SmallInput
        );
    });
}

// ============================================================================
// COLLABORATIVE EDITING BENCHMARKS
// ============================================================================

fn bench_collaborative_editing(c: &mut Criterion) {
    let mut group = c.benchmark_group("collaborative_editing");
    
    for num_nodes in [2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_nodes),
            num_nodes,
            |b, &num_nodes| {
                b.iter_batched(
                    || {
                        // Create N nodes
                        let nodes: Vec<_> = (0..num_nodes)
                            .map(|i| {
                                CausaluxRuntime::new(RuntimeConfig {
                                    node_id: format!("node{}", i),
                                    ..Default::default()
                                })
                            })
                            .collect();
                        
                        // All create same document
                        for node in &nodes {
                            node.create_document("shared".to_string(), "Shared Doc".to_string()).unwrap();
                        }
                        
                        // All edit concurrently
                        for (i, node) in nodes.iter().enumerate() {
                            for j in 0..10 {
                                node.insert_text("shared", j, &format!("N{}", i)).unwrap();
                            }
                        }
                        
                        nodes
                    },
                    |nodes| {
                        // Full mesh sync
                        for i in 0..nodes.len() {
                            for j in 0..nodes.len() {
                                if i != j {
                                    nodes[i].sync_with_node(&nodes[j]).unwrap();
                                }
                            }
                        }
                    },
                    criterion::BatchSize::SmallInput
                );
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// MEMORY BENCHMARKS
// ============================================================================

fn bench_memory_footprint(c: &mut Criterion) {
    c.bench_function("memory_10k_ops", |b| {
        b.iter(|| {
            let runtime = CausaluxRuntime::new(RuntimeConfig {
                snapshot_interval: 1000,
                ..Default::default()
            });
            
            runtime.create_document("doc1".to_string(), "Test".to_string()).unwrap();
            
            // Perform 10K operations (should trigger snapshots)
            for i in 0..10000 {
                runtime.insert_text("doc1", i % 100, "A").unwrap();
            }
            
            // Check metrics
            let metrics = runtime.get_metrics();
            black_box(metrics);
        });
    });
}

// ============================================================================
// CRITERION CONFIGURATION
// ============================================================================

criterion_group!(
    benches,
    bench_document_insert,
    bench_document_delete,
    bench_counter_increment,
    bench_counter_concurrent,
    bench_set_add,
    bench_set_remove,
    bench_sync_small_partition,
    bench_sync_large_partition,
    bench_sync_bidirectional,
    bench_collaborative_editing,
    bench_memory_footprint,
);

criterion_main!(benches);
