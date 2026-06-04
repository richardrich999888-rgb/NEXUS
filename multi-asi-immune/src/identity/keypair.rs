//! Ed25519 keypair for ASI identity.
//!
//! Each ASI instance has a unique, cryptographically verifiable identity based on Ed25519.
//! Identity is self-sovereign - no central registry required.

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 32-byte identifier derived from public key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AsiId(pub [u8; 32]);

impl AsiId {
    /// Creates an AsiId from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    /// Returns the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    /// Returns a hex string representation.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
    
    /// Returns a short hex string (first 8 chars).
    pub fn short_hex(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl fmt::Debug for AsiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AsiId({})", self.short_hex())
    }
}

impl fmt::Display for AsiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_hex())
    }
}

/// Unique, cryptographically verifiable identity for an ASI instance.
/// Based on Ed25519 for speed and security.
#[derive(Clone)]
pub struct AsiIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    /// Stable identifier derived from public key.
    pub id: AsiId,
}

impl AsiIdentity {
    /// Generates a new random identity.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let id = Self::derive_id(&verifying_key);
        
        Self {
            signing_key,
            verifying_key,
            id,
        }
    }
    
    /// Creates identity from existing signing key bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        let id = Self::derive_id(&verifying_key);
        
        Self {
            signing_key,
            verifying_key,
            id,
        }
    }
    
    /// Derives a stable ID from the public key.
    fn derive_id(vk: &VerifyingKey) -> AsiId {
        let mut hasher = Sha256::new();
        hasher.update(b"ASI_ID_V1:");
        hasher.update(vk.as_bytes());
        let hash = hasher.finalize();
        
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        AsiId(id)
    }
    
    /// Signs arbitrary data.
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }
    
    /// Returns the public verifying key for sharing with others.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }
    
    /// Returns the public identity for sharing.
    pub fn public_identity(&self) -> PublicIdentity {
        PublicIdentity {
            id: self.id,
            verifying_key: self.verifying_key,
        }
    }
    
    /// Exports the signing key bytes (SENSITIVE - protect carefully).
    pub fn export_secret(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

impl fmt::Debug for AsiIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsiIdentity")
            .field("id", &self.id)
            .finish()
    }
}

/// Public identity information shareable with other ASIs.
#[derive(Debug, Clone)]
pub struct PublicIdentity {
    pub id: AsiId,
    pub verifying_key: VerifyingKey,
}

impl PublicIdentity {
    /// Creates from raw verifying key bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, ed25519_dalek::SignatureError> {
        let verifying_key = VerifyingKey::from_bytes(bytes)?;
        let id = AsiIdentity::derive_id(&verifying_key);
        Ok(Self { id, verifying_key })
    }
    
    /// Verifies a signature from this identity.
    pub fn verify(&self, data: &[u8], signature: &Signature) -> bool {
        self.verifying_key.verify(data, signature).is_ok()
    }
    
    /// Returns the raw public key bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

/// Wrapper for signature with serialization support.
#[derive(Debug, Clone)]
pub struct SerializableSignature(pub Signature);

impl Serialize for SerializableSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.to_bytes())
    }
}

impl<'de> Deserialize<'de> for SerializableSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(SerializableSignature(Signature::from_bytes(&arr)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_generation() {
        let id1 = AsiIdentity::generate();
        let id2 = AsiIdentity::generate();
        
        // Each identity should be unique
        assert_ne!(id1.id, id2.id);
    }
    
    #[test]
    fn test_sign_verify() {
        let identity = AsiIdentity::generate();
        let message = b"test message";
        
        let signature = identity.sign(message);
        let public = identity.public_identity();
        
        assert!(public.verify(message, &signature));
        
        // Verify fails with wrong message
        assert!(!public.verify(b"wrong message", &signature));
    }
    
    #[test]
    fn test_id_derivation_deterministic() {
        let secret = [42u8; 32];
        let id1 = AsiIdentity::from_bytes(&secret);
        let id2 = AsiIdentity::from_bytes(&secret);
        
        assert_eq!(id1.id, id2.id);
    }
    
    #[test]
    fn test_cross_identity_verification_fails() {
        let id1 = AsiIdentity::generate();
        let id2 = AsiIdentity::generate();
        
        let message = b"test";
        let sig = id1.sign(message);
        
        // id2's public key should not verify id1's signature
        assert!(!id2.public_identity().verify(message, &sig));
    }
}
