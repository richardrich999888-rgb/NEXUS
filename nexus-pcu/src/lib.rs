// NEXUS PCU: Portable Computation Units & Universal State Objects
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
// Patent Pending: IN202501XXXXX
// Inventor: Katta Naga Sri Ganesh
//
// This module implements the category-creating primitives for NEXUS:
// - PCU: Computation where code moves to data, identity is intrinsic
// - USO: Universal state primitive replacing databases, caches, queues

use ed25519_dalek::VerifyingKey;

pub mod pcu;
pub mod identity;
pub mod proof;
pub mod uso;
pub mod routing;
pub mod pqc;
pub mod content_hash;
pub mod invariants;
pub mod crypto;

pub use content_hash::{ContentHash, ContentHasher};
pub use pcu::{PCU, WasmModule, ExecutionConstraints};
pub use identity::{IdentityContext, PrincipalId, Capability, CapabilitySet, DelegationChain, DelegationLink};
pub use proof::{ExecutionProof, NodeAttestation};
pub use uso::{USO, SyncPolicy, CausalHistory, AccessPolicy, SchemaRef, Region};
pub use routing::DataLocator;
pub use pqc::{HybridSignature, HybridKeyPair, PublicKeyBundle, PqcError, PqcResult};

/// Unique identifier for a node in the NEXUS mesh (public key bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub [u8; 32]);

impl NodeId {
    pub fn new(id: [u8; 32]) -> Self {
        NodeId(id)
    }

    /// Create from verifying key
    pub fn from_verifying_key(key: &VerifyingKey) -> Self {
        NodeId(key.to_bytes())
    }

    /// Local node (all zeros)
    pub fn local() -> Self {
        NodeId([0u8; 32])
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hex representation
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Short hex for display
    pub fn short_hex(&self) -> String {
        format!("{}..{}", &self.to_hex()[..8], &self.to_hex()[56..])
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "node:{}", hex::encode(&self.0[..4]))
    }
}

/// Timestamp in milliseconds since Unix epoch
pub type Timestamp = u64;

/// Get current timestamp
pub fn now() -> Timestamp {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
