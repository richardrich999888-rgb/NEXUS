//! Layer 5: Trust Accumulator
//!
//! Manages unforkable trust scores anchored to entropy history.
//! Trust cannot be purchased, only earned through correct behavior over time.

use crate::authority::AgentId;
use crate::membrane::CommitmentProof;
use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Trust score for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    /// Agent this score belongs to.
    pub agent_id: AgentId,
    /// Overall trust score (0.0 - 1.0).
    pub score: f64,
    /// Commitment accuracy rate.
    pub commitment_accuracy: f64,
    /// Total entropy expended.
    pub total_entropy_expended: u64,
    /// Constraint violation count.
    pub violation_count: u64,
    /// Time in protocol (seconds).
    pub tenure_seconds: u64,
    /// Number of successful commitments.
    pub successful_commitments: u64,
    /// Number of rejected commitments.
    pub rejected_commitments: u64,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Merkle root of commitment history.
    pub history_root: [u8; 32],
}

impl TrustScore {
    /// Create a new trust score for an agent.
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            score: 0.0, // Start at zero - must be earned
            commitment_accuracy: 0.0,
            total_entropy_expended: 0,
            violation_count: 0,
            tenure_seconds: 0,
            successful_commitments: 0,
            rejected_commitments: 0,
            updated_at: Utc::now(),
            history_root: [0u8; 32],
        }
    }

    /// Compute the trust score from components.
    pub fn compute_score(&mut self) {
        // Weighted combination of factors
        let accuracy_weight = 0.4;
        let tenure_weight = 0.2;
        let entropy_weight = 0.2;
        let violation_weight = 0.2;

        // Accuracy component (0-1)
        let accuracy = self.commitment_accuracy;

        // Tenure component (logarithmic, maxes out around 1 year)
        let tenure_days = (self.tenure_seconds as f64) / 86400.0;
        let tenure_score = (tenure_days.ln() / 365.0_f64.ln()).clamp(0.0, 1.0);

        // Entropy component (logarithmic)
        let entropy_score = if self.total_entropy_expended > 0 {
            ((self.total_entropy_expended as f64).ln() / 20.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Violation penalty
        let violation_penalty = if self.successful_commitments > 0 {
            1.0 - (self.violation_count as f64 / self.successful_commitments as f64).min(1.0)
        } else {
            0.0
        };

        self.score = (accuracy_weight * accuracy)
            + (tenure_weight * tenure_score)
            + (entropy_weight * entropy_score)
            + (violation_weight * violation_penalty);

        self.score = self.score.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
    }
}

/// A record in the commitment history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentRecord {
    /// Commitment ID.
    pub commitment_id: String,
    /// Decision hash.
    pub decision_hash: [u8; 32],
    /// Entropy consumed.
    pub entropy_consumed: u64,
    /// Whether the commitment was successful.
    pub successful: bool,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Hash link to previous record.
    pub prev_hash: [u8; 32],
}

impl CommitmentRecord {
    /// Compute the record hash.
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.commitment_id.as_bytes());
        hasher.update(&self.decision_hash);
        hasher.update(self.entropy_consumed.to_le_bytes());
        hasher.update(&[if self.successful { 1 } else { 0 }]);
        hasher.update(self.timestamp.timestamp().to_le_bytes());
        hasher.update(&self.prev_hash);
        hasher.finalize().into()
    }
}

/// Complete commitment history for an agent.
#[derive(Debug, Clone, Default)]
pub struct CommitmentHistory {
    /// Ordered list of commitment records.
    records: Vec<CommitmentRecord>,
    /// Current Merkle root.
    root_hash: [u8; 32],
}

impl CommitmentHistory {
    /// Create a new empty history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a commitment record.
    pub fn add(&mut self, commitment_id: String, decision_hash: [u8; 32], entropy: u64, successful: bool) {
        let prev_hash = if let Some(last) = self.records.last() {
            last.hash()
        } else {
            [0u8; 32]
        };

        let record = CommitmentRecord {
            commitment_id,
            decision_hash,
            entropy_consumed: entropy,
            successful,
            timestamp: Utc::now(),
            prev_hash,
        };

        let record_hash = record.hash();
        self.records.push(record);
        self.update_root();
    }

    /// Update the Merkle root.
    fn update_root(&mut self) {
        if self.records.is_empty() {
            self.root_hash = [0u8; 32];
            return;
        }

        // Simple Merkle tree (pairs of hashes)
        let mut layer: Vec<[u8; 32]> = self.records.iter().map(|r| r.hash()).collect();
        
        while layer.len() > 1 {
            let mut next_layer = Vec::new();
            for chunk in layer.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // Duplicate if odd
                }
                next_layer.push(hasher.finalize().into());
            }
            layer = next_layer;
        }

        self.root_hash = layer[0];
    }

    /// Get the Merkle root.
    pub fn root(&self) -> [u8; 32] {
        self.root_hash
    }

    /// Get the number of records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Get total entropy expended.
    pub fn total_entropy(&self) -> u64 {
        self.records.iter().map(|r| r.entropy_consumed).sum()
    }

    /// Get success count.
    pub fn success_count(&self) -> u64 {
        self.records.iter().filter(|r| r.successful).count() as u64
    }

    /// Verify chain integrity.
    pub fn verify_integrity(&self) -> bool {
        if self.records.is_empty() {
            return true;
        }

        // First record should have zero prev_hash
        if self.records[0].prev_hash != [0u8; 32] {
            return false;
        }

        // Each record's prev_hash should match the previous record's hash
        for window in self.records.windows(2) {
            let expected = window[0].hash();
            if window[1].prev_hash != expected {
                return false;
            }
        }

        true
    }
}

/// The trust accumulator.
#[derive(Debug, Default)]
pub struct TrustAccumulator {
    /// Trust scores per agent.
    scores: HashMap<AgentId, TrustScore>,
    /// Commitment histories per agent.
    histories: HashMap<AgentId, CommitmentHistory>,
    /// Registration timestamps.
    registrations: HashMap<AgentId, DateTime<Utc>>,
}

impl TrustAccumulator {
    /// Create a new trust accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an agent.
    pub fn register_agent(&mut self, agent_id: AgentId) {
        if !self.scores.contains_key(&agent_id) {
            self.scores.insert(agent_id.clone(), TrustScore::new(agent_id.clone()));
            self.histories.insert(agent_id.clone(), CommitmentHistory::new());
            self.registrations.insert(agent_id, Utc::now());
        }
    }

    /// Record a commitment.
    pub fn record_commitment(
        &mut self,
        agent_id: &AgentId,
        commitment_id: String,
        decision_hash: [u8; 32],
        entropy_consumed: u64,
        successful: bool,
    ) -> TelosResult<()> {
        let history = self.histories.get_mut(agent_id)
            .ok_or_else(|| TelosError::AgentNotFound(agent_id.0.clone()))?;

        history.add(commitment_id, decision_hash, entropy_consumed, successful);

        // Update trust score
        let score = self.scores.get_mut(agent_id).unwrap();
        score.total_entropy_expended = history.total_entropy();
        
        if successful {
            score.successful_commitments += 1;
        } else {
            score.rejected_commitments += 1;
        }

        // Update accuracy
        let total = score.successful_commitments + score.rejected_commitments;
        if total > 0 {
            score.commitment_accuracy = score.successful_commitments as f64 / total as f64;
        }

        // Update tenure
        if let Some(reg_time) = self.registrations.get(agent_id) {
            score.tenure_seconds = (Utc::now() - *reg_time).num_seconds().max(0) as u64;
        }

        // Update history root
        score.history_root = history.root();

        // Recompute score
        score.compute_score();

        Ok(())
    }

    /// Record a violation.
    pub fn record_violation(&mut self, agent_id: &AgentId) -> TelosResult<()> {
        let score = self.scores.get_mut(agent_id)
            .ok_or_else(|| TelosError::AgentNotFound(agent_id.0.clone()))?;
        
        score.violation_count += 1;
        score.compute_score();
        
        Ok(())
    }

    /// Get trust score for an agent.
    pub fn get_trust_score(&self, agent_id: &AgentId) -> Option<&TrustScore> {
        self.scores.get(agent_id)
    }

    /// Get commitment history for an agent.
    pub fn get_history(&self, agent_id: &AgentId) -> Option<&CommitmentHistory> {
        self.histories.get(agent_id)
    }

    /// Verify trust proof (Merkle inclusion).
    pub fn verify_trust_proof(&self, agent_id: &AgentId, claimed_root: [u8; 32]) -> bool {
        if let Some(history) = self.histories.get(agent_id) {
            history.root() == claimed_root
        } else {
            false
        }
    }

    /// Demonstrate why trust is non-transferable.
    /// 
    /// This returns an error because trust is earned through:
    /// 1. Time in protocol (tenure)
    /// 2. Entropy expenditure (cannot be faked)
    /// 3. Historical behavior (commitment accuracy)
    pub fn transfer_trust(&self, _from: &AgentId, _to: &AgentId) -> TelosResult<()> {
        Err(TelosError::TrustNonTransferable)
    }

    /// Compute what trust a fork would have.
    /// 
    /// A forked network starts fresh with:
    /// - Zero tenure (time restarts)
    /// - Zero entropy history (not anchored)
    /// - Zero commitments (no history)
    pub fn fork_trust_value(&self) -> TrustScore {
        TrustScore {
            agent_id: AgentId::new("fork"),
            score: 0.0,
            commitment_accuracy: 0.0,
            total_entropy_expended: 0,
            violation_count: 0,
            tenure_seconds: 0,
            successful_commitments: 0,
            rejected_commitments: 0,
            updated_at: Utc::now(),
            history_root: [0u8; 32],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_score_new() {
        let score = TrustScore::new(AgentId::new("agent-1"));
        assert_eq!(score.score, 0.0);
        assert_eq!(score.successful_commitments, 0);
    }

    #[test]
    fn test_commitment_history_integrity() {
        let mut history = CommitmentHistory::new();
        
        history.add("c1".into(), [1u8; 32], 1000, true);
        history.add("c2".into(), [2u8; 32], 2000, true);
        history.add("c3".into(), [3u8; 32], 500, false);

        assert!(history.verify_integrity());
        assert_eq!(history.len(), 3);
        assert_eq!(history.total_entropy(), 3500);
        assert_eq!(history.success_count(), 2);
    }

    #[test]
    fn test_trust_accumulation() {
        let mut accumulator = TrustAccumulator::new();
        let agent = AgentId::new("agent-1");

        accumulator.register_agent(agent.clone());

        // Record some successful commitments
        for i in 0..10 {
            accumulator.record_commitment(
                &agent,
                format!("commit-{}", i),
                [i as u8; 32],
                1000,
                true,
            ).unwrap();
        }

        let score = accumulator.get_trust_score(&agent).unwrap();
        assert!(score.score > 0.0);
        assert_eq!(score.successful_commitments, 10);
        assert_eq!(score.total_entropy_expended, 10000);
    }

    #[test]
    fn test_trust_non_transferable() {
        let accumulator = TrustAccumulator::new();
        let agent1 = AgentId::new("agent-1");
        let agent2 = AgentId::new("agent-2");

        let result = accumulator.transfer_trust(&agent1, &agent2);
        assert!(matches!(result, Err(TelosError::TrustNonTransferable)));
    }

    #[test]
    fn test_fork_has_zero_trust() {
        let accumulator = TrustAccumulator::new();
        let fork_trust = accumulator.fork_trust_value();

        assert_eq!(fork_trust.score, 0.0);
        assert_eq!(fork_trust.total_entropy_expended, 0);
        assert_eq!(fork_trust.tenure_seconds, 0);
    }

    #[test]
    fn test_violation_decreases_trust() {
        let mut accumulator = TrustAccumulator::new();
        let agent = AgentId::new("agent-1");

        accumulator.register_agent(agent.clone());

        // Build some trust
        for i in 0..5 {
            accumulator.record_commitment(&agent, format!("c{}", i), [0u8; 32], 1000, true).unwrap();
        }

        let score_before = accumulator.get_trust_score(&agent).unwrap().score;

        // Record violation
        accumulator.record_violation(&agent).unwrap();

        let score_after = accumulator.get_trust_score(&agent).unwrap().score;
        assert!(score_after < score_before);
    }
}
