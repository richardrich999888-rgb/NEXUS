//! Error types for secret management

use thiserror::Error;

pub type SecretResult<T> = Result<T, SecretError>;

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("Secret not found: {0}")]
    NotFound(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Backend not configured: {0}")]
    BackendNotConfigured(String),

    #[error("Key rotation failed: {0}")]
    RotationFailed(String),
}


