//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! Error types for VECTRA operations.
//!
//! All errors are explicit and recoverable.
//! No panics in production code paths.

use crate::types::ArtifactError;
use thiserror::Error;

/// Top-level VECTRA error type.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum VectraError {
    /// Decomposition of payload failed
    #[error("decomposition failed: {reason}")]
    DecompositionFailed { reason: String },

    /// FEE encoding failed
    #[error("FEE encoding failed: {reason}")]
    FeeEncodingFailed { reason: String },

    /// SPE prediction failed
    #[error("SPE prediction failed: {reason}")]
    SpePredictionFailed { reason: String },

    /// EBTA validation rejected the residual
    #[error("EBTA validation failed: entropy {entropy:.4} exceeds maximum {max:.4}")]
    EbtaValidationFailed { entropy: f64, max: f64 },

    /// Artifact construction failed
    #[error("artifact construction failed: {reason}")]
    ArtifactConstructionFailed { reason: String },

    /// Artifact error (deserialization, integrity, version)
    #[error("artifact error: {0}")]
    Artifact(#[from] ArtifactError),

    /// Decoding failed
    #[error("decode failed: {reason}")]
    DecodeFailed { reason: String },

    /// Integrity verification failed
    #[error("integrity verification failed: {reason}")]
    IntegrityFailed { reason: String },

    /// Input validation failed
    #[error("invalid input: {reason}")]
    InvalidInput { reason: String },

    /// Internal invariant violation (should never happen)
    #[error("internal error: {reason}")]
    InternalError { reason: String },
}

/// Result type alias for VECTRA operations.
pub type VectraResult<T> = Result<T, VectraError>;

/// Encoding-specific error (subset of VectraError).
///
/// Used internally to distinguish fail-open conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    /// Decomposition failed - cannot proceed
    Decomposition(String),
    /// FEE failed - cannot encode structure
    Fee(String),
    /// SPE failed - cannot predict
    Spe(String),
    /// EBTA rejected - entropy too high (this triggers fail-open)
    Ebta { entropy: f64, max: f64 },
    /// Artifact assembly failed
    Artifact(String),
}

impl EncodeError {
    /// Check if this error should trigger fail-open behavior.
    ///
    /// EBTA rejection is expected and triggers pass-through.
    /// Other errors indicate bugs or malformed input.
    #[must_use]
    pub fn is_fail_open_condition(&self) -> bool {
        matches!(self, Self::Ebta { .. })
    }
}

impl From<EncodeError> for VectraError {
    fn from(e: EncodeError) -> Self {
        match e {
            EncodeError::Decomposition(reason) => Self::DecompositionFailed { reason },
            EncodeError::Fee(reason) => Self::FeeEncodingFailed { reason },
            EncodeError::Spe(reason) => Self::SpePredictionFailed { reason },
            EncodeError::Ebta { entropy, max } => Self::EbtaValidationFailed { entropy, max },
            EncodeError::Artifact(reason) => Self::ArtifactConstructionFailed { reason },
        }
    }
}

/// Decoding-specific error.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// Artifact integrity check failed
    IntegrityFailed(String),
    /// Version mismatch
    VersionMismatch { expected: u64, found: u64 },
    /// Structure regeneration failed
    StructureRegeneration(String),
    /// Variable reconstruction failed
    VariableReconstruction(String),
    /// Final recomposition failed
    Recomposition(String),
    /// Output hash mismatch (losslessness violation)
    OutputHashMismatch,
}

impl From<DecodeError> for VectraError {
    fn from(e: DecodeError) -> Self {
        match e {
            DecodeError::IntegrityFailed(reason) => Self::IntegrityFailed { reason },
            DecodeError::VersionMismatch { expected, found } => {
                Self::Artifact(ArtifactError::VersionMismatch { expected, found })
            }
            DecodeError::StructureRegeneration(reason)
            | DecodeError::VariableReconstruction(reason)
            | DecodeError::Recomposition(reason) => Self::DecodeFailed { reason },
            DecodeError::OutputHashMismatch => Self::IntegrityFailed {
                reason: "output hash does not match expected value".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebta_error_is_fail_open() {
        let err = EncodeError::Ebta {
            entropy: 5.0,
            max: 4.0,
        };
        assert!(err.is_fail_open_condition());
    }

    #[test]
    fn test_other_errors_not_fail_open() {
        let err = EncodeError::Decomposition("test".to_string());
        assert!(!err.is_fail_open_condition());
    }

    #[test]
    fn test_error_display() {
        let err = VectraError::EbtaValidationFailed {
            entropy: 5.1234,
            max: 4.0,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("5.1234"));
        assert!(msg.contains("4.0"));
    }
}
