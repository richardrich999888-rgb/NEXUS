//! Layer 4: Validator Network
//!
//! External validators that attest to commitment validity.
//! Validators have staked collateral subject to slashing.

#[allow(unused_imports)]
use crate::membrane::CommitmentProof;
use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Unique validator identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidatorId(pub String);

impl ValidatorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Validator status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    /// Active and can attest.
    Active,
    /// Temporarily jailed for misbehavior.
    Jailed,
    /// Exiting the validator set.
    Exiting,
    /// Fully exited.
    Exited,
    /// Slashed and removed.
    Slashed,
}

/// A validator in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Unique identifier.
    pub id: ValidatorId,
    /// Public key (ed25519) as hex string for serde.
    pub public_key: String,
    /// Staked amount.
    pub stake: u64,
    /// Current reputation score (0.0 - 1.0).
    pub reputation: f64,
    /// Supported domains.
    pub supported_domains: Vec<String>,
    /// Registration timestamp.
    pub registered_at: DateTime<Utc>,
    /// Current status.
    pub status: ValidatorStatus,
    /// Attestation count.
    pub attestation_count: u64,
    /// Correct attestation count.
    pub correct_attestations: u64,
}

impl Validator {
    /// Create a new validator.
    pub fn new(id: impl Into<String>, public_key: [u8; 32], stake: u64) -> Self {
        Self {
            id: ValidatorId::new(id),
            public_key: hex::encode(public_key),
            stake,
            reputation: 0.5, // Start at neutral
            supported_domains: vec!["*".into()], // All domains by default
            registered_at: Utc::now(),
            status: ValidatorStatus::Active,
            attestation_count: 0,
            correct_attestations: 0,
        }
    }

    /// Update reputation after an attestation result.
    pub fn update_reputation(&mut self, correct: bool) {
        self.attestation_count += 1;
        if correct {
            self.correct_attestations += 1;
        }
        // Simple EMA-based reputation update
        let alpha = 0.1;
        let new_sample = if correct { 1.0 } else { 0.0 };
        self.reputation = (1.0 - alpha) * self.reputation + alpha * new_sample;
        self.reputation = self.reputation.clamp(0.0, 1.0);
    }

    /// Get accuracy rate.
    pub fn accuracy(&self) -> f64 {
        if self.attestation_count == 0 {
            0.5
        } else {
            self.correct_attestations as f64 / self.attestation_count as f64
        }
    }
}

/// Attestation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationType {
    Approve,
    Reject,
}

/// A validator attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Validator who attested.
    pub validator_id: ValidatorId,
    /// Decision ID being attested.
    pub decision_id: String,
    /// Attestation type.
    pub attestation_type: AttestationType,
    /// Reason (for rejections).
    pub reason: Option<String>,
    /// Timestamp.
    pub attested_at: DateTime<Utc>,
    /// Signature (would be real ed25519 sig in production).
    pub signature: Vec<u8>,
}

impl Attestation {
    /// Create a new attestation.
    pub fn new(
        validator_id: ValidatorId,
        decision_id: impl Into<String>,
        attestation_type: AttestationType,
        reason: Option<String>,
    ) -> Self {
        Self {
            validator_id,
            decision_id: decision_id.into(),
            attestation_type,
            reason,
            attested_at: Utc::now(),
            signature: vec![0u8; 64], // Would be actual signature
        }
    }

    /// Compute attestation hash.
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.validator_id.0.as_bytes());
        hasher.update(self.decision_id.as_bytes());
        hasher.update(&[self.attestation_type as u8]);
        hasher.update(self.attested_at.timestamp().to_le_bytes());
        hasher.finalize().into()
    }
}

/// Consensus parameters.
#[derive(Debug, Clone)]
pub struct ConsensusParams {
    /// Minimum number of validators.
    pub min_validators: usize,
    /// Required attestation weight (fraction).
    pub consensus_threshold: f64,
    /// Attestation timeout in seconds.
    pub attestation_timeout_secs: u64,
    /// Slash fraction for downtime.
    pub slash_fraction_downtime: f64,
    /// Slash fraction for false attestation.
    pub slash_fraction_false_attestation: f64,
    /// Slash fraction for collusion.
    pub slash_fraction_collusion: f64,
    /// Jail duration in seconds.
    pub jail_duration_secs: u64,
    /// Exit delay in seconds.
    pub exit_delay_secs: u64,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            min_validators: 5,
            consensus_threshold: 0.67,
            attestation_timeout_secs: 30,
            slash_fraction_downtime: 0.01,
            slash_fraction_false_attestation: 0.10,
            slash_fraction_collusion: 0.50,
            jail_duration_secs: 86400,      // 1 day
            exit_delay_secs: 604800,        // 7 days
        }
    }
}

/// The validator network.
#[derive(Debug)]
pub struct ValidatorNetwork {
    /// All validators.
    validators: HashMap<ValidatorId, Validator>,
    /// Pending attestations per decision.
    pending_attestations: HashMap<String, Vec<Attestation>>,
    /// Consensus parameters.
    params: ConsensusParams,
    /// Minimum stake required.
    min_stake: u64,
}

impl ValidatorNetwork {
    /// Default minimum stake.
    pub const DEFAULT_MIN_STAKE: u64 = 10000;

    /// Create a new validator network.
    pub fn new() -> Self {
        Self::with_params(ConsensusParams::default())
    }

    /// Create with custom parameters.
    pub fn with_params(params: ConsensusParams) -> Self {
        Self {
            validators: HashMap::new(),
            pending_attestations: HashMap::new(),
            params,
            min_stake: Self::DEFAULT_MIN_STAKE,
        }
    }

    /// Register a new validator.
    pub fn register_validator(&mut self, validator: Validator) -> TelosResult<()> {
        if validator.stake < self.min_stake {
            return Err(TelosError::InsufficientStake {
                validator: validator.id.0.clone(),
                stake: validator.stake,
                minimum: self.min_stake,
            });
        }

        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }

    /// Get active validators.
    pub fn active_validators(&self) -> impl Iterator<Item = &Validator> {
        self.validators.values().filter(|v| v.status == ValidatorStatus::Active)
    }

    /// Get total active stake.
    pub fn total_active_stake(&self) -> u64 {
        self.active_validators().map(|v| v.stake).sum()
    }

    /// Submit an attestation.
    pub fn submit_attestation(&mut self, attestation: Attestation) -> TelosResult<()> {
        let validator = self.validators.get(&attestation.validator_id)
            .ok_or_else(|| TelosError::ValidatorNotFound(attestation.validator_id.0.clone()))?;

        if validator.status != ValidatorStatus::Active {
            return Err(TelosError::ValidatorNotFound(attestation.validator_id.0.clone()));
        }

        self.pending_attestations
            .entry(attestation.decision_id.clone())
            .or_default()
            .push(attestation);

        Ok(())
    }

    /// Check if consensus is reached for a decision.
    pub fn check_consensus(&self, decision_id: &str) -> Option<bool> {
        let attestations = self.pending_attestations.get(decision_id)?;
        let total_stake = self.total_active_stake();
        
        if total_stake == 0 {
            return None;
        }

        let mut approve_stake: u64 = 0;
        let mut reject_stake: u64 = 0;

        for attestation in attestations {
            if let Some(validator) = self.validators.get(&attestation.validator_id) {
                match attestation.attestation_type {
                    AttestationType::Approve => approve_stake += validator.stake,
                    AttestationType::Reject => reject_stake += validator.stake,
                }
            }
        }

        let approve_fraction = approve_stake as f64 / total_stake as f64;
        let reject_fraction = reject_stake as f64 / total_stake as f64;

        if approve_fraction >= self.params.consensus_threshold {
            Some(true) // Approved
        } else if reject_fraction > (1.0 - self.params.consensus_threshold) {
            Some(false) // Rejected
        } else {
            None // No consensus yet
        }
    }

    /// Finalize attestations for a decision.
    pub fn finalize_attestations(&mut self, decision_id: &str) -> Vec<Attestation> {
        self.pending_attestations.remove(decision_id).unwrap_or_default()
    }

    /// Slash a validator.
    pub fn slash(&mut self, validator_id: &ValidatorId, fraction: f64, reason: &str) -> TelosResult<u64> {
        let validator = self.validators.get_mut(validator_id)
            .ok_or_else(|| TelosError::ValidatorNotFound(validator_id.0.clone()))?;

        let slash_amount = (validator.stake as f64 * fraction) as u64;
        validator.stake = validator.stake.saturating_sub(slash_amount);
        
        // If stake falls below minimum, mark as slashed
        if validator.stake < self.min_stake {
            validator.status = ValidatorStatus::Slashed;
        }

        Ok(slash_amount)
    }

    /// Jail a validator.
    pub fn jail(&mut self, validator_id: &ValidatorId) -> TelosResult<()> {
        let validator = self.validators.get_mut(validator_id)
            .ok_or_else(|| TelosError::ValidatorNotFound(validator_id.0.clone()))?;
        validator.status = ValidatorStatus::Jailed;
        Ok(())
    }

    /// Unjail a validator.
    pub fn unjail(&mut self, validator_id: &ValidatorId) -> TelosResult<()> {
        let validator = self.validators.get_mut(validator_id)
            .ok_or_else(|| TelosError::ValidatorNotFound(validator_id.0.clone()))?;
        if validator.status == ValidatorStatus::Jailed {
            validator.status = ValidatorStatus::Active;
        }
        Ok(())
    }

    /// Get validator by ID.
    pub fn get_validator(&self, validator_id: &ValidatorId) -> Option<&Validator> {
        self.validators.get(validator_id)
    }

    /// Get parameters.
    pub fn params(&self) -> &ConsensusParams {
        &self.params
    }
}

impl Default for ValidatorNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator(id: &str, stake: u64) -> Validator {
        Validator::new(id, [0u8; 32], stake)
    }

    #[test]
    fn test_register_validator() {
        let mut network = ValidatorNetwork::new();
        let validator = create_test_validator("v1", 20000);
        
        assert!(network.register_validator(validator).is_ok());
        assert_eq!(network.active_validators().count(), 1);
    }

    #[test]
    fn test_insufficient_stake() {
        let mut network = ValidatorNetwork::new();
        let validator = create_test_validator("v1", 100); // Below minimum
        
        let result = network.register_validator(validator);
        assert!(matches!(result, Err(TelosError::InsufficientStake { .. })));
    }

    #[test]
    fn test_consensus_reached() {
        let mut network = ValidatorNetwork::new();
        
        // Register validators with different stakes
        network.register_validator(create_test_validator("v1", 30000)).unwrap();
        network.register_validator(create_test_validator("v2", 20000)).unwrap();
        network.register_validator(create_test_validator("v3", 50000)).unwrap();
        // Total: 100000. Need 67% = 67000

        // Submit attestations
        network.submit_attestation(Attestation::new(
            ValidatorId::new("v1"),
            "decision-1",
            AttestationType::Approve,
            None,
        )).unwrap();
        
        network.submit_attestation(Attestation::new(
            ValidatorId::new("v3"),
            "decision-1",
            AttestationType::Approve,
            None,
        )).unwrap();
        // 30000 + 50000 = 80000 > 67000

        let consensus = network.check_consensus("decision-1");
        assert_eq!(consensus, Some(true));
    }

    #[test]
    fn test_slashing() {
        let mut network = ValidatorNetwork::new();
        network.register_validator(create_test_validator("v1", 50000)).unwrap();
        
        let slashed = network.slash(&ValidatorId::new("v1"), 0.10, "false attestation").unwrap();
        assert_eq!(slashed, 5000);
        
        let validator = network.get_validator(&ValidatorId::new("v1")).unwrap();
        assert_eq!(validator.stake, 45000);
    }

    #[test]
    fn test_reputation_update() {
        let mut validator = create_test_validator("v1", 20000);
        
        // Start at 0.5
        assert!((validator.reputation - 0.5).abs() < 0.01);
        
        // Correct attestation increases rep
        validator.update_reputation(true);
        assert!(validator.reputation > 0.5);
        
        // Incorrect decreases
        validator.update_reputation(false);
        // Should be slightly above or at 0.5 due to the correct one
    }
}
