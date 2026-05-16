//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited
//!
//! Build Watermarking Module - Tamper Detection and Build Verification

/// Build watermark embedded at compile time
pub const BUILD_WATERMARK: &str = env!("VECTRA_WATERMARK");

/// Build ID (unique per build)
pub const BUILD_ID: &str = env!("VECTRA_BUILD_ID");

/// Git commit hash
pub const GIT_COMMIT: &str = env!("VECTRA_GIT_COMMIT");

/// Build timestamp (Unix epoch)
pub const BUILD_TIMESTAMP: &str = env!("VECTRA_BUILD_TIMESTAMP");

/// Rust compiler version used for this build
pub const RUSTC_VERSION: &str = env!("VECTRA_RUSTC_VERSION");

/// Organization fingerprint
pub const ORG_FINGERPRINT: &str = env!("VECTRA_ORG_FINGERPRINT");

/// Verify build integrity on library load
///
/// This function is automatically called when the library is loaded using the
/// `ctor` crate. It verifies that the build watermark is valid and has not been
/// tampered with.
///
/// # Panics
///
/// Panics if:
/// - Watermark is missing or invalid
/// - Organization fingerprint doesn't match SYNTRIASS
/// - Build appears to be unauthorized or tampered
#[cfg(not(test))]  // Don't run in tests to avoid initialization issues
#[ctor::ctor]
fn verify_build_integrity() {
    // Verify watermark format
    if !BUILD_WATERMARK.starts_with("SYNTRIASS_VECTRA_") {
        panic!(
            "VECTRA: Build integrity check failed - unauthorized build detected. \
             This binary was not built by SYNTRIASS Labs Private Limited. \
             Use of unauthorized builds is prohibited under the VECTRA license. \
             For inquiries: legal@syntriass.com"
        );
    }
    
    // Verify organization fingerprint
    if ORG_FINGERPRINT != "SYNTRIASS_LABS_PVT_LTD" {
        panic!(
            "VECTRA: Organization fingerprint mismatch - tampered binary detected. \
             This may indicate reverse engineering or unauthorized modification. \
             Contact: security@syntriass.com"
        );
    }
    
    // Log build info (in debug mode)
    #[cfg(debug_assertions)]
    {
        eprintln!("VECTRA Build Info:");
        eprintln!("  Build ID: {}", BUILD_ID);
        eprintln!("  Git Commit: {}", GIT_COMMIT);
        eprintln!("  Build Time: {}", BUILD_TIMESTAMP);
        eprintln!("  Rust Version: {}", RUSTC_VERSION);
        eprintln!("  Watermark: {}", BUILD_WATERMARK);
    }
}

/// Get build information as a struct
#[derive(Debug, Clone)]
pub struct BuildInfo {
    /// Unique build identifier
    pub build_id: String,
    
    /// Git commit hash
    pub git_commit: String,
    
    /// Unix timestamp of build
    pub build_timestamp: String,
    
    /// Rust compiler version
    pub rustc_version: String,
    
    /// Full build watermark
    pub watermark: String,
}

impl BuildInfo {
    /// Get current build information
    pub fn current() -> Self {
        Self {
            build_id: BUILD_ID.to_string(),
            git_commit: GIT_COMMIT.to_string(),
            build_timestamp: BUILD_TIMESTAMP.to_string(),
            rustc_version: RUSTC_VERSION.to_string(),
            watermark: BUILD_WATERMARK.to_string(),
        }
    }
    
    /// Verify this build is authorized
    pub fn is_authorized(&self) -> bool {
        self.watermark.starts_with("SYNTRIASS_VECTRA_")
            && ORG_FINGERPRINT == "SYNTRIASS_LABS_PVT_LTD"
    }
    
    /// Get build age in seconds
    pub fn age_seconds(&self) -> Option<u64> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let build_time: u64 = self.build_timestamp.parse().ok()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();
        
        Some(now.saturating_sub(build_time))
    }
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "VECTRA Build Information")?;
        writeln!(f, "  Build ID:      {}", self.build_id)?;
        writeln!(f, "  Git Commit:    {}", self.git_commit)?;
        writeln!(f, "  Build Time:    {}", self.build_timestamp)?;
        writeln!(f, "  Rust Version:  {}", self.rustc_version)?;
        writeln!(f, "  Watermark:     {}", self.watermark)?;
        writeln!(f, "  Authorized:    {}", self.is_authorized())?;
        
        if let Some(age) = self.age_seconds() {
            writeln!(f, "  Build Age:     {} seconds", age)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_info() {
        let info = BuildInfo::current();
        assert!(info.is_authorized());
        assert!(info.watermark.starts_with("SYNTRIASS_VECTRA_"));
    }
    
    #[test]
    fn test_build_constants_exist() {
        assert!(!BUILD_ID.is_empty());
        assert!(!BUILD_WATERMARK.is_empty());
        assert_eq!(ORG_FINGERPRINT, "SYNTRIASS_LABS_PVT_LTD");
    }
}
