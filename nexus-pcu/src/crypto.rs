//! NEXUS Cryptographic Utilities
//!
//! Provides robust Ed25519 key generation, signing, and verification.
//! This module is the single source of truth for cryptographic operations.
//!
//! ## Security Model
//!
//! - **Classical**: Ed25519 for all current operations
//! - **PQC-Ready**: Types support future ML-DSA integration
//! - **Deterministic**: All operations are reproducible
//!
//! Copyright (c) 2025 SYNTRIASS Labs Private Limited
//! Inventor: Katta Naga Sri Ganesh

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::content_hash::ContentHash;

/// Cryptographic errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Invalid signature format")]
    InvalidSignatureFormat,

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

/// Result type for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

// ============================================================================
// KEY GENERATION
// ============================================================================

/// Generate a new Ed25519 signing key using OS-provided entropy
pub fn generate_signing_key() -> SigningKey {
    let mut secret = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut secret);
    SigningKey::from_bytes(&secret)
}

/// Derive verifying (public) key from signing key
pub fn derive_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
    signing_key.verifying_key()
}

// ============================================================================
// SIGNING
// ============================================================================

/// Sign a message with Ed25519
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Sign and return raw bytes
pub fn sign_bytes(signing_key: &SigningKey, message: &[u8]) -> [u8; 64] {
    signing_key.sign(message).to_bytes()
}

// ============================================================================
// VERIFICATION
// ============================================================================

/// Verify an Ed25519 signature
pub fn verify(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> CryptoResult<()> {
    verifying_key
        .verify(message, signature)
        .map_err(|_| CryptoError::VerificationFailed)
}

/// Verify signature from raw bytes
pub fn verify_bytes(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> CryptoResult<()> {
    let signature = Signature::from_bytes(signature_bytes);
    verify(verifying_key, message, &signature)
}

// ============================================================================
// PCU-BOUND LICENSE
// ============================================================================

/// A cryptographically-bound license for PCU execution.
///
/// Unlike hostname-based licenses, this binds to the PCU's content hash,
/// ensuring the license is valid only for specific, verified code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcuLicense {
    /// License ID (unique identifier)
    pub license_id: String,

    /// Organization this license is issued to
    pub organization: String,

    /// Content hash of the PCU code this license covers
    pub pcu_code_hash: ContentHash,

    /// Features enabled by this license
    pub features: Vec<String>,

    /// License tier (Standard, Premium, Enterprise)
    pub tier: LicenseTier,

    /// Expiration timestamp (Unix epoch seconds)
    pub expires_at: u64,

    /// Maximum executions allowed (0 = unlimited)
    pub max_executions: u64,

    /// Ed25519 signature over license data (from SYNTRIASS)
    pub signature: Vec<u8>,
}

/// License tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseTier {
    /// Standard features only
    Standard,
    /// Premium features included
    Premium,
    /// Full enterprise access
    Enterprise,
}

impl PcuLicense {
    /// Create a new unsigned license
    pub fn new(
        license_id: impl Into<String>,
        organization: impl Into<String>,
        pcu_code_hash: ContentHash,
        tier: LicenseTier,
        expires_at: u64,
    ) -> Self {
        Self {
            license_id: license_id.into(),
            organization: organization.into(),
            pcu_code_hash,
            features: Vec::new(),
            tier,
            expires_at,
            max_executions: 0, // unlimited
            signature: Vec::new(),
        }
    }

    /// Add a feature to the license
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Set max executions
    pub fn with_max_executions(mut self, max: u64) -> Self {
        self.max_executions = max;
        self
    }

    /// Get the data to sign (canonical representation)
    fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.license_id.as_bytes());
        data.extend_from_slice(self.organization.as_bytes());
        data.extend_from_slice(self.pcu_code_hash.as_bytes());
        for feature in &self.features {
            data.extend_from_slice(feature.as_bytes());
        }
        data.push(self.tier as u8);
        data.extend_from_slice(&self.expires_at.to_le_bytes());
        data.extend_from_slice(&self.max_executions.to_le_bytes());
        data
    }

    /// Sign this license with the issuer's key
    pub fn sign(&mut self, issuer_key: &SigningKey) {
        let data = self.signing_data();
        let sig = issuer_key.sign(&data);
        self.signature = sig.to_bytes().to_vec();
    }

    /// Verify the license signature
    pub fn verify_signature(&self, issuer_public_key: &VerifyingKey) -> CryptoResult<()> {
        if self.signature.len() != 64 {
            return Err(CryptoError::InvalidSignatureFormat);
        }

        let sig_bytes: [u8; 64] = self.signature.as_slice().try_into()
            .map_err(|_| CryptoError::InvalidSignatureFormat)?;
        
        let signature = Signature::from_bytes(&sig_bytes);
        let data = self.signing_data();
        
        issuer_public_key
            .verify(&data, &signature)
            .map_err(|_| CryptoError::VerificationFailed)
    }

    /// Check if license is valid for a specific PCU code hash
    pub fn is_valid_for_pcu(&self, pcu_code_hash: &ContentHash) -> bool {
        self.pcu_code_hash == *pcu_code_hash
    }

    /// Check if license is expired
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    /// Check if license has a specific feature
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    /// Full validation (signature + expiration + PCU binding)
    pub fn validate(
        &self,
        issuer_public_key: &VerifyingKey,
        pcu_code_hash: &ContentHash,
    ) -> CryptoResult<()> {
        // Check signature
        self.verify_signature(issuer_public_key)?;

        // Check PCU binding
        if !self.is_valid_for_pcu(pcu_code_hash) {
            return Err(CryptoError::VerificationFailed);
        }

        // Check expiration
        if self.is_expired() {
            return Err(CryptoError::VerificationFailed);
        }

        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key1 = generate_signing_key();
        let key2 = generate_signing_key();
        
        // Keys should be different
        assert_ne!(key1.to_bytes(), key2.to_bytes());
    }

    #[test]
    fn test_sign_verify() {
        let key = generate_signing_key();
        let verifying_key = derive_verifying_key(&key);
        let message = b"Test message for signing";

        let signature = sign(&key, message);
        
        // Should verify
        assert!(verify(&verifying_key, message, &signature).is_ok());
        
        // Wrong message should fail
        assert!(verify(&verifying_key, b"Wrong message", &signature).is_err());
    }

    #[test]
    fn test_pcu_license_creation_and_signing() {
        let issuer_key = generate_signing_key();
        let issuer_public = derive_verifying_key(&issuer_key);
        
        let pcu_hash = ContentHash::compute(b"my_pcu_code");
        
        let mut license = PcuLicense::new(
            "LIC-001",
            "ACME Corp",
            pcu_hash,
            LicenseTier::Enterprise,
            u64::MAX, // Never expires for test
        )
        .with_feature("advanced_analytics")
        .with_feature("priority_support");
        
        license.sign(&issuer_key);
        
        // Should verify
        assert!(license.verify_signature(&issuer_public).is_ok());
        assert!(license.is_valid_for_pcu(&pcu_hash));
        assert!(!license.is_expired());
        assert!(license.has_feature("advanced_analytics"));
    }

    #[test]
    fn test_pcu_license_wrong_pcu() {
        let issuer_key = generate_signing_key();
        let issuer_public = derive_verifying_key(&issuer_key);
        
        let licensed_pcu = ContentHash::compute(b"licensed_code");
        let different_pcu = ContentHash::compute(b"different_code");
        
        let mut license = PcuLicense::new(
            "LIC-002",
            "Test Org",
            licensed_pcu,
            LicenseTier::Standard,
            u64::MAX,
        );
        license.sign(&issuer_key);
        
        // Should fail for different PCU
        assert!(license.validate(&issuer_public, &different_pcu).is_err());
        
        // Should pass for licensed PCU
        assert!(license.validate(&issuer_public, &licensed_pcu).is_ok());
    }

    #[test]
    fn test_pcu_license_tamper_detection() {
        let issuer_key = generate_signing_key();
        let issuer_public = derive_verifying_key(&issuer_key);
        
        let pcu_hash = ContentHash::compute(b"code");
        
        let mut license = PcuLicense::new(
            "LIC-003",
            "Org",
            pcu_hash,
            LicenseTier::Standard,
            u64::MAX,
        );
        license.sign(&issuer_key);
        
        // Tamper with the license
        license.tier = LicenseTier::Enterprise;
        
        // Should now fail verification
        assert!(license.verify_signature(&issuer_public).is_err());
    }
}
