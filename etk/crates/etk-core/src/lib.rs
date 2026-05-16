//! ETK cryptographic kernel v1.0.
//!
//! Passive, append-only verifiable execution proof. Deterministic binary encoding.
//! Zero trust assumptions. Offline verifier.

pub mod chain;
pub mod codec;
pub mod constants;
pub mod crypto;
pub mod genesis;
pub mod proof;
pub mod verifier;

pub use chain::{EventChain, ChainError};
pub use codec::{decode_event, decode_proof, encode_event, encode_proof, compute_event_id, CodecError};
pub use constants::{ETK_SCHEMA_VERSION, ZERO_HASH};
pub use crypto::hash256;
pub use genesis::{create_genesis, derive_execution_id, is_genesis};
pub use proof::build_proof;
pub use verifier::{verify, Verdict, VerifierError};

// Re-export types for convenience.
pub use etk_types::{
    ExecutionEvent, ExecutionProof, Hash256, OutcomeCode, ResourceClass,
    EVENT_CANONICAL_LEN, PROOF_CANONICAL_LEN,
};
