//! Error types for the NEXUS executor.
//!
//! All errors are typed and include context for debugging.
//! Errors are designed to be:
//! - Specific enough for programmatic handling
//! - Descriptive enough for debugging
//! - Safe (no sensitive data leaked)

use nexus_pcu::Capability;
use std::time::Duration;
use thiserror::Error;

/// Result type alias for executor operations.
pub type ExecutorResult<T> = Result<T, ExecutorError>;

/// Errors that can occur during PCU execution.
#[derive(Error, Debug)]
pub enum ExecutorError {
    // =========================================================================
    // Validation Errors (before execution)
    // =========================================================================
    /// WASM module is invalid or malformed.
    #[error("Invalid WASM module: {reason}")]
    InvalidModule { 
        /// Reason for invalidation.
        reason: String 
    },

    /// WASM module exceeds size limit.
    #[error("Module too large: {size} bytes (max: {max} bytes)")]
    ModuleTooLarge { 
        /// Actual size in bytes.
        size: usize, 
        /// Maximum allowed size in bytes.
        max: usize 
    },

    /// Module uses disallowed WASM features.
    #[error("Disallowed WASM feature: {feature}")]
    DisallowedFeature { 
        /// Name of the disallowed feature.
        feature: String 
    },

    /// PCU is malformed or invalid.
    #[error("Invalid PCU: {reason}")]
    InvalidPcu { 
        /// Reason for invalidation.
        reason: String 
    },

    /// Execution blocked by guard (biological / accountability constraint).
    #[error("Execution blocked: {reason}")]
    ExecutionBlocked {
        /// Reason from guard (e.g. developmental stage, autonomic risk, TELOS).
        reason: String,
    },

    /// Input data is missing or invalid.
    #[error("Invalid input at index {index}: {reason}")]
    InvalidInput { 
        /// Index of the invalid input.
        index: usize, 
        /// Reason for invalidation.
        reason: String 
    },

    // =========================================================================
    // Identity/Capability Errors
    // =========================================================================
    /// Identity has insufficient capabilities for this operation.
    #[error("Insufficient capabilities: required {required:?}")]
    InsufficientCapabilities { 
        /// The missing capability.
        required: Capability 
    },

    /// Identity has expired.
    #[error("Identity expired at {expired_at}")]
    IdentityExpired { 
        /// Expired at (Unix timestamp).
        expired_at: u64 
    },

    /// Delegation chain is invalid.
    #[error("Invalid delegation chain: {reason}")]
    InvalidDelegation { 
        /// Reason for invalidation.
        reason: String 
    },

    /// Signature verification failed.
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    // =========================================================================
    // Execution Errors
    // =========================================================================
    /// WASM execution trapped (runtime error in WASM code).
    #[error("WASM trap: {message}")]
    WasmTrap { 
        /// Trap message.
        message: String 
    },

    /// Execution exceeded time limit.
    #[error("Execution timeout after {elapsed:?} (limit: {limit:?})")]
    Timeout { 
        /// Total time elapsed.
        elapsed: Duration, 
        /// The time limit.
        limit: Duration 
    },

    /// Execution exceeded fuel (instruction) limit.
    #[error("Fuel exhausted: consumed {consumed} (limit: {limit})")]
    FuelExhausted { 
        /// Total fuel consumed.
        consumed: u64, 
        /// The fuel limit.
        limit: u64 
    },

    /// Execution exceeded memory limit.
    #[error("Memory limit exceeded: requested {requested} bytes (limit: {limit} bytes)")]
    MemoryLimitExceeded { 
        /// Bytes requested.
        requested: usize, 
        /// The memory limit in bytes.
        limit: usize 
    },

    /// Output exceeds maximum allowed size.
    #[error("Output too large: {size} bytes (max: {max} bytes)")]
    OutputTooLarge { 
        /// Actual output size in bytes.
        size: usize, 
        /// Maximum allowed output size in bytes.
        max: usize 
    },

    /// Required entry point not found in WASM module.
    #[error("Entry point not found: tried {tried:?}")]
    EntryPointNotFound { 
        /// List of entry points tried.
        tried: Vec<String> 
    },

    /// WASM module has no memory export.
    #[error("Module has no memory export")]
    NoMemoryExport,

    /// Output length function returned invalid value.
    #[error("Invalid output length: {length}")]
    InvalidOutputLength { 
        /// The invalid length value.
        length: i64 
    },

    // =========================================================================
    // Host Function Errors
    // =========================================================================
    /// Host function called with invalid arguments.
    #[error("Host function error in {function}: {reason}")]
    HostFunctionError { 
        /// Name of the guest function.
        function: String, 
        /// Reason for failure.
        reason: String 
    },

    /// Input index out of bounds in host function.
    #[error("Input index {index} out of bounds (have {count} inputs)")]
    InputIndexOutOfBounds { 
        /// Index requested.
        index: usize, 
        /// Total inputs available.
        count: usize 
    },

    // =========================================================================
    // Proof Errors
    // =========================================================================
    /// Proof generation failed.
    #[error("Failed to generate proof: {reason}")]
    ProofGenerationFailed { 
        /// Reason for failure.
        reason: String 
    },

    /// Proof verification failed.
    #[error("Proof verification failed: {reason}")]
    ProofVerificationFailed { 
        /// Reason for failure.
        reason: String 
    },

    // =========================================================================
    // Cache Errors
    // =========================================================================
    /// Cache lookup failed.
    #[error("Cache error: {reason}")]
    CacheError { 
        /// Reason for cache failure.
        reason: String 
    },

    // =========================================================================
    // System Errors
    // =========================================================================
    /// Wasmtime engine error.
    #[error("Wasmtime error: {0}")]
    WasmtimeError(#[from] wasmtime::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error (should not happen in normal operation).
    #[error("Internal error: {0}")]
    InternalError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ExecutorError {
    /// Check if this error is retryable.
    ///
    /// Some errors may succeed on retry (e.g., timeouts due to system load),
    /// while others are permanent (e.g., invalid module).
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ExecutorError::Timeout { .. }
                | ExecutorError::CacheError { .. }
                | ExecutorError::IoError(_)
        )
    }

    /// Check if this error is due to resource exhaustion.
    pub fn is_resource_exhaustion(&self) -> bool {
        matches!(
            self,
            ExecutorError::Timeout { .. }
                | ExecutorError::FuelExhausted { .. }
                | ExecutorError::MemoryLimitExceeded { .. }
                | ExecutorError::OutputTooLarge { .. }
        )
    }

    /// Check if this error is due to invalid input.
    pub fn is_validation_error(&self) -> bool {
        matches!(
            self,
            ExecutorError::InvalidModule { .. }
                | ExecutorError::ModuleTooLarge { .. }
                | ExecutorError::DisallowedFeature { .. }
                | ExecutorError::InvalidPcu { .. }
                | ExecutorError::InvalidInput { .. }
        )
    }

    /// Check if this error is due to authorization failure.
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            ExecutorError::InsufficientCapabilities { .. }
                | ExecutorError::IdentityExpired { .. }
                | ExecutorError::InvalidDelegation { .. }
                | ExecutorError::SignatureVerificationFailed
                | ExecutorError::ExecutionBlocked { .. }
        )
    }

    /// Get error code for metrics/logging.
    pub fn error_code(&self) -> &'static str {
        match self {
            ExecutorError::InvalidModule { .. } => "INVALID_MODULE",
            ExecutorError::ModuleTooLarge { .. } => "MODULE_TOO_LARGE",
            ExecutorError::DisallowedFeature { .. } => "DISALLOWED_FEATURE",
            ExecutorError::InvalidPcu { .. } => "INVALID_PCU",
            ExecutorError::ExecutionBlocked { .. } => "EXECUTION_BLOCKED",
            ExecutorError::InvalidInput { .. } => "INVALID_INPUT",
            ExecutorError::InsufficientCapabilities { .. } => "INSUFFICIENT_CAPS",
            ExecutorError::IdentityExpired { .. } => "IDENTITY_EXPIRED",
            ExecutorError::InvalidDelegation { .. } => "INVALID_DELEGATION",
            ExecutorError::SignatureVerificationFailed => "SIG_VERIFY_FAILED",
            ExecutorError::WasmTrap { .. } => "WASM_TRAP",
            ExecutorError::Timeout { .. } => "TIMEOUT",
            ExecutorError::FuelExhausted { .. } => "FUEL_EXHAUSTED",
            ExecutorError::MemoryLimitExceeded { .. } => "MEMORY_EXCEEDED",
            ExecutorError::OutputTooLarge { .. } => "OUTPUT_TOO_LARGE",
            ExecutorError::EntryPointNotFound { .. } => "NO_ENTRY_POINT",
            ExecutorError::NoMemoryExport => "NO_MEMORY",
            ExecutorError::InvalidOutputLength { .. } => "INVALID_OUTPUT_LEN",
            ExecutorError::HostFunctionError { .. } => "HOST_FUNC_ERROR",
            ExecutorError::InputIndexOutOfBounds { .. } => "INPUT_OOB",
            ExecutorError::ProofGenerationFailed { .. } => "PROOF_GEN_FAILED",
            ExecutorError::ProofVerificationFailed { .. } => "PROOF_VERIFY_FAILED",
            ExecutorError::CacheError { .. } => "CACHE_ERROR",
            ExecutorError::WasmtimeError(_) => "WASMTIME_ERROR",
            ExecutorError::SerializationError(_) => "SERIALIZATION_ERROR",
            ExecutorError::InternalError(_) => "INTERNAL_ERROR",
            ExecutorError::IoError(_) => "IO_ERROR",
        }
    }
}

impl From<bincode::Error> for ExecutorError {
    fn from(e: bincode::Error) -> Self {
        ExecutorError::SerializationError(e.to_string())
    }
}

impl From<ed25519_dalek::SignatureError> for ExecutorError {
    fn from(_: ed25519_dalek::SignatureError) -> Self {
        ExecutorError::SignatureVerificationFailed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let timeout = ExecutorError::Timeout {
            elapsed: Duration::from_secs(30),
            limit: Duration::from_secs(30),
        };
        assert!(timeout.is_retryable());
        assert!(timeout.is_resource_exhaustion());
        assert!(!timeout.is_validation_error());
        assert!(!timeout.is_auth_error());

        let invalid = ExecutorError::InvalidModule {
            reason: "test".into(),
        };
        assert!(!invalid.is_retryable());
        assert!(!invalid.is_resource_exhaustion());
        assert!(invalid.is_validation_error());

        let auth = ExecutorError::IdentityExpired { expired_at: 0 };
        assert!(!auth.is_retryable());
        assert!(auth.is_auth_error());
    }

    #[test]
    fn test_error_codes() {
        let errors = vec![
            ExecutorError::InvalidModule {
                reason: "test".into(),
            },
            ExecutorError::Timeout {
                elapsed: Duration::from_secs(1),
                limit: Duration::from_secs(1),
            },
            ExecutorError::WasmTrap {
                message: "test".into(),
            },
        ];

        for err in errors {
            assert!(!err.error_code().is_empty());
        }
    }
}
