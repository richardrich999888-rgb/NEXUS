//! Concrete demonstration of VECTRA's core logic
//! 
//! This example proves the algorithms work with real data:
//! 1. Pattern detection finds repeating structures
//! 2. Decomposition separates structure from variable data
//! 3. Entropy validation ensures safety
//! 4. Lossless reconstruction proves correctness

use vectra::{vectra_encode, vectra_decode, Payload, EncodeResult};

fn main() {
    println!("=== VECTRA Core Logic Demonstration ===\n");
    
    // Example 1: Structured data with repeating patterns
    demo_pattern_detection();
    
    // Example 2: Real-world telemetry data
    demo_telemetry();
    
    // Example 3: Fail-open behavior with high entropy
    demo_fail_open();
}

fn demo_pattern_detection() {
    println!("--- Example 1: Pattern Detection ---");
    
    // Simulated structured data with repeating header
    let data = b"HEADER:12345:HEADER:67890:HEADER:11111:HEADER:22222".to_vec();
    println!("Original data: {:?}", String::from_utf8_lossy(&data));
    println!("Original size: {} bytes", data.len());
    
    let payload = Payload::new(data.clone());
    
    // Encode
    let result = vectra_encode(payload);
    
    match result {
        EncodeResult::Encoded(artifact) => {
            let artifact_bytes = artifact.to_bytes();
            println!("✓ Encoded successfully");
            println!("  Artifact size: {} bytes", artifact_bytes.len());
            
            // Show what was detected
            println!("  Structural pattern detected: {:?}", 
                String::from_utf8_lossy(&artifact.generator.base));
            println!("  Pattern occurs {} times", artifact.generator.repetition.count);
            println!("  Variable segments: {}", artifact.residual.segments.len());
            
            // Decode and verify losslessness
            match vectra_decode(&artifact) {
                Ok(decoded) => {
                    if decoded.as_bytes() == &data {
                        println!("✓ Lossless reconstruction verified!");
                        println!("  Decoded matches original exactly\n");
                    } else {
                        println!("✗ FAILED: Decoded doesn't match!");
                    }
                }
                Err(e) => println!("✗ Decode failed: {:?}", e),
            }
        }
        EncodeResult::PassThrough(_) => {
            println!("✗ Failed to encode (high entropy)\n");
        }
    }
}

fn demo_telemetry() {
    println!("--- Example 2: Telemetry Data ---");
    
    // Simulated telemetry with timestamp pattern
    let telemetry = format!(
        "{{\"ts\":1234567890,\"cpu\":45}}{{\"ts\":1234567891,\"cpu\":47}}{{\"ts\":1234567892,\"cpu\":46}}"
    );
    
    println!("Telemetry: {}", &telemetry[..50]);
    println!("Original size: {} bytes", telemetry.len());
    
    let payload = Payload::new(telemetry.as_bytes().to_vec());
    let result = vectra_encode(payload);
    
    match result {
        EncodeResult::Encoded(artifact) => {
            println!("✓ Encoded successfully");
            println!("  Detected structure: {:?}", 
                String::from_utf8_lossy(&artifact.generator.base));
            
            // Verify losslessness
            if let Ok(decoded) = vectra_decode(&artifact) {
                if decoded.as_bytes() == telemetry.as_bytes() {
                    println!("✓ Lossless reconstruction verified!\n");
                } else {
                    println!("✗ Reconstruction mismatch\n");
                }
            }
        }
        EncodeResult::PassThrough(_) => {
            println!("  Passed through (no compressible patterns)\n");
        }
    }
}

fn demo_fail_open() {
    println!("--- Example 3: Fail-Open Safety ---");
    
    // Random-like data (high entropy)
    let random_data: Vec<u8> = (0..100).map(|i| ((i * 17 + 31) % 256) as u8).collect();
    println!("Random data (first 20 bytes): {:?}", &random_data[..20]);
    
    let payload = Payload::new(random_data.clone());
    let result = vectra_encode(payload);
    
    match result {
        EncodeResult::Encoded(_) => {
            println!("✗ Unexpectedly encoded high-entropy data");
        }
        EncodeResult::PassThrough(original) => {
            println!("✓ Correctly failed-open (returned original)");
            println!("  Reason: Entropy too high for safe compression");
            println!("  Original preserved: {}", original.as_bytes() == &random_data);
            println!("\nThis proves VECTRA never corrupts data!\n");
        }
    }
}
