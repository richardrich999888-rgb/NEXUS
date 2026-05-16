//! VECTRA Use Case: Audit Log Compression
//!
//! Demonstrates VECTRA for compliance-critical audit log storage.
//!
//! Key properties:
//! - Deterministic: Same log → same compressed artifact
//! - Lossless: Exact byte reconstruction
//! - Verifiable: Integrity hash proves no tampering
//!
//! Run: cargo run --example audit_log_demo

use std::time::Instant;
use vectra::{vectra_decode, vectra_encode, EncodeResult, Payload};

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("          VECTRA AUDIT LOG COMPRESSION DEMO");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    // Simulate audit log entries (structured, repetitive)
    let audit_log = generate_audit_log(100);
    
    println!("SCENARIO: Compliance-Critical Audit Log Storage");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Step 1: Show original data
    println!("1️⃣  ORIGINAL LOG");
    println!("   Size: {} bytes", audit_log.len());
    println!("   First entry: {}", &audit_log.lines().next().unwrap_or(""));
    println!();

    // Step 2: Compute original hash (for verification)
    let original_hash = sha256_hex(&audit_log.as_bytes());
    println!("2️⃣  ORIGINAL INTEGRITY");
    println!("   SHA-256: {}...", &original_hash[..32]);
    println!();

    // Step 3: Compress with VECTRA
    let start = Instant::now();
    let payload = Payload::new(audit_log.as_bytes().to_vec());
    let result = vectra_encode(payload);
    let encode_time = start.elapsed();

    println!("3️⃣  VECTRA COMPRESSION");
    match &result {
        EncodeResult::Encoded(artifact) => {
            let artifact_bytes = artifact.to_bytes();
            let ratio = audit_log.len() as f64 / artifact_bytes.len() as f64;
            println!("   Status: ✅ Encoded");
            println!("   Artifact size: {} bytes", artifact_bytes.len());
            println!("   Compression ratio: {:.2}x", ratio);
            println!("   Encode time: {:?}", encode_time);
        }
        EncodeResult::PassThrough(_) => {
            println!("   Status: ⚠️ Pass-through (encoding not beneficial)");
        }
    }
    println!();

    // Step 4: Decompress and verify
    if let EncodeResult::Encoded(artifact) = &result {
        let start = Instant::now();
        match vectra_decode(artifact) {
            Ok(decoded) => {
                let decode_time = start.elapsed();
                let decoded_hash = sha256_hex(decoded.as_bytes());
                
                println!("4️⃣  VECTRA DECOMPRESSION");
                println!("   Status: ✅ Decoded");
                println!("   Output size: {} bytes", decoded.len());
                println!("   Decode time: {:?}", decode_time);
                println!();
                
                println!("5️⃣  INTEGRITY VERIFICATION");
                let match_status = if original_hash == decoded_hash {
                    "✅ VERIFIED — Exact byte match"
                } else {
                    "❌ FAILED — Data corruption detected"
                };
                println!("   Original:  {}...", &original_hash[..32]);
                println!("   Decoded:   {}...", &decoded_hash[..32]);
                println!("   Status:    {}", match_status);
                println!();
                
                println!("6️⃣  DETERMINISM CHECK");
                // Encode again and verify identical output
                let payload2 = Payload::new(audit_log.as_bytes().to_vec());
                if let EncodeResult::Encoded(artifact2) = vectra_encode(payload2) {
                    let bytes1 = artifact.to_bytes();
                    let bytes2 = artifact2.to_bytes();
                    if bytes1 == bytes2 {
                        println!("   Status: ✅ DETERMINISTIC — Identical artifacts");
                    } else {
                        println!("   Status: ❌ NON-DETERMINISTIC — Artifacts differ");
                    }
                }
            }
            Err(e) => {
                println!("4️⃣  DECOMPRESSION FAILED: {:?}", e);
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("                    DEMO COMPLETE");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("KEY TAKEAWAYS:");
    println!("  • Audit logs compressed deterministically");
    println!("  • Original data reconstructed exactly");
    println!("  • Integrity hash verifiable");
    println!("  • Suitable for compliance archives");
}

/// Generate simulated audit log entries.
fn generate_audit_log(entries: usize) -> String {
    let mut log = String::new();
    let base_timestamp = 1702900000u64;
    
    for i in 0..entries {
        let timestamp = base_timestamp + (i as u64 * 60);
        let user_id = format!("user_{:04}", i % 50);
        let action = match i % 5 {
            0 => "LOGIN",
            1 => "VIEW_RECORD",
            2 => "UPDATE_RECORD",
            3 => "EXPORT_DATA",
            _ => "LOGOUT",
        };
        let resource = format!("/api/v1/records/{}", 1000 + (i % 100));
        let status = if i % 20 == 0 { "DENIED" } else { "ALLOWED" };
        
        log.push_str(&format!(
            "{}|{}|{}|{}|{}|session_{:08x}\n",
            timestamp, user_id, action, resource, status, i * 7919
        ));
    }
    
    log
}

/// Compute SHA-256 hash as hex string.
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}
