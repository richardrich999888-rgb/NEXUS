//! Offline ETK verifier CLI.
//!
//! Reads proof, event stream, policy snapshot; verifies without trusting runtime.
//! No API calls. Output: VALID or INVALID.

use ed25519_dalek::VerifyingKey;
use nexus_etk::schema::{
    ExecutionEventV1, ExecutionProofV1, EVENT_CANONICAL_LEN, Hash256, PROOF_CANONICAL_LEN,
};
use nexus_etk::verifier::{verify, Verdict};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: etk_verifier <proof_file> <events_file> <policy_file> <verifier_pubkey_file> [--tolerance-ms N] [--verbose]");
        eprintln!("  proof_file: raw canonical bytes of ExecutionProofV1 ({} bytes)", PROOF_CANONICAL_LEN);
        eprintln!("  events_file: concatenated canonical ExecutionEventV1 (each {} bytes)", EVENT_CANONICAL_LEN);
        eprintln!("  policy_file: raw policy snapshot bytes (hashed and checked against proof.policy_ref)");
        eprintln!("  verifier_pubkey_file: raw 32-byte Ed25519 verifying key");
        process::exit(2);
    }

    let proof_path = &args[1];
    let events_path = &args[2];
    let policy_path = &args[3];
    let pubkey_path = &args[4];

    let mut tolerance_ms: u64 = 86400 * 1000; // 1 day default
    let mut verbose = false;
    let mut i = 5;
    while i < args.len() {
        if args[i] == "--verbose" || args[i] == "-v" {
            verbose = true;
        } else if args[i] == "--tolerance-ms" && i + 1 < args.len() {
            tolerance_ms = args[i + 1].parse().unwrap_or(tolerance_ms);
            i += 1;
        }
        i += 1;
    }

    let proof_bytes = fs::read(proof_path).unwrap_or_else(|e| {
        eprintln!("Failed to read proof file: {}", e);
        process::exit(1);
    });
    let proof = ExecutionProofV1::from_canonical_bytes(&proof_bytes).unwrap_or_else(|e| {
        eprintln!("Invalid proof format: {}", e);
        process::exit(1);
    });

    let events_bytes = fs::read(events_path).unwrap_or_else(|e| {
        eprintln!("Failed to read events file: {}", e);
        process::exit(1);
    });
    if events_bytes.len() % EVENT_CANONICAL_LEN != 0 {
        eprintln!(
            "Events file length {} is not a multiple of event size {}",
            events_bytes.len(),
            EVENT_CANONICAL_LEN
        );
        process::exit(1);
    }
    let mut events = Vec::new();
    for chunk in events_bytes.chunks(EVENT_CANONICAL_LEN) {
        let ev = ExecutionEventV1::from_canonical_bytes(chunk).unwrap_or_else(|e| {
            eprintln!("Invalid event in stream: {}", e);
            process::exit(1);
        });
        events.push(ev);
    }

    let policy_bytes = fs::read(policy_path).unwrap_or_else(|e| {
        eprintln!("Failed to read policy file: {}", e);
        process::exit(1);
    });
    // Verifier hashes snapshot and checks hash == proof.policy_ref.
    let policy_resolver = move |_pr: Hash256| Some(policy_bytes.clone());

    let pubkey_bytes: [u8; 32] = fs::read(pubkey_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to read verifier pubkey: {}", e);
            process::exit(1);
        })
        .try_into()
        .unwrap_or_else(|_| {
            eprintln!("Verifier pubkey must be exactly 32 bytes");
            process::exit(1);
        });
    let verifier_pubkey = VerifyingKey::from_bytes(&pubkey_bytes).unwrap_or_else(|e| {
        eprintln!("Invalid Ed25519 verifying key: {}", e);
        process::exit(1);
    });

    match verify(
        &proof,
        &events,
        &policy_resolver,
        &verifier_pubkey,
        tolerance_ms,
    ) {
        Ok(Verdict::Valid) => {
            println!("VALID");
            process::exit(0);
        }
        Ok(Verdict::Invalid) => {
            println!("INVALID");
            process::exit(1);
        }
        Err(e) => {
            if verbose {
                eprintln!("INVALID: {} (code: {:?})", e, e.code());
            } else {
                eprintln!("INVALID: {}", e);
            }
            println!("INVALID");
            process::exit(1);
        }
    }
}
