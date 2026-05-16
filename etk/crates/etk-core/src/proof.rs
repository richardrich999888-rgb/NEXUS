//! ETK proof derivation. One proof per execution; constant size.

use etk_types::{ExecutionEvent, ExecutionProof};

/// Build ExecutionProof from event slice (root = last event_id).
/// Caller must sign; use EventChain::finalize for full flow.
pub fn build_proof(events: &[ExecutionEvent]) -> Option<ExecutionProof> {
    let first = events.first()?;
    let last = events.last()?;
    Some(ExecutionProof {
        execution_id: first.execution_id,
        event_chain_root: last.event_id,
        start_timestamp: first.timestamp_utc,
        end_timestamp: last.timestamp_utc,
        policy_ref: first.policy_ref,
        jurisdiction_code: first.jurisdiction_code,
        verifier_signature: [0u8; 64],
    })
}
