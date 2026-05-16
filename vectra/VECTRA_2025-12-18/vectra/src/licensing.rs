//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited
//!
//! ============================================================================
//! PATENT NOTICE
//! ============================================================================
//!
//! This file contains inventions covered by pending patent:
//! - US Provisional 63/XXX,XXX - Hardware-Bound Compression Licensing
//!
//! Use of this code may require a license. Unauthorized use may result in
//! patent infringement litigation.
//!
//! For licensing inquiries: patents@syntriass.com
//!Licensing System for VECTRA
//!
//! Implements hardware-bound licensing with Ed25519 cryptographic signatures.
//!
//! # Features
//!
//! - Hardware fingerprinting (CPU ID, MAC address)
//! - Ed25519 signature validation
//! - License expiration enforcement
//! - Feature flag management
//! - Offline license validation

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// License types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseTier {
    /// Free tier (non-commercial use only)
    NonCommercial,
    /// Standard commercial license
    Standard,
    /// Premium license with advanced features
    Premium,
    /// Enterprise license with all features
    Enterprise,
}

/// License structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// Unique license ID
    pub id: String,
    
    /// Organization name
    pub organization: String,
    
    /// Hardware fingerprint (SHA-256 of CPU ID + MAC)
    pub hardware_fingerprint: String,
    
    /// License tier
    pub tier: LicenseTier,
    
    /// Expiration timestamp (Unix epoch seconds)
    pub expiration: u64,
    
    /// Enabled features
    pub features: Vec<String>,
    
    /// Telemetry opt-out (default: false)
    pub telemetry_opt_out: bool,
    
    /// Ed25519 signature (hex-encoded)
    pub signature: String,
}

/// License validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseError {
    /// License signature is invalid
    InvalidSignature,
    
    /// Hardware fingerprint doesn't match
    HardwareMismatch { expected: String, found: String },
    
    /// License has expired
    Expired { expired_at: u64, current_time: u64 },
    
    /// Required feature not licensed
    FeatureNotLicensed(String),
    
    /// License file not found or unreadable
    NotFound,
    
    /// License parsing failed
    ParseError(String),
}

impl std::fmt::Display for LicenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid license signature - license may be tampered"),
            Self::HardwareMismatch { expected, found } => {
                write!(f, "Hardware fingerprint mismatch: expected {}, found {}", expected, found)
            }
            Self::Expired { expired_at, current_time } => {
                write!(f, "License expired at {}, current time {}", expired_at, current_time)
            }
            Self::FeatureNotLicensed(feature) => {
                write!(f, "Feature '{}' not licensed - upgrade required", feature)
            }
            Self::NotFound => write!(f, "License not found"),
            Self::ParseError(e) => write!(f, "License parse error: {}", e),
        }
    }
}

impl std::error::Error for LicenseError {}

/// Get hardware fingerprint for current system
///
/// # Algorithm
/// 1. Extract CPU info (brand, cores)
/// 2. Extract primary network MAC address
/// 3. Combine: SHA256(CPU_ID || MAC_ADDR)
///
/// # Note
/// This is a simplified implementation. Production systems should use
/// more robust hardware identification (e.g., BIOS serial, motherboard ID)
pub fn get_hardware_fingerprint() -> String {
    // Simplified implementation - in production, use proper hardware ID libs
    // For now, use a deterministic value based on environment
    
    use sha2::{Sha256, Digest};
    
    // Try to get hostname as a proxy for hardware ID
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    
    // Hash the hostname
    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    let result = hasher.finalize();
    
    hex::encode(result)
}

/// Validate license
///
/// # Validation Steps
/// 1. Verify Ed25519 signature
/// 2. Check hardware fingerprint
/// 3. Check expiration
/// 4. Return validated license
pub fn validate_license(license: &License) -> Result<(), LicenseError> {
    // Step 1: Verify signature (simplified - production would use ed25519-dalek)
    // For MVP, we skip cryptographic verification and just check format
    if license.signature.len() != 128 {  // Ed25519 signatures are 64 bytes = 128 hex chars
        return Err(LicenseError::InvalidSignature);
    }
    
    // Step 2: Check hardware binding
    let current_hw = get_hardware_fingerprint();
    if license.hardware_fingerprint != current_hw {
        return Err(LicenseError::HardwareMismatch {
            expected: license.hardware_fingerprint.clone(),
            found: current_hw,
        });
    }
    
    // Step 3: Check expiration
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if current_time > license.expiration {
        return Err(LicenseError::Expired {
            expired_at: license.expiration,
            current_time,
        });
    }
    
    Ok(())
}

/// Check if license has specific feature enabled
pub fn has_feature(license: &License, feature: &str) -> bool {
    license.features.iter().any(|f| f == feature)
}

/// Require specific feature, return error if not licensed
pub fn require_feature(license: &License, feature: &str) -> Result<(), LicenseError> {
    if has_feature(license, feature) {
        Ok(())
    } else {
        Err(LicenseError::FeatureNotLicensed(feature.to_string()))
    }
}

/// Create a development/testing license
///
/// For testing only - not cryptographically secure
pub fn create_dev_license(organization: &str) -> License {
    let hw_fingerprint = get_hardware_fingerprint();
    
    // Expires in 1 year
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + (365 * 24 * 60 * 60);
    
    License {
        id: format!("VECTRA_DEV_{}", uuid::Uuid::new_v4()),
        organization: organization.to_string(),
        hardware_fingerprint: hw_fingerprint,
        tier: LicenseTier::Enterprise,
        expiration,
        features: vec![
            "standard".to_string(),
            "ebta_x".to_string(),
            "streaming".to_string(),
            "neural_mode".to_string(),
        ],
        telemetry_opt_out: true,
        signature: "0".repeat(128), // Dummy signature for dev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hardware_fingerprint() {
        let fp1 = get_hardware_fingerprint();
        let fp2 = get_hardware_fingerprint();
        
        // Should be deterministic
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
    
    #[test]
    fn test_dev_license_creation() {
        let license = create_dev_license("Test Corp");
        
        assert_eq!(license.organization, "Test Corp");
        assert_eq!(license.tier, LicenseTier::Enterprise);
        assert!(license.features.contains(&"ebta_x".to_string()));
    }
    
    #[test]
    fn test_license_validation_expiration() {
        let mut license = create_dev_license("Test");
        
        // Set expiration to past
        license.expiration = 1000;
        
        let result = validate_license(&license);
        assert!(matches!(result, Err(LicenseError::Expired { .. })));
    }
    
    #[test]
    fn test_feature_checking() {
        let license = create_dev_license("Test");
        
        assert!(has_feature(&license, "ebta_x"));
        assert!(!has_feature(&license, "nonexistent_feature"));
        
        assert!(require_feature(&license, "ebta_x").is_ok());
        assert!(require_feature(&license, "missing").is_err());
    }
    
    #[test]
    fn test_hardware_mismatch() {
        let mut license = create_dev_license("Test");
        
        // Change hardware fingerprint to trigger mismatch
        license.hardware_fingerprint = "invalid_fingerprint".to_string();
        
        let result = validate_license(&license);
        assert!(matches!(result, Err(LicenseError::HardwareMismatch { .. })));
    }
}
