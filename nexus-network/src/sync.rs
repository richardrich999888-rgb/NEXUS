// NEXUS Network: Sync Protocol
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use crate::transport::QuicTransport;
use crate::message::CausalMessage;
use crate::error::NexusNetworkError;
use nexus_core::causal::CausalId;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

pub struct SyncProtocol {
    transport: Arc<QuicTransport>,
}

impl SyncProtocol {
    pub fn new(transport: Arc<QuicTransport>) -> Self {
        Self { transport }
    }

    /// Request sync from a peer using local VersionVector
    pub async fn request_sync(
        &self,
        peer: SocketAddr,
        local_vv: nexus_sync::VersionVector,
    ) -> Result<(), NexusNetworkError> {
        info!("Requesting sync from {} with version {:?}", peer, local_vv);
        
        let conn = self.transport.connect(peer, None).await?;
        let msg = CausalMessage::SyncRequest(local_vv);
        
        self.transport.send(&conn, &msg).await?;
        Ok(())
    }

    /// Send sync update (delta) to a peer
    pub async fn push_sync_delta(
        &self,
        peer: SocketAddr,
        delta: nexus_sync::SyncDelta,
    ) -> Result<(), NexusNetworkError> {
        info!("Pushing sync delta to {}", peer);
        
        let conn = self.transport.connect(peer, None).await?;
        let msg = CausalMessage::SyncResponse(delta);
        
        self.transport.send(&conn, &msg).await?;
        Ok(())
    }

    /// Request missing tensors from a specific node
    pub async fn pull_tensors(
        &self, 
        peer: SocketAddr, 
        ids: Vec<CausalId>
    ) -> Result<(), NexusNetworkError> {
        info!("Pulling {} tensors from {}", ids.len(), peer);
        
        let conn = self.transport.connect(peer, None).await?;
        let msg = CausalMessage::PullRequest(ids);
        
        self.transport.send(&conn, &msg).await?;
        Ok(())
    }
}
