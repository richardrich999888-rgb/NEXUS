// NEXUS Network: Sync Node
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use crate::transport::QuicTransport;
use crate::gossip::GossipProtocol;
use crate::sync::SyncProtocol;
use crate::message::CausalMessage;
use crate::error::NexusNetworkError;
use nexus_sync::NexusSyncEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// SyncNode ties together networking and sync logic
pub struct SyncNode {
    addr: SocketAddr,
    transport: Arc<QuicTransport>,
    gossip: Arc<GossipProtocol>,
    sync_proto: Arc<SyncProtocol>,
    engine: Arc<RwLock<NexusSyncEngine>>,
}

impl SyncNode {
    pub fn new(
        addr: SocketAddr,
        engine: NexusSyncEngine,
    ) -> Result<Self, NexusNetworkError> {
        // Use dev mode for now (self-signed cert)
        // In production, use QuicTransport::new_with_certs() with proper certificates
        let transport = Arc::new(QuicTransport::new_dev(addr, "nexus-node")?);
        let gossip = Arc::new(GossipProtocol::new(transport.clone()));
        let sync_proto = Arc::new(SyncProtocol::new(transport.clone()));
        let engine = Arc::new(RwLock::new(engine));
        
        Ok(Self {
            addr,
            transport,
            gossip,
            sync_proto,
            engine,
        })
    }

    /// Add a peer to this node
    pub fn add_peer(&self, addr: SocketAddr) {
        self.gossip.add_peer(addr);
    }

    /// Start the node listener
    pub async fn run(&self) -> Result<(), NexusNetworkError> {
        info!("NEXUS SyncNode listening on {}", self.addr);
        
        let engine = self.engine.clone();
        let sync_proto = self.sync_proto.clone();
        
        self.transport.listen(move |msg| {
            let engine = engine.clone();
            let _sync_proto = sync_proto.clone();
            
            async move {
                match msg {
                    CausalMessage::SyncRequest(_remote_vv) => {
                        info!("Received SyncRequest from remote");
                        // For demo: respond with delta or ignore
                    }
                    CausalMessage::SyncResponse(delta) => {
                        info!("Received SyncResponse with {} operations", delta.operations.len());
                        let mut engine_lock = engine.write().await;
                        if let Err(e) = engine_lock.merge_remote(delta.operations) {
                            error!("Failed to merge remote ops: {}", e);
                        }
                    }
                    CausalMessage::PCU(pcu) => {
                        info!("Received PCU: {}", pcu.id);
                        // Process PCU
                    }
                    CausalMessage::USO(uso) => {
                        info!("Received USO: {}", uso.id);
                        let mut engine_lock = engine.write().await;
                        engine_lock.register_uso(uso);
                    }
                    _ => {
                        warn!("Received unhandled message type");
                    }
                }
            }
        }).await
    }

    /// Broadcast a local update
    pub async fn broadcast_update(&self, msg: CausalMessage) -> Result<(), NexusNetworkError> {
        self.gossip.broadcast(msg).await
    }
}
