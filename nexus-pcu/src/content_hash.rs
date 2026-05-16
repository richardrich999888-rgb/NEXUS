//! Content-addressed hashing for deterministic identification.
//!
//! Uses BLAKE3 for fast, secure hashing. All content in NEXUS is
//! identified by its cryptographic hash.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-byte BLAKE3 hash used for content addressing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    /// Compute hash of arbitrary bytes.
    #[inline]
    pub fn compute(data: &[u8]) -> Self {
        Self(blake3::hash(data).into())
    }

    /// Compute hash of multiple byte slices (streaming).
    pub fn compute_many(parts: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            hasher.update(part);
        }
        Self(hasher.finalize().into())
    }

    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the underlying bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string.
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Zero hash (for testing/placeholders only).
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Genesis/empty hash
    #[inline]
    pub const fn genesis() -> Self {
        Self([0u8; 32])
    }

    /// Check if this is the zero hash.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", &self.to_hex()[..16])
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        Self::zero()
    }
}

impl AsRef<[u8]> for ContentHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for ContentHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<ContentHash> for [u8; 32] {
    fn from(hash: ContentHash) -> Self {
        hash.0
    }
}

/// Incremental hasher for streaming content.
#[derive(Clone)]
pub struct ContentHasher {
    inner: blake3::Hasher,
}

impl ContentHasher {
    /// Create new hasher.
    pub fn new() -> Self {
        Self {
            inner: blake3::Hasher::new(),
        }
    }

    /// Add data to hasher.
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.inner.update(data);
        self
    }

    /// Finalize and return hash.
    pub fn finalize(self) -> ContentHash {
        ContentHash(self.inner.finalize().into())
    }

    /// Finalize but allow continued hashing.
    pub fn finalize_reset(&mut self) -> ContentHash {
        let hash = ContentHash(self.inner.finalize().into());
        self.inner.reset();
        hash
    }
}

impl Default for ContentHasher {
    fn default() -> Self {
        Self::new()
    }
}
