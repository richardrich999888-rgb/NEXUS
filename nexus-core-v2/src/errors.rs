use thiserror::Error;
use crate::hash::Hash;

#[derive(Error, Debug)]
pub enum NexusError {
    #[error("missing parent: {0}")]
    MissingParent(Hash),
    
    #[error("function not found: {0}")]
    FunctionNotFound(Hash),
    
    #[error("invalid proof")]
    InvalidProof,
    
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("entry not found: {0}")]
    EntryNotFound(Hash),
    
    #[error("storage error: {0}")]
    StorageError(String),
    
    #[error("sync error: {0}")]
    SyncError(String),
}

pub type Result<T> = std::result::Result<T, NexusError>;
