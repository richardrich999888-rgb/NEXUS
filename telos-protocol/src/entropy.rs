//! Layer 2: Entropy Meter
//!
//! Manages verifiable entropy consumption for commitment crossing.
//! Entropy is the scarce resource that makes decisions costly.

use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};

/// Consequence tier determines entropy cost multiplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConsequenceTier {
    /// Tier 1: Minimal consequence (query, log, notify)
    Minimal = 1,
    /// Tier 2: Low consequence (internal state change)
    Low = 2,
    /// Tier 3: Medium consequence (external API call)
    Medium = 3,
    /// Tier 4: High consequence (financial transaction)
    High = 4,
    /// Tier 5: Critical consequence (irreversible physical action)
    Critical = 5,
}

impl ConsequenceTier {
    /// Get the entropy cost multiplier (tier squared).
    pub fn multiplier(&self) -> u64 {
        let tier = *self as u64;
        tier * tier
    }

    /// Parse from integer.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Minimal),
            2 => Some(Self::Low),
            3 => Some(Self::Medium),
            4 => Some(Self::High),
            5 => Some(Self::Critical),
            _ => None,
        }
    }
}

/// Proof of entropy consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyProof {
    /// Type of entropy source used.
    pub source_type: EntropySourceType,
    /// Amount of entropy consumed.
    pub amount: u64,
    /// Source-specific proof data.
    pub proof_data: EntropyProofData,
    /// When the entropy was generated.
    pub generated_at: DateTime<Utc>,
    /// Hash of the proof for verification.
    pub proof_hash: [u8; 32],
}

impl EntropyProof {
    /// Create a new entropy proof.
    pub fn new(source_type: EntropySourceType, amount: u64, proof_data: EntropyProofData) -> Self {
        let generated_at = Utc::now();
        let proof_hash = Self::compute_hash(&source_type, amount, &proof_data, &generated_at);
        Self {
            source_type,
            amount,
            proof_data,
            generated_at,
            proof_hash,
        }
    }

    fn compute_hash(
        source_type: &EntropySourceType,
        amount: u64,
        proof_data: &EntropyProofData,
        generated_at: &DateTime<Utc>,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", source_type).as_bytes());
        hasher.update(amount.to_le_bytes());
        hasher.update(format!("{:?}", proof_data).as_bytes());
        hasher.update(generated_at.timestamp().to_le_bytes());
        hasher.finalize().into()
    }

    /// Verify the proof hash is valid.
    pub fn verify(&self) -> bool {
        let expected = Self::compute_hash(
            &self.source_type,
            self.amount,
            &self.proof_data,
            &self.generated_at,
        );
        self.proof_hash == expected
    }
}

/// Type of entropy source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropySourceType {
    /// Verifiable Delay Function.
    VDF,
    /// External random beacon.
    Beacon,
    /// Proof-of-stake burn.
    StakeBurn,
    /// TEE attestation.
    TEE,
}

/// Source-specific proof data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropyProofData {
    /// VDF proof with input, output, and steps.
    VDF {
        input: Vec<u8>,
        output: Vec<u8>,
        steps: u64,
    },
    /// Beacon proof with round and randomness.
    Beacon {
        beacon_id: String,
        round: u64,
        randomness: Vec<u8>,
    },
    /// Stake burn proof with transaction hash.
    StakeBurn {
        tx_hash: [u8; 32],
        amount: u64,
        block_height: u64,
    },
    /// TEE attestation with measurement.
    TEE {
        enclave_id: String,
        measurement: [u8; 32],
        report: Vec<u8>,
    },
}

/// Entropy meter for managing an agent's entropy budget.
#[derive(Debug, Clone)]
pub struct EntropyMeter {
    /// Total entropy budget.
    budget: u64,
    /// Currently consumed entropy.
    consumed: u64,
    /// Base rate for entropy cost calculation.
    base_rate: u64,
    /// Current load factor (1.0 = normal).
    load_factor: f64,
    /// History of entropy proofs.
    proof_history: Vec<EntropyProof>,
}

impl EntropyMeter {
    /// Default base rate: 1000 telos units.
    pub const DEFAULT_BASE_RATE: u64 = 1000;

    /// Create a new entropy meter with the given budget.
    pub fn new(budget: u64) -> Self {
        Self {
            budget,
            consumed: 0,
            base_rate: Self::DEFAULT_BASE_RATE,
            load_factor: 1.0,
            proof_history: Vec::new(),
        }
    }

    /// Create with custom base rate.
    pub fn with_base_rate(budget: u64, base_rate: u64) -> Self {
        Self {
            budget,
            consumed: 0,
            base_rate,
            load_factor: 1.0,
            proof_history: Vec::new(),
        }
    }

    /// Get available entropy.
    pub fn available(&self) -> u64 {
        self.budget.saturating_sub(self.consumed)
    }

    /// Get total consumed entropy.
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Calculate entropy cost for a commitment.
    pub fn calculate_cost(&self, tier: ConsequenceTier, trust_score: f64) -> u64 {
        // cost = base_rate * tier² * load_factor * trust_discount
        let base = self.base_rate as f64;
        let multiplier = tier.multiplier() as f64;
        let trust_discount = 1.0 - (trust_score.clamp(0.0, 1.0) * 0.5);
        
        (base * multiplier * self.load_factor * trust_discount) as u64
    }

    /// Attempt to consume entropy for a commitment.
    pub fn consume(
        &mut self,
        tier: ConsequenceTier,
        trust_score: f64,
        proof: EntropyProof,
    ) -> TelosResult<u64> {
        // Verify the proof
        if !proof.verify() {
            return Err(TelosError::InvalidEntropyProof("Hash mismatch".into()));
        }

        let cost = self.calculate_cost(tier, trust_score);
        
        // Check if proof covers the cost
        if proof.amount < cost {
            return Err(TelosError::InsufficientEntropy {
                required: cost,
                available: proof.amount,
            });
        }

        // Check if budget covers the cost
        if self.available() < cost {
            return Err(TelosError::InsufficientEntropy {
                required: cost,
                available: self.available(),
            });
        }

        // Consume the entropy
        self.consumed += cost;
        self.proof_history.push(proof);

        Ok(cost)
    }

    /// Refund entropy (e.g., on rejection).
    pub fn refund(&mut self, amount: u64, processing_fee: u64) {
        let refund = amount.saturating_sub(processing_fee);
        self.consumed = self.consumed.saturating_sub(refund);
    }

    /// Update load factor.
    pub fn set_load_factor(&mut self, factor: f64) {
        self.load_factor = factor.max(1.0);
    }

    /// Get proof history.
    pub fn proof_history(&self) -> &[EntropyProof] {
        &self.proof_history
    }

    /// Replenish budget (called by authority).
    pub fn replenish(&mut self, amount: u64) {
        self.budget = self.budget.saturating_add(amount);
    }
}

/// Simple VDF implementation for testing (NOT cryptographically secure).
#[cfg(feature = "vdf")]
pub mod vdf {
    use super::*;

    /// Generate a simple VDF proof (for testing only).
    pub fn generate_proof(input: &[u8], steps: u64) -> EntropyProof {
        let mut output = input.to_vec();
        
        // Sequential hashing (simple VDF simulation)
        for _ in 0..steps {
            let mut hasher = Sha256::new();
            hasher.update(&output);
            output = hasher.finalize().to_vec();
        }

        let proof_data = EntropyProofData::VDF {
            input: input.to_vec(),
            output: output.clone(),
            steps,
        };

        // Amount is proportional to steps
        let amount = steps * 100;

        EntropyProof::new(EntropySourceType::VDF, amount, proof_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consequence_tier_multiplier() {
        assert_eq!(ConsequenceTier::Minimal.multiplier(), 1);
        assert_eq!(ConsequenceTier::Low.multiplier(), 4);
        assert_eq!(ConsequenceTier::Medium.multiplier(), 9);
        assert_eq!(ConsequenceTier::High.multiplier(), 16);
        assert_eq!(ConsequenceTier::Critical.multiplier(), 25);
    }

    #[test]
    fn test_entropy_meter_basic() {
        let meter = EntropyMeter::new(10000);
        assert_eq!(meter.available(), 10000);
        assert_eq!(meter.consumed(), 0);
    }

    #[test]
    fn test_cost_calculation() {
        let meter = EntropyMeter::new(10000);
        
        // Base cost for minimal tier, no trust
        let cost = meter.calculate_cost(ConsequenceTier::Minimal, 0.0);
        assert_eq!(cost, 1000); // base_rate * 1 * 1.0 * 1.0
        
        // High tier with 50% trust
        let cost = meter.calculate_cost(ConsequenceTier::High, 0.5);
        // 1000 * 16 * 1.0 * 0.75 = 12000
        assert_eq!(cost, 12000);
    }

    #[test]
    fn test_entropy_proof_verification() {
        let proof = EntropyProof::new(
            EntropySourceType::Beacon,
            5000,
            EntropyProofData::Beacon {
                beacon_id: "test".into(),
                round: 42,
                randomness: vec![1, 2, 3, 4],
            },
        );
        
        assert!(proof.verify());
    }

    #[test]
    fn test_consume_entropy() {
        let mut meter = EntropyMeter::new(10000);
        
        let proof = EntropyProof::new(
            EntropySourceType::Beacon,
            5000,
            EntropyProofData::Beacon {
                beacon_id: "test".into(),
                round: 1,
                randomness: vec![0; 32],
            },
        );
        
        let result = meter.consume(ConsequenceTier::Low, 0.0, proof);
        assert!(result.is_ok());
        
        // Cost was 1000 * 4 = 4000
        assert_eq!(meter.consumed(), 4000);
        assert_eq!(meter.available(), 6000);
    }

    #[test]
    fn test_insufficient_entropy() {
        let mut meter = EntropyMeter::new(1000);
        
        let proof = EntropyProof::new(
            EntropySourceType::Beacon,
            100, // Only 100 entropy in proof
            EntropyProofData::Beacon {
                beacon_id: "test".into(),
                round: 1,
                randomness: vec![0; 32],
            },
        );
        
        // Critical tier needs 25000 entropy
        let result = meter.consume(ConsequenceTier::Critical, 0.0, proof);
        assert!(matches!(result, Err(TelosError::InsufficientEntropy { .. })));
    }
}
