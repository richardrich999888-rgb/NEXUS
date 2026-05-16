//! ETK Execution Event Schema v1.0 (LOCKED).
//!
//! Canonical data structures and serialization. No optional fields, no nulls.
//! Same event → same hash on any machine. Field order is fixed.

use sha2::{Digest, Sha256};
use std::fmt;

/// 256-bit hash. ETK uses SHA-256 for canonical, certification-friendly hashing.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash256(pub [u8; 32]);

impl Hash256 {
    /// Hash arbitrary bytes (SHA-256).
    pub fn of(data: &[u8]) -> Self {
        let mut h = Sha256::new();
        h.update(data);
        Self(h.finalize().into())
    }

    /// Hash multiple segments in order (canonical concatenation).
    pub fn of_segments(segments: &[&[u8]]) -> Self {
        let mut h = Sha256::new();
        for s in segments {
            h.update(s);
        }
        Self(h.finalize().into())
    }

    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256({})", hex::encode(&self.0[..8]))
    }
}

/// Resource class (LOCKED enum). Future hardware extends here without schema change.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceClass {
    Unknown = 0,
    Cpu = 1,
    Gpu = 2,
    Edge = 3,
    Satellite = 4,
    Neuromorphic = 5,
    Quantum = 6,
    Reserved = 255,
}

impl ResourceClass {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Unknown,
            1 => Self::Cpu,
            2 => Self::Gpu,
            3 => Self::Edge,
            4 => Self::Satellite,
            5 => Self::Neuromorphic,
            6 => Self::Quantum,
            _ => Self::Reserved,
        }
    }
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Outcome code (LOCKED). Nothing else allowed.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutcomeCode {
    Unknown = 0,
    Success = 1,
    Failure = 2,
    Terminated = 3,
    Degraded = 4,
}

impl OutcomeCode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Unknown,
            1 => Self::Success,
            2 => Self::Failure,
            3 => Self::Terminated,
            4 => Self::Degraded,
            _ => Self::Unknown,
        }
    }
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Terminal outcomes that end the execution lifecycle.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            OutcomeCode::Success | OutcomeCode::Failure | OutcomeCode::Terminated | OutcomeCode::Degraded
        )
    }
}

/// Canonical ExecutionEvent v1.0. Field order is fixed for deterministic serialization.
#[derive(Clone, PartialEq, Eq)]
pub struct ExecutionEventV1 {
    pub event_id: Hash256,
    pub execution_id: Hash256,
    pub sequence_number: u64,
    pub timestamp_utc: u64,
    pub actor_id: Hash256,
    pub workload_id: Hash256,
    pub execution_context: Hash256,
    pub resource_class: ResourceClass,
    pub jurisdiction_code: u16,
    pub policy_ref: Hash256,
    pub outcome_code: OutcomeCode,
    pub previous_event_hash: Hash256,
}

impl ExecutionEventV1 {
    /// Canonical serialization: field order exactly as schema. Big-endian. No optional fields.
    /// Used for storage/wire. event_id is included.
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 * 8 + 8 + 8 + 1 + 2 + 1);
        out.extend_from_slice(&self.event_id.0);
        out.extend_from_slice(&self.execution_id.0);
        out.extend_from_slice(&self.sequence_number.to_be_bytes());
        out.extend_from_slice(&self.timestamp_utc.to_be_bytes());
        out.extend_from_slice(&self.actor_id.0);
        out.extend_from_slice(&self.workload_id.0);
        out.extend_from_slice(&self.execution_context.0);
        out.push(self.resource_class.to_u8());
        out.extend_from_slice(&self.jurisdiction_code.to_be_bytes());
        out.extend_from_slice(&self.policy_ref.0);
        out.push(self.outcome_code.to_u8());
        out.extend_from_slice(&self.previous_event_hash.0);
        out
    }

    /// Bytes used to compute event_id: same order but WITHOUT event_id (all other fields).
    fn to_canonical_bytes_for_event_id(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 * 7 + 8 + 8 + 1 + 2 + 1);
        out.extend_from_slice(&self.execution_id.0);
        out.extend_from_slice(&self.sequence_number.to_be_bytes());
        out.extend_from_slice(&self.timestamp_utc.to_be_bytes());
        out.extend_from_slice(&self.actor_id.0);
        out.extend_from_slice(&self.workload_id.0);
        out.extend_from_slice(&self.execution_context.0);
        out.push(self.resource_class.to_u8());
        out.extend_from_slice(&self.jurisdiction_code.to_be_bytes());
        out.extend_from_slice(&self.policy_ref.0);
        out.push(self.outcome_code.to_u8());
        out.extend_from_slice(&self.previous_event_hash.0);
        out
    }

    /// Compute event_id from canonical payload (hash of event excluding event_id).
    pub fn compute_event_id(&self) -> Hash256 {
        Hash256::of(&self.to_canonical_bytes_for_event_id())
    }

    /// Parse from canonical bytes. Returns error if length or format invalid.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, SchemaError> {
        if bytes.len() != EVENT_CANONICAL_LEN {
            return Err(SchemaError::InvalidLength);
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

        let ev = Self {
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
        if ev.compute_event_id() != event_id {
            return Err(SchemaError::EventIdMismatch);
        }
        Ok(ev)
    }
}

impl fmt::Debug for ExecutionEventV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionEventV1")
            .field("event_id", &self.event_id)
            .field("execution_id", &self.execution_id)
            .field("sequence_number", &self.sequence_number)
            .field("timestamp_utc", &self.timestamp_utc)
            .field("outcome_code", &self.outcome_code)
            .finish_non_exhaustive()
    }
}

/// ExecutionProof v1.0. Constant size. event_chain_root = hash of final event.
#[derive(Clone, PartialEq, Eq)]
pub struct ExecutionProofV1 {
    pub execution_id: Hash256,
    pub event_chain_root: Hash256,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub policy_ref: Hash256,
    pub jurisdiction_code: u16,
    pub verifier_signature: [u8; 64],
}

impl ExecutionProofV1 {
    /// Canonical bytes for signing (all fields except signature).
    pub fn to_signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + 32 + 8 + 8 + 32 + 2);
        out.extend_from_slice(&self.execution_id.0);
        out.extend_from_slice(&self.event_chain_root.0);
        out.extend_from_slice(&self.start_timestamp.to_be_bytes());
        out.extend_from_slice(&self.end_timestamp.to_be_bytes());
        out.extend_from_slice(&self.policy_ref.0);
        out.extend_from_slice(&self.jurisdiction_code.to_be_bytes());
        out
    }

    /// Serialize proof for storage/wire (including signature).
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = self.to_signing_bytes();
        out.extend_from_slice(&self.verifier_signature);
        out
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, SchemaError> {
        if bytes.len() != PROOF_CANONICAL_LEN {
            return Err(SchemaError::InvalidLength);
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
        Ok(Self {
            execution_id: read_hash(&mut off),
            event_chain_root: read_hash(&mut off),
            start_timestamp: read_u64_be(&mut off),
            end_timestamp: read_u64_be(&mut off),
            policy_ref: read_hash(&mut off),
            jurisdiction_code: u16::from_be_bytes(bytes[off..off + 2].try_into().unwrap()),
            verifier_signature: bytes[off + 2..off + 66].try_into().unwrap(),
        })
    }
}

impl fmt::Debug for ExecutionProofV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionProofV1")
            .field("execution_id", &self.execution_id)
            .field("event_chain_root", &self.event_chain_root)
            .field("start_timestamp", &self.start_timestamp)
            .field("end_timestamp", &self.end_timestamp)
            .finish_non_exhaustive()
    }
}

/// Canonical byte length of ExecutionEventV1. Used for stream parsing.
pub const EVENT_CANONICAL_LEN: usize =
    32 + 32 + 8 + 8 + 32 + 32 + 32 + 1 + 2 + 32 + 1 + 32; // 276

/// Canonical byte length of ExecutionProofV1 (including signature).
pub const PROOF_CANONICAL_LEN: usize = 32 + 32 + 8 + 8 + 32 + 2 + 64; // 178

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Invalid canonical byte length")]
    InvalidLength,
    #[error("event_id does not match computed hash")]
    EventIdMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_canonical_roundtrip() {
        let ev = ExecutionEventV1 {
            event_id: Hash256::of(b"eid"),
            execution_id: Hash256::of(b"xid"),
            sequence_number: 0,
            timestamp_utc: 1_000_000,
            actor_id: Hash256::of(b"actor"),
            workload_id: Hash256::of(b"workload"),
            execution_context: Hash256::of(b"ctx"),
            resource_class: ResourceClass::Cpu,
            jurisdiction_code: 840,
            policy_ref: Hash256::of(b"policy"),
            outcome_code: OutcomeCode::Unknown,
            previous_event_hash: Hash256::zero(),
        };
        let ev2 = ev.compute_event_id();
        let mut ev_correct = ev;
        ev_correct.event_id = ev2;
        let bytes = ev_correct.to_canonical_bytes();
        let parsed = ExecutionEventV1::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(parsed.event_id, ev_correct.event_id);
        assert_eq!(parsed.sequence_number, ev_correct.sequence_number);
    }

    #[test]
    fn proof_canonical_roundtrip() {
        let proof = ExecutionProofV1 {
            execution_id: Hash256::of(b"xid"),
            event_chain_root: Hash256::of(b"root"),
            start_timestamp: 1000,
            end_timestamp: 2000,
            policy_ref: Hash256::of(b"policy"),
            jurisdiction_code: 840,
            verifier_signature: [0u8; 64],
        };
        let bytes = proof.to_canonical_bytes();
        let parsed = ExecutionProofV1::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(parsed.execution_id, proof.execution_id);
        assert_eq!(parsed.event_chain_root, proof.event_chain_root);
    }
}
