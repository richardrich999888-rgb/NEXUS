//! TELOS Protocol error types.

use thiserror::Error;

/// Errors that can occur in the TELOS protocol.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum TelosError {
    // ─────────────────────────────────────────────────────────────
    // Layer 1: Commitment Membrane Errors
    // ─────────────────────────────────────────────────────────────
    
    /// Decision is still in reversible zone.
    #[error("Decision '{0}' has not crossed the commitment membrane")]
    NotCommitted(String),
    
    /// Attempted to modify a committed decision.
    #[error("Cannot modify committed decision '{0}'")]
    AlreadyCommitted(String),
    
    /// Crossing was rejected.
    #[error("Crossing rejected: {reason}")]
    CrossingRejected { reason: String },

    // ─────────────────────────────────────────────────────────────
    // Layer 2: Entropy Errors
    // ─────────────────────────────────────────────────────────────
    
    /// Insufficient entropy budget.
    #[error("Insufficient entropy: required {required}, available {available}")]
    InsufficientEntropy { required: u64, available: u64 },
    
    /// Entropy source failure.
    #[error("Entropy source failure: {0}")]
    EntropySourceFailure(String),
    
    /// Invalid entropy proof.
    #[error("Invalid entropy proof: {0}")]
    InvalidEntropyProof(String),

    // ─────────────────────────────────────────────────────────────
    // Layer 3: Authority Errors
    // ─────────────────────────────────────────────────────────────
    
    /// Agent not registered.
    #[error("Agent '{0}' not found in registry")]
    AgentNotFound(String),
    
    /// Insufficient authority for decision.
    #[error("Agent '{agent}' lacks authority for scope '{scope}'")]
    InsufficientAuthority { agent: String, scope: String },
    
    /// Delegation constraint violation.
    #[error("Delegation violates constraint: {0}")]
    ConstraintViolation(String),
    
    /// Authority has been revoked.
    #[error("Authority for agent '{0}' has been revoked")]
    AuthorityRevoked(String),
    
    /// Authority chain is broken.
    #[error("Authority chain broken at '{0}'")]
    BrokenAuthorityChain(String),

    // ─────────────────────────────────────────────────────────────
    // Layer 4: Validator Errors
    // ─────────────────────────────────────────────────────────────
    
    /// Insufficient validator attestations.
    #[error("Insufficient attestations: got {got}, need {need}")]
    InsufficientAttestations { got: usize, need: usize },
    
    /// Validator not registered.
    #[error("Validator '{0}' not registered")]
    ValidatorNotFound(String),
    
    /// Validator stake insufficient.
    #[error("Validator '{validator}' stake {stake} below minimum {minimum}")]
    InsufficientStake { validator: String, stake: u64, minimum: u64 },
    
    /// Attestation signature invalid.
    #[error("Invalid attestation signature from validator '{0}'")]
    InvalidAttestationSignature(String),
    
    /// Validator timeout.
    #[error("Validator '{0}' timed out")]
    ValidatorTimeout(String),

    // ─────────────────────────────────────────────────────────────
    // Layer 5: Trust Errors
    // ─────────────────────────────────────────────────────────────
    
    /// History corruption detected.
    #[error("Commitment history corrupted at index {0}")]
    HistoryCorrupted(u64),
    
    /// Trust score cannot be transferred.
    #[error("Trust scores are non-transferable")]
    TrustNonTransferable,

    // ─────────────────────────────────────────────────────────────
    // Protocol-Level Errors
    // ─────────────────────────────────────────────────────────────
    
    /// Circuit breaker triggered.
    #[error("Circuit breaker active: {0}")]
    CircuitBreakerActive(String),
    
    /// Protocol in emergency shutdown.
    #[error("Protocol in emergency shutdown")]
    EmergencyShutdown,
    
    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for TELOS operations.
pub type TelosResult<T> = Result<T, TelosError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TelosError::InsufficientEntropy { required: 100, available: 50 };
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("50"));
    }

    #[test]
    fn test_error_equality() {
        let e1 = TelosError::AgentNotFound("alice".into());
        let e2 = TelosError::AgentNotFound("alice".into());
        assert_eq!(e1, e2);
    }
}
