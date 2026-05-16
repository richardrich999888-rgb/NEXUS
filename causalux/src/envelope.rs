// Sovereign Data Envelope - End-to-end encrypted operations
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Inventor: Katta Naga Sri Ganesh
//
// Patent Claim: "A cryptographic envelope for causal operations enabling 
// end-to-end encrypted synchronization where intermediary sync nodes 
// cannot access plaintext content, combined with key derivation for 
// selective document sharing."

use crate::causal_op::CausalOp;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use zeroize::Zeroize;

// ============================================================================
// KEY MANAGEMENT
// ============================================================================

/// Represents a derived key for a specific access scope
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivedKey {
    /// Key identifier
    pub key_id: String,
    
    /// The actual key bytes (256-bit)
    pub key_bytes: [u8; 32],
    
    /// Scope this key grants access to
    pub scope: KeyScope,
    
    /// Expiration timestamp (0 = never)
    pub expires_at: u64,
}

/// Defines what a key can access
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyScope {
    /// Full document access
    FullDocument,
    
    /// Read-only access
    ReadOnly,
    
    /// Access to specific sections only
    Section(String),
    
    /// Time-limited access
    Temporary { until: u64 },
}

/// Key derivation manager for hierarchical key generation
#[derive(Clone, Debug)]
pub struct KeyDerivation {
    /// Master key (kept secret on owner's device)
    master_key: [u8; 32],
    
    /// Derived keys by ID
    derived_keys: HashMap<String, DerivedKey>,
    
    /// Document ID this key tree is for
    document_id: String,
}

impl KeyDerivation {
    /// Create new key derivation from master secret
    pub fn new(master_key: [u8; 32], document_id: String) -> Self {
        Self {
            master_key,
            derived_keys: HashMap::new(),
            document_id,
        }
    }

    /// Generate master key from passphrase
    pub fn from_passphrase(passphrase: &str, document_id: String) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        hasher.update(document_id.as_bytes());
        hasher.update(b"causalux-master-key-v1");
        
        let hash = hasher.finalize();
        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&hash);
        
        Self::new(master_key, document_id)
    }

    /// Derive a new key for specific scope
    pub fn derive_key(&mut self, scope: KeyScope, expires_at: u64) -> DerivedKey {
        let key_id = self.generate_key_id(&scope);
        
        let mut hasher = Sha256::new();
        hasher.update(&self.master_key);
        hasher.update(key_id.as_bytes());
        hasher.update(&serde_json::to_vec(&scope).unwrap());
        
        let hash = hasher.finalize();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&hash);
        
        let key = DerivedKey {
            key_id: key_id.clone(),
            key_bytes,
            scope,
            expires_at,
        };
        
        self.derived_keys.insert(key_id, key.clone());
        key
    }

    /// Get a derived key by ID
    pub fn get_key(&self, key_id: &str) -> Option<&DerivedKey> {
        self.derived_keys.get(key_id)
    }

    /// Check if a key is valid (not expired)
    pub fn is_key_valid(&self, key_id: &str) -> bool {
        if let Some(key) = self.derived_keys.get(key_id) {
            if key.expires_at == 0 {
                return true; // Never expires
            }
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            key.expires_at > now
        } else {
            false
        }
    }

    /// Revoke a derived key
    pub fn revoke_key(&mut self, key_id: &str) -> bool {
        self.derived_keys.remove(key_id).is_some()
    }

    fn generate_key_id(&self, scope: &KeyScope) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.document_id.as_bytes());
        hasher.update(&serde_json::to_vec(scope).unwrap());
        hasher.update(&rand_bytes());
        format!("key_{:x}", hasher.finalize())[..24].to_string()
    }
}

fn rand_bytes() -> [u8; 16] {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
}

// ============================================================================
// ENCRYPTED ENVELOPE
// ============================================================================

/// Encrypted operation envelope
/// 
/// Contains a CausalOp encrypted so that only holders of the
/// appropriate derived key can decrypt and read the content.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SovereignEnvelope {
    /// Envelope version
    pub version: u8,
    
    /// Document ID (plaintext for routing)
    pub document_id: String,
    
    /// Key ID used for encryption
    pub key_id: String,
    
    /// Encrypted operation (AES-256-GCM)
    pub ciphertext: Vec<u8>,
    
    /// Nonce for decryption
    pub nonce: [u8; 12],
    
    /// Authentication tag
    pub tag: [u8; 16],
    
    /// Plaintext metadata (for routing/filtering without decryption)
    pub metadata: EnvelopeMetadata,
}

/// Plaintext metadata visible without decryption
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeMetadata {
    /// Operation ID (hash, not content)
    pub operation_id: String,
    
    /// Creator identity (public key hash)
    pub creator_id: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Dependencies (operation IDs)
    pub dependencies: Vec<String>,
    
    /// Version vector summary (node_id -> count)
    pub version_summary: HashMap<String, u64>,
}

impl SovereignEnvelope {
    /// Encrypt an operation into an envelope using AES-256-GCM
    /// 
    /// PRODUCTION-GRADE: Uses authenticated encryption with AES-GCM
    pub fn seal(op: &CausalOp, key: &DerivedKey, document_id: String) -> Result<Self, EnvelopeError> {
        let plaintext = serde_json::to_vec(op)
            .map_err(|_| EnvelopeError::InvalidContent)?;
        
        // Initialize AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(&key.key_bytes)
            .map_err(|_| EnvelopeError::InvalidKey)?;
        
        // Generate cryptographically secure nonce (96 bits for GCM)
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt with authenticated encryption (includes authentication tag)
        let ciphertext_with_tag = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|_| EnvelopeError::EncryptionFailure)?;
        
        // GCM appends 16-byte tag to ciphertext
        let (ciphertext, tag_bytes) = ciphertext_with_tag.split_at(ciphertext_with_tag.len() - 16);
        let mut tag = [0u8; 16];
        tag.copy_from_slice(tag_bytes);
        
        Ok(Self {
            version: 1,
            document_id,
            key_id: key.key_id.clone(),
            ciphertext: ciphertext.to_vec(),
            nonce: nonce_bytes,
            tag,
            metadata: EnvelopeMetadata {
                operation_id: op.id.clone(),
                creator_id: op.identity.clone(),
                timestamp: op.wall_clock,
                dependencies: op.dependencies.iter().cloned().collect(),
                version_summary: op.version_vector.versions.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            },
        })
    }

    /// Decrypt an envelope to get the operation using AES-256-GCM
    pub fn unseal(&self, key: &DerivedKey) -> Result<CausalOp, EnvelopeError> {
        // Verify key ID matches
        if self.key_id != key.key_id {
            return Err(EnvelopeError::WrongKey);
        }
        
        // Initialize AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(&key.key_bytes)
            .map_err(|_| EnvelopeError::InvalidKey)?;
        
        let nonce = Nonce::from_slice(&self.nonce);
        
        // Reconstruct ciphertext with tag (GCM expects them together)
        let mut ciphertext_with_tag = self.ciphertext.clone();
        ciphertext_with_tag.extend_from_slice(&self.tag);
        
        // Decrypt with authentication verification
        let plaintext = cipher.decrypt(nonce, ciphertext_with_tag.as_ref())
            .map_err(|_| EnvelopeError::IntegrityFailure)?;  // Auth tag verification happens here
        
        let op: CausalOp = serde_json::from_slice(&plaintext)
            .map_err(|_| EnvelopeError::InvalidContent)?;
        
        Ok(op)
    }

    fn generate_nonce() -> [u8; 12] {
        use rand::RngCore;
        let mut nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        nonce
    }
}

/// Envelope errors
#[derive(Debug, Clone)]
pub enum EnvelopeError {
    WrongKey,
    InvalidKey,
    IntegrityFailure,
    InvalidContent,
    KeyExpired,
    EncryptionFailure,
    DecryptionFailure,
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvelopeError::WrongKey => write!(f, "Wrong decryption key"),
            EnvelopeError::InvalidKey => write!(f, "Invalid key format"),
            EnvelopeError::IntegrityFailure => write!(f, "Envelope integrity check failed (authentication tag mismatch)"),
            EnvelopeError::InvalidContent => write!(f, "Invalid content after decryption"),
            EnvelopeError::KeyExpired => write!(f, "Decryption key has expired"),
            EnvelopeError::EncryptionFailure => write!(f, "Encryption operation failed"),
            EnvelopeError::DecryptionFailure => write!(f, "Decryption operation failed"),
        }
    }
}

impl std::error::Error for EnvelopeError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version_vector::VersionVector;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use std::collections::BTreeSet;

    fn create_test_op() -> CausalOp {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        let keypair = SigningKey::from_bytes(&bytes);
        let mut vv = VersionVector::new();
        vv.increment("node1");
        
        CausalOp::new(
            "test".to_string(),
            serde_json::json!({"data": "secret"}),
            BTreeSet::new(),
            vv,
            "node1".to_string(),
            &keypair,
        )
    }

    #[test]
    fn test_key_derivation() {
        let mut kd = KeyDerivation::from_passphrase("my-secret", "doc123".to_string());
        
        let key1 = kd.derive_key(KeyScope::FullDocument, 0);
        let key2 = kd.derive_key(KeyScope::ReadOnly, 0);
        
        // Keys should be different
        assert_ne!(key1.key_bytes, key2.key_bytes);
        
        // Keys should be retrievable
        assert!(kd.get_key(&key1.key_id).is_some());
    }

    #[test]
    fn test_seal_unseal() {
        let mut kd = KeyDerivation::from_passphrase("my-secret", "doc123".to_string());
        let key = kd.derive_key(KeyScope::FullDocument, 0);
        
        let op = create_test_op();
        
        // Seal
        let envelope = SovereignEnvelope::seal(&op, &key, "doc123".to_string()).unwrap();
        
        // Metadata should be visible
        assert_eq!(envelope.metadata.operation_id, op.id);
        
        // Unseal
        let decrypted = envelope.unseal(&key).unwrap();
        assert_eq!(decrypted.id, op.id);
    }

    #[test]
    fn test_wrong_key() {
        let mut kd = KeyDerivation::from_passphrase("my-secret", "doc123".to_string());
        let key1 = kd.derive_key(KeyScope::FullDocument, 0);
        let key2 = kd.derive_key(KeyScope::ReadOnly, 0);
        
        let op = create_test_op();
        let envelope = SovereignEnvelope::seal(&op, &key1, "doc123".to_string()).unwrap();
        
        // Should fail with wrong key
        let result = envelope.unseal(&key2);
        assert!(matches!(result, Err(EnvelopeError::WrongKey)));
    }

    #[test]
    fn test_key_revocation() {
        let mut kd = KeyDerivation::from_passphrase("my-secret", "doc123".to_string());
        let key = kd.derive_key(KeyScope::FullDocument, 0);
        
        assert!(kd.get_key(&key.key_id).is_some());
        
        kd.revoke_key(&key.key_id);
        
        assert!(kd.get_key(&key.key_id).is_none());
    }
}
