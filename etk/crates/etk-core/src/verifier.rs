//! ETK Verifier Specification v1.0 (LOCKED).
//!
//! Binary verdict: VALID or INVALID. No trust in runtime/cloud/actor.
//! Offline-capable.

use etk_types::{ExecutionEvent, ExecutionProof, Hash256};
use crate::codec::{compute_event_id, encode_proof_signing_bytes};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Valid,
    Invalid,
}

#[derive(Debug, Error)]
pub enum VerifierError {
    #[error("E1: Schema violation")]
    E1Schema(#[from] crate::codec::CodecError),
    #[error("E2: Signature invalid")]
    E2Signature,
    #[error("E3: Hash chain broken")]
    E3HashChain,
    #[error("E4: Policy reference invalid")]
    E4PolicyRef,
    #[error("E5: Temporal anomaly")]
    E5Temporal,
    #[error("E6: Execution incomplete")]
    E6Incomplete,
}

/// Policy resolver: given policy_ref, return policy snapshot bytes if available.
/// Verifier hashes and checks hash == policy_ref.
pub type PolicyResolver = dyn Fn(Hash256) -> Option<Vec<u8>>;

/// Verify proof and event stream. Offline; no API calls.
/// Events may be unordered; verifier sorts by sequence_number.
pub fn verify(
    proof: &ExecutionProof,
    events: &[ExecutionEvent],
    policy_resolver: &PolicyResolver,
    verifier_pubkey: &VerifyingKey,
    tolerance_ms: u64,
) -> Result<Verdict, VerifierError> {
    if events.is_empty() {
        return Err(VerifierError::E6Incomplete);
    }

    let mut sorted: Vec<_> = events.to_vec();
    sorted.sort_by_key(|e| e.sequence_number);

    // Phase 1: event_id consistency
    for ev in &sorted {
        if compute_event_id(ev) != ev.event_id {
            return Err(VerifierError::E3HashChain);
        }
    }

    // Phase 2: Signature
    let signing_bytes = encode_proof_signing_bytes(proof);
    let sig = Signature::from_bytes(&proof.verifier_signature);
    verifier_pubkey
        .verify(&signing_bytes, &sig)
        .map_err(|_| VerifierError::E2Signature)?;

    // Phase 3 & 4: Execution consistency + hash chain
    let execution_id = proof.execution_id;
    let policy_ref = proof.policy_ref;
    let jurisdiction_code = proof.jurisdiction_code;

    for (i, ev) in sorted.iter().enumerate() {
        if ev.execution_id != execution_id || ev.policy_ref != policy_ref
            || ev.jurisdiction_code != jurisdiction_code
        {
            return Err(VerifierError::E3HashChain);
        }
        if ev.sequence_number != i as u64 {
            return Err(VerifierError::E3HashChain);
        }
        if i == 0 {
            if !ev.previous_event_hash.is_zero() {
                return Err(VerifierError::E3HashChain);
            }
        } else if ev.previous_event_hash != sorted[i - 1].event_id {
            return Err(VerifierError::E3HashChain);
        }
    }

    if proof.event_chain_root != sorted.last().unwrap().event_id {
        return Err(VerifierError::E3HashChain);
    }

    // Phase 5: Temporal sanity
    for i in 1..sorted.len() {
        if sorted[i].timestamp_utc < sorted[i - 1].timestamp_utc {
            return Err(VerifierError::E5Temporal);
        }
    }
    if proof.end_timestamp < proof.start_timestamp {
        return Err(VerifierError::E5Temporal);
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if proof.end_timestamp > now_ms.saturating_add(tolerance_ms) {
        return Err(VerifierError::E5Temporal);
    }

    // Phase 6: Policy reference
    let snapshot = policy_resolver(policy_ref).ok_or(VerifierError::E4PolicyRef)?;
    if crate::crypto::hash256(&snapshot) != policy_ref {
        return Err(VerifierError::E4PolicyRef);
    }

    // Phase 7: Exactly one terminal event, last
    let terminal_idx = sorted
        .iter()
        .position(|e| e.outcome_code.is_terminal())
        .ok_or(VerifierError::E6Incomplete)?;
    if terminal_idx != sorted.len() - 1 {
        return Err(VerifierError::E6Incomplete);
    }

    Ok(Verdict::Valid)
}
