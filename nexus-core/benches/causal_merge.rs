// NEXUS Core: Causal Merge Benchmarks
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;

fn bench_causal_merge(c: &mut Criterion) {
    let signing_key = generate_signing_key();
    
    // Setup for Idempotent Merge (same tensor)
    let mut clock = VectorClock::new();
    let tensor = CausalTensor::new(
        vec![0u8; 1024], // 1KB data
        vec![],
        1,
        &mut clock,
        &signing_key,
    ).unwrap();

    c.bench_function("merge_idempotent", |b| {
        b.iter(|| {
            CausalTensor::merge(
                black_box(&tensor),
                black_box(&tensor),
                1,
                &mut clock.clone(),
                &signing_key,
            ).unwrap()
        });
    });

    // Setup for Causal Monotonicity (one is newer)
    let mut clock_newer = clock.clone();
    let tensor_newer = CausalTensor::new(
        vec![0u8; 1024],
        vec![tensor.id],
        1,
        &mut clock_newer,
        &signing_key,
    ).unwrap();

    c.bench_function("merge_monotonic", |b| {
        b.iter(|| {
            CausalTensor::merge(
                black_box(&tensor),
                black_box(&tensor_newer),
                1,
                &mut clock.clone(), // Reset clock for each iter
                &signing_key,
            ).unwrap()
        });
    });

    // Setup for Concurrent Merge (conflict resolution)
    let mut clock_a = VectorClock::new();
    let tensor_a = CausalTensor::new(
        vec![0u8; 1024],
        vec![],
        1,
        &mut clock_a,
        &signing_key,
    ).unwrap();

    let mut clock_b = VectorClock::new();
    let tensor_b = CausalTensor::new(
        vec![1u8; 1024],
        vec![],
        2,
        &mut clock_b,
        &signing_key,
    ).unwrap();

    c.bench_function("merge_concurrent_1kb", |b| {
        b.iter(|| {
            let mut merged_clock = VectorClock::new();
            CausalTensor::merge(
                black_box(&tensor_a),
                black_box(&tensor_b),
                3,
                &mut merged_clock,
                &signing_key,
            ).unwrap()
        });
    });
    
    // Large data merge (1MB)
    let mut clock_large_a = VectorClock::new();
    let tensor_large_a = CausalTensor::new(
        vec![0u8; 1024 * 1024], // 1MB
        vec![],
        1,
        &mut clock_large_a,
        &signing_key,
    ).unwrap();

    let mut clock_large_b = VectorClock::new();
    let tensor_large_b = CausalTensor::new(
        vec![1u8; 1024 * 1024], // 1MB
        vec![],
        2,
        &mut clock_large_b,
        &signing_key,
    ).unwrap();

    c.bench_function("merge_concurrent_1mb", |b| {
        b.iter(|| {
            let mut merged_clock = VectorClock::new();
            CausalTensor::merge(
                black_box(&tensor_large_a),
                black_box(&tensor_large_b),
                3,
                &mut merged_clock,
                &signing_key,
            ).unwrap()
        });
    });
}

criterion_group!(benches, bench_causal_merge);
criterion_main!(benches);
