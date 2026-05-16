//! Execution proofs for verifiable computation.
//!
//! Every PCU execution produces a cryptographic proof that:
//! - The correct code was executed
//! - The claimed inputs were used
//! - The output hash is accurate
//! - A specific node performed the execution

use nexus_pcu::{ContentHash, IdentityContext, PCU, NodeId};
use crate::types::ExecutionResult;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Attestation from a node that executed computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAttestation {
    pub node_pubkey: [u8; 32],
    pub signature: Signature,
    pub tee_quote: Option<TeeQuote>,
}

impl NodeAttestation {
    /// Create a new attestation for a node.
    pub fn new(node_pubkey: [u8; 32]) -> Self {
        Self {
            node_pubkey,
            signature: Signature::from_bytes(&[0u8; 64]),
            tee_quote: None,
        }
    }

    /// Verify the attestation signature against provided data.
    pub fn verify(&self, data: &[u8]) -> Result<(), AttestationError> {
        let pubkey = VerifyingKey::from_bytes(&self.node_pubkey)
            .map_err(|_| AttestationError::InvalidPublicKey)?;
        pubkey
            .verify(data, &self.signature)
            .map_err(|_| AttestationError::InvalidSignature)
    }

    /// Get the NodeId corresponding to this attestation.
    pub fn node_id(&self) -> Result<NodeId, AttestationError> {
        let pubkey = VerifyingKey::from_bytes(&self.node_pubkey)
            .map_err(|_| AttestationError::InvalidPublicKey)?;
        Ok(NodeId::from_verifying_key(&pubkey))
    }
}

impl Default for NodeAttestation {
    fn default() -> Self {
        Self::new([0u8; 32])
    }
}

/// Hardware TEE attestation quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeQuote {
    pub tee_type: TeeType,
    pub quote: Vec<u8>,
    pub endorsements: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub report_data: Vec<u8>,
}

/// Type of Trusted Execution Environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeeType {
    /// Intel SGX
    IntelSgx,
    /// ARM TrustZone
    ArmTrustZone,
    /// AMD SEV
    AmdSev,
    /// No TEE (Software only)
    None,
}

/// Errors that can occur during attestation verification.
#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    /// Public key is malformed.
    #[error("Invalid public key")]
    InvalidPublicKey,
    /// Signature is invalid.
    #[error("Invalid signature")]
    InvalidSignature,
}

/// Proof of correct execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProof {
    /// Hash of the PCU code executed.
    pub pcu_hash: ContentHash,
    /// Hashes of the inputs used for execution.
    pub input_hashes: Vec<ContentHash>,
    /// Hash of the execution output.
    pub output_hash: ContentHash,
    /// Hash of the identity context used.
    pub identity_hash: ContentHash,
    /// ID of the node that performed the execution.
    pub executor_node: NodeId,
    /// Unix timestamp of execution.
    pub executed_at: u64,
    /// Duration of execution in milliseconds.
    pub duration_ms: u64,
    /// Fuel consumed during execution.
    pub fuel_consumed: u64,
    /// Peak memory usage in bytes.
    pub peak_memory: usize,
    /// Node attestation (signature + optional TEE quote).
    pub attestation: NodeAttestation,
}

impl ExecutionProof {
    /// Create a new execution proof.
    pub fn create(
        pcu: &PCU,
        inputs: &[(ContentHash, Vec<u8>)],
        result: &ExecutionResult,
        identity: &IdentityContext,
        node_key: &SigningKey,
    ) -> Self {
        let pcu_hash = pcu.content_hash();
        let input_hashes: Vec<_> = inputs.iter().map(|(h, _)| *h).collect();
        let identity_hash = identity.content_hash();
        let executor_node = NodeId::from_verifying_key(&node_key.verifying_key());

        let executed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::ZERO)
            .as_secs();

        let mut proof = Self {
            pcu_hash,
            input_hashes,
            output_hash: result.output_hash,
            identity_hash,
            executor_node,
            executed_at,
            duration_ms: result.duration.as_millis() as u64,
            fuel_consumed: result.fuel_consumed,
            peak_memory: result.peak_memory,
            attestation: NodeAttestation::new(node_key.verifying_key().to_bytes()),
        };

        let signing_bytes = proof.signing_bytes();
        proof.attestation.signature = node_key.sign(&signing_bytes);

        proof
    }

    /// Get the bytes to be signed for this proof.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(256);
        bytes.extend_from_slice(self.pcu_hash.as_bytes());
        bytes.extend_from_slice(&(self.input_hashes.len() as u32).to_le_bytes());
        for hash in &self.input_hashes {
            bytes.extend_from_slice(hash.as_bytes());
        }
        bytes.extend_from_slice(self.output_hash.as_bytes());
        bytes.extend_from_slice(self.identity_hash.as_bytes());
        bytes.extend_from_slice(&self.executor_node.0);
        bytes.extend_from_slice(&self.executed_at.to_le_bytes());
        bytes.extend_from_slice(&self.duration_ms.to_le_bytes());
        bytes.extend_from_slice(&self.fuel_consumed.to_le_bytes());
        bytes.extend_from_slice(&(self.peak_memory as u64).to_le_bytes());
        bytes.extend_from_slice(&self.attestation.node_pubkey);
        bytes
    }

    /// Verify the proof signature.
    pub fn verify(&self) -> Result<(), ProofError> {
        let signing_bytes = self.signing_bytes();
        self.attestation
            .verify(&signing_bytes)
            .map_err(|_| ProofError::InvalidSignature)
    }

    /// Verify that the proof matches the provided output.
    pub fn verify_output(&self, output: &[u8]) -> Result<(), ProofError> {
        self.verify()?;
        let expected = ContentHash::compute(output);
        if self.output_hash != expected {
            return Err(ProofError::OutputHashMismatch {
                expected,
                actual: self.output_hash,
            });
        }
        Ok(())
    }

    /// Get the content hash of the entire proof.
    pub fn content_hash(&self) -> ContentHash {
        let bytes = bincode::serialize(self).unwrap_or_default();
        ContentHash::compute(&bytes)
    }
}

impl Default for ExecutionProof {
    fn default() -> Self {
        Self {
            pcu_hash: ContentHash::zero(),
            input_hashes: Vec::new(),
            output_hash: ContentHash::zero(),
            identity_hash: ContentHash::zero(),
            executor_node: NodeId::local(),
            executed_at: 0,
            duration_ms: 0,
            fuel_consumed: 0,
            peak_memory: 0,
            attestation: NodeAttestation::default(),
        }
    }
}

/// Errors that can occur during proof verification.
#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    /// The attestation signature is invalid.
    #[error("Invalid attestation signature")]
    InvalidSignature,
    /// The PCU hash in the proof does not match the actual PCU hash.
    #[error("PCU hash mismatch: expected {expected}, got {actual}")]
    PcuHashMismatch {
        /// Expected PCU hash.
        expected: ContentHash,
        /// Actual PCU hash.
        actual: ContentHash,
    },
    /// Input hashes do not match.
    #[error("Input hashes do not match")]
    InputHashMismatch,
    /// Output hash mismatch.
    #[error("Output hash mismatch: expected {expected}, got {actual}")]
    OutputHashMismatch {
        /// Expected output hash.
        expected: ContentHash,
        /// Actual output hash.
        actual: ContentHash,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use nexus_pcu::{PCU, WasmModule, IdentityContext};
    use std::time::Duration;

    #[test]
    fn test_proof_creation_and_verification() {
        let node_key = SigningKey::generate(&mut OsRng);
        // Create a minimal PCU for testing
        let wasm = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
        let pcu = PCU::new(
            wasm,
            vec![],
            vec![],
            IdentityContext::anonymous(),
        );
        let inputs: Vec<(ContentHash, Vec<u8>)> = vec![];
        let result = ExecutionResult {
            output: b"test output".to_vec(),
            output_hash: ContentHash::compute(b"test output"),
            fuel_consumed: 1000,
            peak_memory: 1024,
            duration: Duration::from_millis(10),
        };
        let identity = IdentityContext::anonymous();

        let proof = ExecutionProof::create(&pcu, &inputs, &result, &identity, &node_key);
        assert!(proof.verify().is_ok());
        assert!(proof.verify_output(b"test output").is_ok());
        assert!(proof.verify_output(b"wrong output").is_err());
    }
}
