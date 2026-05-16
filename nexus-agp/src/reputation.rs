//! # CRDT-Based Reputation Convergence
//!
//! PATENT CLAIM 5: Reputation records stored as conflict-free replicated data types
//! that automatically converge across distributed nodes.
//!
//! ## Why Unforkable
//!
//! - Reputation data lives in distributed network, not in fork's code
//! - CRDTs automatically merge without conflicts
//! - Merkle proofs enable portable reputation claims

use crate::identity::AgentFingerprint;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Grow-only counter for monotonic reputation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GCounter {
    /// Node ID -> count mapping
    counts: HashMap<String, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        Self { counts: HashMap::new() }
    }
    
    /// Increment counter for this node
    pub fn increment(&mut self, node_id: &str) {
        *self.counts.entry(node_id.to_string()).or_insert(0) += 1;
    }
    
    /// Add specific value for this node
    pub fn add(&mut self, node_id: &str, value: u64) {
        *self.counts.entry(node_id.to_string()).or_insert(0) += value;
    }
    
    /// Get total value across all nodes
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }
    
    /// Merge with another GCounter (takes max of each node)
    pub fn merge(&mut self, other: &Self) {
        for (node, count) in &other.counts {
            let entry = self.counts.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
    }
}

/// Task type for reputation categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskType {
    pub domain: String,    // L0: inference, training, data
    pub category: String,  // L1: nlp, vision, tabular
    pub specific: String,  // L2: sentiment, classification, ner
}

impl TaskType {
    pub fn new(domain: &str, category: &str, specific: &str) -> Self {
        Self {
            domain: domain.to_lowercase(),
            category: category.to_lowercase(),
            specific: specific.to_lowercase(),
        }
    }
    
    /// Compute hierarchical similarity
    pub fn similarity(&self, other: &Self) -> f64 {
        if self == other {
            1.0
        } else if self.domain == other.domain && self.category == other.category {
            0.7
        } else if self.domain == other.domain {
            0.3
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.domain, self.category, self.specific)
    }
}

/// CRDT-based reputation record
///
/// PATENT CLAIM 5: Reputation as CRDT that converges automatically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationCRDT {
    /// Agent this reputation belongs to
    pub agent_fingerprint: AgentFingerprint,
    /// Task type this reputation is for
    pub task_type: TaskType,
    /// Monotonic success counter
    pub successes: GCounter,
    /// Monotonic failure counter
    pub failures: GCounter,
    /// Last update timestamp (LWW for metadata)
    pub last_updated: u64,
    /// Merkle root for portable proofs
    pub merkle_root: [u8; 32],
}

impl ReputationCRDT {
    /// Create new reputation record for agent + task type
    pub fn new(agent_fingerprint: AgentFingerprint, task_type: TaskType) -> Self {
        Self {
            agent_fingerprint,
            task_type,
            successes: GCounter::new(),
            failures: GCounter::new(),
            last_updated: 0,
            merkle_root: [0u8; 32],
        }
    }
    
    /// Record a successful execution
    pub fn record_success(&mut self, node_id: &str, timestamp: u64) {
        self.successes.increment(node_id);
        self.last_updated = self.last_updated.max(timestamp);
        self.update_merkle_root();
    }
    
    /// Record a failed execution
    pub fn record_failure(&mut self, node_id: &str, timestamp: u64) {
        self.failures.increment(node_id);
        self.last_updated = self.last_updated.max(timestamp);
        self.update_merkle_root();
    }
    
    /// Compute reputation score [0, 1]
    pub fn score(&self) -> f64 {
        let s = self.successes.value() as f64;
        let f = self.failures.value() as f64;
        // Bayesian with prior (avoids division by zero)
        (s + 1.0) / (s + f + 2.0)
    }
    
    /// Total executions
    pub fn total_executions(&self) -> u64 {
        self.successes.value() + self.failures.value()
    }
    
    /// Merge with remote replica (CRDT merge)
    pub fn merge(&mut self, other: &Self) {
        if self.agent_fingerprint != other.agent_fingerprint {
            return; // Only merge same agent
        }
        if self.task_type != other.task_type {
            return; // Only merge same task type
        }
        
        self.successes.merge(&other.successes);
        self.failures.merge(&other.failures);
        self.last_updated = self.last_updated.max(other.last_updated);
        self.update_merkle_root();
    }
    
    /// Update Merkle root after changes
    fn update_merkle_root(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.agent_fingerprint.0);
        hasher.update(self.task_type.to_string().as_bytes());
        hasher.update(&self.successes.value().to_le_bytes());
        hasher.update(&self.failures.value().to_le_bytes());
        hasher.update(&self.last_updated.to_le_bytes());
        let result = hasher.finalize();
        self.merkle_root.copy_from_slice(&result);
    }
}

/// Merkle proof for portable reputation claims
///
/// PATENT CLAIM 7: Enables agents to port reputation to external chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationProof {
    /// Agent fingerprint
    pub agent_fingerprint: AgentFingerprint,
    /// Task type
    pub task_type: TaskType,
    /// Claimed score
    pub score: f64,
    /// Total executions (for weight)
    pub total_executions: u64,
    /// Merkle root
    pub merkle_root: [u8; 32],
    /// Merkle proof siblings
    pub proof_siblings: Vec<[u8; 32]>,
    /// Proof timestamp
    pub timestamp: u64,
}

impl ReputationProof {
    /// Create proof from reputation CRDT
    pub fn create(reputation: &ReputationCRDT, proof_siblings: Vec<[u8; 32]>) -> Self {
        Self {
            agent_fingerprint: reputation.agent_fingerprint.clone(),
            task_type: reputation.task_type.clone(),
            score: reputation.score(),
            total_executions: reputation.total_executions(),
            merkle_root: reputation.merkle_root,
            proof_siblings,
            timestamp: reputation.last_updated,
        }
    }
    
    /// Verify proof against a known root
    pub fn verify(&self, expected_root: &[u8; 32]) -> bool {
        // In production: verify full Merkle path
        // For now: check root matches
        &self.merkle_root == expected_root
    }
}

/// Reputation store (in-memory, CRDT-based)
#[derive(Debug, Default)]
pub struct ReputationStore {
    /// Agent fingerprint -> task type -> reputation
    records: HashMap<[u8; 32], HashMap<String, ReputationCRDT>>,
}

impl ReputationStore {
    pub fn new() -> Self {
        Self { records: HashMap::new() }
    }
    
    /// Get or create reputation for agent + task type
    pub fn get_or_create(
        &mut self,
        agent: &AgentFingerprint,
        task_type: &TaskType,
    ) -> &mut ReputationCRDT {
        let agent_records = self.records
            .entry(agent.0)
            .or_insert_with(HashMap::new);
        
        agent_records
            .entry(task_type.to_string())
            .or_insert_with(|| ReputationCRDT::new(agent.clone(), task_type.clone()))
    }
    
    /// Merge remote reputation (for gossip)
    pub fn merge_remote(&mut self, remote: ReputationCRDT) {
        let local = self.get_or_create(&remote.agent_fingerprint, &remote.task_type);
        local.merge(&remote);
    }
    
    /// Get all reputations for an agent
    pub fn get_all(&self, agent: &AgentFingerprint) -> Vec<&ReputationCRDT> {
        self.records
            .get(&agent.0)
            .map(|m| m.values().collect())
            .unwrap_or_default()
    }
    
    /// Compute aggregate score across all task types
    pub fn aggregate_score(&self, agent: &AgentFingerprint) -> f64 {
        let reputations = self.get_all(agent);
        if reputations.is_empty() {
            return 0.5; // Default
        }
        
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        
        for rep in reputations {
            let weight = (rep.total_executions() as f64).ln_1p();
            weighted_sum += rep.score() * weight;
            weight_total += weight;
        }
        
        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.5
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gcounter_merge() {
        let mut c1 = GCounter::new();
        let mut c2 = GCounter::new();
        
        c1.increment("node_a");
        c1.increment("node_a");
        c2.increment("node_b");
        c2.increment("node_b");
        c2.increment("node_b");
        
        c1.merge(&c2);
        
        assert_eq!(c1.value(), 5); // 2 + 3
    }
    
    #[test]
    fn test_reputation_convergence() {
        let fp = AgentFingerprint([0u8; 32]);
        let tt = TaskType::new("inference", "nlp", "sentiment");
        
        let mut rep1 = ReputationCRDT::new(fp.clone(), tt.clone());
        let mut rep2 = ReputationCRDT::new(fp.clone(), tt.clone());
        
        // Concurrent updates on different nodes
        rep1.record_success("node_a", 1000);
        rep1.record_success("node_a", 1001);
        
        rep2.record_success("node_b", 1002);
        rep2.record_failure("node_b", 1003);
        
        // Merge (order shouldn't matter for CRDTs)
        rep1.merge(&rep2);
        rep2.merge(&rep1);
        
        assert_eq!(rep1.successes.value(), rep2.successes.value());
        assert_eq!(rep1.failures.value(), rep2.failures.value());
        assert_eq!(rep1.score(), rep2.score());
    }
    
    #[test]
    fn test_reputation_score() {
        let fp = AgentFingerprint([0u8; 32]);
        let tt = TaskType::new("inference", "nlp", "sentiment");
        let mut rep = ReputationCRDT::new(fp, tt);
        
        // Add successes
        for _ in 0..9 {
            rep.record_success("node", 1000);
        }
        rep.record_failure("node", 1001);
        
        // 9 success, 1 failure → score ≈ 0.83 (with Bayesian prior)
        let score = rep.score();
        assert!(score > 0.8 && score < 0.9);
    }
}
