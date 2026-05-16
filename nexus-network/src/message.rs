// NEXUS Network: Message Types
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use nexus_core::causal::{CausalTensor, CausalId};
use serde::{Deserialize, Serialize};

/// CausalMessage - Self-ordering transport envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CausalMessage {
    /// Broadcast a new tensor
    /// Broadcast a new tensor (Legacy)
    Tensor(CausalTensor),
    
    /// Portable Computation Unit transport
    PCU(nexus_pcu::pcu::PCU),
    
    /// Universal State Object transport
    USO(nexus_pcu::uso::USO),

    /// Synchronization request (VersionVector-based)
    SyncRequest(nexus_sync::VersionVector),

    /// Synchronization response (SyncDelta)
    SyncResponse(nexus_sync::SyncDelta),
    
    /// Request missing tensors by ID (Sync reconciliation)
    PullRequest(Vec<CausalId>),
    
    /// Response to PullRequest
    PullResponse(Vec<CausalTensor>),
    
    /// Gossip node availability/peers
    KeepAlive {
        node_id: u64,
        timestamp: i64,
    },
}

impl CausalMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}
