// NEXUS Storage: Error Types
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NexusStorageError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Write operation failed: {0}")]
    WriteFailed(String),

    #[error("Read operation failed: {0}")]
    ReadFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Index corruption: {0}")]
    IndexCorruption(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Causal violation: {0}")]
    CausalViolation(String),
}
