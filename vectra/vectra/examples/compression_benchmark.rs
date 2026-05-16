//! VECTRA Compression Benchmark Harness
//!
//! Measures baseline compression metrics:
//! - Input/output size
//! - Compression ratio
//! - Structural coverage (% bytes explained by patterns)
//! - Residual entropy
//!
//! Run: cargo run --example compression_benchmark

use std::time::Instant;
use vectra::{vectra_decode, vectra_encode, EncodeResult, Payload};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    VECTRA COMPRESSION BENCHMARK");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();

    // Test cases: (name, data generator)
    let test_cases: Vec<(&str, Vec<u8>)> = vec![
        // Highly repetitive - should compress well
        ("repetitive_text", generate_repetitive_text(1_000)),
        
        // Semi-structured logs - moderate compression
        ("structured_logs", generate_structured_logs(20)),
        
        // Random binary - should pass-through (no compression)
        ("random_binary", generate_random_binary(500)),
        
        // JSON-like data - structural patterns
        ("json_like", generate_json_like(15)),
        
        // Alternating patterns
        ("alternating", generate_alternating(500)),
        
        // Single pattern repeated
        ("single_pattern", b"HEADER\n".repeat(100)),
    ];

    println!("┌─────────────────────┬──────────┬──────────┬──────────┬────────────┬─────────────┐");
    println!("│ Test Case           │ Input    │ Output   │ Ratio    │ Struct Cov │ Resid Entr  │");
    println!("├─────────────────────┼──────────┼──────────┼──────────┼────────────┼─────────────┤");

    let mut results = Vec::new();

    for (name, data) in &test_cases {
        let metrics = benchmark_compression(name, data);
        results.push((name.to_string(), metrics.clone()));
        
        println!(
            "│ {:<19} │ {:>6} B │ {:>6} B │ {:>7.2}x │ {:>9.1}% │ {:>7.3} b/B │",
            name,
            metrics.input_size,
            metrics.output_size,
            metrics.compression_ratio,
            metrics.structural_coverage * 100.0,
            metrics.residual_entropy
        );
    }

    println!("└─────────────────────┴──────────┴──────────┴──────────┴────────────┴─────────────┘");
    println!();

    // Summary
    println!("SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let compressed: Vec<_> = results.iter().filter(|(_, m)| m.compression_ratio > 1.0).collect();
    let passthrough: Vec<_> = results.iter().filter(|(_, m)| m.compression_ratio <= 1.0).collect();
    
    println!("Compressed: {}/{}", compressed.len(), results.len());
    println!("Pass-through: {}/{}", passthrough.len(), results.len());
    
    if !compressed.is_empty() {
        let avg_ratio: f64 = compressed.iter().map(|(_, m)| m.compression_ratio).sum::<f64>() 
            / compressed.len() as f64;
        let avg_coverage: f64 = compressed.iter().map(|(_, m)| m.structural_coverage).sum::<f64>() 
            / compressed.len() as f64;
        println!("Avg compression ratio (compressed only): {:.2}x", avg_ratio);
        println!("Avg structural coverage (compressed only): {:.1}%", avg_coverage * 100.0);
    }
    
    println!();
    println!("BASELINE RECORDED — Ready for optimization experiments");
}

#[derive(Clone, Debug)]
struct CompressionMetrics {
    input_size: usize,
    output_size: usize,
    compression_ratio: f64,
    structural_coverage: f64,
    residual_entropy: f64,
    encode_time_ms: u128,
    lossless: bool,
}

fn benchmark_compression(name: &str, data: &[u8]) -> CompressionMetrics {
    let input_size = data.len();
    
    // Encode
    let start = Instant::now();
    let payload = Payload::new(data.to_vec());
    let result = vectra_encode(payload);
    let encode_time = start.elapsed();
    
    match result {
        EncodeResult::Encoded(ref artifact) => {
            let artifact_bytes = artifact.to_bytes();
            let output_size = artifact_bytes.len() + 4; // +4 for VCTR magic
            
            // Calculate structural coverage from residual
            let residual_bytes: usize = artifact.residual.segments.iter()
                .map(|s| s.delta.len())
                .sum();
            let structural_coverage = 1.0 - (residual_bytes as f64 / input_size as f64);
            
            // Calculate residual entropy
            let residual_entropy = calculate_entropy(&collect_residual_bytes(&artifact.residual));
            
            // Verify losslessness
            let lossless = match vectra_decode(artifact) {
                Ok(decoded) => decoded.as_bytes() == data,
                Err(_) => false,
            };
            
            CompressionMetrics {
                input_size,
                output_size,
                compression_ratio: input_size as f64 / output_size as f64,
                structural_coverage,
                residual_entropy,
                encode_time_ms: encode_time.as_millis(),
                lossless,
            }
        }
        EncodeResult::PassThrough(_) => {
            // Pass-through: entropy too high or no benefit
            let residual_entropy = calculate_entropy(data);
            
            CompressionMetrics {
                input_size,
                output_size: input_size + 4, // +4 for PASS magic
                compression_ratio: input_size as f64 / (input_size + 4) as f64,
                structural_coverage: 0.0,
                residual_entropy,
                encode_time_ms: encode_time.as_millis(),
                lossless: true, // PassThrough is always lossless
            }
        }
    }
}

/// Calculate Shannon entropy in bits per byte.
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    
    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

/// Collect all residual bytes into a single vector.
fn collect_residual_bytes(residual: &vectra::Residual) -> Vec<u8> {
    residual.segments.iter()
        .flat_map(|s| s.delta.iter().copied())
        .collect()
}

// Test data generators

fn generate_repetitive_text(size: usize) -> Vec<u8> {
    let pattern = b"The quick brown fox jumps over the lazy dog.\n";
    let mut data = Vec::with_capacity(size);
    while data.len() < size {
        data.extend_from_slice(pattern);
    }
    data.truncate(size);
    data
}

fn generate_structured_logs(entries: usize) -> Vec<u8> {
    let mut log = String::new();
    for i in 0..entries {
        let level = match i % 4 {
            0 => "INFO",
            1 => "DEBUG",
            2 => "WARN",
            _ => "ERROR",
        };
        let timestamp = 1702900000 + i * 60;
        log.push_str(&format!(
            "[{}] {} - Processing request {} from client_{:04}\n",
            timestamp, level, i * 1000, i % 50
        ));
    }
    log.into_bytes()
}

fn generate_random_binary(size: usize) -> Vec<u8> {
    // Pseudo-random using simple LCG (deterministic)
    let mut data = Vec::with_capacity(size);
    let mut state: u64 = 12345;
    for _ in 0..size {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        data.push((state >> 16) as u8);
    }
    data
}

fn generate_json_like(objects: usize) -> Vec<u8> {
    let mut json = String::from("[\n");
    for i in 0..objects {
        json.push_str(&format!(
            r#"  {{"id": {}, "name": "item_{:04}", "value": {}, "active": {}}}"#,
            i, i, i * 100, i % 2 == 0
        ));
        if i < objects - 1 {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("]\n");
    json.into_bytes()
}

fn generate_alternating(size: usize) -> Vec<u8> {
    (0..size).map(|i| if i % 2 == 0 { 0xAA } else { 0x55 }).collect()
}
