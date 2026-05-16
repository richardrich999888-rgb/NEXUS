//! Merkle Tree utilities for commitment proof anchoring.
//!
//! Provides cryptographic inclusion proofs for commitments in blocks.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// A Merkle tree built from leaf hashes.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Original leaf hashes.
    leaves: Vec<[u8; 32]>,
    /// All tree layers (leaves at 0, root at last).
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build a Merkle tree from leaf hashes.
    pub fn from_leaves(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves: vec![],
                layers: vec![vec![[0u8; 32]]],
            };
        }

        let mut layers = vec![leaves.clone()];
        let mut current = leaves.clone();

        // Build tree bottom-up
        while current.len() > 1 {
            let mut next = Vec::new();
            
            for chunk in current.chunks(2) {
                let left = chunk[0];
                let right = if chunk.len() > 1 { chunk[1] } else { chunk[0] };
                next.push(Self::hash_pair(&left, &right));
            }
            
            layers.push(next.clone());
            current = next;
        }

        Self { leaves, layers }
    }

    /// Hash two nodes together.
    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    /// Get the Merkle root.
    pub fn root(&self) -> [u8; 32] {
        self.layers.last()
            .and_then(|l| l.first().copied())
            .unwrap_or([0u8; 32])
    }

    /// Get the number of leaves.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if tree is empty.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Generate inclusion proof for a leaf at given index.
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let leaf = self.leaves[index];
        let mut siblings = Vec::new();
        let mut path = Vec::new();
        let mut idx = index;

        // Walk up the tree
        for layer in &self.layers[..self.layers.len().saturating_sub(1)] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sibling = if sibling_idx < layer.len() {
                layer[sibling_idx]
            } else {
                layer[idx] // Duplicate if odd
            };
            
            siblings.push(sibling);
            path.push(idx % 2 == 1); // true = right, false = left
            idx /= 2;
        }

        Some(MerkleProof {
            root: self.root(),
            leaf,
            leaf_index: index,
            siblings,
            path,
        })
    }
}

/// A Merkle inclusion proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Root hash of the tree.
    pub root: [u8; 32],
    /// Leaf hash being proven.
    pub leaf: [u8; 32],
    /// Index of the leaf.
    pub leaf_index: usize,
    /// Sibling hashes along the path.
    pub siblings: Vec<[u8; 32]>,
    /// Path direction (true = leaf is on right, false = left).
    pub path: Vec<bool>,
}

impl MerkleProof {
    /// Verify this proof.
    pub fn verify(&self) -> bool {
        let mut current = self.leaf;

        for (sibling, is_right) in self.siblings.iter().zip(self.path.iter()) {
            current = if *is_right {
                MerkleTree::hash_pair(sibling, &current)
            } else {
                MerkleTree::hash_pair(&current, sibling)
            };
        }

        current == self.root
    }

    /// Serialize to hex string for transport.
    pub fn to_hex(&self) -> String {
        format!(
            "{}:{}:{}",
            hex::encode(self.root),
            hex::encode(self.leaf),
            self.siblings.iter().map(hex::encode).collect::<Vec<_>>().join(",")
        )
    }
}

/// Compute leaf hash from commitment data.
pub fn commitment_hash(commitment_id: &str, decision_hash: &[u8; 32], entropy: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(commitment_id.as_bytes());
    hasher.update(decision_hash);
    hasher.update(entropy.to_le_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::from_leaves(vec![]);
        assert!(tree.is_empty());
        assert_eq!(tree.root(), [0u8; 32]);
    }

    #[test]
    fn test_single_leaf() {
        let leaf = [1u8; 32];
        let tree = MerkleTree::from_leaves(vec![leaf]);
        assert_eq!(tree.len(), 1);
        // Root equals the single leaf
        assert_eq!(tree.root(), leaf);
    }

    #[test]
    fn test_two_leaves() {
        let leaf1 = [1u8; 32];
        let leaf2 = [2u8; 32];
        let tree = MerkleTree::from_leaves(vec![leaf1, leaf2]);
        
        let expected_root = MerkleTree::hash_pair(&leaf1, &leaf2);
        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let leaves: Vec<[u8; 32]> = (0..8).map(|i| [i as u8; 32]).collect();
        let tree = MerkleTree::from_leaves(leaves);

        // Verify all proofs
        for i in 0..8 {
            let proof = tree.proof(i).expect("proof should exist");
            assert!(proof.verify(), "proof {} should verify", i);
        }
    }

    #[test]
    fn test_proof_out_of_bounds() {
        let tree = MerkleTree::from_leaves(vec![[1u8; 32], [2u8; 32]]);
        assert!(tree.proof(5).is_none());
    }

    #[test]
    fn test_odd_number_of_leaves() {
        let leaves: Vec<[u8; 32]> = (0..5).map(|i| [i as u8; 32]).collect();
        let tree = MerkleTree::from_leaves(leaves);
        
        // All proofs should still verify
        for i in 0..5 {
            let proof = tree.proof(i).unwrap();
            assert!(proof.verify());
        }
    }

    #[test]
    fn test_commitment_hash() {
        let h1 = commitment_hash("commit-1", &[0u8; 32], 1000);
        let h2 = commitment_hash("commit-1", &[0u8; 32], 1000);
        let h3 = commitment_hash("commit-2", &[0u8; 32], 1000);
        
        assert_eq!(h1, h2); // Same inputs = same hash
        assert_ne!(h1, h3); // Different id = different hash
    }
}
