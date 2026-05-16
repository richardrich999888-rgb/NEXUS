//! Composite execution guard: runs multiple guards in sequence.
//!
//! All guards must Allow; if any Deny, execution is blocked.
//!
//! **FROZEN INTERFACE:** First-Deny-wins ordering and the `add`/`from_guards` contract must not change.

use crate::guard::{ExecutionGuard, GuardDecision};
use nexus_pcu::PCU;
use crate::types::ExecutionContext;
use std::sync::Arc;

/// Guard that runs a list of guards in order. First Deny wins.
pub struct CompositeGuard {
    guards: Vec<Arc<dyn ExecutionGuard>>,
}

impl CompositeGuard {
    /// Create an empty composite (allows all).
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Add a guard to the chain (order: first added = first checked).
    pub fn add(mut self, guard: Arc<dyn ExecutionGuard>) -> Self {
        self.guards.push(guard);
        self
    }

    /// Build from a list of guards.
    pub fn from_guards(guards: Vec<Arc<dyn ExecutionGuard>>) -> Self {
        Self { guards }
    }
}

impl Default for CompositeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionGuard for CompositeGuard {
    fn check(&self, pcu: &PCU, ctx: &ExecutionContext) -> GuardDecision {
        for guard in &self.guards {
            match guard.check(pcu, ctx) {
                GuardDecision::Allow => {}
                GuardDecision::Deny(reason) => return GuardDecision::Deny(reason),
            }
        }
        GuardDecision::Allow
    }
}
