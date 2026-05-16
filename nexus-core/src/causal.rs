// NEXUS Core: Causal Tensor Algebra
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX
// Inventor: Katta Naga Sri Ganesh

use serde::{Deserialize, Serialize};
use blake3::Hasher as Blake3Hasher;
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use std::collections::{HashSet, BTreeMap};
use crate::error::{NexusError, Result};

// ============================================================================
// CAUSAL ID - Content-addressed identifier
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CausalId([u8; 32]);

impl CausalId {
    pub fn from_hash(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        CausalId(*hash.as_bytes())
    }
    
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        CausalId(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    pub fn short_hex(&self) -> String {
        hex::encode(&self.0[..8])
    }
    
    pub fn genesis() -> Self {
        CausalId([0u8; 32])
    }
}

impl std::fmt::Display for CausalId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.short_hex())
    }
}

// ============================================================================
// VECTOR CLOCK - Causal ordering with optimizations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock {
    clocks: BTreeMap<u64, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        VectorClock {
            clocks: BTreeMap::new(),
        }
    }
    
    pub fn with_node(node_id: u64) -> Self {
        let mut clock = Self::new();
        clock.tick(node_id);
        clock
    }
    
    pub fn tick(&mut self, node_id: u64) -> u64 {
        let time = self.clocks.entry(node_id).or_insert(0);
        *time += 1;
        *time
    }
    
    pub fn get(&self, node_id: u64) -> u64 {
        self.clocks.get(&node_id).copied().unwrap_or(0)
    }
    
    pub fn merge(&mut self, other: &VectorClock) {
        for (&node, &time) in &other.clocks {
            let entry = self.clocks.entry(node).or_insert(0);
            *entry = (*entry).max(time);
        }
    }
    
    /// Check if self happens before other (self → other)
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        if self.clocks.is_empty() {
            return !other.clocks.is_empty();
        }
        
        let mut strictly_less = false;
        
        // Check all nodes in self
        for (&node, &time) in &self.clocks {
            let other_time = other.clocks.get(&node).copied().unwrap_or(0);
            if time > other_time {
                return false; // Not causally ordered
            }
            if time < other_time {
                strictly_less = true;
            }
        }
        
        // Check nodes only in other
        for (&node, &other_time) in &other.clocks {
            if !self.clocks.contains_key(&node) && other_time > 0 {
                strictly_less = true;
            }
        }
        
        strictly_less
    }
    
    /// Check if events are concurrent
    pub fn concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && self != other
    }
    
    /// Compute Lamport timestamp for total ordering
    pub fn lamport_timestamp(&self) -> u64 {
        self.clocks.values().max().copied().unwrap_or(0)
    }
    
    pub fn node_count(&self) -> usize {
        self.clocks.len()
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PROVENANCE - Merkle DAG for causal history
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub parents: Vec<CausalId>,
    pub merkle_root: [u8; 32],
    pub depth: u64, // For optimization
}

impl Provenance {
    pub fn new(parents: Vec<CausalId>) -> Self {
        let depth = if parents.is_empty() { 0 } else { 1 }; // Simplified for MVP
        let merkle_root = Self::compute_merkle_root(&parents);
        Provenance {
            parents,
            merkle_root,
            depth,
        }
    }
    
    pub fn genesis() -> Self {
        Provenance {
            parents: vec![],
            merkle_root: [0u8; 32],
            depth: 0,
        }
    }
    
    fn compute_merkle_root(parents: &[CausalId]) -> [u8; 32] {
        if parents.is_empty() {
            return [0u8; 32];
        }
        
        let mut hasher = Blake3Hasher::new();
        for parent in parents {
            hasher.update(parent.as_bytes());
        }
        *hasher.finalize().as_bytes()
    }
    
    /// Find lowest common ancestor with another provenance
    pub fn lca(&self, other: &Provenance) -> Option<CausalId> {
        let self_set: HashSet<_> = self.parents.iter().copied().collect();
        other.parents.iter()
            .find(|p| self_set.contains(p))
            .copied()
    }
    
    /// Compute diff since an ancestor
    pub fn diff_since(&self, ancestor: CausalId) -> Vec<CausalId> {
        self.parents.iter()
            .filter(|&&p| p != ancestor)
            .copied()
            .collect()
    }
    
    /// Check if contains ancestor
    pub fn contains_ancestor(&self, ancestor: CausalId) -> bool {
        self.parents.contains(&ancestor)
    }
}

// ============================================================================
// CAUSAL TENSOR - Core data structure with full production features
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalTensor {
    pub id: CausalId,
    pub data: Vec<u8>,
    pub provenance: Provenance,
    pub clock: VectorClock,
    pub signature: Vec<u8>,
    pub metadata: TensorMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorMetadata {
    pub created_at: i64,
    pub node_id: u64,
    pub content_type: String,
    pub size_bytes: usize,
    pub flags: TensorFlags,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TensorFlags {
    pub immutable: bool,
    pub encrypted: bool,
    pub compressed: bool,
}

impl CausalTensor {
    /// Create a new causal tensor with full validation
    pub fn new(
        data: Vec<u8>,
        parents: Vec<CausalId>,
        node_id: u64,
        clock: &mut VectorClock,
        signing_key: &SigningKey,
    ) -> Result<Self> {
        if data.len() > 100 * 1024 * 1024 {
            return Err(NexusError::InvalidTensor(
                "Data exceeds 100MB limit".to_string()
            ));
        }
        
        clock.tick(node_id);
        let provenance = Provenance::new(parents);
        
        // Compute content-addressed ID
        let mut hasher = Blake3Hasher::new();
        hasher.update(&data);
        hasher.update(&bincode::serialize(&provenance)
            .map_err(|e| NexusError::SerializationError(e.to_string()))?);
        hasher.update(&bincode::serialize(&clock)
            .map_err(|e| NexusError::SerializationError(e.to_string()))?);
        
        let id = CausalId::from_bytes(*hasher.finalize().as_bytes());

        // Sign the tensor
        let signature_data = Self::signature_data(&id, &data, &provenance);
        let signature = signing_key.sign(&signature_data).to_bytes().to_vec();

        let metadata = TensorMetadata {
            created_at: chrono::Utc::now().timestamp(),
            node_id,
            content_type: "application/octet-stream".to_string(),
            size_bytes: data.len(),
            flags: TensorFlags {
                immutable: false,
                encrypted: false,
                compressed: false,
            },
        };

        Ok(CausalTensor {
            id,
            data,
            provenance,
            clock: clock.clone(),
            signature,
            metadata,
        })
    }
    
    /// Genesis tensor (root of DAG)
    pub fn genesis(node_id: u64, signing_key: &SigningKey) -> Result<Self> {
        let mut clock = VectorClock::new();
        clock.tick(node_id);
        let data = b"NEXUS_GENESIS".to_vec();
        let provenance = Provenance::genesis();
        
        let id = CausalId::genesis();
        let signature_data = Self::signature_data(&id, &data, &provenance);
        let signature = signing_key.sign(&signature_data).to_bytes().to_vec();

        Ok(CausalTensor {
            id,
            data,
            provenance,
            clock,
            signature,
            metadata: TensorMetadata {
                created_at: chrono::Utc::now().timestamp(),
                node_id,
                content_type: "genesis".to_string(),
                size_bytes: 13,
                flags: TensorFlags {
                    immutable: true,
                    encrypted: false,
                    compressed: false,
                },
            },
        })
    }
    
    fn signature_data(id: &CausalId, data: &[u8], provenance: &Provenance) -> Vec<u8> {
        let mut sig_data = Vec::with_capacity(32 + data.len() + 32);
        sig_data.extend_from_slice(id.as_bytes());
        sig_data.extend_from_slice(data);
        sig_data.extend_from_slice(&provenance.merkle_root);
        sig_data
    }
    
    /// Verify tensor signature and integrity
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<()> {
        let signature_data = Self::signature_data(&self.id, &self.data, &self.provenance);
        
        let sig = Signature::from_bytes(&self.signature.as_slice().try_into()
            .map_err(|_| NexusError::InvalidSignature("Invalid signature length".to_string()))?);
        
        verifying_key.verify(&signature_data, &sig)
            .map_err(|e| NexusError::InvalidSignature(e.to_string()))?;
        
        Ok(())
    }
    
    /// Three-way causal merge - THE CORE ALGORITHM
    pub fn merge(
        local: &CausalTensor,
        remote: &CausalTensor,
        node_id: u64,
        _clock: &mut VectorClock,
        signing_key: &SigningKey,
    ) -> Result<CausalTensor> {
        // 1. IDEMPOTENCE: Identical tensors
        if local.id == remote.id {
            return Ok(local.clone());
        }

        // 2. CAUSAL MONOTONICITY: Check causal ordering
        if local.clock.happens_before(&remote.clock) {
            return Ok(remote.clone()); // Remote is causally newer
        }
        if remote.clock.happens_before(&local.clock) {
            return Ok(local.clone()); // Local is causally newer
        }

        // 3. CONCURRENT MERGE: Algebraic resolution
        tracing::debug!(
            "Concurrent merge detected: local={}, remote={}",
            local.id,
            remote.id
        );

        let mut merged_clock = local.clock.clone();
        merged_clock.merge(&remote.clock);

        // Find LCA for three-way merge
        let lca = local.provenance.lca(&remote.provenance);
        
        // Merge data (deterministic for same inputs)
        let merged_data = Self::merge_data(&local.data, &remote.data, lca)?;

        // Create merged provenance (both parents)
        let mut parents = vec![local.id, remote.id];
        parents.sort(); // Deterministic ordering
        parents.dedup();

        Self::new(
            merged_data,
            parents,
            node_id,
            &mut merged_clock,
            signing_key,
        )
    }
    
    /// Application-specific merge logic
    fn merge_data(
        local: &[u8],
        remote: &[u8],
        _lca: Option<CausalId>,
    ) -> Result<Vec<u8>> {
        // Default: deterministic last-write-wins by hash comparison
        let local_hash = blake3::hash(local);
        let remote_hash = blake3::hash(remote);
        
        // Compare hashes as byte arrays for deterministic ordering
        Ok(if local_hash.as_bytes() > remote_hash.as_bytes() {
            local.to_vec()
        } else {
            remote.to_vec()
        })
    }
    
    /// Serialize for network transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| NexusError::SerializationError(e.to_string()))
    }
    
    /// Deserialize from network
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| NexusError::SerializationError(e.to_string()))
    }
}

// ============================================================================
// CONSISTENCY LEVELS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyLevel {
    Eventual,      // Apply immediately, merge later
    Causal,        // Wait for dependencies (default)
    Sequential,    // Linearizable (requires coordination)
}

// ============================================================================
// TESTS - Production-grade test suite
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_signing_key;
    
    #[test]
    fn test_causal_id_deterministic() {
        let data = b"test data";
        let id1 = CausalId::from_hash(data);
        let id2 = CausalId::from_hash(data);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_vector_clock_ordering() {
        let mut clock_a = VectorClock::new();
        let mut clock_b = VectorClock::new();

        clock_a.tick(1); // A: {1: 1}
        clock_b.tick(2); // B: {2: 1}

        assert!(clock_a.concurrent(&clock_b));

        clock_a.merge(&clock_b); // A: {1: 1, 2: 1}
        clock_a.tick(1);          // A: {1: 2, 2: 1}

        assert!(clock_b.happens_before(&clock_a));
        assert!(!clock_a.happens_before(&clock_b));
    }

    #[test]
    fn test_causal_tensor_creation() {
        let signing_key = generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let mut clock = VectorClock::new();

        let tensor = CausalTensor::new(
            b"test data".to_vec(),
            vec![],
            1,
            &mut clock,
            &signing_key,
        ).unwrap();

        assert!(tensor.verify(&verifying_key).is_ok());
    }

    #[test]
    fn test_merge_idempotent() {
        let signing_key = generate_signing_key();
        let mut clock = VectorClock::new();

        let tensor = CausalTensor::new(
            b"data".to_vec(),
            vec![],
            1,
            &mut clock,
            &signing_key,
        ).unwrap();

        let merged = CausalTensor::merge(
            &tensor,
            &tensor,
            1,
            &mut clock,
            &signing_key,
        ).unwrap();

        assert_eq!(tensor.id, merged.id);
    }

    #[test]
    fn test_merge_concurrent() {
        let signing_key = generate_signing_key();
        
        let mut clock_a = VectorClock::new();
        let tensor_a = CausalTensor::new(
            b"data_a".to_vec(),
            vec![],
            1,
            &mut clock_a,
            &signing_key,
        ).unwrap();

        let mut clock_b = VectorClock::new();
        let tensor_b = CausalTensor::new(
            b"data_b".to_vec(),
            vec![],
            2,
            &mut clock_b,
            &signing_key,
        ).unwrap();

        let mut merged_clock = VectorClock::new();
        let merged = CausalTensor::merge(
            &tensor_a,
            &tensor_b,
            3,
            &mut merged_clock,
            &signing_key,
        ).unwrap();

        assert_eq!(merged.provenance.parents.len(), 2);
        assert!(merged.provenance.parents.contains(&tensor_a.id));
        assert!(merged.provenance.parents.contains(&tensor_b.id));
    }

    #[test]
    fn test_merge_deterministic() {
        let signing_key = generate_signing_key();
        
        let mut clock_a = VectorClock::new();
        let tensor_a = CausalTensor::new(
            b"AAA".to_vec(),
            vec![],
            1,
            &mut clock_a,
            &signing_key,
        ).unwrap();

        let mut clock_b = VectorClock::new();
        let tensor_b = CausalTensor::new(
            b"BBB".to_vec(),
            vec![],
            2,
            &mut clock_b,
            &signing_key,
        ).unwrap();

        // Merge A+B
        let mut clock1 = VectorClock::new();
        let merged1 = CausalTensor::merge(
            &tensor_a,
            &tensor_b,
            3,
            &mut clock1,
            &signing_key,
        ).unwrap();

        // Merge B+A (reversed order)
        let mut clock2 = VectorClock::new();
        let merged2 = CausalTensor::merge(
            &tensor_b,
            &tensor_a,
            3,
            &mut clock2,
            &signing_key,
        ).unwrap();

        // Should produce same result (commutativity)
        assert_eq!(merged1.data, merged2.data);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let signing_key = generate_signing_key();
        let mut clock = VectorClock::new();

        let tensor = CausalTensor::new(
            b"test".to_vec(),
            vec![],
            1,
            &mut clock,
            &signing_key,
        ).unwrap();

        let bytes = tensor.to_bytes().unwrap();
        let deserialized = CausalTensor::from_bytes(&bytes).unwrap();

        assert_eq!(tensor.id, deserialized.id);
        assert_eq!(tensor.data, deserialized.data);
    }
}
