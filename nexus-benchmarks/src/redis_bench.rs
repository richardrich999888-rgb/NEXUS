//! Redis benchmark module

use crate::BenchmarkResult;
use std::time::Instant;

pub fn run_benchmarks() -> Result<Vec<BenchmarkResult>, String> {
    let client = redis::Client::open("redis://127.0.0.1/")
        .map_err(|e| e.to_string())?;
    let mut con = client.get_connection()
        .map_err(|e| format!("Redis not running: {}", e))?;

    let mut results = Vec::new();
    let iterations = 10_000u64;

    // SET benchmark
    {
        let start = Instant::now();
        for i in 0..iterations {
            let _: () = redis::cmd("SET")
                .arg(format!("key:{}", i))
                .arg("value")
                .query(&mut con)
                .map_err(|e| e.to_string())?;
        }
        let duration = start.elapsed();
        let total_us = duration.as_micros() as u64;
        
        results.push(BenchmarkResult {
            system: "Redis".to_string(),
            operation: "SET".to_string(),
            iterations,
            total_duration_us: total_us,
            ops_per_second: (iterations as f64) / duration.as_secs_f64(),
            avg_latency_us: total_us as f64 / iterations as f64,
        });
        println!("  ✓ Redis SET: {:.0} ops/sec", (iterations as f64) / duration.as_secs_f64());
    }

    // GET benchmark
    {
        let start = Instant::now();
        for i in 0..iterations {
            let _: String = redis::cmd("GET")
                .arg(format!("key:{}", i))
                .query(&mut con)
                .map_err(|e| e.to_string())?;
        }
        let duration = start.elapsed();
        let total_us = duration.as_micros() as u64;
        
        results.push(BenchmarkResult {
            system: "Redis".to_string(),
            operation: "GET".to_string(),
            iterations,
            total_duration_us: total_us,
            ops_per_second: (iterations as f64) / duration.as_secs_f64(),
            avg_latency_us: total_us as f64 / iterations as f64,
        });
        println!("  ✓ Redis GET: {:.0} ops/sec", (iterations as f64) / duration.as_secs_f64());
    }

    // Cleanup
    let _: () = redis::cmd("FLUSHDB").query(&mut con).unwrap_or(());

    Ok(results)
}
