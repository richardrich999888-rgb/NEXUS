//! Execution Truth Kernel (ETK) v1.0.
//!
//! Passive, append-only system that generates verifiable cryptographic proofs
//! that a specific execution occurred under specific constraints.
//!
//! Not: enforcement, scheduling, intelligence, policy engines. Just truth capture.

pub mod chain;
pub mod genesis;
pub mod schema;
pub mod verifier;

pub use chain::{EventChain, ChainError};
pub use genesis::{create_genesis, derive_execution_id, is_genesis};
pub use schema::{
    ExecutionEventV1, ExecutionProofV1, Hash256, OutcomeCode, ResourceClass, SchemaError,
};
pub use verifier::{verify, verify_verdict, Verdict, VerifierError, VerifierErrorCode};

/// Schema version. Immutable; any change requires new version.
pub const ETK_SCHEMA_VERSION: &str = "1.0";
