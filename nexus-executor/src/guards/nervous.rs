//! Nervous-system execution guard: developmental gates + autonomic risk.
//!
//! When this guard is set, every PCU execution passes through the coordinator.
//! Infant stage cannot execute; CALM mode blocks high-risk input.

use crate::guard::{ExecutionGuard, GuardDecision};
use nexus_pcu::PCU;
use crate::types::ExecutionContext;
use nervous_system::{NervousSystemCoordinator, CoordinatorConfig};
use nervous_system::decision::engine::{DecisionResult, ProposedAction};
use nervous_system::perception::processor::InputType;
use developmental_gates::Capability;
use parking_lot::Mutex;

/// Guard that delegates to the nervous-system coordinator.
/// Holds coordinator behind a mutex because process() takes &mut self.
pub struct NervousSystemGuard {
    coordinator: Mutex<NervousSystemCoordinator>,
}

impl NervousSystemGuard {
    /// Create a new guard with default coordinator config.
    pub fn new() -> Self {
        Self::with_config(CoordinatorConfig::default())
    }

    /// Create with custom coordinator config.
    pub fn with_config(config: CoordinatorConfig) -> Self {
        Self {
            coordinator: Mutex::new(NervousSystemCoordinator::new(config)),
        }
    }
}

impl Default for NervousSystemGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionGuard for NervousSystemGuard {
    fn check(&self, _pcu: &PCU, ctx: &ExecutionContext) -> GuardDecision {
        let estimated_risk = ctx.biological_risk.unwrap_or(0.5);
        let proposed = ProposedAction {
            action_type: "execute".to_string(),
            target: None,
            parameters: vec![],
            required_capability: Capability::Execute,
            estimated_risk,
        };

        let mut coord = self.coordinator.lock();
        let output = coord.process(
            InputType::Network {
                source: "pcu_execute".to_string(),
                payload: vec![],
            },
            Some(proposed),
        );

        match output.decision {
            DecisionResult::Approved { .. } => GuardDecision::Allow,
            DecisionResult::Modified { reason, .. } => {
                GuardDecision::Deny(format!("Modified execution not allowed: {}", reason))
            }
            DecisionResult::Blocked { reason, .. } => GuardDecision::Deny(reason),
            DecisionResult::NoAction => GuardDecision::Deny("NoAction".to_string()),
        }
    }
}
