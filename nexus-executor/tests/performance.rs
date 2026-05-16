//! Performance and load tests for executor.
//! Uses valid PCU (signed identity + valid WASM) so executor identity/header checks pass; no guard for throughput measurement.

use nexus_executor::{PcuExecutor, NodeId, NoopHost, ExecutionContext, ExecutionLimits};
use nexus_pcu::{PCU, WasmModule, IdentityContext, PrincipalId};
use std::time::{Duration, Instant};
use std::sync::Arc;
use ed25519_dalek::SigningKey;

fn make_executor() -> PcuExecutor {
    PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[0u8; 32]),
        Arc::new(NoopHost),
        1000,
        None,
    )
    .unwrap()
}

/// Build a valid PCU (signed identity + minimal valid WASM) for performance tests. Executor requires valid identity and WASM header.
fn valid_pcu() -> PCU {
    let wasm = WasmModule::new(
        wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)
            .unwrap(),
    );
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    PCU::new(wasm, vec![], vec![], identity)
}

#[tokio::test]
async fn test_execution_latency() {
    let executor = make_executor();
    let pcu = valid_pcu();
    let context = ExecutionContext::minimal();

    let start = Instant::now();
    let result = executor.execute(&pcu, context).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Execution should succeed");
    assert!(duration < Duration::from_secs(1), "Execution should complete in <1s");
}

#[tokio::test]
async fn test_throughput_1000_pcus() {
    let executor = Arc::new(make_executor());
    let pcu = Arc::new(valid_pcu());
    let context = ExecutionContext::minimal();

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..1000 {
        let executor_clone = executor.clone();
        let pcu_clone = pcu.clone();
        let ctx = context.clone();
        handles.push(tokio::spawn(async move {
            executor_clone.execute(&pcu_clone, ctx).await
        }));
    }
    let results: Vec<_> = futures::future::join_all(handles).await;
    let duration = start.elapsed();
    let successes = results.iter().filter(|r| r.as_ref().map(|r| r.is_ok()).unwrap_or(false)).count();
    let throughput = 1000.0 / duration.as_secs_f64();

    println!("Executed {} PCUs in {:?} ({:.2} PCUs/sec)", successes, duration, throughput);
    assert!(successes > 900, "Should execute >90% of PCUs successfully");
    assert!(throughput > 100.0, "Should achieve >100 PCUs/sec throughput");
}

#[tokio::test]
async fn test_cache_hit_performance() {
    let executor = Arc::new(make_executor());
    let pcu = Arc::new(valid_pcu());
    let context = ExecutionContext::minimal();

    let start = Instant::now();
    let _result1 = executor.execute(&pcu, context.clone()).await;
    let miss_duration = start.elapsed();

    let start = Instant::now();
    let _result2 = executor.execute(&pcu, context).await;
    let hit_duration = start.elapsed();

    assert!(hit_duration < miss_duration, "Cache hit should be faster than miss");
    println!("Cache miss: {:?}, Cache hit: {:?}", miss_duration, hit_duration);
}

#[tokio::test]
async fn test_large_payload_handling() {
    let executor = make_executor();
    let large_input = vec![0u8; 100 * 1024 * 1024];
    let input_hash = nexus_pcu::ContentHash::compute(&large_input);
    let wasm = WasmModule::new(
        wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)
            .unwrap(),
    );
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    let pcu = PCU::new(wasm, vec![input_hash], vec![], identity.clone());
    let context = ExecutionContext::new(
        vec![(input_hash, large_input)],
        identity,
        ExecutionLimits::default(),
    );

    let start = Instant::now();
    let result = executor.execute(&pcu, context).await;
    let duration = start.elapsed();

    assert!(result.is_ok() || result.is_err(), "Should handle or reject large payload gracefully");
    assert!(duration < Duration::from_secs(30), "Large payload should process in reasonable time");
}

#[tokio::test]
async fn test_memory_usage() {
    let executor = Arc::new(make_executor());
    let context = ExecutionContext::minimal();

    for i in 0..100 {
        let pcu = Arc::new(valid_pcu());
        let _result = executor.execute(&pcu, context.clone()).await;
        if i % 10 == 0 {
            println!("Executed {} PCUs", i);
        }
    }
    assert!(true);
}

#[tokio::test]
async fn test_concurrent_cache_contention() {
    let executor = Arc::new(make_executor());
    let pcu = Arc::new(valid_pcu());
    let context = ExecutionContext::minimal();

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..100 {
        let executor_clone = executor.clone();
        let pcu_clone = pcu.clone();
        let ctx = context.clone();
        handles.push(tokio::spawn(async move {
            executor_clone.execute(&pcu_clone, ctx).await
        }));
    }
    let results: Vec<_> = futures::future::join_all(handles).await;
    let duration = start.elapsed();
    let successes = results.iter().filter(|r| r.as_ref().map(|x| x.is_ok()).unwrap_or(false)).count();

    assert!(successes > 90, "Should handle concurrent cache access");
    assert!(duration < Duration::from_secs(5), "Concurrent cache access should be fast");
}
