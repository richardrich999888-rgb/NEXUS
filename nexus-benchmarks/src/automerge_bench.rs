//! Automerge benchmark module

use crate::BenchmarkResult;
use automerge::{AutoCommit, ObjType, ROOT};
use automerge::transaction::Transactable;
use std::time::Instant;

pub fn run_benchmarks() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    let iterations = 10_000u64;

    // Document Insert benchmark
    {
        let mut doc = AutoCommit::new();
        let list = doc.put_object(ROOT, "items", ObjType::List).unwrap();
        
        let start = Instant::now();
        for i in 0..iterations {
            doc.insert(&list, i as usize, format!("item_{}", i)).unwrap();
        }
        let duration = start.elapsed();
        let total_us = duration.as_micros() as u64;
        
        results.push(BenchmarkResult {
            system: "Automerge".to_string(),
            operation: "List Insert".to_string(),
            iterations,
            total_duration_us: total_us,
            ops_per_second: (iterations as f64) / duration.as_secs_f64(),
            avg_latency_us: total_us as f64 / iterations as f64,
        });
        println!("  ✓ Automerge Insert: {:.0} ops/sec", (iterations as f64) / duration.as_secs_f64());
    }

    // Document Merge benchmark
    {
        let merge_iterations = 1_000u64;
        
        let start = Instant::now();
        for _ in 0..merge_iterations {
            let mut doc1 = AutoCommit::new();
            let mut doc2 = AutoCommit::new();
            
            doc1.put(ROOT, "key1", "value1").unwrap();
            doc2.put(ROOT, "key2", "value2").unwrap();
            
            doc1.merge(&mut doc2).unwrap();
        }
        let duration = start.elapsed();
        let total_us = duration.as_micros() as u64;
        
        results.push(BenchmarkResult {
            system: "Automerge".to_string(),
            operation: "Doc Merge".to_string(),
            iterations: merge_iterations,
            total_duration_us: total_us,
            ops_per_second: (merge_iterations as f64) / duration.as_secs_f64(),
            avg_latency_us: total_us as f64 / merge_iterations as f64,
        });
        println!("  ✓ Automerge Merge: {:.0} ops/sec", (merge_iterations as f64) / duration.as_secs_f64());
    }

    results
}
