//! Ledger for immutable commitment proof anchoring.
//!
//! Provides a tamper-evident chain of blocks containing commitment proofs.

use crate::merkle::{MerkleTree, MerkleProof, commitment_hash};
use crate::membrane::CommitmentProof;
use crate::validator::ValidatorId;
use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// A block in the commitment ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block height (0-indexed).
    pub height: u64,
    /// Hash of the previous block.
    pub prev_hash: [u8; 32],
    /// Merkle root of commitments in this block.
    pub merkle_root: [u8; 32],
    /// Commitments included in this block.
    pub commitments: Vec<CommitmentProof>,
    /// Block creation timestamp.
    pub timestamp: DateTime<Utc>,
    /// Validator who proposed this block.
    pub proposer: ValidatorId,
    /// Block hash (computed).
    pub hash: [u8; 32],
}

impl Block {
    /// Create a new block.
    pub fn new(
        height: u64,
        prev_hash: [u8; 32],
        commitments: Vec<CommitmentProof>,
        proposer: ValidatorId,
    ) -> Self {
        let timestamp = Utc::now();
        
        // Build Merkle tree from commitment hashes
        let leaves: Vec<[u8; 32]> = commitments.iter()
            .map(|c| commitment_hash(&c.commitment_id, &c.decision_hash, c.entropy_consumed))
            .collect();
        
        let tree = MerkleTree::from_leaves(leaves);
        let merkle_root = tree.root();
        
        // Compute block hash
        let hash = Self::compute_hash(height, &prev_hash, &merkle_root, &timestamp, &proposer);

        Self {
            height,
            prev_hash,
            merkle_root,
            commitments,
            timestamp,
            proposer,
            hash,
        }
    }

    /// Compute block hash.
    fn compute_hash(
        height: u64,
        prev_hash: &[u8; 32],
        merkle_root: &[u8; 32],
        timestamp: &DateTime<Utc>,
        proposer: &ValidatorId,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(height.to_le_bytes());
        hasher.update(prev_hash);
        hasher.update(merkle_root);
        hasher.update(timestamp.timestamp().to_le_bytes());
        hasher.update(proposer.0.as_bytes());
        hasher.finalize().into()
    }

    /// Verify block hash is correct.
    pub fn verify_hash(&self) -> bool {
        let expected = Self::compute_hash(
            self.height,
            &self.prev_hash,
            &self.merkle_root,
            &self.timestamp,
            &self.proposer,
        );
        self.hash == expected
    }

    /// Get Merkle tree for this block.
    pub fn merkle_tree(&self) -> MerkleTree {
        let leaves: Vec<[u8; 32]> = self.commitments.iter()
            .map(|c| commitment_hash(&c.commitment_id, &c.decision_hash, c.entropy_consumed))
            .collect();
        MerkleTree::from_leaves(leaves)
    }

    /// Generate inclusion proof for a commitment in this block.
    pub fn inclusion_proof(&self, commitment_id: &str) -> Option<MerkleProof> {
        let index = self.commitments.iter()
            .position(|c| c.commitment_id == commitment_id)?;
        self.merkle_tree().proof(index)
    }
}

/// Position of a commitment in the ledger.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LedgerPosition {
    /// Block height.
    pub block_height: u64,
    /// Transaction index within block.
    pub tx_index: usize,
    /// Merkle root of the block.
    pub merkle_root: [u8; 32],
}

/// The commitment ledger.
#[derive(Debug)]
pub struct Ledger {
    /// Chain of blocks.
    blocks: Vec<Block>,
    /// Pending commitments (not yet in a block).
    pending: Vec<CommitmentProof>,
    /// Index: commitment_id → ledger position.
    index: HashMap<String, LedgerPosition>,
    /// Block interval in seconds.
    block_interval_secs: u64,
    /// Genesis block hash.
    genesis_hash: [u8; 32],
}

impl Ledger {
    /// Default block interval: 10 seconds.
    pub const DEFAULT_BLOCK_INTERVAL: u64 = 10;

    /// Create a new empty ledger.
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            pending: Vec::new(),
            index: HashMap::new(),
            block_interval_secs: Self::DEFAULT_BLOCK_INTERVAL,
            genesis_hash: [0u8; 32],
        }
    }

    /// Create with custom block interval.
    pub fn with_interval(block_interval_secs: u64) -> Self {
        Self {
            blocks: Vec::new(),
            pending: Vec::new(),
            index: HashMap::new(),
            block_interval_secs,
            genesis_hash: [0u8; 32],
        }
    }

    /// Get the latest block hash.
    pub fn latest_hash(&self) -> [u8; 32] {
        self.blocks.last()
            .map(|b| b.hash)
            .unwrap_or(self.genesis_hash)
    }

    /// Get the current height.
    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// Add a commitment to the pending pool.
    pub fn append_commitment(&mut self, commitment: CommitmentProof) {
        self.pending.push(commitment);
    }

    /// Get pending commitment count.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Propose a new block from pending commitments.
    pub fn propose_block(&mut self, proposer: ValidatorId) -> Option<Block> {
        if self.pending.is_empty() {
            return None;
        }

        let commitments = std::mem::take(&mut self.pending);
        let block = Block::new(
            self.height(),
            self.latest_hash(),
            commitments,
            proposer,
        );

        Some(block)
    }

    /// Validate a proposed block.
    pub fn validate_block(&self, block: &Block) -> TelosResult<()> {
        // Check height
        if block.height != self.height() {
            return Err(TelosError::HistoryCorrupted(block.height));
        }

        // Check prev_hash
        if block.prev_hash != self.latest_hash() {
            return Err(TelosError::HistoryCorrupted(block.height));
        }

        // Verify block hash
        if !block.verify_hash() {
            return Err(TelosError::HistoryCorrupted(block.height));
        }

        // Verify Merkle root
        let tree = block.merkle_tree();
        if tree.root() != block.merkle_root {
            return Err(TelosError::HistoryCorrupted(block.height));
        }

        Ok(())
    }

    /// Append a validated block to the chain.
    pub fn append_block(&mut self, block: Block) -> TelosResult<()> {
        self.validate_block(&block)?;

        // Index all commitments
        for (tx_index, commitment) in block.commitments.iter().enumerate() {
            self.index.insert(
                commitment.commitment_id.clone(),
                LedgerPosition {
                    block_height: block.height,
                    tx_index,
                    merkle_root: block.merkle_root,
                },
            );
        }

        self.blocks.push(block);
        Ok(())
    }

    /// Get a block by height.
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(height as usize)
    }

    /// Get ledger position for a commitment.
    pub fn get_position(&self, commitment_id: &str) -> Option<LedgerPosition> {
        self.index.get(commitment_id).copied()
    }

    /// Get inclusion proof for a commitment.
    pub fn get_inclusion_proof(&self, commitment_id: &str) -> Option<MerkleProof> {
        let position = self.get_position(commitment_id)?;
        let block = self.get_block(position.block_height)?;
        block.inclusion_proof(commitment_id)
    }

    /// Verify chain integrity from genesis.
    pub fn verify_chain(&self) -> bool {
        let mut prev_hash = self.genesis_hash;

        for block in &self.blocks {
            if block.prev_hash != prev_hash {
                return false;
            }
            if !block.verify_hash() {
                return false;
            }
            prev_hash = block.hash;
        }

        true
    }

    /// Get all blocks.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Get total commitment count.
    pub fn total_commitments(&self) -> usize {
        self.blocks.iter().map(|b| b.commitments.len()).sum()
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_commitment(id: &str) -> CommitmentProof {
        CommitmentProof {
            commitment_id: id.to_string(),
            decision_id: format!("decision-{}", id),
            decision_hash: [0u8; 32],
            entropy_consumed: 1000,
            entropy_proof_hash: [0u8; 32],
            authority_chain: vec!["root".into()],
            attestation_hashes: vec![],
            committed_at: Utc::now(),
            ledger_index: 0,
        }
    }

    #[test]
    fn test_empty_ledger() {
        let ledger = Ledger::new();
        assert_eq!(ledger.height(), 0);
        assert_eq!(ledger.pending_count(), 0);
        assert!(ledger.verify_chain());
    }

    #[test]
    fn test_append_commitment() {
        let mut ledger = Ledger::new();
        ledger.append_commitment(mock_commitment("c1"));
        ledger.append_commitment(mock_commitment("c2"));
        assert_eq!(ledger.pending_count(), 2);
    }

    #[test]
    fn test_propose_and_append_block() {
        let mut ledger = Ledger::new();
        ledger.append_commitment(mock_commitment("c1"));
        ledger.append_commitment(mock_commitment("c2"));

        let proposer = ValidatorId::new("validator-1");
        let block = ledger.propose_block(proposer).unwrap();
        
        assert_eq!(block.height, 0);
        assert_eq!(block.commitments.len(), 2);
        assert!(block.verify_hash());

        ledger.append_block(block).unwrap();
        assert_eq!(ledger.height(), 1);
        assert_eq!(ledger.pending_count(), 0);
    }

    #[test]
    fn test_inclusion_proof() {
        let mut ledger = Ledger::new();
        ledger.append_commitment(mock_commitment("c1"));
        ledger.append_commitment(mock_commitment("c2"));
        ledger.append_commitment(mock_commitment("c3"));

        let proposer = ValidatorId::new("validator-1");
        let block = ledger.propose_block(proposer).unwrap();
        ledger.append_block(block).unwrap();

        // Get and verify proof
        let proof = ledger.get_inclusion_proof("c2").unwrap();
        assert!(proof.verify());

        // Position should be correct
        let pos = ledger.get_position("c2").unwrap();
        assert_eq!(pos.block_height, 0);
        assert_eq!(pos.tx_index, 1);
    }

    #[test]
    fn test_chain_verification() {
        let mut ledger = Ledger::new();
        let proposer = ValidatorId::new("v1");

        // Create 5 blocks
        for i in 0..5 {
            ledger.append_commitment(mock_commitment(&format!("c{}", i)));
            let block = ledger.propose_block(proposer.clone()).unwrap();
            ledger.append_block(block).unwrap();
        }

        assert_eq!(ledger.height(), 5);
        assert!(ledger.verify_chain());
        assert_eq!(ledger.total_commitments(), 5);
    }

    #[test]
    fn test_invalid_block_height() {
        let mut ledger = Ledger::new();
        
        // Try to append block with wrong height
        let bad_block = Block::new(
            5, // Wrong height
            [0u8; 32],
            vec![mock_commitment("c1")],
            ValidatorId::new("v1"),
        );

        let result = ledger.append_block(bad_block);
        assert!(result.is_err());
    }
}
