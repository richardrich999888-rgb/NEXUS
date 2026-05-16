use worker::*;
use serde::{Deserialize, Serialize};
use nexus_pcu::{USO, PrincipalId};
use causalux_v2::VersionVector;
use std::collections::HashMap;

// ============================================================================
// API Types
// ============================================================================

#[derive(Serialize)]
struct BenchmarkResponse {
    operation: String,
    duration_us: u64,
    operations_per_second: f64,
    timestamp: u64,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    region: String,
    version: &'static str,
}

#[derive(Deserialize)]
struct CreateUsoRequest {
    data: String,
}

#[derive(Serialize)]
struct CreateUsoResponse {
    id: String,
    duration_us: u64,
}

// ============================================================================
// Worker Logic
// ============================================================================

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    
    console_error_panic_hook::set_once();
    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("NEXUS Edge Substrate v0.1.1 - Debug Mode"))
        
        .get("/health", |_, _| Response::ok("healthy"))

        // DEBUG: Test Randomness (still useful to try, but optional)
        .get("/api/debug/random", |_, _| {
            let mut buf = [0u8; 32];
            // If getrandom fails, return error cleanly
            match getrandom::getrandom(&mut buf) {
                Ok(_) => Response::ok(hex::encode(buf)),
                Err(e) => Response::error(format!("RNG Error: {}", e), 500)
            }
        })

        .get("/api/benchmark/causal-merge", |_, _| {
            let iterations = 100_000; // Reduced to avoid CPU timeout (10ms limit on free tier)
            let start = js_sys::Date::now();
            
            for _ in 0..iterations {
                let mut vv1 = VersionVector::new();
                let mut vv2 = VersionVector::new();
                
                vv1.increment("node1");
                vv2.increment("node2");
                let _merged = vv1.merge(&vv2);
            }
            
            let end = js_sys::Date::now();
            let duration_ms = end - start;
            let duration_us = (duration_ms * 1000.0) as u64;
            let ops_per_sec = (iterations as f64) / ((duration_ms.max(1.0)) / 1000.0);
            
            let resp = BenchmarkResponse {
                operation: "causal_merge".to_string(),
                duration_us,
                operations_per_second: ops_per_sec,
                timestamp: (js_sys::Date::now() / 1000.0) as u64,
            };
            Response::from_json(&resp)
        })

        .get("/api/benchmark/uso-creation", |_, _| {
            let iterations = 100; // Expensive crypto; reduce to 100
            let start = js_sys::Date::now();
            
            console_log!("Starting USO creation benchmark (deterministic)");
            
            for i in 0..iterations {
                let data = format!("test_data_{}", i);
                let mut bytes = [0u8; 32];
                let i_bytes = (i as u64).to_le_bytes();
                bytes[0..8].copy_from_slice(&i_bytes); 
                
                let _uso = USO::new(data.as_bytes().to_vec(), PrincipalId::from_bytes(bytes));
            }
            
            let end = js_sys::Date::now();
            let duration_ms = end - start;
            let duration_us = (duration_ms * 1000.0) as u64;
            let ops_per_sec = (iterations as f64) / ((duration_ms.max(1.0)) / 1000.0);
            
            let resp = BenchmarkResponse {
                operation: "uso_creation".to_string(),
                duration_us,
                operations_per_second: ops_per_sec,
                timestamp: (js_sys::Date::now() / 1000.0) as u64,
            };
            Response::from_json(&resp)
        })

        // Detailed latency benchmark with percentiles
        .get("/api/bench/latency", |_, _| {
            let iterations = 1000;
            let mut latencies: Vec<f64> = Vec::with_capacity(iterations);
            
            for i in 0..iterations {
                let start = js_sys::Date::now();
                
                // Measure single USO creation
                let data = format!("latency_test_{}", i);
                let mut bytes = [0u8; 32];
                bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
                let _uso = USO::new(data.as_bytes().to_vec(), PrincipalId::from_bytes(bytes));
                
                let end = js_sys::Date::now();
                latencies.push(end - start);
            }
            
            // Sort for percentiles
            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let p50 = latencies[iterations / 2];
            let p95 = latencies[(iterations * 95) / 100];
            let p99 = latencies[(iterations * 99) / 100];
            let avg = latencies.iter().sum::<f64>() / iterations as f64;
            
            let resp = serde_json::json!({
                "operation": "uso_latency_percentiles",
                "iterations": iterations,
                "p50_ms": p50,
                "p95_ms": p95,
                "p99_ms": p99,
                "avg_ms": avg,
                "timestamp": (js_sys::Date::now() / 1000.0) as u64,
            });
            Response::from_json(&resp)
        })

        // Hash lookup benchmark (simulates cache hit)
        .get("/api/bench/hash-lookup", |_, _| {
            let iterations = 10_000;
            let mut map: HashMap<String, Vec<u8>> = HashMap::new();
            
            // Pre-populate with test data
            for i in 0..100 {
                let key = format!("key_{}", i);
                let value = vec![i as u8; 64];
                map.insert(key, value);
            }
            
            let start = js_sys::Date::now();
            
            for i in 0..iterations {
                let key = format!("key_{}", i % 100);
                let _val = map.get(&key);
            }
            
            let end = js_sys::Date::now();
            let duration_ms = end - start;
            let ops_per_sec = (iterations as f64) / ((duration_ms.max(0.001)) / 1000.0);
            
            let resp = BenchmarkResponse {
                operation: "hash_lookup".to_string(),
                duration_us: (duration_ms * 1000.0) as u64,
                operations_per_second: ops_per_sec,
                timestamp: (js_sys::Date::now() / 1000.0) as u64,
            };
            Response::from_json(&resp)
        })

        .post_async("/api/uso", |mut req, _| async move {
            let create_req: CreateUsoRequest = match req.json().await {
                Ok(r) => r,
                Err(_) => return Response::error("Bad Request", 400),
            };

            let start = js_sys::Date::now();
            
            // Deterministic ID
            let mut bytes = [0u8; 32];
            bytes[0] = 0xDE;
            bytes[1] = 0xAD;
            bytes[2] = 0xBE;
            bytes[3] = 0xEF;
            
            let uso = USO::new(create_req.data.as_bytes().to_vec(), PrincipalId::from_bytes(bytes));
            let id = hex::encode(uso.id.as_bytes());
            let end = js_sys::Date::now();
            
            let resp = CreateUsoResponse {
                id,
                duration_us: ((end - start) * 1000.0) as u64,
            };
            Response::from_json(&resp)
        })

        .run(req, env)
        .await
}
