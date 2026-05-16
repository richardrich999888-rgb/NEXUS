//! Encryption utilities for secret storage

use crate::error::{SecretError, SecretResult};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use zeroize::Zeroize;
use rand::RngCore;

/// Encrypt data with AES-256-GCM
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> SecretResult<Vec<u8>> {
    if key.len() != 32 {
        return Err(SecretError::Encryption("Key must be 32 bytes".to_string()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SecretError::Encryption(format!("Failed to create cipher: {}", e)))?;

    // Generate random nonce (12 bytes for GCM)
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| SecretError::Encryption(format!("Encryption failed: {}", e)))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(key: &[u8; 32], ciphertext: &[u8]) -> SecretResult<Vec<u8>> {
    if key.len() != 32 {
        return Err(SecretError::Decryption("Key must be 32 bytes".to_string()));
    }

    if ciphertext.len() < 12 {
        return Err(SecretError::Decryption("Ciphertext too short".to_string()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SecretError::Decryption(format!("Failed to create cipher: {}", e)))?;

    // Extract nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let encrypted_data = &ciphertext[12..];

    let plaintext = cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| SecretError::Decryption(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

/// Generate a random encryption key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    key
}

/// Derive key from password using PBKDF2
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    use sha2::Sha256;
    use pbkdf2::pbkdf2_hmac;
    
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100000, &mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = b"secret data";
        
        let ciphertext = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let plaintext = b"secret data";
        
        let ciphertext = encrypt(&key1, plaintext).unwrap();
        assert!(decrypt(&key2, &ciphertext).is_err());
    }
}

