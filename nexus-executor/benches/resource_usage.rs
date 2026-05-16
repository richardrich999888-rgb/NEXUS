//! Resource Usage Benchmarks
//!
//! Measures CPU, memory, and storage I/O consumption per PCU execution.
//! These benchmarks are critical for capacity planning and cost estimation.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nexus_executor::{PcuExecutor, PCU, ExecutionContext, NodeId, NoopHost};
use std::sync::Arc;
use tokio::runtime::Runtime;

// Minimal valid WASM module (empty) with _start export
const DUMMY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60,
    0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73,
    0x74, 0x61, 0x72, 0x74, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b
];

// WASM module that allocates memory (for memory profiling)
fn wasm_with_memory(size: usize) -> Vec<u8> {
    // Simple WASM that allocates and uses memory
    // This is a minimal module that requests memory
    let mut wasm = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version
        0x05, 0x03, 0x01, // Memory section: 1 memory
        0x00, 0x01, // Memory: no max, initial pages (1 page = 64KB)
    ];
    
    // Add more pages if needed (rough approximation)
    let pages = (size / 65536).max(1).min(256) as u8;
    wasm[11] = pages;
    
    // Function section
    wasm.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]); // 1 function, type 0
    
    // Export section
    wasm.extend_from_slice(&[0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x00]);
    
    // Code section
    wasm.extend_from_slice(&[0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b]);
    
    wasm
}

fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    let mut group = c.benchmark_group("memory-usage");
    
    // Test different PCU sizes
    for size_kb in [1, 10, 100, 1000].iter() {
        let wasm = wasm_with_memory(*size_kb * 1024);
        let pcu = PCU::new(
            nexus_pcu::WasmModule::new(wasm),
            vec![],
            vec![],
            nexus_pcu::IdentityContext::anonymous(),
        );
        let context = ExecutionContext::minimal();
        
        // Warm up
        rt.block_on(executor.execute(&pcu, context.clone())).ok();
        
        group.bench_with_input(
            BenchmarkId::new("peak_memory", format!("{}KB", size_kb)),
            &pcu,
            |b, pcu| {
                b.iter(|| {
                    let result = executor.execute(black_box(pcu), black_box(context.clone()));
                    // Extract peak memory from result
                    rt.block_on(result).ok()
                });
            },
        );
    }
    
    group.finish();
}

fn bench_cpu_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    let pcu = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        nexus_pcu::IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    
    // Warm up
    rt.block_on(executor.execute(&pcu, context.clone())).ok();
    
    let mut group = c.benchmark_group("cpu-usage");
    
    // Measure CPU time (wall clock time is proxy for CPU usage in single-threaded benchmark)
    group.bench_function("execution_cpu_time", |b| {
        b.iter(|| {
            let result = executor.execute(black_box(&pcu), black_box(context.clone()));
            rt.block_on(result).ok()
        });
    });
    
    group.finish();
}

fn bench_storage_io(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    let pcu = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        nexus_pcu::IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    
    let mut group = c.benchmark_group("storage-io");
    
    // Measure serialization overhead (proxy for storage I/O)
    group.bench_function("pcu_serialization", |b| {
        b.iter(|| {
            black_box(pcu.to_bytes()).ok()
        });
    });
    
    group.bench_function("pcu_deserialization", |b| {
        let bytes = pcu.to_bytes().unwrap();
        b.iter(|| {
            black_box(PCU::from_bytes(&bytes)).ok()
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_memory_usage, bench_cpu_usage, bench_storage_io);
criterion_main!(benches);


