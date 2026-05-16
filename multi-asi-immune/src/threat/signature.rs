//! Signed threat reports.

use crate::identity::keypair::{AsiId, AsiIdentity, PublicIdentity};
use crate::threat::pattern::ThreatPattern;
pub use crate::threat::pattern::ThreatCategory;
use ed25519_dalek::Signature;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// A signed threat report from an ASI.
#[derive(Debug, Clone)]
pub struct SignedThreatReport {
    /// The threat pattern being reported.
    pub pattern: ThreatPattern,
    /// ID of the reporting ASI.
    pub reporter: AsiId,
    /// Confidence in this threat [0, 1].
    pub confidence: f64,
    /// Monotonic timestamp.
    pub timestamp: u64,
    /// Cryptographic signature over (pattern, reporter, confidence, timestamp).
    pub signature: Signature,
}

impl SignedThreatReport {
    /// Creates and signs a new threat report.
    pub fn new(
        identity: &AsiIdentity,
        pattern: ThreatPattern,
        confidence: f64,
        timestamp: u64,
    ) -> Self {
        let confidence = confidence.clamp(0.0, 1.0);
        let signature = Self::sign_report(identity, &pattern, confidence, timestamp);
        
        Self {
            pattern,
            reporter: identity.id,
            confidence,
            timestamp,
            signature,
        }
    }
    
    fn sign_report(
        identity: &AsiIdentity,
        pattern: &ThreatPattern,
        confidence: f64,
        timestamp: u64,
    ) -> Signature {
        let message = Self::build_message(pattern, identity.id, confidence, timestamp);
        identity.sign(&message)
    }
    
    fn build_message(
        pattern: &ThreatPattern,
        reporter: AsiId,
        confidence: f64,
        timestamp: u64,
    ) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(b"THREAT_REPORT_V1:");
        hasher.update(&pattern.pattern_hash);
        hasher.update(&[pattern.category as u8]);
        hasher.update(&pattern.severity.to_le_bytes());
        hasher.update(&reporter.0);
        hasher.update(&confidence.to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.finalize().to_vec()
    }
    
    /// Verifies the signature on this report.
    pub fn verify(&self, public_identity: &PublicIdentity) -> bool {
        if public_identity.id != self.reporter {
            return false;
        }
        
        let message = Self::build_message(
            &self.pattern,
            self.reporter,
            self.confidence,
            self.timestamp,
        );
        
        public_identity.verify(&message, &self.signature)
    }
    
    /// Computes unique identifier for deduplication.
    pub fn report_id(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.reporter.0);
        hasher.update(&self.pattern.pattern_hash);
        hasher.update(&self.timestamp.to_le_bytes());
        
        let hash = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        id
    }
    
    /// Returns the effective severity (pattern severity * confidence).
    pub fn effective_severity(&self) -> f64 {
        self.pattern.effective_severity() * self.confidence
    }
}

/// Serializable version of SignedThreatReport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableThreatReport {
    pub pattern: ThreatPattern,
    pub reporter: [u8; 32],
    pub confidence: f64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

impl From<&SignedThreatReport> for SerializableThreatReport {
    fn from(report: &SignedThreatReport) -> Self {
        Self {
            pattern: report.pattern.clone(),
            reporter: report.reporter.0,
            confidence: report.confidence,
            timestamp: report.timestamp,
            signature: report.signature.to_bytes().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sign_verify() {
        let identity = AsiIdentity::generate();
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [42; 32], 0.9);
        
        let report = SignedThreatReport::new(&identity, pattern, 0.85, 100);
        
        assert!(report.verify(&identity.public_identity()));
    }
    
    #[test]
    fn test_wrong_identity_fails() {
        let identity1 = AsiIdentity::generate();
        let identity2 = AsiIdentity::generate();
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [42; 32], 0.9);
        
        let report = SignedThreatReport::new(&identity1, pattern, 0.85, 100);
        
        // Verification with wrong identity should fail
        assert!(!report.verify(&identity2.public_identity()));
    }
    
    #[test]
    fn test_report_id_unique() {
        let identity = AsiIdentity::generate();
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [42; 32], 0.9);
        
        let report1 = SignedThreatReport::new(&identity, pattern.clone(), 0.85, 100);
        let report2 = SignedThreatReport::new(&identity, pattern, 0.85, 101);
        
        // Different timestamps = different IDs
        assert_ne!(report1.report_id(), report2.report_id());
    }
}
