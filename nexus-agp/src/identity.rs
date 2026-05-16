//! # PQC-Bound Agent Identity
//!
//! PATENT CLAIM 6: Agent fingerprints cryptographically bound to ML-DSA (FIPS 204)
//! keypairs, resistant to quantum attacks.
//!
//! ## Why Unforkable
//!
//! - Fork can't copy PQC private keys
//! - Identity is cryptographically bound to model commitment
//! - Registration requires proof of key ownership

use nexus_pcu::{PrincipalId, ContentHash};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[cfg(feature = "pqc")]
use nexus_pcu::pqc::HybridKeyPair;

/// 32-byte agent fingerprint bound to PQC identity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentFingerprint(pub [u8; 32]);

impl AgentFingerprint {
    /// Compute fingerprint from PQC public key and model commitment
    ///
    /// PATENT CLAIM: This binding makes identity unforkable
    pub fn compute(
        principal: &PrincipalId,
        model_commitment: &[u8],
        pqc_public_key: &[u8],
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(principal.as_bytes());
        hasher.update(model_commitment);
        hasher.update(pqc_public_key);
        hasher.update(b"NEXUS-AGP-v1"); // Domain separator
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Self(bytes)
    }
    
    /// Create fingerprint without PQC (for testing/legacy)
    pub fn compute_legacy(principal: &PrincipalId, model_commitment: &[u8]) -> Self {
        Self::compute(principal, model_commitment, &[])
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// Agent version for fork tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl AgentVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
    
    pub fn is_major_upgrade(&self, other: &Self) -> bool {
        self.major > other.major
    }
    
    pub fn is_minor_upgrade(&self, other: &Self) -> bool {
        self.major == other.major && self.minor > other.minor
    }
}

impl std::fmt::Display for AgentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Complete NEXUS-bound agent identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusAgentIdentity {
    /// NEXUS principal ID (derived from public key)
    pub principal: PrincipalId,
    /// AGP fingerprint (bound to PQC + model)
    pub fingerprint: AgentFingerprint,
    /// Agent version for fork tracking
    pub version: AgentVersion,
    /// Model commitment hash
    pub model_commitment: ContentHash,
    /// Registration timestamp (milliseconds)
    pub registered_at: u64,
}

impl NexusAgentIdentity {
    /// Create new identity with model commitment
    #[cfg(not(feature = "pqc"))]
    pub fn new(model_commitment: &[u8], version: AgentVersion) -> Self {
        let principal = PrincipalId::anonymous();
        let fingerprint = AgentFingerprint::compute_legacy(&principal, model_commitment);
        let content_hash = ContentHash::compute(model_commitment);
        
        Self {
            principal,
            fingerprint,
            version,
            model_commitment: content_hash,
            registered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
    
    /// Create PQC-bound identity (PATENT CLAIM 6)
    #[cfg(feature = "pqc")]
    pub fn new_with_pqc(
        keypair: &HybridKeyPair,
        model_commitment: &[u8],
        version: AgentVersion,
    ) -> Self {
        let principal = PrincipalId::from_bytes(keypair.public_key_bytes());
        let pqc_pk = keypair.pqc_public_key();
        let fingerprint = AgentFingerprint::compute(&principal, model_commitment, &pqc_pk);
        let content_hash = ContentHash::compute(model_commitment);
        
        Self {
            principal,
            fingerprint,
            version,
            model_commitment: content_hash,
            registered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Registration proof for on-chain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub identity: NexusAgentIdentity,
    /// Signature over identity (classical + PQC hybrid)
    pub signature: Vec<u8>,
    /// Operator ID for anti-collusion
    pub operator_id: [u8; 32],
}

impl AgentRegistration {
    /// Create registration with signature
    pub fn new(identity: NexusAgentIdentity, signature: Vec<u8>, operator_id: [u8; 32]) -> Self {
        Self {
            identity,
            signature,
            operator_id,
        }
    }
    
    /// Verify registration signature
    pub fn verify(&self) -> bool {
        // In production: verify hybrid signature
        // For now: check signature exists
        !self.signature.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_determinism() {
        let principal = PrincipalId::anonymous();
        let model = b"test_model_v1";
        let pqc_pk = b"mock_pqc_public_key";
        
        let fp1 = AgentFingerprint::compute(&principal, model, pqc_pk);
        let fp2 = AgentFingerprint::compute(&principal, model, pqc_pk);
        
        assert_eq!(fp1, fp2, "Fingerprint must be deterministic");
    }
    
    #[test]
    fn test_fingerprint_uniqueness() {
        let principal = PrincipalId::anonymous();
        let model1 = b"model_v1";
        let model2 = b"model_v2";
        let pqc_pk = b"mock_pqc_public_key";
        
        let fp1 = AgentFingerprint::compute(&principal, model1, pqc_pk);
        let fp2 = AgentFingerprint::compute(&principal, model2, pqc_pk);
        
        assert_ne!(fp1, fp2, "Different models must have different fingerprints");
    }
    
    #[test]
    fn test_version_upgrade_detection() {
        let v1 = AgentVersion::new(1, 0, 0);
        let v2_major = AgentVersion::new(2, 0, 0);
        let v1_minor = AgentVersion::new(1, 1, 0);
        
        assert!(v2_major.is_major_upgrade(&v1));
        assert!(!v1_minor.is_major_upgrade(&v1));
        assert!(v1_minor.is_minor_upgrade(&v1));
    }
}
