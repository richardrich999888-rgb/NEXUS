//! ETK Schema Lock v1.0 — types only.
//!
//! Fixed-size binary types. No strings. No runtime parsing ambiguity.
//! Canonical encoding is defined in etk-core (codec).

use std::fmt;

/// 256-bit hash. Opaque; hashing and encoding live in etk-core.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash256(pub [u8; 32]);

impl Hash256 {
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
        write!(f, "Hash256({}..)", hex::encode(&self.0[..4]))
    }
}

/// Resource class (LOCKED). Future hardware extends here without schema change.
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

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            OutcomeCode::Success
                | OutcomeCode::Failure
                | OutcomeCode::Terminated
                | OutcomeCode::Degraded
        )
    }
}

/// ExecutionEvent v1.0. Field order fixed for canonical encoding (etk-core).
#[derive(Clone, PartialEq, Eq)]
pub struct ExecutionEvent {
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

impl fmt::Debug for ExecutionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionEvent")
            .field("execution_id", &self.execution_id)
            .field("sequence_number", &self.sequence_number)
            .field("outcome_code", &self.outcome_code)
            .finish_non_exhaustive()
    }
}

/// ExecutionProof v1.0. Constant size. event_chain_root = final event's event_id.
#[derive(Clone, PartialEq, Eq)]
pub struct ExecutionProof {
    pub execution_id: Hash256,
    pub event_chain_root: Hash256,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub policy_ref: Hash256,
    pub jurisdiction_code: u16,
    pub verifier_signature: [u8; 64],
}

impl fmt::Debug for ExecutionProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionProof")
            .field("execution_id", &self.execution_id)
            .field("event_chain_root", &self.event_chain_root)
            .finish_non_exhaustive()
    }
}

/// Canonical byte lengths (for stream parsing / verification).
pub const EVENT_CANONICAL_LEN: usize =
    32 + 32 + 8 + 8 + 32 + 32 + 32 + 1 + 2 + 32 + 1 + 32; // 276
pub const PROOF_CANONICAL_LEN: usize = 32 + 32 + 8 + 8 + 32 + 2 + 64; // 178
