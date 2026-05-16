//! ETK canonical binary codec v1.0 (LOCKED).
//!
//! Big-endian. Field order fixed. No optional fields. No nulls.
//! event_id = hash(canonical bytes excluding event_id).

use etk_types::{
    ExecutionEvent, ExecutionProof, Hash256, OutcomeCode, ResourceClass, EVENT_CANONICAL_LEN,
    PROOF_CANONICAL_LEN,
};
use crate::crypto::hash256;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Invalid length")]
    InvalidLength,
    #[error("event_id mismatch")]
    EventIdMismatch,
}

/// Encode event to canonical bytes (storage/wire). Includes event_id.
pub fn encode_event(ev: &ExecutionEvent) -> Vec<u8> {
    let mut out = Vec::with_capacity(EVENT_CANONICAL_LEN);
    out.extend_from_slice(&ev.event_id.0);
    out.extend_from_slice(&ev.execution_id.0);
    out.extend_from_slice(&ev.sequence_number.to_be_bytes());
    out.extend_from_slice(&ev.timestamp_utc.to_be_bytes());
    out.extend_from_slice(&ev.actor_id.0);
    out.extend_from_slice(&ev.workload_id.0);
    out.extend_from_slice(&ev.execution_context.0);
    out.push(ev.resource_class.to_u8());
    out.extend_from_slice(&ev.jurisdiction_code.to_be_bytes());
    out.extend_from_slice(&ev.policy_ref.0);
    out.push(ev.outcome_code.to_u8());
    out.extend_from_slice(&ev.previous_event_hash.0);
    out
}

/// Bytes used to compute event_id (all fields except event_id).
fn encode_event_for_hash(ev: &ExecutionEvent) -> Vec<u8> {
    let mut out = Vec::with_capacity(EVENT_CANONICAL_LEN - 32);
    out.extend_from_slice(&ev.execution_id.0);
    out.extend_from_slice(&ev.sequence_number.to_be_bytes());
    out.extend_from_slice(&ev.timestamp_utc.to_be_bytes());
    out.extend_from_slice(&ev.actor_id.0);
    out.extend_from_slice(&ev.workload_id.0);
    out.extend_from_slice(&ev.execution_context.0);
    out.push(ev.resource_class.to_u8());
    out.extend_from_slice(&ev.jurisdiction_code.to_be_bytes());
    out.extend_from_slice(&ev.policy_ref.0);
    out.push(ev.outcome_code.to_u8());
    out.extend_from_slice(&ev.previous_event_hash.0);
    out
}

/// Compute event_id from event (hash of canonical bytes excluding event_id).
pub fn compute_event_id(ev: &ExecutionEvent) -> Hash256 {
    hash256(&encode_event_for_hash(ev))
}

/// Decode event from canonical bytes. Validates event_id.
pub fn decode_event(bytes: &[u8]) -> Result<ExecutionEvent, CodecError> {
    if bytes.len() != EVENT_CANONICAL_LEN {
        return Err(CodecError::InvalidLength);
    }
    let mut off = 0;
    let read_hash = |off: &mut usize| {
        let h: [u8; 32] = bytes[*off..*off + 32].try_into().unwrap();
        *off += 32;
        Hash256(h)
    };
    let read_u64_be = |off: &mut usize| {
        let b: [u8; 8] = bytes[*off..*off + 8].try_into().unwrap();
        *off += 8;
        u64::from_be_bytes(b)
    };
    let event_id = read_hash(&mut off);
    let execution_id = read_hash(&mut off);
    let sequence_number = read_u64_be(&mut off);
    let timestamp_utc = read_u64_be(&mut off);
    let actor_id = read_hash(&mut off);
    let workload_id = read_hash(&mut off);
    let execution_context = read_hash(&mut off);
    let resource_class = ResourceClass::from_u8(bytes[off]);
    off += 1;
    let jurisdiction_code = u16::from_be_bytes(bytes[off..off + 2].try_into().unwrap());
    off += 2;
    let policy_ref = read_hash(&mut off);
    let outcome_code = OutcomeCode::from_u8(bytes[off]);
    off += 1;
    let previous_event_hash = read_hash(&mut off);

    let ev = ExecutionEvent {
        event_id,
        execution_id,
        sequence_number,
        timestamp_utc,
        actor_id,
        workload_id,
        execution_context,
        resource_class,
        jurisdiction_code,
        policy_ref,
        outcome_code,
        previous_event_hash,
    };
    if compute_event_id(&ev) != event_id {
        return Err(CodecError::EventIdMismatch);
    }
    Ok(ev)
}

/// Encode proof (signing payload: all fields except signature).
pub fn encode_proof_signing_bytes(p: &ExecutionProof) -> Vec<u8> {
    let mut out = Vec::with_capacity(PROOF_CANONICAL_LEN - 64);
    out.extend_from_slice(&p.execution_id.0);
    out.extend_from_slice(&p.event_chain_root.0);
    out.extend_from_slice(&p.start_timestamp.to_be_bytes());
    out.extend_from_slice(&p.end_timestamp.to_be_bytes());
    out.extend_from_slice(&p.policy_ref.0);
    out.extend_from_slice(&p.jurisdiction_code.to_be_bytes());
    out
}

/// Encode proof to canonical bytes (storage/wire). Includes signature.
pub fn encode_proof(p: &ExecutionProof) -> Vec<u8> {
    let mut out = encode_proof_signing_bytes(p);
    out.extend_from_slice(&p.verifier_signature);
    out
}

/// Decode proof from canonical bytes.
pub fn decode_proof(bytes: &[u8]) -> Result<ExecutionProof, CodecError> {
    if bytes.len() != PROOF_CANONICAL_LEN {
        return Err(CodecError::InvalidLength);
    }
    let mut off = 0;
    let read_hash = |off: &mut usize| {
        let h: [u8; 32] = bytes[*off..*off + 32].try_into().unwrap();
        *off += 32;
        Hash256(h)
    };
    let read_u64_be = |off: &mut usize| {
        let b: [u8; 8] = bytes[*off..*off + 8].try_into().unwrap();
        *off += 8;
        u64::from_be_bytes(b)
    };
    Ok(ExecutionProof {
        execution_id: read_hash(&mut off),
        event_chain_root: read_hash(&mut off),
        start_timestamp: read_u64_be(&mut off),
        end_timestamp: read_u64_be(&mut off),
        policy_ref: read_hash(&mut off),
        jurisdiction_code: u16::from_be_bytes(bytes[off..off + 2].try_into().unwrap()),
        verifier_signature: bytes[off + 2..off + 66].try_into().unwrap(),
    })
}
