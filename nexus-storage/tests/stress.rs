// NEXUS Storage: Stress Tests
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use nexus_storage::ProvenanceLog;
use tempfile::tempdir;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[test]
fn stress_test_throughput() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    let log = Arc::new(ProvenanceLog::open(path).expect("Failed to open log"));
    
    let num_threads = 4;
    let ops_per_thread = 25000;
    let total_ops = num_threads * ops_per_thread;
    
    println!("Starting stress test: {} total operations across {} threads...", total_ops, num_threads);
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for t in 0..num_threads {
        let log_clone = Arc::clone(&log);
        let signing_key = generate_signing_key();
        
        let handle = thread::spawn(move || {
            let mut clock = VectorClock::new();
            for i in 0..ops_per_thread {
                let data = vec![0u8; 100]; // 100 bytes of data
                let tensor = CausalTensor::new(
                    data,
                    vec![],
                    (t * 100 + t) as u64, // Unique node ID
                    &mut clock,
                    &signing_key,
                ).unwrap();
                log_clone.append(&tensor).expect("Append failed");
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("Stress test complete!");
    println!("Total time: {:?}", duration);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
    
    // Approximate count
    let count = log.count_approximate();
    println!("Approximate key count: {}", count);
    
    assert!(ops_per_sec > 1000.0, "Throughput too low (was {:.2})", ops_per_sec);
}

#[test]
fn stress_test_batch_throughput() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    let log = Arc::new(ProvenanceLog::open(path).expect("Failed to open log"));
    
    let total_ops = 100000;
    let batch_size = 1000;
    let num_batches = total_ops / batch_size;
    
    let signing_key = generate_signing_key();
    let mut clock = VectorClock::new();
    
    println!("Starting batch stress test: {} operations in {} batches of {}...", total_ops, num_batches, batch_size);
    
    let start = Instant::now();
    
    for _ in 0..num_batches {
        let mut batch = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            let tensor = CausalTensor::new(
                vec![0u8; 100],
                vec![],
                1,
                &mut clock,
                &signing_key,
            ).unwrap();
            batch.push(tensor);
        }
        log.append_batch(&batch).expect("Batch append failed");
    }
    
    let duration = start.elapsed();
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("Batch stress test complete!");
    println!("Total time: {:?}", duration);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
    
    assert!(ops_per_sec > 50000.0, "Batch throughput too low (was {:.2})", ops_per_sec);
}
