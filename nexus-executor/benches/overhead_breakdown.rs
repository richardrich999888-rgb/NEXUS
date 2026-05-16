//! Execution Overhead Breakdown Benchmarks
//!
//! Separates and measures individual overhead components:
//! - Serialization overhead
//! - Cache lookup overhead
//! - Proof generation overhead
//! - Module compilation overhead (Wasmtime)
//!
//! This helps identify optimization opportunities and understand
//! where time is spent in the execution pipeline.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nexus_executor::{PcuExecutor, PCU, ExecutionContext, NodeId, NoopHost, ExecutionProof};
use nexus_pcu::{ContentHash, IdentityContext};
use std::sync::Arc;
use tokio::runtime::Runtime;

const DUMMY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60,
    0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73,
    0x74, 0x61, 0x72, 0x74, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b
];

fn bench_serialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("overhead-serialization");
    
    // Test different PCU sizes
    for size in [100, 1000, 10000, 100000].iter() {
        let code = vec![0u8; *size];
        let pcu = PCU::new(
            nexus_pcu::WasmModule::new(code),
            vec![ContentHash::compute(b"input1"), ContentHash::compute(b"input2")],
            vec![1, 2, 3, 4, 5],
            IdentityContext::anonymous(),
        );
        
        group.bench_with_input(
            BenchmarkId::new("pcu_to_bytes", format!("{}B", size)),
            &pcu,
            |b, pcu| {
                b.iter(|| {
                    black_box(pcu.to_bytes()).ok()
                });
            },
        );
        
        let bytes = pcu.to_bytes().unwrap();
        group.bench_with_input(
            BenchmarkId::new("pcu_from_bytes", format!("{}B", size)),
            &bytes,
            |b, bytes| {
                b.iter(|| {
                    black_box(PCU::from_bytes(bytes)).ok()
                });
            },
        );
    }
    
    group.finish();
}

fn bench_cache_lookup_overhead(c: &mut Criterion) {
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
    
    // Pre-populate cache
    rt.block_on(executor.execute(&pcu, context.clone())).ok();
    
    let mut group = c.benchmark_group("overhead-cache");
    
    // Cache hit overhead
    group.bench_function("cache_lookup_hit", |b| {
        b.iter(|| {
            rt.block_on(executor.execute(black_box(&pcu), black_box(context.clone()))).ok()
        });
    });
    
    // Cache miss overhead (force miss by changing inputs)
    let mut pcu_miss = pcu.clone();
    pcu_miss.inputs.push(ContentHash::compute(b"force-miss"));
    
    group.bench_function("cache_lookup_miss", |b| {
        b.iter(|| {
            rt.block_on(executor.execute(black_box(&pcu_miss), black_box(context.clone()))).ok()
        });
    });
    
    group.finish();
}

fn bench_proof_generation_overhead(c: &mut Criterion) {
    let node_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    
    let pcu = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![ContentHash::compute(b"input1"), ContentHash::compute(b"input2")],
        vec![],
        IdentityContext::anonymous(),
    );
    
    let inputs: Vec<(ContentHash, Vec<u8>)> = vec![
        (ContentHash::compute(b"input1"), b"data1".to_vec()),
        (ContentHash::compute(b"input2"), b"data2".to_vec()),
    ];
    
    let result = nexus_executor::ExecutionResult::new(
        b"output".to_vec(),
        1000,
        1024,
        std::time::Duration::from_millis(10),
    );
    
    let identity = IdentityContext::anonymous();
    
    let mut group = c.benchmark_group("overhead-proof");
    
    // Proof creation
    group.bench_function("proof_create", |b| {
        b.iter(|| {
            black_box(ExecutionProof::create(
                &pcu,
                &inputs,
                &result,
                &identity,
                &node_key,
            ))
        });
    });
    
    // Proof verification
    let proof = ExecutionProof::create(&pcu, &inputs, &result, &identity, &node_key);
    group.bench_function("proof_verify", |b| {
        b.iter(|| {
            black_box(proof.verify()).ok()
        });
    });
    
    // Proof signing bytes computation
    group.bench_function("proof_signing_bytes", |b| {
        b.iter(|| {
            black_box(proof.signing_bytes())
        });
    });
    
    group.finish();
}

fn bench_module_compilation_overhead(c: &mut Criterion) {
    use wasmtime::{Config, Engine, Module};
    
    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config).unwrap();
    
    let mut group = c.benchmark_group("overhead-compilation");
    
    // Test different WASM module sizes
    for size in [100, 1000, 10000, 100000].iter() {
        let wasm = vec![0u8; *size];
        // Make it valid WASM by prepending header
        let mut valid_wasm = DUMMY_WASM.to_vec();
        valid_wasm.extend_from_slice(&wasm);
        
        group.bench_with_input(
            BenchmarkId::new("module_compile", format!("{}B", size)),
            &valid_wasm,
            |b, wasm| {
                b.iter(|| {
                    black_box(Module::new(&engine, wasm)).ok()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_serialization_overhead,
    bench_cache_lookup_overhead,
    bench_proof_generation_overhead,
    bench_module_compilation_overhead
);
criterion_main!(benches);


