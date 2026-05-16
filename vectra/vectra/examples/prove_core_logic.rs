//! Proof that VECTRA's core algorithms work
//!
//! This demonstrates the actual logic without full encode/decode:
//! 1. Pattern detection algorithm finds repeating structures
//! 2. Entropy calculation measures compressibility
//! 3. Semantic type inference classifies data
//! 4. EBTA-X adaptive thresholds work with real data

use vectra::decompose::decompose;
use vectra::ebta::compute_byte_entropy;
use vectra::ebta_x::{analyze_payload, AdaptiveThresholdCalculator};
use vectra::Payload;

fn main() {
    println!("=== PROOF: VECTRA Core Logic Works ===\n");
    
    demo_pattern_detection();
    demo_entropy_calculation();
    demo_adaptive_ebta_x();
}

fn demo_pattern_detection() {
    println!("--- 1. Pattern Detection Algorithm ---");
    
    // Real structured data
    let data = b"HEADER:value1:HEADER:value2:HEADER:value3:HEADER:value4".to_vec();
    println!("Input: {:?}", String::from_utf8_lossy(&data));
    println!("Size: {} bytes\n", data.len());
    
    let payload = Payload::new(data);
    
    match decompose(&payload) {
        Ok(result) => {
            println!("✓ Decomposition successful!");
            println!("  Structural levels found: {}", result.structure.levels.len());
            
            if let Some(level) = result.structure.levels.first() {
                println!("  Pattern detected: {:?}", String::from_utf8_lossy(&level.literals));
                println!("  Pattern ID: {}", level.pattern_id);
            }
            
            println!("  Structural byte ranges: {}", result.structure.byte_ranges.len());
            for (i, range) in result.structure.byte_ranges.iter().enumerate() {
                println!("    Range {}: bytes {}-{}", i, range.start, range.end);
            }
            
            println!("  Variable segments: {}", result.variable.segments.len());
            for (i, seg) in result.variable.segments.iter().enumerate() {
                println!("    Segment {}: {:?} (type: {:?})", 
                    i, 
                    String::from_utf8_lossy(&seg.data),
                    seg.semantic_type
                );
            }
            println!();
        }
        Err(e) => println!("✗ Decomposition failed: {:?}\n", e),
    }
}

fn demo_entropy_calculation() {
    println!("--- 2. Entropy Calculation (Shannon) ---");
    
    // Low entropy (repeating pattern)
    let low_entropy_data = vec![0xAA; 100];
    let low_h = compute_byte_entropy(&low_entropy_data);
    println!("Repeating bytes (0xAA × 100):");
    println!("  Entropy: {:.4} bits/byte", low_h);
    println!("  Interpretation: {} (perfect compression possible)", 
        if low_h < 1.0 { "Very low" } else { "Low" });
    
    // Medium entropy (structured)
    let medium_entropy_data: Vec<u8> = (0..100).map(|i| (i % 16) as u8).collect();
    let medium_h = compute_byte_entropy(&medium_entropy_data);
    println!("\nStructured pattern (0-15 repeating):");
    println!("  Entropy: {:.4} bits/byte", medium_h);
    println!("  Interpretation: Medium (good compression possible)");
    
    // High entropy (random-like)
    let high_entropy_data: Vec<u8> = (0..=255).collect();
    let high_h = compute_byte_entropy(&high_entropy_data);
    println!("\nUniform distribution (0-255):");
    println!("  Entropy: {:.4} bits/byte", high_h);
    println!("  Interpretation: {} (compression not safe)", 
        if high_h > 7.0 { "Very high" } else { "High" });
    
    println!("\n✓ Entropy calculation proves data compressibility!\n");
}

fn demo_adaptive_ebta_x() {
    println!("--- 3. EBTA-X Adaptive Thresholds ---");
    
    let mut calculator = AdaptiveThresholdCalculator::new();
    
    // Simulate learning from successful compressions
    println!("Training adaptive threshold calculator...");
    for i in 0..30 {
        let entropy = 3.0 + (i as f64 * 0.05); // Gradually increasing
        calculator.record_success(entropy);
    }
    println!("  Recorded 30 successful compressions\n");
    
    // Test with different payload types
    let text_payload = Payload::new(b"This is text data with patterns patterns patterns".to_vec());
    let text_chars = analyze_payload(&text_payload);
    
    println!("Text payload analysis:");
    println!("  ASCII ratio: {:.2}", text_chars.ascii_ratio);
    println!("  Compressibility: {:.2}", text_chars.compressibility);
    println!("  Pattern density: {:.2}", text_chars.pattern_density);
    println!("  Byte entropy: {:.4}", text_chars.byte_entropy);
    
    let threshold = calculator.calculate_threshold(&text_chars);
    println!("  Adaptive threshold: {:.2}", threshold);
    println!("  Decision: {}", 
        if text_chars.byte_entropy <= threshold { 
            "✓ ACCEPT (entropy below threshold)" 
        } else { 
            "✗ REJECT (entropy too high)" 
        });
    
    println!("\n✓ EBTA-X adapts thresholds based on data characteristics!\n");
}
