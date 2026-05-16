//! ETK Verifier Specification v1.0 (LOCKED).
//!
//! Binary verdict: VALID or INVALID. Seven phases in order. No trust in runtime/cloud/actor.

use crate::schema::{ExecutionEventV1, ExecutionProofV1, Hash256, OutcomeCode, SchemaError};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use thiserror::Error;

/// Verifier verdict. Binary only.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Valid,
    Invalid,
}

/// Failure reason codes (non-normative; helps audits).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VerifierErrorCode {
    E1SchemaViolation,
    E2SignatureInvalid,
    E3HashChainBroken,
    E4PolicyReferenceInvalid,
    E5TemporalAnomaly,
    E6ExecutionIncomplete,
}

#[derive(Debug, Error)]
pub enum VerifierError {
    #[error("E1: Schema violation")]
    E1SchemaViolation(#[from] SchemaError),
    #[error("E2: Signature invalid")]
    E2SignatureInvalid,
    #[error("E3: Hash chain broken")]
    E3HashChainBroken,
    #[error("E4: Policy reference invalid")]
    E4PolicyReferenceInvalid,
    #[error("E5: Temporal anomaly")]
    E5TemporalAnomaly,
    #[error("E6: Execution incomplete")]
    E6ExecutionIncomplete,
}

impl VerifierError {
    pub fn code(&self) -> VerifierErrorCode {
        match self {
            VerifierError::E1SchemaViolation(_) => VerifierErrorCode::E1SchemaViolation,
            VerifierError::E2SignatureInvalid => VerifierErrorCode::E2SignatureInvalid,
            VerifierError::E3HashChainBroken => VerifierErrorCode::E3HashChainBroken,
            VerifierError::E4PolicyReferenceInvalid => VerifierErrorCode::E4PolicyReferenceInvalid,
            VerifierError::E5TemporalAnomaly => VerifierErrorCode::E5TemporalAnomaly,
            VerifierError::E6ExecutionIncomplete => VerifierErrorCode::E6ExecutionIncomplete,
        }
    }
}

/// Policy snapshot resolver: given policy_ref, return policy blob if valid.
/// Verifier hashes blob and checks hash == policy_ref. Caller provides resolution (e.g. file, store).
pub type PolicyResolver = dyn Fn(Hash256) -> Option<Vec<u8>>;

/// Verify ExecutionProof and event stream. Offline; no API calls.
/// Events may be in any order; verifier sorts by sequence_number and validates chain.
pub fn verify(
    proof: &ExecutionProofV1,
    events: &[ExecutionEventV1],
    policy_resolver: &PolicyResolver,
    verifier_pubkey: &VerifyingKey,
    tolerance_ms: u64,
) -> Result<Verdict, VerifierError> {
    // Phase 1: Schema & version
    if events.is_empty() {
        return Err(VerifierError::E6ExecutionIncomplete);
    }

    let mut events_sorted: Vec<_> = events.to_vec();
    events_sorted.sort_by_key(|e| e.sequence_number);

    for ev in &events_sorted {
        if ev.compute_event_id() != ev.event_id {
            return Err(VerifierError::E1SchemaViolation(SchemaError::EventIdMismatch));
        }
    }

    // Phase 2: Signature
    let signing_bytes = proof.to_signing_bytes();
    let sig = Signature::from_bytes(&proof.verifier_signature);
    verifier_pubkey
        .verify(&signing_bytes, &sig)
        .map_err(|_| VerifierError::E2SignatureInvalid)?;

    // Phase 3: Execution consistency
    let execution_id = proof.execution_id;
    let policy_ref = proof.policy_ref;
    let jurisdiction_code = proof.jurisdiction_code;

    for (i, ev) in events_sorted.iter().enumerate() {
        if ev.execution_id != execution_id {
            return Err(VerifierError::E3HashChainBroken);
        }
        if ev.policy_ref != policy_ref {
            return Err(VerifierError::E3HashChainBroken);
        }
        if ev.jurisdiction_code != jurisdiction_code {
            return Err(VerifierError::E3HashChainBroken);
        }
        if ev.sequence_number != i as u64 {
            return Err(VerifierError::E3HashChainBroken);
        }
    }

    // Phase 4: Hash chain
    for (i, ev) in events_sorted.iter().enumerate() {
        if i == 0 {
            if !ev.previous_event_hash.is_zero() {
                return Err(VerifierError::E3HashChainBroken);
            }
        } else {
            let prev_id = events_sorted[i - 1].event_id;
            if ev.previous_event_hash != prev_id {
                return Err(VerifierError::E3HashChainBroken);
            }
        }
    }

    let last = events_sorted.last().unwrap();
    if proof.event_chain_root != last.event_id {
        return Err(VerifierError::E3HashChainBroken);
    }

    // Phase 5: Temporal sanity (non-trusting)
    for i in 1..events_sorted.len() {
        if events_sorted[i].timestamp_utc < events_sorted[i - 1].timestamp_utc {
            return Err(VerifierError::E5TemporalAnomaly);
        }
    }
    if proof.end_timestamp < proof.start_timestamp {
        return Err(VerifierError::E5TemporalAnomaly);
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if proof.end_timestamp > now_ms.saturating_add(tolerance_ms) {
        return Err(VerifierError::E5TemporalAnomaly);
    }

    // Phase 6: Policy reference
    let snapshot = policy_resolver(policy_ref).ok_or(VerifierError::E4PolicyReferenceInvalid)?;
    let snapshot_hash = Hash256::of(&snapshot);
    if snapshot_hash != policy_ref {
        return Err(VerifierError::E4PolicyReferenceInvalid);
    }

    // Phase 7: Outcome consistency — exactly one terminal, no events after
    let terminal_idx = events_sorted
        .iter()
        .position(|e| e.outcome_code.is_terminal())
        .ok_or(VerifierError::E6ExecutionIncomplete)?;
    if terminal_idx != events_sorted.len() - 1 {
        return Err(VerifierError::E6ExecutionIncomplete);
    }
    let terminal = &events_sorted[terminal_idx];
    match terminal.outcome_code {
        OutcomeCode::Success | OutcomeCode::Failure | OutcomeCode::Terminated | OutcomeCode::Degraded => {}
        _ => return Err(VerifierError::E6ExecutionIncomplete),
    }

    Ok(Verdict::Valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::EventChain;
    use crate::schema::{Hash256, ResourceClass};
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use rand::rngs::OsRng;

    #[test]
    fn full_flow_valid() {
        let actor = Hash256::of(b"actor");
        let workload = Hash256::of(b"workload");
        let ctx = Hash256::of(b"context");
        let policy_ref = Hash256::of(b"policy-snapshot-bytes");
        let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy_ref);
        // Use monotonic timestamps: genesis uses "now", append with now+1ms, now+2ms.
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        chain.append(now_ms + 1, OutcomeCode::Unknown).unwrap();
        chain.append(now_ms + 2, OutcomeCode::Success).unwrap();
        let signing_key = SigningKey::generate(&mut OsRng);
        let proof = chain.finalize(&signing_key).unwrap();
        let events: Vec<_> = chain.events().to_vec();
        let verifier_pubkey: VerifyingKey = signing_key.verifying_key();
        let policy_resolver = |_pr: Hash256| Some(b"policy-snapshot-bytes".to_vec());
        let result = verify(
            &proof,
            &events,
            &policy_resolver,
            &verifier_pubkey,
            86400_000,
        );
        assert!(matches!(result, Ok(Verdict::Valid)));
    }

    #[test]
    fn invalid_when_policy_hash_mismatch() {
        let actor = Hash256::of(b"a");
        let workload = Hash256::of(b"w");
        let ctx = Hash256::of(b"c");
        let policy_ref = Hash256::of(b"correct-policy");
        let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy_ref);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        chain.append(now_ms + 1, OutcomeCode::Success).unwrap();
        let signing_key = SigningKey::generate(&mut OsRng);
        let proof = chain.finalize(&signing_key).unwrap();
        let events: Vec<_> = chain.events().to_vec();
        let verifier_pubkey = signing_key.verifying_key();
        // Resolver returns wrong policy bytes → hash won't match policy_ref.
        let policy_resolver = |_pr: Hash256| Some(b"wrong-policy".to_vec());
        let result = verify(&proof, &events, &policy_resolver, &verifier_pubkey, 86400_000);
        assert!(matches!(result, Err(VerifierError::E4PolicyReferenceInvalid)));
    }
}

/// Same as verify but returns Verdict only (discards error). Useful for CLI.
pub fn verify_verdict(
    proof: &ExecutionProofV1,
    events: &[ExecutionEventV1],
    policy_resolver: &PolicyResolver,
    verifier_pubkey: &VerifyingKey,
    tolerance_ms: u64,
) -> Verdict {
    match verify(proof, events, policy_resolver, verifier_pubkey, tolerance_ms) {
        Ok(Verdict::Valid) => Verdict::Valid,
        _ => Verdict::Invalid,
    }
}
