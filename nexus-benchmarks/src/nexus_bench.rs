//! NEXUS benchmark module

use crate::BenchmarkResult;
use nexus_pcu::{USO, PrincipalId};
use causalux_v2::VersionVector;
use std::time::Instant;

pub fn run_benchmarks() -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    let iterations = 100_000u64;

    // USO Create benchmark
    {
        let start = Instant::now();
        for i in 0..iterations {
            let data = format!("data_{}", i);
            let _uso = USO::new(data.as_bytes().to_vec(), PrincipalId::generate());
        }
        let duration = start.elapsed();
        let total_us = duration.as_micros() as u64;
        
        results.push(BenchmarkResult {
            system: "NEXUS".to_string(),
            operation: "USO Create".to_string(),
            iterations,
            total_duration_us: total_us,
            ops_per_second: (iterations as f64) / duration.as_secs_f64(),
            avg_latency_us: total_us as f64 / iterations as f64,
        });
        println!("  ✓ NEXUS USO Create: {:.0} ops/sec", (iterations as f64) / duration.as_secs_f64());
    }

    // Version Vector Merge benchmark (THE CORE CLAIM)
    {
        let merge_iterations = 1_000_000u64;
        
        let start = Instant::now();
        for _ in 0..merge_iterations {
            let mut vv1 = VersionVector::new();
            let mut vv2 = VersionVector::new();
            
            vv1.increment("node1");
            vv2.increment("node2");
            
            let _merged = vv1.merge(&vv2);
        }
        let duration = start.elapsed();
        let total_us = duration.as_micros() as u64;
        
        results.push(BenchmarkResult {
            system: "NEXUS".to_string(),
            operation: "Version Vector Merge".to_string(),
            iterations: merge_iterations,
            total_duration_us: total_us,
            ops_per_second: (merge_iterations as f64) / duration.as_secs_f64(),
            avg_latency_us: total_us as f64 / merge_iterations as f64,
        });
        println!("  ✓ NEXUS VV Merge: {:.0} ops/sec", (merge_iterations as f64) / duration.as_secs_f64());
    }

    results
}
