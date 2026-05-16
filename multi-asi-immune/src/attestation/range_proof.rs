//! Range proofs for homeostatic attestation.
//! 
//! Allows proving "metric X is in range [a, b]" without revealing exact value.
//! This is a simplified implementation - production would use bulletproofs.

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// A range proof for a homeostatic metric.
/// 
/// Proves that a value is within a specified range without revealing the exact value.
/// This is a simplified commitment-based scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleRangeProof {
    /// Metric identifier.
    pub metric_id: u32,
    /// Lower bound being proven.
    pub lower: f64,
    /// Upper bound being proven.
    pub upper: f64,
    /// Commitment to the actual value.
    pub commitment: [u8; 32],
    /// Proof data (simplified).
    pub proof: Vec<u8>,
}

impl SimpleRangeProof {
    /// Creates a range proof for a value.
    /// 
    /// # Returns
    /// 
    /// Some(proof) if value is in range, None otherwise.
    pub fn create(
        metric_id: u32,
        actual_value: f64,
        lower: f64,
        upper: f64,
        blinding_factor: &[u8; 32],
    ) -> Option<Self> {
        // Check that value is actually in range
        if actual_value < lower || actual_value > upper {
            return None;
        }
        
        // Create commitment
        let commitment = Self::commit(actual_value, blinding_factor);
        
        // Create proof (simplified - real impl would use bulletproofs)
        // This just commits to the bounds and value
        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&actual_value.to_le_bytes());
        proof_data.extend_from_slice(blinding_factor);
        
        let mut hasher = Sha256::new();
        hasher.update(b"RANGE_PROOF_V1:");
        hasher.update(&proof_data);
        let proof = hasher.finalize().to_vec();
        
        Some(Self {
            metric_id,
            lower,
            upper,
            commitment,
            proof,
        })
    }
    
    /// Creates a commitment to a value.
    fn commit(value: f64, blinding: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"COMMITMENT_V1:");
        hasher.update(&value.to_le_bytes());
        hasher.update(blinding);
        
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        result
    }
    
    /// Verifies the proof (simplified - requires knowing the value).
    /// 
    /// In a real implementation, this would verify without knowing the value.
    pub fn verify_with_value(&self, actual_value: f64, blinding_factor: &[u8; 32]) -> bool {
        // Check range
        if actual_value < self.lower || actual_value > self.upper {
            return false;
        }
        
        // Check commitment
        let expected_commitment = Self::commit(actual_value, blinding_factor);
        self.commitment == expected_commitment
    }
    
    /// Returns the claimed range.
    pub fn range(&self) -> (f64, f64) {
        (self.lower, self.upper)
    }
}

/// Batch of range proofs for multiple metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeostaticAttestation {
    /// Proofs for each metric.
    pub proofs: Vec<SimpleRangeProof>,
    /// Timestamp of attestation.
    pub timestamp: u64,
    /// Attester identifier.
    pub attester: [u8; 32],
}

impl HomeostaticAttestation {
    /// Creates a new attestation.
    pub fn new(attester: [u8; 32], timestamp: u64) -> Self {
        Self {
            proofs: Vec::new(),
            timestamp,
            attester,
        }
    }
    
    /// Adds a proof to the attestation.
    pub fn add_proof(&mut self, proof: SimpleRangeProof) {
        self.proofs.push(proof);
    }
    
    /// Gets proof for a specific metric.
    pub fn get_proof(&self, metric_id: u32) -> Option<&SimpleRangeProof> {
        self.proofs.iter().find(|p| p.metric_id == metric_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_range_proof() {
        let blinding = [42u8; 32];
        
        let proof = SimpleRangeProof::create(1, 0.5, 0.0, 1.0, &blinding);
        
        assert!(proof.is_some());
        let proof = proof.unwrap();
        assert!(proof.verify_with_value(0.5, &blinding));
    }
    
    #[test]
    fn test_out_of_range_fails() {
        let blinding = [42u8; 32];
        
        // Value outside range should fail to create proof
        let proof = SimpleRangeProof::create(1, 1.5, 0.0, 1.0, &blinding);
        assert!(proof.is_none());
    }
    
    #[test]
    fn test_wrong_value_verification_fails() {
        let blinding = [42u8; 32];
        
        let proof = SimpleRangeProof::create(1, 0.5, 0.0, 1.0, &blinding).unwrap();
        
        // Wrong value should fail verification
        assert!(!proof.verify_with_value(0.6, &blinding));
    }
}
