// NEXUS Proof: Execution proofs for trust without re-verification
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Key innovation: Once a PCU executes, the proof allows any node to
// trust the result without re-executing. This eliminates redundant computation.

use serde::{Deserialize, Serialize};
use ed25519_dalek::{SigningKey, Signature, VerifyingKey, Signer, Verifier};

use crate::{NodeId, content_hash::ContentHash, Timestamp};

// ============================================================================
// NODE ATTESTATION - Proof from the executing node
// ============================================================================

/// Attestation from the node that executed the PCU
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAttestation {
    /// Which node executed this
    pub node_id: NodeId,
    
    /// When execution completed
    pub executed_at: Timestamp,
    
    /// Hardware security level (0 = software, 1 = SGX, 2 = HSM)
    pub security_level: u8,
    
    /// Ed25519 signature over proof contents
    pub signature: Vec<u8>,
}

impl NodeAttestation {
    /// Create new attestation (unsigned)
    pub fn new(node_id: NodeId, security_level: u8) -> Self {
        NodeAttestation {
            node_id,
            executed_at: crate::now(),
            security_level,
            signature: Vec::new(),
        }
    }

    /// Sign attestation with node's private key
    pub fn sign(&mut self, signing_key: &SigningKey, proof_contents: &[u8]) {
        let mut data = Vec::new();
        data.extend_from_slice(self.node_id.as_bytes());
        data.extend_from_slice(&self.executed_at.to_le_bytes());
        data.push(self.security_level);
        data.extend_from_slice(proof_contents);
        
        let signature: Signature = signing_key.sign(&data);
        self.signature = signature.to_bytes().to_vec();
    }

    /// Verify attestation signature
    pub fn verify(&self, proof_contents: &[u8]) -> bool {
        if self.signature.len() != 64 {
            return false;
        }
        
        let verifying_key = match VerifyingKey::from_bytes(&self.node_id.0) {
            Ok(k) => k,
            Err(_) => return false,
        };
        
        let signature = match Signature::from_slice(&self.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        
        let mut data = Vec::new();
        data.extend_from_slice(self.node_id.as_bytes());
        data.extend_from_slice(&self.executed_at.to_le_bytes());
        data.push(self.security_level);
        data.extend_from_slice(proof_contents);
        
        verifying_key.verify(&data, &signature).is_ok()
    }
}

// ============================================================================
// EXECUTION PROOF - Cryptographic proof of correct execution
// ============================================================================

/// Execution proof enables trust without re-verification
/// 
/// Contains:
/// - What inputs were used (by hash)
/// - What code executed (by hash)
/// - What output was produced (by hash)
/// - Attestation from executing node
///
/// Any node can verify this proof and trust the result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionProof {
    /// Hash of the PCU that was executed
    pub pcu_hash: ContentHash,
    
    /// Hashes of inputs that were used
    pub input_hashes: Vec<ContentHash>,
    
    /// Hash of code that executed
    pub code_hash: ContentHash,
    
    /// Hash of output produced
    pub output_hash: ContentHash,
    
    /// Execution duration in microseconds (for audit)
    pub duration_us: u64,
    
    /// Memory used in bytes (for audit)
    pub memory_bytes: u64,
    
    /// Attestation from executing node
    pub attestation: NodeAttestation,
}

impl ExecutionProof {
    /// Create new execution proof
    pub fn new(
        pcu_hash: ContentHash,
        input_hashes: Vec<ContentHash>,
        code_hash: ContentHash,
        output_hash: ContentHash,
        duration_us: u64,
        memory_bytes: u64,
        attestation: NodeAttestation,
    ) -> Self {
        ExecutionProof {
            pcu_hash,
            input_hashes,
            code_hash,
            output_hash,
            duration_us,
            memory_bytes,
            attestation,
        }
    }

    /// Get ID of node that produced this proof
    pub fn node_id(&self) -> NodeId {
        self.attestation.node_id
    }

    /// Compute deterministic proof content hash (for signing)
    pub fn content_hash(&self) -> ContentHash {
        let mut hasher = blake3::Hasher::new();
        
        hasher.update(self.pcu_hash.as_bytes());
        
        for input in &self.input_hashes {
            hasher.update(input.as_bytes());
        }
        
        hasher.update(self.code_hash.as_bytes());
        hasher.update(self.output_hash.as_bytes());
        hasher.update(&self.duration_us.to_le_bytes());
        hasher.update(&self.memory_bytes.to_le_bytes());
        
        ContentHash(*hasher.finalize().as_bytes())
    }

    /// Sign this proof with a node's signing key
    pub fn sign(&mut self, signing_key: &SigningKey) {
        let content = self.content_hash();
        self.attestation.sign(signing_key, content.as_bytes());
    }

    /// Verify the proof's attestation
    pub fn verify(&self) -> bool {
        let content = self.content_hash();
        self.attestation.verify(content.as_bytes())
    }

    /// Check if proof matches expected inputs and code
    pub fn matches(&self, expected_inputs: &[ContentHash], expected_code: ContentHash) -> bool {
        if self.code_hash != expected_code {
            return false;
        }
        
        if self.input_hashes.len() != expected_inputs.len() {
            return false;
        }
        
        self.input_hashes.iter()
            .zip(expected_inputs.iter())
            .all(|(a, b)| a == b)
    }
}

/// Builder for creating execution proofs
pub struct ExecutionProofBuilder {
    pcu_hash: ContentHash,
    input_hashes: Vec<ContentHash>,
    code_hash: ContentHash,
    output_hash: Option<ContentHash>,
    duration_us: u64,
    memory_bytes: u64,
}

impl ExecutionProofBuilder {
    /// Start building a proof
    pub fn new(pcu_hash: ContentHash, code_hash: ContentHash) -> Self {
        ExecutionProofBuilder {
            pcu_hash,
            input_hashes: Vec::new(),
            code_hash,
            output_hash: None,
            duration_us: 0,
            memory_bytes: 0,
        }
    }

    /// Add input hashes
    pub fn with_inputs(mut self, inputs: Vec<ContentHash>) -> Self {
        self.input_hashes = inputs;
        self
    }

    /// Set output hash
    pub fn with_output(mut self, output_hash: ContentHash) -> Self {
        self.output_hash = Some(output_hash);
        self
    }

    /// Set execution metrics
    pub fn with_metrics(mut self, duration_us: u64, memory_bytes: u64) -> Self {
        self.duration_us = duration_us;
        self.memory_bytes = memory_bytes;
        self
    }

    /// Build and sign the proof
    pub fn build_signed(self, signing_key: &SigningKey) -> ExecutionProof {
        let node_id = NodeId::from_verifying_key(&signing_key.verifying_key());
        let attestation = NodeAttestation::new(node_id, 0);
        
        let mut proof = ExecutionProof::new(
            self.pcu_hash,
            self.input_hashes,
            self.code_hash,
            self.output_hash.unwrap_or(ContentHash::genesis()),
            self.duration_us,
            self.memory_bytes,
            attestation,
        );
        
        proof.sign(signing_key);
        proof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn generate_signing_key() -> SigningKey {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn test_node_id_from_key() {
        let key = generate_signing_key();
        let node_id = NodeId::from_verifying_key(&key.verifying_key());
        assert_eq!(node_id.0, key.verifying_key().to_bytes());
    }

    #[test]
    fn test_attestation_sign_verify() {
        let key = generate_signing_key();
        let node_id = NodeId::from_verifying_key(&key.verifying_key());
        
        let mut attestation = NodeAttestation::new(node_id, 0);
        let proof_content = b"test proof content";
        
        attestation.sign(&key, proof_content);
        assert!(attestation.verify(proof_content));
    }

    #[test]
    fn test_attestation_invalid_content() {
        let key = generate_signing_key();
        let node_id = NodeId::from_verifying_key(&key.verifying_key());
        
        let mut attestation = NodeAttestation::new(node_id, 0);
        attestation.sign(&key, b"original content");
        
        // Verify with different content should fail
        assert!(!attestation.verify(b"different content"));
    }

    #[test]
    fn test_execution_proof_builder() {
        let key = generate_signing_key();
        let pcu_hash = ContentHash::compute(b"pcu");
        let code_hash = ContentHash::compute(b"code");
        let output_hash = ContentHash::compute(b"output");
        
        let proof = ExecutionProofBuilder::new(pcu_hash, code_hash)
            .with_inputs(vec![ContentHash::compute(b"input1")])
            .with_output(output_hash)
            .with_metrics(1000, 65536)
            .build_signed(&key);
        
        assert!(proof.verify());
        assert_eq!(proof.pcu_hash, pcu_hash);
        assert_eq!(proof.output_hash, output_hash);
    }

    #[test]
    fn test_proof_matches() {
        let key = generate_signing_key();
        let code_hash = ContentHash::compute(b"code");
        let inputs = vec![ContentHash::compute(b"input1"), ContentHash::compute(b"input2")];
        
        let proof = ExecutionProofBuilder::new(ContentHash::compute(b"pcu"), code_hash)
            .with_inputs(inputs.clone())
            .with_output(ContentHash::compute(b"output"))
            .build_signed(&key);
        
        assert!(proof.matches(&inputs, code_hash));
        assert!(!proof.matches(&inputs, ContentHash::compute(b"wrong code")));
    }
}
