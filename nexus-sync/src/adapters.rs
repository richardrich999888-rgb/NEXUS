// NEXUS-CAUSALUX Adapters
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Adapters to bridge NEXUS and CAUSALUX type systems

use nexus_pcu::ContentHash;

/// Adapter utilities for NEXUS-CAUSALUX integration
/// 
/// Note: CAUSALUX ContentAddress is for position-independent text references,
/// while NEXUS ContentHash is for content-addressed storage. They serve
/// different purposes and are not directly interchangeable.
pub struct ContentHashAdapter;

impl ContentHashAdapter {
    /// Compute NEXUS ContentHash from data
    pub fn hash_of(data: &[u8]) -> ContentHash {
        ContentHash::compute(data)
    }

    /// Compute hex string of a content hash (for use in CAUSALUX operations)
    pub fn to_hex(hash: &ContentHash) -> String {
        hash.to_hex()
    }

    /// Parse hex string to ContentHash
    pub fn from_hex(hex: &str) -> Option<ContentHash> {
        if hex.len() != 64 {
            return None;
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&hex[i*2..i*2+2], 16).ok()?;
        }
        Some(ContentHash::from_bytes(bytes))
    }
}

/// Marker trait for types that can be synchronized
pub trait Syncable {
    /// Get unique identifier for sync
    fn sync_id(&self) -> ContentHash;
    
    /// Get version for conflict detection
    fn version(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let data = b"test data";
        let hash = ContentHashAdapter::hash_of(data);
        assert!(!hash.to_hex().is_empty());
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = b"test data for roundtrip";
        let hash = ContentHashAdapter::hash_of(data);
        let hex = ContentHashAdapter::to_hex(&hash);
        let restored = ContentHashAdapter::from_hex(&hex).unwrap();
        assert_eq!(hash, restored);
    }
}
