//! Immune + reputation execution guard.
//!
//! When set, execution is denied if the principal is isolated due to defection
//! or if aggregated reputation is below threshold.

use crate::guard::{ExecutionGuard, GuardDecision};
use nexus_pcu::PCU;
use crate::types::ExecutionContext;
use multi_asi_immune::node::state::{AsiNode, NodeConfig};
use parking_lot::Mutex;

/// Guard that delegates to the multi-ASI immune node (reputation + defection).
pub struct ImmuneGuard {
    node: Mutex<AsiNode>,
    /// Minimum aggregated reputation to allow execution [0, 1]. Unknown principals get INITIAL (0.5).
    min_reputation: f64,
}

impl ImmuneGuard {
    /// Create a new guard with default node config and min_reputation 0.0 (only defection blocks).
    pub fn new() -> Self {
        Self::with_config(NodeConfig::default(), 0.0)
    }

    /// Create with custom node config and minimum reputation threshold.
    pub fn with_config(config: NodeConfig, min_reputation: f64) -> Self {
        Self {
            node: Mutex::new(AsiNode::new(config)),
            min_reputation,
        }
    }

    /// Set minimum reputation threshold.
    pub fn set_min_reputation(&mut self, min_reputation: f64) {
        self.min_reputation = min_reputation;
    }

    /// Access the underlying node (e.g. to add peers, record observations).
    pub fn node_mut(&self) -> parking_lot::MutexGuard<'_, AsiNode> {
        self.node.lock()
    }
}

impl Default for ImmuneGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionGuard for ImmuneGuard {
    fn check(&self, pcu: &PCU, _ctx: &ExecutionContext) -> GuardDecision {
        let principal = pcu.identity.effective_principal();
        if principal.is_anonymous() {
            return GuardDecision::Deny(
                "Anonymous principal not permitted by immune guard".to_string(),
            );
        }
        let bytes = *principal.as_bytes();
        let mut node = self.node.lock();
        match node.allow_execution_by(bytes, self.min_reputation) {
            Ok(()) => GuardDecision::Allow,
            Err(reason) => GuardDecision::Deny(reason),
        }
    }
}
