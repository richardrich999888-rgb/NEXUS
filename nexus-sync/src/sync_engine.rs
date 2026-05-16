// NEXUS Sync Engine - Wraps CAUSALUX CausalDAG for NEXUS
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh

use causalux_v2::{CausalDAG, CausalOp, ConflictPolicy, VersionVector};
use nexus_pcu::{ContentHash, USO, PrincipalId};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NEXUS Sync Engine - manages synchronized state across nodes
/// 
/// Wraps CAUSALUX CausalDAG with NEXUS-specific adaptations:
/// - Uses ContentHash for operation IDs
/// - Integrates with USO for state management
/// - Provides PCU-aware sync protocols
pub struct NexusSyncEngine {
    /// Underlying CAUSALUX DAG
    dag: CausalDAG,
    
    /// Node ID
    node_id: String,
    
    /// Keypair for signing operations
    keypair: Option<SigningKey>,
    
    /// USO registry - tracked USOs by their ID
    uso_registry: HashMap<ContentHash, USO>,
}

impl NexusSyncEngine {
    /// Create new sync engine
    pub fn new(node_id: impl Into<String>, conflict_policy: ConflictPolicy) -> Self {
        let node_id = node_id.into();
        let dag = CausalDAG::new(
            node_id.clone(),
            10_000, // Snapshot every 10K ops
            conflict_policy,
        );
        
        NexusSyncEngine {
            dag,
            node_id,
            keypair: None,
            uso_registry: HashMap::new(),
        }
    }

    /// Set keypair for signing operations
    pub fn with_keypair(mut self, keypair: SigningKey) -> Self {
        self.keypair = Some(keypair);
        self
    }

    /// Get current version vector
    pub fn version_vector(&self) -> &VersionVector {
        self.dag.get_version_vector()
    }

    /// Get operation count
    pub fn operation_count(&self) -> usize {
        self.dag.operation_count()
    }

    /// Register a USO for sync tracking
    pub fn register_uso(&mut self, uso: USO) {
        self.uso_registry.insert(uso.id, uso);
    }

    /// Get a tracked USO
    pub fn get_uso(&self, id: &ContentHash) -> Option<&USO> {
        self.uso_registry.get(id)
    }

    /// Update a USO and create sync operation
    pub fn update_uso(
        &mut self,
        uso_id: &ContentHash,
        new_data: Vec<u8>,
        principal: PrincipalId,
    ) -> Result<ContentHash, SyncError> {
        let uso = self.uso_registry.get_mut(uso_id)
            .ok_or(SyncError::UsoNotFound)?;
        
        uso.update(new_data.clone(), principal, &self.node_id);
        
        // Create causal operation for sync
        if let Some(ref keypair) = self.keypair {
            let version = uso.history.vector_clock.get(&self.node_id).copied().unwrap_or(0);
            let mut vv = VersionVector::new();
            for _ in 0..version {
                vv.increment(&self.node_id);
            }
            
            let op = CausalOp::new(
                "uso_update".to_string(),
                serde_json::json!({
                    "uso_id": hex::encode(uso_id.as_bytes()),
                    "data_hash": hex::encode(uso.id.as_bytes()),
                }),
                std::collections::BTreeSet::new(),
                vv,
                self.node_id.clone(),
                keypair,
            );
            
            self.dag.insert(op).map_err(|e| SyncError::DagError(e.to_string()))?;
        }
        
        Ok(uso.id)
    }

    /// Get operations since a version vector (for sync)
    pub fn get_operations_since(&self, since_lamport: u64) -> Vec<&CausalOp> {
        self.dag.get_operations_after(since_lamport)
    }

    /// Merge remote operations into local DAG
    pub fn merge_remote(&mut self, ops: Vec<CausalOp>) -> Result<usize, SyncError> {
        let mut merged = 0;
        for op in ops {
            match self.dag.insert(op) {
                Ok(_) => merged += 1,
                Err(e) => {
                    // Log but continue - some ops may conflict
                    eprintln!("Merge warning: {}", e);
                }
            }
        }
        Ok(merged)
    }

    /// Get sync delta for a peer
    pub fn get_sync_delta(&self, peer_vv: &VersionVector) -> SyncDelta {
        let local_vv = self.version_vector();
        
        // Find common point
        let mut min_lamport = u64::MAX;
        for (node, count) in &peer_vv.versions {
            let local_count = local_vv.get(node);
            if local_count > *count {
                min_lamport = min_lamport.min(*count);
            }
        }
        
        let ops = if min_lamport == u64::MAX {
            vec![]
        } else {
            self.get_operations_since(min_lamport)
                .into_iter()
                .cloned()
                .collect()
        };
        
        SyncDelta {
            from_version: peer_vv.clone(),
            to_version: local_vv.clone(),
            operations: ops,
        }
    }
}

/// Sync delta for incremental sync
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncDelta {
    pub from_version: VersionVector,
    pub to_version: VersionVector,
    pub operations: Vec<CausalOp>,
}

/// Sync errors
#[derive(Debug, Clone)]
pub enum SyncError {
    UsoNotFound,
    DagError(String),
    InvalidOperation(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SyncError::UsoNotFound => write!(f, "USO not found"),
            SyncError::DagError(e) => write!(f, "DAG error: {}", e),
            SyncError::InvalidOperation(e) => write!(f, "Invalid operation: {}", e),
        }
    }
}

impl std::error::Error for SyncError {}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    fn create_test_keypair() -> SigningKey {
        let mut rng = OsRng;
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rng, &mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn test_sync_engine_creation() {
        let engine = NexusSyncEngine::new("node1", ConflictPolicy::LastWriterWins);
        assert_eq!(engine.operation_count(), 0);
    }

    #[test]
    fn test_uso_registration() {
        let mut engine = NexusSyncEngine::new("node1", ConflictPolicy::LastWriterWins);
        let uso = USO::new(b"test data".to_vec(), PrincipalId::generate());
        
        let uso_id = uso.id;
        engine.register_uso(uso);
        
        assert!(engine.get_uso(&uso_id).is_some());
    }

    #[test]
    fn test_uso_update_with_keypair() {
        let keypair = create_test_keypair();
        let mut engine = NexusSyncEngine::new("node1", ConflictPolicy::LastWriterWins)
            .with_keypair(keypair);
        
        let principal = PrincipalId::generate();
        let uso = USO::new(b"initial".to_vec(), principal);
        let uso_id = uso.id;
        
        engine.register_uso(uso);
        
        let new_id = engine.update_uso(&uso_id, b"updated".to_vec(), principal);
        assert!(new_id.is_ok());
        
        // Should have created an operation
        assert_eq!(engine.operation_count(), 1);
    }
}
