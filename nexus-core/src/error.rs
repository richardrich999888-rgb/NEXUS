// NEXUS Core: Error Types
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NexusError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
    
    #[error("Causal conflict detected: {0}")]
    CausalConflict(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
    
    #[error("Merge error: {0}")]
    MergeError(String),
    
    #[error("Invalid tensor: {0}")]
    InvalidTensor(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

pub type Result<T> = std::result::Result<T, NexusError>;
