// NEXUS Compression Layer - Powered by VECTRA
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// This module integrates VECTRA's deterministic lossless compression
// with NEXUS's PCU and USO abstractions.

pub mod pcu_compress;
pub mod uso_compress;

// Re-export VECTRA types for convenience
pub use vectra::{
    // Core encode/decode
    vectra_encode,
    vectra_decode,
    try_encode,
    EncodeResult,
    
    // Artifact format
    Artifact,
    Payload,
    
    // Types
    VectraError,
    VectraResult,
    
    // Utilities
    compression_ratio,
    is_encoding_beneficial,
    compute_byte_entropy,
    
    // Version
    VERSION as VECTRA_VERSION,
};

// Re-export our integration modules
pub use pcu_compress::CompressedPCU;
pub use uso_compress::CompressedUSO;

/// NEXUS-VECTRA integration version
pub const COMPRESS_VERSION: &str = "1.0.0";
