//! ETK CLI — regulator/auditor tooling.
//!
//! Usage: etk verify <proof.bin> <events.bin> <policy.bin> <pubkey.bin> [--tolerance-ms N]

use ed25519_dalek::VerifyingKey;
use etk_core::{
    decode_event, decode_proof, verify, Verdict,
    EVENT_CANONICAL_LEN, PROOF_CANONICAL_LEN,
};
use etk_types::Hash256;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(2);
    }
    match args[1].as_str() {
        "verify" => run_verify(&args[2..]),
        "version" => {
            let v = option_env!("ETK_CLI_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
            println!("etk {}", v);
            process::exit(0);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(2);
        }
    }
}

fn print_usage() {
    eprintln!("ETK — Execution Truth Kernel CLI");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  etk verify <proof.bin> <events.bin> <policy.bin> <pubkey.bin> [--tolerance-ms N] [--verbose]");
    eprintln!("  etk version");
    eprintln!();
    eprintln!("verify: Offline verification. Proof {} bytes, each event {} bytes.", PROOF_CANONICAL_LEN, EVENT_CANONICAL_LEN);
}

fn run_verify(args: &[String]) -> ! {
    if args.len() < 4 {
        eprintln!("etk verify requires: proof.bin events.bin policy.bin pubkey.bin");
        process::exit(2);
    }
    let proof_path = &args[0];
    let events_path = &args[1];
    let policy_path = &args[2];
    let pubkey_path = &args[3];

    let mut tolerance_ms: u64 = 86400 * 1000; // 1 day default
    let mut verbose = false;
    let mut i = 4;
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
        eprintln!("Failed to read proof: {}", e);
        process::exit(1);
    });
    let proof = decode_proof(&proof_bytes).unwrap_or_else(|e| {
        eprintln!("Invalid proof format: {}", e);
        process::exit(1);
    });

    let events_bytes = fs::read(events_path).unwrap_or_else(|e| {
        eprintln!("Failed to read events: {}", e);
        process::exit(1);
    });
    if events_bytes.len() % EVENT_CANONICAL_LEN != 0 {
        eprintln!(
            "Events file length {} is not a multiple of {}",
            events_bytes.len(),
            EVENT_CANONICAL_LEN
        );
        process::exit(1);
    }
    let mut events = Vec::new();
    for chunk in events_bytes.chunks(EVENT_CANONICAL_LEN) {
        let ev = decode_event(chunk).unwrap_or_else(|e| {
            eprintln!("Invalid event in stream: {}", e);
            process::exit(1);
        });
        events.push(ev);
    }

    let policy_bytes = fs::read(policy_path).unwrap_or_else(|e| {
        eprintln!("Failed to read policy: {}", e);
        process::exit(1);
    });
    let policy_resolver = move |_pr: Hash256| Some(policy_bytes.clone());

    let pubkey_bytes: [u8; 32] = fs::read(pubkey_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to read pubkey: {}", e);
            process::exit(1);
        })
        .try_into()
        .unwrap_or_else(|_| {
            eprintln!("Pubkey must be exactly 32 bytes");
            process::exit(1);
        });
    let verifier_pubkey = VerifyingKey::from_bytes(&pubkey_bytes).unwrap_or_else(|e| {
        eprintln!("Invalid Ed25519 pubkey: {}", e);
        process::exit(1);
    });

    let result = verify(
        &proof,
        &events,
        &policy_resolver,
        &verifier_pubkey,
        tolerance_ms,
    );
    match result {
        Ok(Verdict::Valid) => {
            println!("VALID");
            process::exit(0);
        }
        Ok(Verdict::Invalid) | Err(_) => {
            if verbose {
                if let Err(e) = result {
                    eprintln!("INVALID: {}", e);
                }
            }
            println!("INVALID");
            process::exit(1);
        }
    }
}
