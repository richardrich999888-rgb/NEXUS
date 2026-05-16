//! Fuzz test for VECTRA losslessness invariant.
//!
//! Tests that `decode(encode(data)) == data` for random inputs.

use rand::Rng;
use vectra::{vectra_decode, vectra_encode, EncodeResult, Payload};

/// Generate random bytes of specified size.
fn random_bytes(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

/// Test losslessness invariant for a single payload.
fn check_losslessness(data: &[u8], test_name: &str) -> bool {
    let payload = Payload::new(data.to_vec());
    
    match vectra_encode(payload) {
        EncodeResult::Encoded(artifact) => {
            // Must decode successfully
            match vectra_decode(&artifact) {
                Ok(decoded) => {
                    if decoded.as_bytes() == data {
                        true
                    } else {
                        eprintln!(
                            "[{}] FAIL: Output mismatch. Input len={}, Output len={}",
                            test_name,
                            data.len(),
                            decoded.len()
                        );
                        false
                    }
                }
                Err(e) => {
                    eprintln!("[{}] FAIL: Decode error: {:?}", test_name, e);
                    false
                }
            }
        }
        EncodeResult::PassThrough(original) => {
            // Pass-through is acceptable (fail-open behavior)
            if original.as_bytes() == data {
                true
            } else {
                eprintln!("[{}] FAIL: PassThrough corrupted data", test_name);
                false
            }
        }
    }
}

#[test]
fn fuzz_losslessness_100_iterations() {
    const ITERATIONS: usize = 100;
    const MIN_SIZE: usize = 1024;      // 1KB
    const MAX_SIZE: usize = 100 * 1024; // 100KB
    
    let mut rng = rand::thread_rng();
    let mut passed = 0;
    let mut failed = 0;
    
    println!("\n=== VECTRA Fuzz Test ===");
    println!("Iterations: {}", ITERATIONS);
    println!("Size range: {}KB - {}KB\n", MIN_SIZE / 1024, MAX_SIZE / 1024);
    
    for i in 0..ITERATIONS {
        let size = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let data = random_bytes(size);
        let test_name = format!("fuzz_{:03}", i);
        
        if check_losslessness(&data, &test_name) {
            passed += 1;
            if (i + 1) % 10 == 0 {
                println!("Progress: {}/{} passed", passed, i + 1);
            }
        } else {
            failed += 1;
            // Fail fast on first error
            panic!(
                "Fuzz test failed at iteration {} with {} byte input",
                i, size
            );
        }
    }
    
    println!("\n=== Results ===");
    println!("Passed: {}/{}", passed, ITERATIONS);
    println!("Failed: {}", failed);
    
    assert_eq!(failed, 0, "Fuzz test had {} failures", failed);
}

#[test]
fn fuzz_edge_cases() {
    // Test specific edge cases
    let test_cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        ("single_byte", vec![0x42]),
        ("all_zeros_1kb", vec![0u8; 1024]),
        ("all_ones_1kb", vec![0xFF; 1024]),
        ("alternating", (0..1024).map(|i| if i % 2 == 0 { 0xAA } else { 0x55 }).collect()),
        ("sequential", (0..=255).cycle().take(1024).collect()),
        ("repeated_pattern", b"HEADER:DATA\n".repeat(100).to_vec()),
    ];
    
    println!("\n=== Edge Case Tests ===");
    
    for (name, data) in test_cases {
        print!("Testing {}: ", name);
        if check_losslessness(&data, name) {
            println!("✓ PASS");
        } else {
            println!("✗ FAIL");
            panic!("Edge case '{}' failed", name);
        }
    }
}
