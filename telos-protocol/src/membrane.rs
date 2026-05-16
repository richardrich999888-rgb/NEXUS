//! Layer 1: Commitment Membrane
//!
//! The boundary between reversible reasoning and irreversible action.
//! Crossing the membrane requires entropy, authority, and validation.

use crate::entropy::{EntropyMeter, EntropyProof, ConsequenceTier};
use crate::authority::{AuthorityRegistry, AgentId};
use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// A decision that may cross the commitment membrane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique decision identifier.
    pub id: String,
    /// Decision domain (hierarchical namespace).
    pub domain: String,
    /// Action type within the domain.
    pub action: String,
    /// Action parameters.
    pub parameters: HashMap<String, serde_json::Value>,
    /// Consequence tier of this decision.
    pub consequence_tier: ConsequenceTier,
    /// Hash of reasoning trace (optional, for audit).
    pub reasoning_trace_hash: Option<[u8; 32]>,
    /// When the decision was created.
    pub created_at: DateTime<Utc>,
    /// Current state.
    pub state: DecisionState,
}

impl Decision {
    /// Create a new decision in DRAFT state.
    pub fn new(domain: impl Into<String>, action: impl Into<String>, tier: ConsequenceTier) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            domain: domain.into(),
            action: action.into(),
            parameters: HashMap::new(),
            consequence_tier: tier,
            reasoning_trace_hash: None,
            created_at: Utc::now(),
            state: DecisionState::Draft,
        }
    }

    /// Add a parameter to the decision.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.parameters.insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }

    /// Attach a reasoning trace hash.
    pub fn with_reasoning_trace(mut self, hash: [u8; 32]) -> Self {
        self.reasoning_trace_hash = Some(hash);
        self
    }

    /// Compute the decision hash.
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.domain.as_bytes());
        hasher.update(self.action.as_bytes());
        hasher.update(format!("{:?}", self.parameters).as_bytes());
        hasher.update(&[self.consequence_tier as u8]);
        hasher.finalize().into()
    }
}

/// State of a decision in the commitment flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionState {
    /// In reversible zone, can be modified.
    Draft,
    /// Submitted, awaiting validation.
    Pending,
    /// Being validated by validators.
    Validating,
    /// Rejected, returned to reversible zone.
    Rejected,
    /// Committed, irreversible.
    Committed,
}

/// Result of attempting to cross the membrane.
#[derive(Debug, Clone)]
pub enum CrossingResult {
    /// Successfully committed.
    Committed {
        decision_id: String,
        committed_at: DateTime<Utc>,
        entropy_consumed: u64,
        commitment_hash: [u8; 32],
    },
    /// Rejected with reason.
    Rejected {
        decision_id: String,
        reason: String,
    },
    /// Needs more attestations.
    PendingValidation {
        decision_id: String,
        attestations_received: usize,
        attestations_needed: usize,
    },
}

/// A commitment proof (generated after successful crossing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentProof {
    /// Unique commitment identifier.
    pub commitment_id: String,
    /// Reference to the decision.
    pub decision_id: String,
    /// Hash of the committed decision.
    pub decision_hash: [u8; 32],
    /// Entropy consumed.
    pub entropy_consumed: u64,
    /// Entropy proof.
    pub entropy_proof_hash: [u8; 32],
    /// Authority chain (root → leaf).
    pub authority_chain: Vec<String>,
    /// Validator attestation hashes.
    pub attestation_hashes: Vec<[u8; 32]>,
    /// Commitment timestamp.
    pub committed_at: DateTime<Utc>,
    /// Ledger position.
    pub ledger_index: u64,
}

/// The commitment membrane: the protocol boundary.
#[derive(Debug)]
pub struct CommitmentMembrane {
    /// Decisions in the reversible zone.
    reversible_zone: HashMap<String, Decision>,
    /// Commitment history (irreversible zone).
    commitment_history: Vec<CommitmentProof>,
    /// Next ledger index.
    next_ledger_index: u64,
    /// Circuit breaker state.
    circuit_breaker_active: bool,
    /// Circuit breaker reason.
    circuit_breaker_reason: Option<String>,
}

impl CommitmentMembrane {
    /// Create a new commitment membrane.
    pub fn new() -> Self {
        Self {
            reversible_zone: HashMap::new(),
            commitment_history: Vec::new(),
            next_ledger_index: 0,
            circuit_breaker_active: false,
            circuit_breaker_reason: None,
        }
    }

    /// Add a decision to the reversible zone.
    pub fn add_decision(&mut self, decision: Decision) -> TelosResult<String> {
        if self.circuit_breaker_active {
            return Err(TelosError::CircuitBreakerActive(
                self.circuit_breaker_reason.clone().unwrap_or_default()
            ));
        }

        let id = decision.id.clone();
        self.reversible_zone.insert(id.clone(), decision);
        Ok(id)
    }

    /// Modify a decision in the reversible zone.
    pub fn modify_decision<F>(&mut self, decision_id: &str, modifier: F) -> TelosResult<()>
    where
        F: FnOnce(&mut Decision),
    {
        let decision = self.reversible_zone.get_mut(decision_id)
            .ok_or_else(|| TelosError::NotCommitted(decision_id.to_string()))?;

        if decision.state == DecisionState::Committed {
            return Err(TelosError::AlreadyCommitted(decision_id.to_string()));
        }

        modifier(decision);
        Ok(())
    }

    /// Request crossing from reversible to irreversible zone.
    pub fn request_crossing(
        &mut self,
        decision_id: &str,
        agent_id: &AgentId,
        entropy_meter: &mut EntropyMeter,
        entropy_proof: EntropyProof,
        authority_registry: &AuthorityRegistry,
        trust_score: f64,
    ) -> TelosResult<CrossingResult> {
        // Check circuit breaker
        if self.circuit_breaker_active {
            return Err(TelosError::CircuitBreakerActive(
                self.circuit_breaker_reason.clone().unwrap_or_default()
            ));
        }

        // Get the decision
        let decision = self.reversible_zone.get_mut(decision_id)
            .ok_or_else(|| TelosError::NotCommitted(decision_id.to_string()))?;

        // Verify authority
        let authority = authority_registry.verify_authority(
            agent_id,
            &decision.domain,
            decision.consequence_tier,
        )?;

        // Consume entropy
        let entropy_consumed = entropy_meter.consume(
            decision.consequence_tier,
            trust_score,
            entropy_proof.clone(),
        )?;

        // Mark as pending validation
        decision.state = DecisionState::Validating;

        // In a full implementation, we would wait for validator attestations here.
        // For now, we simulate immediate commitment (single-node mode).
        
        // Create commitment proof
        let commitment_id = uuid::Uuid::new_v4().to_string();
        let decision_hash = decision.hash();
        let committed_at = Utc::now();

        let proof = CommitmentProof {
            commitment_id: commitment_id.clone(),
            decision_id: decision_id.to_string(),
            decision_hash,
            entropy_consumed,
            entropy_proof_hash: entropy_proof.proof_hash,
            authority_chain: authority.delegation_chain.clone(),
            attestation_hashes: Vec::new(), // Would be filled by validators
            committed_at,
            ledger_index: self.next_ledger_index,
        };

        // Move to irreversible zone
        decision.state = DecisionState::Committed;
        let committed_decision = self.reversible_zone.remove(decision_id).unwrap();
        
        self.commitment_history.push(proof);
        self.next_ledger_index += 1;

        // Compute commitment hash
        let mut hasher = Sha256::new();
        hasher.update(&decision_hash);
        hasher.update(entropy_consumed.to_le_bytes());
        hasher.update(committed_at.timestamp().to_le_bytes());
        let commitment_hash: [u8; 32] = hasher.finalize().into();

        Ok(CrossingResult::Committed {
            decision_id: decision_id.to_string(),
            committed_at,
            entropy_consumed,
            commitment_hash,
        })
    }

    /// Activate circuit breaker (emergency halt).
    pub fn activate_circuit_breaker(&mut self, reason: impl Into<String>) {
        self.circuit_breaker_active = true;
        self.circuit_breaker_reason = Some(reason.into());
    }

    /// Deactivate circuit breaker.
    pub fn deactivate_circuit_breaker(&mut self) {
        self.circuit_breaker_active = false;
        self.circuit_breaker_reason = None;
    }

    /// Get commitment history.
    pub fn commitment_history(&self) -> &[CommitmentProof] {
        &self.commitment_history
    }

    /// Get a commitment proof by ID.
    pub fn get_commitment(&self, commitment_id: &str) -> Option<&CommitmentProof> {
        self.commitment_history.iter().find(|p| p.commitment_id == commitment_id)
    }

    /// Get decisions in reversible zone.
    pub fn reversible_decisions(&self) -> impl Iterator<Item = &Decision> {
        self.reversible_zone.values()
    }
}

impl Default for CommitmentMembrane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_creation() {
        let decision = Decision::new("finance.trading", "execute_trade", ConsequenceTier::High)
            .with_param("symbol", "AAPL")
            .with_param("quantity", 100);

        assert_eq!(decision.domain, "finance.trading");
        assert_eq!(decision.action, "execute_trade");
        assert_eq!(decision.state, DecisionState::Draft);
        assert!(decision.parameters.contains_key("symbol"));
    }

    #[test]
    fn test_membrane_add_decision() {
        let mut membrane = CommitmentMembrane::new();
        let decision = Decision::new("test.domain", "test_action", ConsequenceTier::Minimal);
        
        let id = membrane.add_decision(decision).unwrap();
        assert!(!id.is_empty());
        assert_eq!(membrane.reversible_zone.len(), 1);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut membrane = CommitmentMembrane::new();
        membrane.activate_circuit_breaker("Emergency");
        
        let decision = Decision::new("test", "action", ConsequenceTier::Minimal);
        let result = membrane.add_decision(decision);
        
        assert!(matches!(result, Err(TelosError::CircuitBreakerActive(_))));
    }

    #[test]
    fn test_decision_hash_deterministic() {
        let d1 = Decision::new("domain", "action", ConsequenceTier::Low);
        let d2 = Decision {
            id: d1.id.clone(),
            domain: "domain".into(),
            action: "action".into(),
            parameters: HashMap::new(),
            consequence_tier: ConsequenceTier::Low,
            reasoning_trace_hash: None,
            created_at: d1.created_at,
            state: DecisionState::Draft,
        };
        
        assert_eq!(d1.hash(), d2.hash());
    }
}
