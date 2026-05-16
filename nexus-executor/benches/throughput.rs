//! Throughput Benchmarks
//!
//! Measures PCU execution throughput (PCUs/second) under various conditions:
//! - Single-threaded sequential execution
//! - Concurrent execution (multiple PCUs in parallel)
//! - Mixed workloads (cache hits + misses)
//!
//! These benchmarks are critical for understanding system capacity
//! and identifying bottlenecks under load.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nexus_executor::{PcuExecutor, PCU, ExecutionContext, NodeId, NoopHost};
use nexus_pcu::{ContentHash, IdentityContext};
use std::sync::Arc;
use tokio::runtime::Runtime;

const DUMMY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60,
    0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73,
    0x74, 0x61, 0x72, 0x74, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b
];

fn bench_sequential_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    let pcu = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    
    // Warm up
    rt.block_on(executor.execute(&pcu, context.clone())).ok();
    
    let mut group = c.benchmark_group("throughput-sequential");
    
    // Measure throughput for different batch sizes
    for batch_size in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("pcus_per_second", format!("batch_{}", batch_size)),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    for _ in 0..batch_size {
                        let result = executor.execute(black_box(&pcu), black_box(context.clone()));
                        rt.block_on(result).ok();
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_concurrent_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    let pcu = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    
    // Warm up
    rt.block_on(executor.execute(&pcu, context.clone())).ok();
    
    let mut group = c.benchmark_group("throughput-concurrent");
    
    // Measure concurrent execution throughput
    for concurrency in [1, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_pcus", format!("{}_parallel", concurrency)),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    use futures::future;
                    let futures: Vec<_> = (0..concurrency)
                        .map(|_| executor.execute(black_box(&pcu), black_box(context.clone())))
                        .collect();
                    rt.block_on(future::join_all(futures));
                });
            },
        );
    }
    
    group.finish();
}

fn bench_mixed_workload_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    // Create PCU and pre-populate cache (for hits)
    let pcu_hit = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    
    // Pre-populate cache
    rt.block_on(executor.execute(&pcu_hit, context.clone())).ok();
    
    // Create PCU for misses
    let mut pcu_miss = pcu_hit.clone();
    pcu_miss.inputs.push(ContentHash::compute(b"unique"));
    
    let mut group = c.benchmark_group("throughput-mixed");
    
    // 50% hit rate
    group.bench_function("mixed_50pct_hit", |b| {
        b.iter(|| {
            // Alternate between hit and miss
            rt.block_on(executor.execute(black_box(&pcu_hit), black_box(context.clone()))).ok();
            rt.block_on(executor.execute(black_box(&pcu_miss), black_box(context.clone()))).ok();
        });
    });
    
    // 90% hit rate (simulate warm cache)
    group.bench_function("mixed_90pct_hit", |b| {
        b.iter(|| {
            // 9 hits, 1 miss
            for _ in 0..9 {
                rt.block_on(executor.execute(black_box(&pcu_hit), black_box(context.clone()))).ok();
            }
            rt.block_on(executor.execute(black_box(&pcu_miss), black_box(context.clone()))).ok();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_throughput,
    bench_concurrent_throughput,
    bench_mixed_workload_throughput
);
criterion_main!(benches);


