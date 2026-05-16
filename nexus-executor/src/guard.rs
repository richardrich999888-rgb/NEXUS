//! Execution guard: single choke point for biological + accountability constraints.
//!
//! No intelligent action occurs without passing through the guard when one is set.
//!
//! **FROZEN INTERFACE:** Do not change the signature of `ExecutionGuard::check` or the meaning of
//! `GuardDecision` (Allow/Deny). Regulator and patent claims depend on this contract.

use nexus_pcu::PCU;
use crate::types::ExecutionContext;

/// Decision from the execution guard.
#[derive(Debug, Clone)]
pub enum GuardDecision {
    /// Execution is allowed to proceed.
    Allow,
    /// Execution is blocked; reason is logged and returned to caller.
    Deny(String),
}

/// Guard that every PCU execution must pass through when set.
///
/// Implementations may delegate to nervous-system coordinator, TELOS membrane,
/// or other enforcement. When no guard is set, execution is unconstrained.
pub trait ExecutionGuard: Send + Sync {
    /// Check whether this PCU execution is allowed.
    fn check(&self, pcu: &PCU, ctx: &ExecutionContext) -> GuardDecision;
}
