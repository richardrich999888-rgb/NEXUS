use vectra::*;

/// Helper to generate deterministic pseudo-random data
fn generate_deterministic_random(seed: u64, length: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(length);
    let mut state = seed;
    for _ in 0..length {
        // Simple LCG
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        data.push((state >> 33) as u8);
    }
    data
}

/// Helper to check exact reconstruction
fn check_round_trip(name: &str, data: &[u8]) {
    let payload = Payload::new(data.to_vec());
    let result = vectra_encode(payload.clone());

    match result {
        EncodeResult::Encoded(artifact) => {
            println!("Test '{}': Encoded to {} bytes (Ratio: {:.2})", 
                name, 
                estimate_artifact_size(&artifact),
                compression_ratio(&payload, &artifact)
            );
            
            let decoded = vectra_decode(&artifact).expect("Decode failed");
            assert_eq!(decoded.as_bytes(), data, "Test '{}': Decoded data mismatch", name);
        }
        EncodeResult::PassThrough(p) => {
            println!("Test '{}': Passed through (Entropy too high or no structure)", name);
            assert_eq!(p.as_bytes(), data, "Test '{}': Pass-through data mismatch", name);
        }
    }
}

#[test]
fn test_mixed_semantic_types() {
    // Construct a payload that simulates a log file with mixed types
    // Structure: "LogID: [Counter] TS: [Timestamp] Metric: [Value]"
    // The simple decomp split on ':' will create structural and variable parts.
    
    let mut payload = Vec::new();
    let mut counter = 1000;
    let mut ts = 1600000000;
    
    for i in 0..20 {
        counter += 1;
        ts += 60;
        
        let line = format!(
            "LogID:{}\nTS:{}\nMetric:{}\n", 
            counter, 
            ts, 
            // "Metric" value changes slightly
            50 + (i % 10)
        );
        payload.extend_from_slice(line.as_bytes());
    }

    check_round_trip("mixed_semantic_types", &payload);
}

#[test]
fn test_entropy_boundary() {
    // 1. Low Entropy: Repeating pattern
    let low_entropy = b"AABBCCAABBCCAABBCC".repeat(50);
    let payload_low = Payload::new(low_entropy.clone());
    
    if let EncodeResult::PassThrough(_) = vectra_encode(payload_low) {
        // It might pass through if FEE doesn't find structure effectively in raw bytes 
        // without separators, but let's see. The simple decomposer might put it all in one variable segment.
        // If it's all one variable segment, low entropy should pass EBTA.
        // However, if decompose puts it in one chunk, FEE won't run on it.
        // Let's ensure we use the public API, so we just expect round trip.
    }
    check_round_trip("low_entropy", &low_entropy);

    // 2. High Entropy: Random data
    let high_entropy = generate_deterministic_random(12345, 1024);
    let payload_high = Payload::new(high_entropy.clone());
    
    let result = vectra_encode(payload_high);
    match result {
        EncodeResult::Encoded(_) => {
            // It is technically possible for random data to pass if H < H_MAX accidentally,
            // but unlikely for 1KB.
            // Or if H_MAX is set very high (currently 4.0).
            // A uniform distribution has entropy ~8.0.
            panic!("High entropy data (random) should have failed encoding gate");
        }
        EncodeResult::PassThrough(p) => {
            // Desired behavior
            assert_eq!(p.as_bytes(), &high_entropy);
        }
    }
}

#[test]
fn test_large_payload_stability() {
    // 1MB payload
    // Repetitive structure to encourage encoding
    let chunk = b"key:value\ncounter\n"; // "key:value" is struct, "counter" is var
    let mut payload = Vec::with_capacity(1024 * 1024);
    for _ in 0..50000 {
        payload.extend_from_slice(chunk);
    }
    
    check_round_trip("large_payload_1MB", &payload);
}

#[test]
fn test_predictor_overflow_resilience() {
    // Test robustness against numeric parsing of massive numbers
    // that might overflow standard integer types if not handled carefully.
    
    let mut payload = Vec::new();
    // Generate numbers close to i64::MAX
    let base = i64::MAX - 100;
    
    for i in 0..200 {
        // "val" lines are variable
        let line = format!("val\n{}\n", base.wrapping_add(i)); 
        payload.extend_from_slice(line.as_bytes());
    }
    
    check_round_trip("predictor_overflow", &payload);
}

#[test]
fn test_empty_lines_edge_case() {
    let payload = b"\n\n\n\n".to_vec();
    check_round_trip("empty_lines", &payload);
}

#[test]
fn test_all_structural() {
    // Every line has a colon, so everything should be structural
    let payload = b"key:val\nfoo:bar\nstruct:only\n".to_vec();
    check_round_trip("all_structural", &payload);
}

#[test]
fn test_no_structural() {
    // No colons, all variable
    // Should be subject to entropy check
    let payload = b"just\nwords\nlist\nof\nthings\n".to_vec();
    check_round_trip("no_structural", &payload);
}
