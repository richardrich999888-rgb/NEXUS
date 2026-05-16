// NEXUS Network: Gossip Protocol
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use crate::transport::QuicTransport;
use crate::message::CausalMessage;
use crate::error::NexusNetworkError;
use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn};

pub struct GossipProtocol {
    transport: Arc<QuicTransport>,
    peers: Arc<RwLock<Vec<SocketAddr>>>,
}

impl GossipProtocol {
    pub fn new(transport: Arc<QuicTransport>) -> Self {
        Self {
            transport,
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a peer to the gossip network
    pub fn add_peer(&self, addr: SocketAddr) {
        let mut peers = self.peers.write();
        if !peers.contains(&addr) {
            peers.push(addr);
            info!("Added peer: {}", addr);
        }
    }

    /// Broadcast a message to all known peers (fanout=all for small networks)
    pub async fn broadcast(&self, msg: CausalMessage) -> Result<(), NexusNetworkError> {
        let peers = self.peers.read().clone();
        
        for peer in peers {
            match self.transport.connect(peer, None).await {
                Ok(conn) => {
                    if let Err(e) = self.transport.send(&conn, &msg).await {
                        warn!("Failed to send gossip to {}: {}", peer, e);
                    }
                }
                Err(e) => {
                    warn!("Failed to connect to peer {}: {}", peer, e);
                }
            }
        }
        
        Ok(())
    }
}
