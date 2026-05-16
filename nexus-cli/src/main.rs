// NEXUS CLI - Unified Substrate Management
// Copyright (c) 2025 SYNTRIASS Labs Private Limited

use anyhow::Result;
use clap::{Parser, Subcommand};
use nexus_pcu::pcu::{PCU, WasmModule};
use nexus_pcu::{USO, PrincipalId};
use nexus_sync::NexusSyncEngine;
use nexus_network::node::SyncNode;
use nexus_sync::ConflictPolicy;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "nexus")]
#[command(about = "Network-Embedded eXecution Unified Substrate CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new node
    Init {
        #[arg(short, long, default_value = "node1")]
        node_id: String,
    },
    /// Manage and run nodes
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },
    /// Manage Portable Computation Units
    Pcu {
        #[command(subcommand)]
        action: PcuAction,
    },
}

#[derive(Subcommand)]
enum NodeAction {
    /// Start a long-running nexus node
    Run {
        #[arg(short, long, default_value = "node1")]
        node_id: String,
        #[arg(short, long, default_value = "127.0.0.1:8000")]
        bind: SocketAddr,
        #[arg(short, long)]
        peers: Vec<SocketAddr>,
    },
    /// Add a peer to a running node (via CLI/socket in real implementation, but for demo we use args)
    Peer {
        #[arg(short, long)]
        add: SocketAddr,
    },
}

#[derive(Subcommand)]
enum PcuAction {
    /// Submit a PCU for execution in the cluster
    Submit {
        #[arg(short, long)]
        code: PathBuf,
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long)]
        target: SocketAddr,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { node_id } => {
            println!("Initializing NEXUS node: {}", node_id);
            println!("Generating keys and genesis USO...");
        }
        Commands::Node { action } => match action {
            NodeAction::Run { node_id, bind, peers } => {
                info!("Starting NEXUS node '{}' on {}", node_id, bind);
                let engine = NexusSyncEngine::new(node_id, ConflictPolicy::LastWriterWins);
                let node = SyncNode::new(bind, engine)?;
                
                for peer in peers {
                    node.add_peer(peer);
                }
                
                // For demo purposes, we log the node addr
                println!("Node started. PID: {}", std::process::id());
                
                node.run().await?;
            }
            NodeAction::Peer { add } => {
                println!("Peering command requested for: {}", add);
                // Implementation would require a control socket to the running node
            }
        },
        Commands::Pcu { action } => match action {
            PcuAction::Submit { code, input: _, target } => {
                info!("Submitting PCU from {:?} to {}", code, target);
                // 1. Load WASM code
                let bytecode = if code.exists() {
                    std::fs::read(code)?
                } else {
                    vec![0x00, 0x61, 0x73, 0x6d]
                };
                let wasm = WasmModule::new(bytecode);
                
                // 2. Create PCU
                let principal = PrincipalId::generate();
                let identity = nexus_pcu::identity::IdentityContext::new(
                    principal,
                    nexus_pcu::identity::CapabilitySet::default(),
                );
                
                let pcu = PCU::new(wasm, vec![], vec![], identity);
                let pcu_id = pcu.id;
                
                // 3. Connect and send
                let addr = "127.0.0.1:0".parse()?;
                let transport = nexus_network::QuicTransport::new_dev(addr, "nexus-cli")?;
                let conn = transport.connect(target, None).await?;
                let msg = nexus_network::CausalMessage::PCU(pcu);
                transport.send(&conn, &msg).await?;
                
                info!("Successfully submitted PCU: {}", pcu_id);
                println!("PCU Hash: {}", pcu_id);
            }
        },
    }

    Ok(())
}
