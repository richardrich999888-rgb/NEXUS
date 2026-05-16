// NEXUS Post-Quantum Cryptography Module
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Implements hybrid cryptography: Ed25519 (classical) + ML-DSA (post-quantum)
// for quantum-resistant signatures during the transition period.
//
// Design Philosophy:
// - Hybrid signatures: both classical and PQC signatures are computed
// - Defense-in-depth: either signature validating is sufficient
// - Backward compatible: classical-only clients can still verify
//
// NOTE: Full PQC implementation pending ml-dsa/ml-kem stabilization.
// Currently provides classical-only with PQC-ready types.
// When `--features pqc` is enabled, PQC fields are reserved but not populated.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};

#[cfg(feature = "pqc")]
use fips204::ml_dsa_65;
#[cfg(feature = "pqc")]
use fips204::traits::{SerDes, Signer as PqcSigner, Verifier as PqcVerifier};
#[cfg(feature = "pqc")]
use rand::RngCore;

/// PQC-related errors
#[derive(Debug, Error)]
pub enum PqcError {
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("PQC not yet implemented - awaiting ml-dsa stabilization")]
    PqcNotImplemented,
}

/// Result type for PQC operations
pub type PqcResult<T> = Result<T, PqcError>;

// ============================================================================
// HYBRID SIGNATURE
// ============================================================================

/// Hybrid signature containing both classical (Ed25519) and post-quantum (ML-DSA) signatures.
///
/// During the transition period (2025-2030), we compute both signatures.
/// Verification passes if EITHER signature is valid, providing defense-in-depth.
///
/// # Current Status
/// 
/// Full PQC support pending ml-dsa crate stabilization (requires rand_core 0.9).
/// Currently only classical signatures are computed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridSignature {
    /// Classical Ed25519 signature (64 bytes)
    pub classical: Vec<u8>,

    /// Post-quantum ML-DSA-65 signature (~3,293 bytes)
    /// Reserved for future PQC implementation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pqc: Option<Vec<u8>>,

    /// Signature scheme version for forward compatibility
    pub version: u8,
}

impl HybridSignature {
    /// Current signature scheme version
    pub const VERSION: u8 = 1;

    /// Create a classical-only signature
    pub fn classical_only(signature: &Ed25519Signature) -> Self {
        Self {
            classical: signature.to_bytes().to_vec(),
            pqc: None,
            version: Self::VERSION,
        }
    }

    /// Create a hybrid signature with both classical and PQC
    /// Reserved for future PQC implementation
    pub fn hybrid(classical: &Ed25519Signature, pqc: &[u8]) -> Self {
        Self {
            classical: classical.to_bytes().to_vec(),
            pqc: Some(pqc.to_vec()),
            version: Self::VERSION,
        }
    }

    /// Get classical signature bytes
    pub fn classical_bytes(&self) -> &[u8] {
        &self.classical
    }

    /// Get PQC signature bytes (if present)
    pub fn pqc_bytes(&self) -> Option<&[u8]> {
        self.pqc.as_deref()
    }

    /// Check if this is a hybrid signature (has PQC component)
    pub fn is_hybrid(&self) -> bool {
        self.pqc.is_some()
    }

    /// Total size in bytes
    pub fn size(&self) -> usize {
        self.classical.len() + self.pqc.as_ref().map(|p| p.len()).unwrap_or(0)
    }

    /// Verify the classical (Ed25519) signature
    pub fn verify_classical(&self, message: &[u8], public_key: &VerifyingKey) -> bool {
        if self.classical.len() != 64 {
            return false;
        }

        let sig_bytes: [u8; 64] = match self.classical.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };

        let signature = Ed25519Signature::from_bytes(&sig_bytes);
        public_key.verify(message, &signature).is_ok()
    }

    /// Verify the PQC (ML-DSA) signature
    pub fn verify_pqc(&self, message: &[u8], public_key: &[u8]) -> bool {
        #[cfg(feature = "pqc")]
        {
            if let Some(sig_bytes) = &self.pqc {
                let Ok(pk_arr) = <[u8; 1952]>::try_from(public_key) else {
                    return false;
                };
                let Ok(pk) = ml_dsa_65::PublicKey::try_from_bytes(pk_arr) else {
                    return false;
                };
                let Ok(sig_arr) = <[u8; 3309]>::try_from(sig_bytes.as_slice()) else {
                    return false;
                };
                return pk.verify(message, &sig_arr, &[]);
            }
        }
        false
    }

    /// Verify hybrid signature: passes if EITHER signature is valid.
    /// This provides defense-in-depth during the quantum transition.
    pub fn verify_hybrid(
        &self,
        message: &[u8],
        classical_pk: &VerifyingKey,
        pqc_pk: Option<&[u8]>,
    ) -> bool {
        let classical_ok = self.verify_classical(message, classical_pk);

        // PQC verification not yet implemented
        let pqc_ok = if let (Some(pk), Some(_sig)) = (pqc_pk, &self.pqc) {
            self.verify_pqc(message, pk)
        } else {
            false
        };

        // Either signature passing is sufficient (defense-in-depth)
        classical_ok || pqc_ok
    }
}

// ============================================================================
// HYBRID KEYPAIR
// ============================================================================

/// Hybrid keypair containing both classical (Ed25519) and post-quantum (ML-DSA) keys.
///
/// # Current Status
/// 
/// Full PQC support pending ml-dsa crate stabilization.
/// Currently only classical keys are generated.
pub struct HybridKeyPair {
    /// Classical Ed25519 signing key
    pub classical: SigningKey,

    /// ML-DSA-65 private key for PQC signatures
    #[cfg(feature = "pqc")]
    pub pqc_sk: Option<ml_dsa_65::PrivateKey>,

    /// ML-DSA-65 public key for verification
    #[cfg(feature = "pqc")]
    pub pqc_pk: Option<ml_dsa_65::PublicKey>,
    
    /// Reserved for when pqc feature is disabled
    #[cfg(not(feature = "pqc"))]
    _pqc_reserved: Option<()>,
}

impl HybridKeyPair {
    /// Generate a new hybrid keypair
    /// Currently generates classical-only; PQC coming soon
    pub fn generate() -> PqcResult<Self> {
        use rand::RngCore;

        // 1. Generate classical Ed25519 key
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let classical = SigningKey::from_bytes(&secret);

        // 2. Generate PQC ML-DSA-65 key if enabled
        #[cfg(feature = "pqc")]
        {
            let mut rng = rand::rngs::OsRng;
            let (pk, sk) = ml_dsa_65::try_keygen_with_rng(&mut rng)
                .map_err(|e| PqcError::KeyGenerationFailed(format!("{:?}", e)))?;
            
            Ok(Self {
                classical,
                pqc_sk: Some(sk),
                pqc_pk: Some(pk),
            })
        }

        #[cfg(not(feature = "pqc"))]
        Ok(Self {
            classical,
            _pqc_reserved: None,
        })
    }

    /// Create from existing Ed25519 key (classical-only mode)
    pub fn from_classical(signing_key: SigningKey) -> Self {
        Self {
            classical: signing_key,
            #[cfg(feature = "pqc")]
            pqc_sk: None,
            #[cfg(feature = "pqc")]
            pqc_pk: None,
            #[cfg(not(feature = "pqc"))]
            _pqc_reserved: None,
        }
    }

    /// Get the classical verifying (public) key
    pub fn classical_verifying_key(&self) -> VerifyingKey {
        self.classical.verifying_key()
    }

    pub fn pqc_public_key(&self) -> Option<Vec<u8>> {
        #[cfg(feature = "pqc")]
        {
            if let Some(pk) = &self.pqc_pk {
                return Some(pk.clone().into_bytes().to_vec());
            }
        }
        None
    }

    /// Sign a message with hybrid signature
    /// Currently produces classical-only signature
    pub fn sign(&self, message: &[u8]) -> HybridSignature {
        let classical_sig = self.classical.sign(message);
        
        #[cfg(feature = "pqc")]
        {
            if let Some(sk) = &self.pqc_sk {
                if let Ok(pqc_sig) = sk.try_sign(message, &[]) {
                    return HybridSignature::hybrid(&classical_sig, &pqc_sig);
                }
            }
        }
        
        HybridSignature::classical_only(&classical_sig)
    }

    /// Check if this keypair has PQC capability
    pub fn has_pqc(&self) -> bool {
        #[cfg(feature = "pqc")]
        return self.pqc_sk.is_some();
        #[cfg(not(feature = "pqc"))]
        false
    }
}

// ============================================================================
// PUBLIC KEY BUNDLE
// ============================================================================

/// Bundle of public keys for verification (supports both classical and PQC)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKeyBundle {
    /// Classical Ed25519 public key (32 bytes)
    pub classical: Vec<u8>,

    /// Post-quantum ML-DSA public key (~1,952 bytes for ML-DSA-65)
    /// Reserved for future PQC implementation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pqc: Option<Vec<u8>>,

    /// Key bundle version
    pub version: u8,
}

impl PublicKeyBundle {
    /// Current key bundle version
    pub const VERSION: u8 = 1;

    /// Create classical-only bundle
    pub fn classical_only(public_key: &VerifyingKey) -> Self {
        Self {
            classical: public_key.to_bytes().to_vec(),
            pqc: None,
            version: Self::VERSION,
        }
    }

    /// Create hybrid bundle (for future use)
    pub fn hybrid(classical: &VerifyingKey, pqc: Vec<u8>) -> Self {
        Self {
            classical: classical.to_bytes().to_vec(),
            pqc: Some(pqc),
            version: Self::VERSION,
        }
    }

    /// Get Ed25519 verifying key
    pub fn classical_verifying_key(&self) -> PqcResult<VerifyingKey> {
        let bytes: [u8; 32] = self
            .classical
            .as_slice()
            .try_into()
            .map_err(|_| PqcError::InvalidKeyFormat("Invalid classical key length".into()))?;

        VerifyingKey::from_bytes(&bytes)
            .map_err(|e| PqcError::InvalidKeyFormat(e.to_string()))
    }

    /// Verify a hybrid signature
    pub fn verify(&self, message: &[u8], signature: &HybridSignature) -> bool {
        let Ok(classical_pk) = self.classical_verifying_key() else {
            return false;
        };

        signature.verify_hybrid(message, &classical_pk, self.pqc.as_deref())
    }

    /// Total size in bytes
    pub fn size(&self) -> usize {
        self.classical.len() + self.pqc.as_ref().map(|p| p.len()).unwrap_or(0)
    }

    /// Check if this bundle has PQC key
    pub fn has_pqc(&self) -> bool {
        self.pqc.is_some()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_keypair_generation() {
        let kp = HybridKeyPair::generate().expect("Keypair generation failed");
        let _ = kp.classical_verifying_key();
        
        #[cfg(feature = "pqc")]
        assert!(kp.has_pqc());
        #[cfg(not(feature = "pqc"))]
        assert!(!kp.has_pqc());
    }

    #[test]
    fn test_classical_signing_and_verification() {
        let kp = HybridKeyPair::generate().expect("Keypair generation failed");
        let message = b"Hello, post-quantum world!";

        let signature = kp.sign(message);

        // Verify classical
        assert!(signature.verify_classical(message, &kp.classical_verifying_key()));

        // Wrong message should fail
        assert!(!signature.verify_classical(b"Wrong message", &kp.classical_verifying_key()));
    }

    #[test]
    fn test_hybrid_signature_serialization() {
        let kp = HybridKeyPair::generate().expect("Keypair generation failed");
        let message = b"Test message for serialization";

        let signature = kp.sign(message);

        // Serialize and deserialize
        let json = serde_json::to_string(&signature).expect("Serialization failed");
        let restored: HybridSignature =
            serde_json::from_str(&json).expect("Deserialization failed");

        // Should still verify
        assert!(restored.verify_classical(message, &kp.classical_verifying_key()));
    }

    #[test]
    fn test_public_key_bundle() {
        let kp = HybridKeyPair::generate().expect("Keypair generation failed");
        let message = b"Bundle verification test";

        let bundle = PublicKeyBundle::classical_only(&kp.classical_verifying_key());
        let signature = kp.sign(message);

        assert!(bundle.verify(message, &signature));
    }

    #[test]
    fn test_classical_only_mode() {
        use rand::RngCore;
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);

        let kp = HybridKeyPair::from_classical(signing_key);
        let message = b"Classical only test";

        let signature = kp.sign(message);
        assert!(!signature.is_hybrid()); // Should be classical-only
        assert!(signature.verify_classical(message, &kp.classical_verifying_key()));
    }

    #[test]
    fn test_signature_size() {
        let kp = HybridKeyPair::generate().expect("Keypair generation failed");
        let signature = kp.sign(b"Size test");

        // Classical signature should be 64 bytes
        assert_eq!(signature.classical.len(), 64);
        
        #[cfg(feature = "pqc")]
        assert_eq!(signature.size(), 64 + 3309); // Classical + ML-DSA-65
        #[cfg(not(feature = "pqc"))]
        assert_eq!(signature.size(), 64); // Classical only
    }
}
