//! ETK crypto. SHA-256 for schema v1.0 (certification-friendly).
//! Crypto agility layer for regulator-mandated migration (e.g. Blake3).

use etk_types::Hash256;
use sha2::{Digest, Sha256};

/// Crypto suite for agility. Schema v1.0 uses Sha256 only; future versions may allow Blake3.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CryptoSuite {
    /// SHA-256. Default for ETK v1.0; FIPS/certification-friendly.
    Sha256,
    /// BLAKE3. Optional; enable with feature "blake3" for migration or future schema.
    Blake3,
}

impl Default for CryptoSuite {
    fn default() -> Self {
        CryptoSuite::Sha256
    }
}

/// Hash256(data). Deterministic; same input → same hash on any machine.
/// Schema v1.0 canonical hashing. Do not change without schema version bump.
#[inline]
pub fn hash256(data: &[u8]) -> Hash256 {
    hash_with_suite(data, CryptoSuite::Sha256)
}

/// Hash with selected suite. Used for future schema versions or crypto migration.
pub fn hash_with_suite(data: &[u8], suite: CryptoSuite) -> Hash256 {
    match suite {
        CryptoSuite::Sha256 => {
            let mut h = Sha256::new();
            h.update(data);
            Hash256(h.finalize().into())
        }
        CryptoSuite::Blake3 => {
            #[cfg(feature = "blake3")]
            {
                Hash256(*blake3::hash(data).as_bytes())
            }
            #[cfg(not(feature = "blake3"))]
            {
                // Fallback to SHA-256 when blake3 feature not enabled.
                let mut h = Sha256::new();
                h.update(data);
                Hash256(h.finalize().into())
            }
        }
    }
}
