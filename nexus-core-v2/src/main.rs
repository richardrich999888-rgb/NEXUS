mod core;
mod errors;
mod executor;
mod hash;
mod log;
mod merge;
mod op;
mod replay;
mod storage;
mod sync;

use clap::{Parser, Subcommand};
use core::NexusCore;
use storage::Storage;
use std::fs;

#[derive(Parser)]
#[command(name = "nexus-core")]
#[command(about = "NEXUS Core - Deterministic State Machine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Exec {
        #[arg(short, long)]
        wasm: String,
        #[arg(short, long)]
        input: String,
    },
    Replay,
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let storage = Storage::open("nexus.log").unwrap();
            let core = NexusCore::new().with_storage(storage).unwrap();
            println!("NEXUS Core initialized");
            println!("Log entries: {}", core.log_len());
        }
        
        Commands::Exec { wasm, input } => {
            let storage = Storage::open("nexus.log").unwrap();
            let mut core = NexusCore::new().with_storage(storage).unwrap();
            
            let wasm_bytes = fs::read(&wasm).expect("read wasm failed");
            let input_bytes = fs::read(&input).expect("read input failed");
            
            let hash = core.register_function(wasm_bytes);
            let id = core.execute(hash, input_bytes, vec![]).expect("execute failed");
            
            println!("Executed: {}", id);
        }
        
        Commands::Replay => {
            let storage = Storage::open("nexus.log").unwrap();
            let mut core = NexusCore::new().with_storage(storage).unwrap();
            
            core.replay().expect("replay failed");
            println!("Replay successful: {} entries", core.log_len());
        }
        
        Commands::Status => {
            let storage = Storage::open("nexus.log").unwrap();
            let core = NexusCore::new().with_storage(storage).unwrap();
            
            println!("NEXUS Core v0.1");
            println!("Log entries: {}", core.log_len());
        }
    }
}
