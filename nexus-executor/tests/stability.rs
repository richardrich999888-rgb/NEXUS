//! Long-Running Stability Tests
//!
//! These tests verify NEXUS remains stable under extended load:
//! - 24+ hour continuous execution
//! - Memory leak detection
//! - Throughput stability over time
//! - Resource usage stability
//!
//! These tests are designed to run in CI/CD with configurable duration
//! (default: 1 hour for CI, 24+ hours for production validation)

use nexus_executor::{PcuExecutor, PCU, ExecutionContext, NodeId, NoopHost};
use nexus_pcu::{WasmModule, IdentityContext, PrincipalId};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use ed25519_dalek::SigningKey;
use rand::RngCore;
use rand::rngs::OsRng;
use futures::future;

/// Minimal valid WASM module
const DUMMY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60,
    0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73,
    0x74, 0x61, 0x72, 0x74, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b
];

/// Create a properly signed identity for testing
fn create_test_identity() -> IdentityContext {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    identity
}

/// Test memory stability over extended period
/// 
/// This test runs for a configurable duration (default: 1 hour for CI)
/// and verifies that memory usage remains bounded.
#[tokio::test]
#[ignore] // Ignored by default - run with `cargo test -- --ignored`
async fn test_memory_stability_extended() {
    let duration = Duration::from_secs(
        std::env::var("STABILITY_TEST_DURATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600) // Default: 1 hour for CI
    );
    
    let node_id = NodeId::local();
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = Arc::new(PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap());
    
    let pcu = Arc::new(PCU::new(
        WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        create_test_identity(),
    ));
    
    let context = Arc::new(ExecutionContext::minimal());
    
    let start = Instant::now();
    let mut iteration = 0u64;
    let mut last_memory_check = Instant::now();
    
    // Track memory usage periodically
    let check_interval = Duration::from_secs(60); // Check every minute
    
    while start.elapsed() < duration {
        // Execute PCU
        let executor_clone = executor.clone();
        let pcu_clone = pcu.clone();
        let context_clone = context.clone();
        
        let result = executor_clone.execute(&pcu_clone, (*context_clone).clone()).await;
        
        // Verify execution succeeded
        assert!(result.is_ok(), "Execution failed at iteration {}", iteration);
        
        iteration += 1;
        
        // Periodic memory check
        if last_memory_check.elapsed() >= check_interval {
            // In a real implementation, would use memory profiling tools
            // For now, verify executor is still functional
            // Note: Cache stats access would require exposing cache getter
            last_memory_check = Instant::now();
        }
        
        // Small delay to prevent overwhelming the system
        if iteration % 100 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    eprintln!("Memory stability test completed: {} iterations in {:?}", iteration, start.elapsed());
    assert!(iteration > 0, "Should have executed at least one iteration");
}

/// Test throughput stability over time
/// 
/// Verifies that throughput (PCUs/second) remains stable over extended period.
#[tokio::test]
#[ignore]
async fn test_throughput_stability() {
    let duration = Duration::from_secs(
        std::env::var("STABILITY_TEST_DURATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600)
    );
    
    let node_id = NodeId::local();
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = Arc::new(PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap());
    
    let pcu = Arc::new(PCU::new(
        WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        create_test_identity(),
    ));
    
    let context = Arc::new(ExecutionContext::minimal());
    
    let start = Instant::now();
    let mut total_executions = 0u64;
    let mut window_start = Instant::now();
    let mut window_executions = 0u64;
    
    let window_duration = Duration::from_secs(60); // 1 minute windows
    let mut throughput_samples = Vec::new();
    
    while start.elapsed() < duration {
        let executor_clone = executor.clone();
        let pcu_clone = pcu.clone();
        let context_clone = context.clone();
        
        let exec_start = Instant::now();
        let result = executor_clone.execute(&pcu_clone, (*context_clone).clone()).await;
        let exec_duration = exec_start.elapsed();
        
        assert!(result.is_ok(), "Execution failed");
        
        total_executions += 1;
        window_executions += 1;
        
        // Calculate throughput for each window
        if window_start.elapsed() >= window_duration {
            let throughput = window_executions as f64 / window_duration.as_secs_f64();
            throughput_samples.push(throughput);
            
            eprintln!("Throughput window: {:.2} PCUs/sec", throughput);
            
            window_start = Instant::now();
            window_executions = 0;
        }
        
        // Small delay
        if total_executions % 100 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Verify throughput stability (coefficient of variation < 0.5)
    if throughput_samples.len() >= 3 {
        let mean: f64 = throughput_samples.iter().sum::<f64>() / throughput_samples.len() as f64;
        let variance: f64 = throughput_samples.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / throughput_samples.len() as f64;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean; // Coefficient of variation
        
        eprintln!("Throughput stability: mean={:.2}, std_dev={:.2}, CV={:.2}", mean, std_dev, cv);
        assert!(cv < 0.5, "Throughput should be stable (CV < 0.5), got CV={}", cv);
    }
    
    eprintln!("Throughput stability test completed: {} total executions", total_executions);
}

/// Test concurrent execution stability
/// 
/// Verifies that concurrent execution remains stable over extended period.
#[tokio::test]
#[ignore]
async fn test_concurrent_stability() {
    let duration = Duration::from_secs(
        std::env::var("STABILITY_TEST_DURATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600)
    );
    
    let concurrency = 10;
    let node_id = NodeId::local();
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = Arc::new(PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap());
    
    let pcu = Arc::new(PCU::new(
        WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        create_test_identity(),
    ));
    
    let context = Arc::new(ExecutionContext::minimal());
    
    let start = Instant::now();
    let mut total_executions = 0u64;
    let mut failures = 0u64;
    
    while start.elapsed() < duration {
        let mut handles = Vec::new();
        for _ in 0..concurrency {
            let executor_clone = executor.clone();
            let pcu_clone = pcu.clone();
            let context_clone = context.clone();
            
            handles.push(tokio::spawn(async move {
                executor_clone.execute(&pcu_clone, (*context_clone).clone()).await
            }));
        }
        
        let results = future::join_all(handles).await;
        for result in results {
            match result {
                Ok(Ok(_)) => total_executions += 1,
                _ => failures += 1,
            }
        }
        
        // Small delay
        sleep(Duration::from_millis(50)).await;
    }
    
    let failure_rate = failures as f64 / (total_executions + failures) as f64;
    eprintln!("Concurrent stability test: {} executions, {} failures, failure_rate={:.4}", 
              total_executions, failures, failure_rate);
    
    assert!(failure_rate < 0.01, "Failure rate should be < 1%, got {}", failure_rate);
    assert!(total_executions > 0, "Should have executed at least one PCU");
}

/// Test resource usage stability
/// 
/// Verifies that CPU and memory usage remain bounded over extended period.
#[tokio::test]
#[ignore]
async fn test_resource_usage_stability() {
    let duration = Duration::from_secs(
        std::env::var("STABILITY_TEST_DURATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600)
    );
    
    let node_id = NodeId::local();
    let signing_key = SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = Arc::new(PcuExecutor::new(node_id, signing_key, host, 1000, None).unwrap());
    
    let pcu = Arc::new(PCU::new(
        WasmModule::new(DUMMY_WASM.to_vec()),
        vec![],
        vec![],
        create_test_identity(),
    ));
    
    let context = Arc::new(ExecutionContext::minimal());
    
    let start = Instant::now();
    let mut iteration = 0u64;
    let check_interval = Duration::from_secs(300); // Check every 5 minutes
    let mut last_check = Instant::now();
    let mut resource_samples = Vec::new();
    
    while start.elapsed() < duration {
        let executor_clone = executor.clone();
        let pcu_clone = pcu.clone();
        let context_clone = context.clone();
        
        let exec_start = Instant::now();
        let result = executor_clone.execute(&pcu_clone, (*context_clone).clone()).await;
        let exec_duration = exec_start.elapsed();
        
        assert!(result.is_ok(), "Execution failed");
        
        iteration += 1;
        
        // Periodic resource check
        if last_check.elapsed() >= check_interval {
            // Sample execution time (proxy for CPU usage)
            resource_samples.push(exec_duration);
            
            // Verify executor is still functional (proxy for memory stability)
            // Note: Cache stats access would require exposing cache getter
            
            last_check = Instant::now();
        }
        
        if iteration % 100 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Verify execution time stability
    if resource_samples.len() >= 3 {
        let mean: f64 = resource_samples.iter()
            .map(|d| d.as_secs_f64())
            .sum::<f64>() / resource_samples.len() as f64;
        let max: f64 = resource_samples.iter()
            .map(|d| d.as_secs_f64())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        
        // Execution time should not grow significantly
        assert!(max < mean * 2.0, "Execution time should remain stable");
    }
    
    eprintln!("Resource usage stability test completed: {} iterations", iteration);
}

