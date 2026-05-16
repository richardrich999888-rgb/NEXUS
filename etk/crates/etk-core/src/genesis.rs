//! ETK Genesis Specification v1.0 (LOCKED).
//!
//! Genesis establishes non-repudiable origin. execution_id is deterministically derived.

use etk_types::{ExecutionEvent, Hash256, OutcomeCode, ResourceClass};
use crate::codec::compute_event_id;
use crate::crypto::hash256;
use std::time::{SystemTime, UNIX_EPOCH};

/// Derive execution_id from genesis inputs. Same inputs → same execution_id.
pub fn derive_execution_id(
    actor: Hash256,
    workload: Hash256,
    ctx: Hash256,
    jurisdiction: u16,
    policy: Hash256,
    ts_ms: u64,
) -> Hash256 {
    let mut buf = Vec::with_capacity(32 * 5 + 2 + 8);
    buf.extend_from_slice(&actor.0);
    buf.extend_from_slice(&workload.0);
    buf.extend_from_slice(&ctx.0);
    buf.extend_from_slice(&jurisdiction.to_be_bytes());
    buf.extend_from_slice(&policy.0);
    buf.extend_from_slice(&ts_ms.to_be_bytes());
    hash256(&buf)
}

/// Create Genesis event. sequence_number=0, previous_event_hash=ZERO, outcome=Unknown.
pub fn create_genesis(
    actor: Hash256,
    workload: Hash256,
    ctx: Hash256,
    resource: ResourceClass,
    jurisdiction: u16,
    policy: Hash256,
) -> ExecutionEvent {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let execution_id = derive_execution_id(actor, workload, ctx, jurisdiction, policy, ts);

    let mut event = ExecutionEvent {
        event_id: Hash256::zero(),
        execution_id,
        sequence_number: 0,
        timestamp_utc: ts,
        actor_id: actor,
        workload_id: workload,
        execution_context: ctx,
        resource_class: resource,
        jurisdiction_code: jurisdiction,
        policy_ref: policy,
        outcome_code: OutcomeCode::Unknown,
        previous_event_hash: Hash256::zero(),
    };
    event.event_id = compute_event_id(&event);
    event
}

/// True iff event is Genesis (sequence 0, previous_event_hash zero).
pub fn is_genesis(ev: &ExecutionEvent) -> bool {
    ev.sequence_number == 0 && ev.previous_event_hash.is_zero()
}
