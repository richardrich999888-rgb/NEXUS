//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited
//!
//! ============================================================================
//! PATENT NOTICE
//! ============================================================================
//!
//! This file contains inventions covered by pending patent:
//! - US Provisional 63/XXX,XXX - Cryptographic Artifact Fingerprinting
//!
//! Use of this code may require a license. Unauthorized use may result in
//! patent infringement litigation.
//!
//! For licensing inquiries: patents@syntriass.com
//! ============================================================================

//! Cryptographic Fingerprinting for VECTRA Artifacts
//!
//! Provides:
//! - Unique artifact fingerprints for tracking and verification
//! - Chain-of-custody tracking through artifact lineage
//! - Tamper detection with multi-layer hashing
//! - Optional hardware binding for deployment verification

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use crate::types::{Artifact, Payload};

/// Cryptographic fingerprint for an artifact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFingerprint {
    /// Primary fingerprint (SHA-256 of artifact content)
    pub primary: [u8; 32],
    
    /// Payload fingerprint (SHA-256 of original payload)
    pub payload: [u8; 32],
    
    /// Structure fingerprint (SHA-256 of structural components)
    pub structure: [u8; 32],
    
    /// Combined fingerprint (SHA-256 of all above)
    pub combined: [u8; 32],
    
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    
    /// Optional hardware binding fingerprint
    pub hardware_binding: Option<String>,
}

impl ArtifactFingerprint {
    /// Generate fingerprint for an artifact
    pub fn generate(artifact: &Artifact, original_payload: &Payload) -> Self {
        // Primary: hash of serialized artifact
        let artifact_bytes = artifact.to_bytes();
        let primary = sha256(&artifact_bytes);
        
        // Payload: hash of original payload
        let payload_hash = sha256(original_payload.as_bytes());
        
        // Structure: hash of generator + mappings
        let structure_bytes = format!(
            "{:?}{:?}",
            artifact.generator,
            artifact.mappings
        );
        let structure = sha256(structure_bytes.as_bytes());
        
        // Combined: hash of all three
        let mut combined_input = Vec::new();
        combined_input.extend_from_slice(&primary);
        combined_input.extend_from_slice(&payload_hash);
        combined_input.extend_from_slice(&structure);
        let combined = sha256(&combined_input);
        
        // Timestamp
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Hardware binding (optional)
        let hardware_binding = get_hardware_binding();
        
        Self {
            primary,
            payload: payload_hash,
            structure,
            combined,
            created_at,
            hardware_binding,
        }
    }
    
    /// Verify fingerprint matches artifact
    pub fn verify(&self, artifact: &Artifact, original_payload: &Payload) -> bool {
        let fresh = Self::generate(artifact, original_payload);
        
        self.primary == fresh.primary
            && self.payload == fresh.payload
            && self.structure == fresh.structure
    }
    
    /// Get fingerprint as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.combined)
    }
    
    /// Create fingerprint from hex string (combined only)
    pub fn from_hex(hex_str: &str) -> Option<[u8; 32]> {
        let bytes = hex::decode(hex_str).ok()?;
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Some(arr)
        } else {
            None
        }
    }
}

/// Chain of custody entry for artifact tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEntry {
    /// Fingerprint at this point in chain
    pub fingerprint: [u8; 32],
    
    /// Timestamp of custody transfer
    pub timestamp: u64,
    
    /// Organization holding custody
    pub organization: String,
    
    /// Action performed (created, transferred, verified)
    pub action: CustodyAction,
    
    /// Previous entry fingerprint (for chain integrity)
    pub previous: Option<[u8; 32]>,
}

/// Actions in chain of custody
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyAction {
    /// Artifact was created
    Created,
    /// Artifact was transferred to new holder
    Transferred,
    /// Artifact was verified
    Verified,
    /// Artifact was modified (re-encoded)
    Modified,
}

/// Chain of custody for an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustody {
    /// Artifact fingerprint
    pub artifact_fingerprint: [u8; 32],
    
    /// Chain of custody entries
    pub entries: Vec<CustodyEntry>,
}

impl ChainOfCustody {
    /// Create new chain of custody for artifact
    pub fn new(fingerprint: &ArtifactFingerprint, organization: &str) -> Self {
        let entry = CustodyEntry {
            fingerprint: fingerprint.combined,
            timestamp: fingerprint.created_at,
            organization: organization.to_string(),
            action: CustodyAction::Created,
            previous: None,
        };
        
        Self {
            artifact_fingerprint: fingerprint.combined,
            entries: vec![entry],
        }
    }
    
    /// Add custody transfer
    pub fn transfer(&mut self, to_organization: &str) {
        let previous = self.entries.last().map(|e| e.fingerprint);
        
        self.entries.push(CustodyEntry {
            fingerprint: self.artifact_fingerprint,
            timestamp: current_timestamp(),
            organization: to_organization.to_string(),
            action: CustodyAction::Transferred,
            previous,
        });
    }
    
    /// Record verification
    pub fn verify(&mut self, organization: &str) {
        let previous = self.entries.last().map(|e| e.fingerprint);
        
        self.entries.push(CustodyEntry {
            fingerprint: self.artifact_fingerprint,
            timestamp: current_timestamp(),
            organization: organization.to_string(),
            action: CustodyAction::Verified,
            previous,
        });
    }
    
    /// Verify chain integrity
    pub fn verify_chain(&self) -> bool {
        for i in 1..self.entries.len() {
            let current = &self.entries[i];
            let previous = &self.entries[i - 1];
            
            if current.previous != Some(previous.fingerprint) {
                return false;
            }
        }
        true
    }
}

/// Compute SHA-256 hash
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    arr
}

/// Get optional hardware binding fingerprint
fn get_hardware_binding() -> Option<String> {
    // Use the licensing module's hardware fingerprint if available
    Some(crate::licensing::get_hardware_fingerprint())
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    
    fn create_test_artifact() -> Artifact {
        Artifact {
            generator: Generator {
                base: vec![0x00, 0x01],
                repetition: RepetitionSpec { count: 1, stride: 2, start_offset: 0, byte_ranges: vec![] },
            },
            mappings: MappingSet { mappings: vec![] },
            predictor_state: PredictorState {
                version: VERSION_ID,
                parameters: PredictorParameters::default(),
            },
            residual: Residual { segments: vec![] },
            constraints: ReconstructionConstraints {
                output_length: 10,
                output_hash: [0u8; 32],
            },
            integrity: IntegrityMeta {
                payload_hash: [0u8; 32],
                artifact_hash: [0u8; 32],
                version: VERSION_ID,
                encoded_at: 0,
            },
        }
    }
    
    #[test]
    fn test_fingerprint_generation() {
        let artifact = create_test_artifact();
        let payload = Payload::new(vec![1, 2, 3, 4, 5]);
        
        let fingerprint = ArtifactFingerprint::generate(&artifact, &payload);
        
        assert_ne!(fingerprint.primary, [0u8; 32]);
        assert_ne!(fingerprint.payload, [0u8; 32]);
        assert_ne!(fingerprint.combined, [0u8; 32]);
        
        println!("Fingerprint: {}", fingerprint.to_hex());
    }
    
    #[test]
    fn test_fingerprint_verification() {
        let artifact = create_test_artifact();
        let payload = Payload::new(vec![1, 2, 3, 4, 5]);
        
        let fingerprint = ArtifactFingerprint::generate(&artifact, &payload);
        
        // Should verify correctly
        assert!(fingerprint.verify(&artifact, &payload));
        
        // Different payload should fail
        let different_payload = Payload::new(vec![9, 8, 7]);
        assert!(!fingerprint.verify(&artifact, &different_payload));
    }
    
    #[test]
    fn test_chain_of_custody() {
        let artifact = create_test_artifact();
        let payload = Payload::new(vec![1, 2, 3, 4, 5]);
        let fingerprint = ArtifactFingerprint::generate(&artifact, &payload);
        
        let mut chain = ChainOfCustody::new(&fingerprint, "SYNTRIASS Labs");
        
        chain.transfer("Partner Corp");
        chain.verify("Partner Corp");
        chain.transfer("End User Inc");
        
        assert_eq!(chain.entries.len(), 4);
        assert!(chain.verify_chain());
        
        for entry in &chain.entries {
            println!("{:?}: {} at {}", entry.action, entry.organization, entry.timestamp);
        }
    }
    
    #[test]
    fn test_deterministic_fingerprint() {
        let artifact = create_test_artifact();
        let payload = Payload::new(vec![1, 2, 3, 4, 5]);
        
        let fp1 = ArtifactFingerprint::generate(&artifact, &payload);
        let fp2 = ArtifactFingerprint::generate(&artifact, &payload);
        
        // Primary fingerprints should be identical (deterministic)
        assert_eq!(fp1.primary, fp2.primary);
        assert_eq!(fp1.payload, fp2.payload);
        assert_eq!(fp1.structure, fp2.structure);
    }
}
