use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexus_executor::{PcuExecutor, PCU, ExecutionContext, NodeId, NoopHost};
use std::sync::Arc;
use tokio::runtime::Runtime;

// Minimal valid WASM module (empty) with _start export
// (module (func (export "_start")))
const DUMMY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60,
    0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73,
    0x74, 0x61, 0x72, 0x74, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b
];

fn bench_executor(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup executor
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap();
    
    // Create a simple PCU
    let pcu = PCU::new(
        nexus_pcu::WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        nexus_pcu::IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();

    let mut group = c.benchmark_group("nexus-executor");
    
    // Warm up the engine (Wasmtime compilation cache)
    rt.block_on(executor.execute(&pcu, context.clone())).unwrap();

    // Benchmark 1: Execution with Semantic Cache MISS (but Wasmtime engine warm)
    let mut pcu_miss = pcu.clone();
    pcu_miss.inputs.push(nexus_executor::ContentHash::compute(b"force-miss"));

    group.bench_function("execution-miss", |b| {
        b.iter(|| {
            rt.block_on(executor.execute(black_box(&pcu_miss), black_box(context.clone()))).ok()
        });
    });

    // Benchmark 2: Execution with Semantic Cache HIT
    group.bench_function("execution-hit", |b| {
        b.iter(|| {
            rt.block_on(executor.execute(black_box(&pcu), black_box(context.clone()))).ok()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_executor);
criterion_main!(benches);
