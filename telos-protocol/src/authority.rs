//! Layer 3: Authority Registry
//!
//! Manages decision rights with delegation chains and additive constraints.
//! Authority can only be attenuated, never amplified, through delegation.

use crate::entropy::ConsequenceTier;
use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

/// Unique agent identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Decision domain specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionDomain {
    /// Domain identifier (hierarchical, e.g., "finance.trading.equity").
    pub domain_id: String,
    /// Maximum consequence tier allowed.
    pub max_tier: ConsequenceTier,
}

impl DecisionDomain {
    pub fn new(domain_id: impl Into<String>, max_tier: ConsequenceTier) -> Self {
        Self {
            domain_id: domain_id.into(),
            max_tier,
        }
    }

    /// Check if this domain covers the given domain and tier.
    pub fn covers(&self, domain: &str, tier: ConsequenceTier) -> bool {
        // Check domain hierarchy (parent covers children)
        let is_parent = domain.starts_with(&self.domain_id) || 
                        self.domain_id == "*"; // Wildcard
        
        // Check tier
        let tier_allowed = tier <= self.max_tier;
        
        is_parent && tier_allowed
    }
}

/// A constraint predicate that must be satisfied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// Time-based constraint.
    Temporal {
        /// Allowed hours (0-23).
        allowed_hours: Option<(u8, u8)>,
        /// Blackout periods.
        blackout_periods: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    },
    /// Rate limit constraint.
    RateLimit {
        /// Maximum commitments per window.
        max_commitments: u32,
        /// Window duration in seconds.
        window_seconds: u64,
    },
    /// Resource constraint.
    Resource {
        /// Maximum entropy per commitment.
        max_entropy_per_commitment: u64,
    },
    /// Approval required from other agents.
    Approval {
        /// Required approvers.
        required_approvers: Vec<AgentId>,
        /// Minimum approvals needed.
        min_approvals: u32,
    },
}

impl Constraint {
    /// Check if constraint is satisfied (simplified).
    pub fn is_satisfied(&self, _context: &ConstraintContext) -> bool {
        // In a full implementation, each constraint type would have its evaluation logic.
        // For now, we return true (placeholder).
        true
    }
}

/// Context for constraint evaluation.
#[derive(Debug, Default)]
pub struct ConstraintContext {
    pub current_time: DateTime<Utc>,
    pub recent_commitments: u32,
    pub entropy_offered: u64,
    pub approvals: Vec<AgentId>,
}

/// Authority record in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authority {
    /// Unique authority identifier.
    pub authority_id: String,
    /// Agent holding this authority.
    pub holder: AgentId,
    /// Decision domains covered.
    pub domains: Vec<DecisionDomain>,
    /// Constraints that must be satisfied.
    pub constraints: Vec<Constraint>,
    /// Entropy budget allocated.
    pub entropy_budget: u64,
    /// Delegated from (None = root authority).
    pub delegated_from: Option<String>,
    /// Validity period.
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Current status.
    pub status: AuthorityStatus,
    /// Delegation chain (cached).
    pub delegation_chain: Vec<String>,
}

/// Authority status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityStatus {
    Active,
    Suspended,
    Revoked,
    Expired,
}

/// Authority verification result.
#[derive(Debug, Clone)]
pub struct AuthorityVerification {
    pub authority_id: String,
    pub holder: AgentId,
    pub delegation_chain: Vec<String>,
    pub verified_at: DateTime<Utc>,
}

/// The authority registry.
#[derive(Debug, Default)]
pub struct AuthorityRegistry {
    /// All authority records.
    authorities: HashMap<String, Authority>,
    /// Index: agent → authority IDs.
    agent_index: HashMap<AgentId, Vec<String>>,
}

impl AuthorityRegistry {
    /// Create a new authority registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a root authority (no delegator).
    pub fn create_root_authority(
        &mut self,
        holder: AgentId,
        domains: Vec<DecisionDomain>,
        entropy_budget: u64,
        valid_days: i64,
    ) -> TelosResult<String> {
        let authority_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let authority = Authority {
            authority_id: authority_id.clone(),
            holder: holder.clone(),
            domains,
            constraints: Vec::new(),
            entropy_budget,
            delegated_from: None,
            valid_from: now,
            expires_at: now + chrono::Duration::days(valid_days),
            status: AuthorityStatus::Active,
            delegation_chain: vec![authority_id.clone()],
        };

        self.authorities.insert(authority_id.clone(), authority);
        self.agent_index
            .entry(holder)
            .or_default()
            .push(authority_id.clone());

        Ok(authority_id)
    }

    /// Delegate authority to another agent (with attenuation).
    pub fn delegate_authority(
        &mut self,
        delegator_authority_id: &str,
        delegate: AgentId,
        domains: Vec<DecisionDomain>,
        constraints: Vec<Constraint>,
        entropy_budget: u64,
        valid_days: i64,
    ) -> TelosResult<String> {
        // Get delegator's authority
        let delegator = self.authorities.get(delegator_authority_id)
            .ok_or_else(|| TelosError::BrokenAuthorityChain(delegator_authority_id.to_string()))?;

        // Verify delegator is active
        if delegator.status != AuthorityStatus::Active {
            return Err(TelosError::AuthorityRevoked(delegator.holder.0.clone()));
        }

        // Verify attenuation: delegated domains must be subset of delegator's
        for domain in &domains {
            let covered = delegator.domains.iter().any(|d| d.covers(&domain.domain_id, domain.max_tier));
            if !covered {
                return Err(TelosError::ConstraintViolation(
                    format!("Domain {} not covered by delegator", domain.domain_id)
                ));
            }
        }

        // Verify entropy budget attenuation
        if entropy_budget > delegator.entropy_budget {
            return Err(TelosError::ConstraintViolation(
                "Entropy budget cannot exceed delegator's".into()
            ));
        }

        // Build delegation chain
        let mut delegation_chain = delegator.delegation_chain.clone();
        let authority_id = uuid::Uuid::new_v4().to_string();
        delegation_chain.push(authority_id.clone());

        let now = Utc::now();

        // Inherit delegator's constraints + new ones
        let mut all_constraints = delegator.constraints.clone();
        all_constraints.extend(constraints);

        let authority = Authority {
            authority_id: authority_id.clone(),
            holder: delegate.clone(),
            domains,
            constraints: all_constraints,
            entropy_budget,
            delegated_from: Some(delegator_authority_id.to_string()),
            valid_from: now,
            expires_at: now + chrono::Duration::days(valid_days),
            status: AuthorityStatus::Active,
            delegation_chain,
        };

        self.authorities.insert(authority_id.clone(), authority);
        self.agent_index
            .entry(delegate)
            .or_default()
            .push(authority_id.clone());

        Ok(authority_id)
    }

    /// Revoke an authority (cascades to delegations).
    pub fn revoke_authority(&mut self, authority_id: &str, _reason: &str) -> TelosResult<Vec<String>> {
        let mut revoked = Vec::new();

        // Revoke this authority
        if let Some(authority) = self.authorities.get_mut(authority_id) {
            authority.status = AuthorityStatus::Revoked;
            revoked.push(authority_id.to_string());
        } else {
            return Err(TelosError::BrokenAuthorityChain(authority_id.to_string()));
        }

        // Find and revoke all downstream delegations
        let downstream: Vec<String> = self.authorities.iter()
            .filter(|(_, a)| a.delegation_chain.contains(&authority_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect();

        for id in downstream {
            if let Some(authority) = self.authorities.get_mut(&id) {
                if authority.status == AuthorityStatus::Active {
                    authority.status = AuthorityStatus::Revoked;
                    revoked.push(id);
                }
            }
        }

        Ok(revoked)
    }

    /// Verify an agent has authority for a decision.
    pub fn verify_authority(
        &self,
        agent_id: &AgentId,
        domain: &str,
        tier: ConsequenceTier,
    ) -> TelosResult<AuthorityVerification> {
        // Find active authorities for this agent
        let authority_ids = self.agent_index.get(agent_id)
            .ok_or_else(|| TelosError::AgentNotFound(agent_id.0.clone()))?;

        for auth_id in authority_ids {
            if let Some(authority) = self.authorities.get(auth_id) {
                // Check status
                if authority.status != AuthorityStatus::Active {
                    continue;
                }

                // Check expiration
                if Utc::now() > authority.expires_at {
                    continue;
                }

                // Check domain coverage
                let covers = authority.domains.iter().any(|d| d.covers(domain, tier));
                if !covers {
                    continue;
                }

                // Found valid authority
                return Ok(AuthorityVerification {
                    authority_id: auth_id.clone(),
                    holder: agent_id.clone(),
                    delegation_chain: authority.delegation_chain.clone(),
                    verified_at: Utc::now(),
                });
            }
        }

        Err(TelosError::InsufficientAuthority {
            agent: agent_id.0.clone(),
            scope: domain.to_string(),
        })
    }

    /// Get an authority by ID.
    pub fn get_authority(&self, authority_id: &str) -> Option<&Authority> {
        self.authorities.get(authority_id)
    }

    /// Get delegation chain for an authority.
    pub fn get_delegation_chain(&self, authority_id: &str) -> TelosResult<Vec<&Authority>> {
        let authority = self.authorities.get(authority_id)
            .ok_or_else(|| TelosError::BrokenAuthorityChain(authority_id.to_string()))?;

        let chain: Vec<&Authority> = authority.delegation_chain.iter()
            .filter_map(|id| self.authorities.get(id))
            .collect();

        Ok(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_root_authority() {
        let mut registry = AuthorityRegistry::new();
        let agent = AgentId::new("agent-001");
        let domains = vec![
            DecisionDomain::new("finance.trading", ConsequenceTier::High),
        ];

        let auth_id = registry.create_root_authority(agent.clone(), domains, 10000, 365)
            .unwrap();

        let authority = registry.get_authority(&auth_id).unwrap();
        assert_eq!(authority.holder, agent);
        assert_eq!(authority.status, AuthorityStatus::Active);
        assert!(authority.delegated_from.is_none());
    }

    #[test]
    fn test_delegate_authority() {
        let mut registry = AuthorityRegistry::new();
        let root_agent = AgentId::new("root");
        let delegate_agent = AgentId::new("delegate");

        // Create root
        let root_auth = registry.create_root_authority(
            root_agent.clone(),
            vec![DecisionDomain::new("finance", ConsequenceTier::Critical)],
            10000,
            365,
        ).unwrap();

        // Delegate (attenuated)
        let delegated = registry.delegate_authority(
            &root_auth,
            delegate_agent.clone(),
            vec![DecisionDomain::new("finance.trading", ConsequenceTier::High)],
            vec![],
            5000,
            30,
        ).unwrap();

        let authority = registry.get_authority(&delegated).unwrap();
        assert_eq!(authority.holder, delegate_agent);
        assert_eq!(authority.delegation_chain.len(), 2);
    }

    #[test]
    fn test_revoke_cascades() {
        let mut registry = AuthorityRegistry::new();
        let root = AgentId::new("root");
        let child = AgentId::new("child");

        let root_auth = registry.create_root_authority(
            root.clone(),
            vec![DecisionDomain::new("*", ConsequenceTier::Critical)],
            10000,
            365,
        ).unwrap();

        let child_auth = registry.delegate_authority(
            &root_auth,
            child.clone(),
            vec![DecisionDomain::new("finance", ConsequenceTier::High)],
            vec![],
            5000,
            30,
        ).unwrap();

        // Revoke root
        let revoked = registry.revoke_authority(&root_auth, "test").unwrap();
        
        // Both should be revoked
        assert!(revoked.contains(&root_auth));
        assert!(revoked.contains(&child_auth));
    }

    #[test]
    fn test_verify_authority() {
        let mut registry = AuthorityRegistry::new();
        let agent = AgentId::new("agent");

        registry.create_root_authority(
            agent.clone(),
            vec![DecisionDomain::new("finance.trading", ConsequenceTier::High)],
            10000,
            365,
        ).unwrap();

        // Should succeed
        let result = registry.verify_authority(&agent, "finance.trading.equity", ConsequenceTier::Medium);
        assert!(result.is_ok());

        // Should fail (different domain)
        let result = registry.verify_authority(&agent, "healthcare", ConsequenceTier::Minimal);
        assert!(result.is_err());

        // Should fail (tier too high)
        let result = registry.verify_authority(&agent, "finance.trading", ConsequenceTier::Critical);
        assert!(result.is_err());
    }
}
