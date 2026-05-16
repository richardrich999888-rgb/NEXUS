// NEXUS Storage: Benchmarks
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use criterion::{criterion_group, criterion_main, Criterion};
use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use nexus_storage::AlgebraicIndex;
use tempfile::tempdir;

fn bench_storage_query(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path = dir.path();
    let index = AlgebraicIndex::open(path).unwrap();
    let signing_key = generate_signing_key();
    
    // Setup: Index 1000 tensors
    for i in 0..1000 {
        let mut clock = VectorClock::new();
        let tensor = CausalTensor::new(
            vec![0u8; 1024], // 1KB data
            vec![],
            (i % 10) as u64, // 10 different nodes
            &mut clock,
            &signing_key,
        ).unwrap();
        index.index_tensor(&tensor).unwrap();
    }
    
    c.bench_function("get_tensor_by_id", |b| {
        // We'll just grab one ID to query
        let tensors = index.get_by_node(1).unwrap();
        let id = tensors[0];
        b.iter(|| {
            index.get_tensor(&id).unwrap();
        });
    });

    c.bench_function("query_by_node", |b| {
        b.iter(|| {
            index.get_by_node(1).unwrap();
        });
    });

    c.bench_function("query_by_depth", |b| {
        b.iter(|| {
            index.get_by_depth(0).unwrap();
        });
    });
}

criterion_group!(benches, bench_storage_query);
criterion_main!(benches);
