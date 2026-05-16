//! VECTRA Command-Line Interface
//!
//! Usage:
//!     vectra encode <input_file> <output_file>
//!     vectra decode <input_file> <output_file>

use std::env;
use std::fs;
use std::process;

use vectra::{vectra_decode, vectra_encode, EncodeResult, Payload};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 4 {
        print_usage();
        process::exit(1);
    }
    
    let command = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];
    
    match command.as_str() {
        "encode" => {
            if let Err(e) = encode_file(input_path, output_path) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        "decode" => {
            if let Err(e) = decode_file(input_path, output_path) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("VECTRA - Deterministic Lossless Compression");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  vectra encode <input_file> <output_file>");
    eprintln!("  vectra decode <input_file> <output_file>");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  vectra encode data.bin data.vectra");
    eprintln!("  vectra decode data.vectra data.bin");
}

/// Encode a file and write the artifact to output.
fn encode_file(input_path: &str, output_path: &str) -> Result<(), String> {
    // Magic bytes for file format identification
    const MAGIC_ARTIFACT: &[u8] = b"VCTR";
    const MAGIC_PASSTHROUGH: &[u8] = b"PASS";
    
    // Read input file
    let data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    let input_size = data.len();
    println!("Input: {} bytes", input_size);
    
    // Create payload and encode
    let payload = Payload::new(data);
    
    match vectra_encode(payload) {
        EncodeResult::Encoded(artifact) => {
            // Serialize artifact to bytes with magic header
            let artifact_bytes = artifact.to_bytes();
            let mut output_bytes = Vec::with_capacity(4 + artifact_bytes.len());
            output_bytes.extend_from_slice(MAGIC_ARTIFACT);
            output_bytes.extend_from_slice(&artifact_bytes);
            
            let output_size = output_bytes.len();
            
            // Write to output file
            fs::write(output_path, &output_bytes)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
            
            let ratio = if output_size > 0 {
                input_size as f64 / output_size as f64
            } else {
                1.0
            };
            
            println!("Output: {} bytes", output_size);
            println!("Ratio: {:.2}x", ratio);
            println!("Status: Encoded successfully");
            Ok(())
        }
        EncodeResult::PassThrough(original) => {
            // Encoding not beneficial, write original with marker
            let mut output_bytes = Vec::with_capacity(4 + original.len());
            output_bytes.extend_from_slice(MAGIC_PASSTHROUGH);
            output_bytes.extend_from_slice(original.as_bytes());
            
            fs::write(output_path, &output_bytes)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
            
            println!("Output: {} bytes (pass-through)", output_bytes.len());
            println!("Status: Data passed through (encoding not beneficial)");
            Ok(())
        }
    }
}

/// Decode an artifact file and write the original payload to output.
fn decode_file(input_path: &str, output_path: &str) -> Result<(), String> {
    // Magic bytes for file format identification
    const MAGIC_ARTIFACT: &[u8] = b"VCTR";
    const MAGIC_PASSTHROUGH: &[u8] = b"PASS";
    
    // Read input file
    let data = fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    if data.len() < 4 {
        return Err("Input file too small (missing magic header)".to_string());
    }
    
    println!("Input: {} bytes", data.len());
    
    let magic = &data[0..4];
    let content = &data[4..];
    
    if magic == MAGIC_PASSTHROUGH {
        // Pass-through: just write content
        fs::write(output_path, content)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        
        println!("Output: {} bytes", content.len());
        println!("Status: Pass-through decoded");
        return Ok(());
    }
    
    if magic != MAGIC_ARTIFACT {
        return Err(format!(
            "Invalid file format. Expected VCTR or PASS magic, got {:?}",
            std::str::from_utf8(magic).unwrap_or("invalid")
        ));
    }
    
    // Normal decode: parse artifact and decode
    let artifact = vectra::Artifact::from_bytes(content)
        .map_err(|e| format!("Failed to parse artifact: {:?}", e))?;
    
    let payload = vectra_decode(&artifact)
        .map_err(|e| format!("Decode failed: {:?}", e))?;
    
    // Write to output file
    fs::write(output_path, payload.as_bytes())
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    
    println!("Output: {} bytes", payload.len());
    println!("Status: Decoded successfully");
    println!("Integrity: Verified ✓");
    
    Ok(())
}
