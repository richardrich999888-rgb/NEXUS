//! NEXUS Execution CLI (`nexus-exec`)
//!
//! A standalone CLI for executing PCUs in the NEXUS sandboxed runtime.

use anyhow::{Context, Result};
use nexus_executor::{
    ExecutionContext, ExecutionLimits, NodeId, NoopHost,
};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
        
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("NEXUS Executor v{}", nexus_executor::VERSION);

    // 1. Collect args
    let mut args = std::env::args().skip(1);
    let wasm_path = args.next().context("Usage: nexus-exec <wasm_file> [input_files...]")?;
    let input_paths: Vec<PathBuf> = args.map(PathBuf::from).collect();

    // 2. Load WASM
    info!("Loading WASM from {}", wasm_path);
    let wasm_bytes = std::fs::read(&wasm_path)
        .with_context(|| format!("Failed to read WASM file: {}", wasm_path))?;

    // 3. Load Inputs
    let mut inputs = Vec::new();
    for path in input_paths {
        info!("Loading input from {:?}", path);
        let data = std::fs::read(&path)
            .with_context(|| format!("Failed to read input file: {:?}", path))?;
        let hash = nexus_pcu::ContentHash::compute(&data);
        inputs.push((hash, data));
    }

    // 4. Setup Executor (production build: guard required; no execution without passing guard)
    let node_id = NodeId::local();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let host = Arc::new(NoopHost);
    let executor = nexus_executor::ExecutorBuilder::production(node_id, signing_key, host)
        .cache_capacity(1000)
        .build()?;

    // 5. Setup Resource Limits from Environment
    let mut limits = ExecutionLimits::default();
    if let Ok(fuel) = std::env::var("NEXUS_MAX_FUEL") {
        limits.max_fuel = fuel.parse().context("Invalid NEXUS_MAX_FUEL")?;
    }
    if let Ok(timeout_secs) = std::env::var("NEXUS_TIMEOUT") {
        limits.max_time = std::time::Duration::from_secs(timeout_secs.parse().context("Invalid NEXUS_TIMEOUT")?);
    }
    if let Ok(memory_mb) = std::env::var("NEXUS_MAX_MEMORY_MB") {
        let mb: usize = memory_mb.parse().context("Invalid NEXUS_MAX_MEMORY_MB")?;
        limits.max_memory = mb * 1024 * 1024;
    }

    // 6. Build PCU
    // Create PCU directly using nexus-pcu types
    let code = nexus_pcu::WasmModule::new(wasm_bytes);
    let pcu_inputs: Vec<nexus_pcu::ContentHash> = inputs.iter().map(|(h, _)| *h).collect();
    let identity = nexus_pcu::IdentityContext::anonymous();
    let pcu = nexus_pcu::PCU::new(code, pcu_inputs, vec![], identity);

    let context = ExecutionContext::new(
        inputs,
        nexus_pcu::IdentityContext::anonymous(),
        limits,
    );

    // 7. Execute
    info!("Executing PCU...");
    let start = std::time::Instant::now();
    let response = executor.execute(&pcu, context).await?;
    let duration = start.elapsed();

    // 8. Output Result
    info!("Execution finished in {:?}", duration);
    info!("Fuel Consumed: {}", response.result.fuel_consumed);
    info!("Peak Memory: {} bytes", response.result.peak_memory);
    info!("Output Hash: {}", response.result.output_hash);
    info!("Cached: {}", response.cached);

    if !response.result.output.is_empty() {
        info!("Output ({} bytes):", response.result.output.len());
        // In a real CLI, we might write to stdout or a file.
        // For now, we'll hex dump if small or just report size.
        if response.result.output.len() < 1024 {
            let hex_out = hex::encode(&response.result.output);
            info!("Hex: {}", hex_out);
        }
    }

    info!("Generating Proof...");
    info!("Proof Verified: {}", response.proof.verify().is_ok());

    Ok(())
}
