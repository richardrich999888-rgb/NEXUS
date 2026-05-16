// NEXUS Core: Cryptographic Primitives
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature, SECRET_KEY_LENGTH};
use rand::rngs::OsRng;
use rand::RngCore;
use crate::error::{NexusError, Result};

/// Generate a new Ed25519 signing key
pub fn generate_signing_key() -> SigningKey {
    let mut secret = [0u8; SECRET_KEY_LENGTH];
    OsRng.fill_bytes(&mut secret);
    SigningKey::from_bytes(&secret)
}

/// Sign data with Ed25519
pub fn sign(signing_key: &SigningKey, data: &[u8]) -> Vec<u8> {
    signing_key.sign(data).to_bytes().to_vec()
}

/// Verify Ed25519 signature
pub fn verify(verifying_key: &VerifyingKey, data: &[u8], signature: &[u8]) -> Result<()> {
    let sig = Signature::from_bytes(signature.try_into()
        .map_err(|_| NexusError::InvalidSignature("Invalid signature length".to_string()))?);
    
    verifying_key.verify(data, &sig)
        .map_err(|e| NexusError::InvalidSignature(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sign_verify_roundtrip() {
        let signing_key = generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let data = b"test data";
        
        let signature = sign(&signing_key, data);
        assert!(verify(&verifying_key, data, &signature).is_ok());
    }
    
    #[test]
    fn test_verify_tampered_data() {
        let signing_key = generate_signing_key();
        let verifying_key = signing_key.verifying_key();
        let data = b"original data";
        let tampered = b"tampered data";
        
        let signature = sign(&signing_key, data);
        assert!(verify(&verifying_key, tampered, &signature).is_err());
    }
}
