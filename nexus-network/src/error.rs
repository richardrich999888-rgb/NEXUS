// NEXUS Network: Error Types
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NexusNetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Identity verification failed: {0}")]
    AuthError(String),
    
    #[error("Gossip error: {0}")]
    GossipError(String),
    
    #[error("Sync conflict: {0}")]
    SyncError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}
