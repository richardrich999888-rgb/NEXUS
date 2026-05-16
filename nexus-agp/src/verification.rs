//! # NEXUS Verification Integration
//!
//! Integrates AGP tiered verification with NEXUS proof infrastructure.
//!
//! - OPTIMISTIC: NEXUS dispute mechanism
//! - TEE: NEXUS TEE attestation
//! - ZKML: NEXUS zkML prover

use crate::identity::AgentFingerprint;
use crate::reputation::ReputationCRDT;
use serde::{Serialize, Deserialize};

/// Verification tier (matches AGP Python implementation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationTier {
    Optimistic,
    Tee,
    Zkml,
}

/// Risk assessment for tier selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub value_risk: f64,
    pub reputation_risk: f64,
    pub sensitivity_risk: f64,
    pub combined_risk: f64,
}

impl RiskAssessment {
    /// Compute risk from task parameters
    pub fn compute(
        stake_at_risk: u64,
        total_network_stake: u64,
        agent_reputation: f64,
        task_sensitivity: f64,
    ) -> Self {
        let value_risk = if total_network_stake > 0 {
            (stake_at_risk as f64 / total_network_stake as f64 * 10.0).min(1.0)
        } else {
            0.5
        };
        
        let reputation_risk = 1.0 - agent_reputation;
        let sensitivity_risk = task_sensitivity;
        
        let weighted = 0.4 * value_risk + 0.3 * reputation_risk + 0.3 * sensitivity_risk;
        let combined = weighted.max(
            [value_risk, reputation_risk, sensitivity_risk]
                .into_iter()
                .fold(0.0_f64, |a, b| a.max(b)) * 0.9
        ).min(1.0);
        
        Self {
            value_risk,
            reputation_risk,
            sensitivity_risk,
            combined_risk: combined,
        }
    }
}

/// Verification decision with parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDecision {
    pub tier: VerificationTier,
    pub risk: RiskAssessment,
    pub estimated_cost: u64,
    pub reasoning: String,
}

/// Select verification tier based on risk
pub fn select_tier(risk: &RiskAssessment) -> (VerificationTier, String) {
    const ZKML_THRESHOLD: f64 = 0.8;
    const TEE_THRESHOLD: f64 = 0.4;
    
    if risk.combined_risk >= ZKML_THRESHOLD {
        (VerificationTier::Zkml, "High risk requires zkML proof".to_string())
    } else if risk.combined_risk >= TEE_THRESHOLD {
        (VerificationTier::Tee, "Medium risk uses TEE attestation".to_string())
    } else {
        (VerificationTier::Optimistic, "Low risk allows optimistic verification".to_string())
    }
}

/// Verification result from NEXUS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verified: bool,
    pub tier_used: VerificationTier,
    pub proof_bytes: Vec<u8>,
    pub timestamp: u64,
}

/// NEXUS verifier that integrates with executor
pub struct NexusVerifier {
    /// Network total stake (for risk calculation)
    pub total_stake: u64,
}

impl NexusVerifier {
    pub fn new(total_stake: u64) -> Self {
        Self { total_stake }
    }
    
    /// Determine verification requirements for a task
    pub fn decide(
        &self,
        stake_at_risk: u64,
        agent_reputation: f64,
        task_sensitivity: f64,
    ) -> VerificationDecision {
        let risk = RiskAssessment::compute(
            stake_at_risk,
            self.total_stake,
            agent_reputation,
            task_sensitivity,
        );
        
        let (tier, reasoning) = select_tier(&risk);
        
        let estimated_cost = match tier {
            VerificationTier::Zkml => 10000,
            VerificationTier::Tee => 1000,
            VerificationTier::Optimistic => 100,
        };
        
        VerificationDecision {
            tier,
            risk,
            estimated_cost,
            reasoning,
        }
    }
    
    /// Verify task output (mock implementation)
    pub fn verify(
        &self,
        decision: &VerificationDecision,
        _task_output: &[u8],
    ) -> VerificationResult {
        // In production: call NEXUS executor
        VerificationResult {
            verified: true,
            tier_used: decision.tier,
            proof_bytes: vec![0u8; 32], // Mock proof
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_tiers() {
        let verifier = NexusVerifier::new(1_000_000);
        
        // Low risk
        let low = verifier.decide(100, 0.95, 0.2);
        assert_eq!(low.tier, VerificationTier::Optimistic);
        
        // Medium risk
        let med = verifier.decide(10_000, 0.7, 0.5);
        assert_eq!(med.tier, VerificationTier::Tee);
        
        // High risk
        let high = verifier.decide(500_000, 0.3, 0.95);
        assert_eq!(high.tier, VerificationTier::Zkml);
    }
}
