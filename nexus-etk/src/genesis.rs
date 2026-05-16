//! ETK Genesis Specification v1.0 (LOCKED).
//!
//! Genesis establishes non-repudiable origin for an execution lifecycle.
//! execution_id is deterministically derived; Genesis is the only event with previous_event_hash = ZERO_HASH.

use crate::schema::{ExecutionEventV1, Hash256, OutcomeCode, ResourceClass};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Derive execution_id from genesis inputs. Canonical: same inputs → same execution_id.
/// Prevents duplicate executions, ID reuse, binds identity/jurisdiction/policy at birth.
pub fn derive_execution_id(
    actor_id: Hash256,
    workload_id: Hash256,
    execution_context: Hash256,
    jurisdiction_code: u16,
    policy_ref: Hash256,
    genesis_timestamp_ms: u64,
) -> Hash256 {
    let mut h = Sha256::new();
    h.update(&actor_id.0);
    h.update(&workload_id.0);
    h.update(&execution_context.0);
    h.update(&jurisdiction_code.to_be_bytes());
    h.update(&policy_ref.0);
    h.update(&genesis_timestamp_ms.to_be_bytes());
    Hash256(h.finalize().into())
}

/// Create Genesis event. sequence_number=0, previous_event_hash=ZERO_HASH, outcome=UNKNOWN.
/// Genesis MUST occur before execution begins. Only authorized launcher/runtime may create.
pub fn create_genesis(
    actor_id: Hash256,
    workload_id: Hash256,
    execution_context: Hash256,
    resource_class: ResourceClass,
    jurisdiction_code: u16,
    policy_ref: Hash256,
) -> ExecutionEventV1 {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let execution_id = derive_execution_id(
        actor_id,
        workload_id,
        execution_context,
        jurisdiction_code,
        policy_ref,
        timestamp_ms,
    );

    let mut ev = ExecutionEventV1 {
        event_id: Hash256::zero(), // computed below
        execution_id,
        sequence_number: 0,
        timestamp_utc: timestamp_ms,
        actor_id,
        workload_id,
        execution_context,
        resource_class,
        jurisdiction_code,
        policy_ref,
        outcome_code: OutcomeCode::Unknown,
        previous_event_hash: Hash256::zero(),
    };
    ev.event_id = ev.compute_event_id();
    ev
}

/// Check if event is Genesis: sequence_number==0 and previous_event_hash==ZERO_HASH.
pub fn is_genesis(ev: &ExecutionEventV1) -> bool {
    ev.sequence_number == 0 && ev.previous_event_hash.is_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_has_sequence_zero_and_zero_prev_hash() {
        let actor = Hash256::of(b"actor");
        let workload = Hash256::of(b"workload");
        let ctx = Hash256::of(b"context");
        let policy = Hash256::of(b"policy");
        let g = create_genesis(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
        assert!(is_genesis(&g));
        assert_eq!(g.sequence_number, 0);
        assert!(g.previous_event_hash.is_zero());
        assert_eq!(g.outcome_code, OutcomeCode::Unknown);
    }

    #[test]
    fn execution_id_deterministic() {
        let actor = Hash256::of(b"a");
        let workload = Hash256::of(b"w");
        let ctx = Hash256::of(b"c");
        let policy = Hash256::of(b"p");
        let t = 1_000_000u64;
        let id1 = derive_execution_id(actor, workload, ctx, 840, policy, t);
        let id2 = derive_execution_id(actor, workload, ctx, 840, policy, t);
        assert_eq!(id1, id2);
    }

    #[test]
    fn execution_id_changes_with_any_input() {
        let actor = Hash256::of(b"a");
        let workload = Hash256::of(b"w");
        let ctx = Hash256::of(b"c");
        let policy = Hash256::of(b"p");
        let base = derive_execution_id(actor, workload, ctx, 840, policy, 1000);
        let diff_actor = derive_execution_id(Hash256::of(b"a2"), workload, ctx, 840, policy, 1000);
        let diff_time = derive_execution_id(actor, workload, ctx, 840, policy, 1001);
        assert_ne!(base, diff_actor);
        assert_ne!(base, diff_time);
    }
}
