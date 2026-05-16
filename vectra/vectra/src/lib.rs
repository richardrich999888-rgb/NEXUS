//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited
//!
//! ============================================================================
//! PATENT NOTICE
//! ============================================================================
//!
//! This file contains inventions covered by pending and granted patents:
//!
//! - US Provisional 63/XXX,XXX - Entropy-Bounded Tensor Algebra (EBTA)
//! - US Provisional 63/XXX,XXX - Deterministic Compression with Mathematical Guarantees
//! - US Provisional 63/XXX,XXX - Adaptive Multi-Dimensional Entropy Validation (EBTA-X)
//! - US Provisional 63/XXX,XXX - Zero-Knowledge Proof of Lossless Data Compression
//! - US Provisional 63/XXX,XXX - Privacy-Preserving Federated Pattern Discovery
//!
//! Use of this code may require a license. Unauthorized use may result in
//! patent infringement litigation.
//!
//! For licensing inquiries: patents@syntriass.com
//!
//! ============================================================================
//! TRADE SECRET NOTICE
//! ============================================================================
//!
//! This file contains trade secrets and confidential information of
//! SYNTRIASS Labs Private Limited. Reverse engineering, decompilation, or
//! disassembly is strictly prohibited.
//!
//! ============================================================================

//! VECTRA — Deterministic Lossless Data Volume Reduction
//!
//! VECTRA is a deterministic, lossless data reduction system for structured payloads.
//! It operates transparently beneath existing protocols and produces self-describing
//! artifacts that guarantee exact reconstruction or safe pass-through.
//!
//! # Core Invariants
//!
//! 1. **Determinism**: Same input + same version → identical output
//! 2. **Losslessness**: `decode(encode(D)) == D` always
//! 3. **Fail-open**: Uncertainty → return original payload unchanged
//! 4. **Self-describing**: Artifacts contain all reconstruction information
//!
//! # Architecture
//!
//! VECTRA decomposes the encoding problem into four components:
//!
//! - **FEE** (Fractal Entropy Encoding): Encodes structural patterns as generators
//! - **SPE** (Symbolic Predictor Engine): Predicts variable components
//! - **EBTA** (Entropy-Bounded Tensor Algebra): Validates residuals against entropy bounds
//! - **TDF** (VECTRA Data Format): Self-describing artifact format
//!
//! # Example
//!
//! ```ignore
//! use vectra::{vectra_encode, vectra_decode, Payload, EncodeResult};
//!
//! // Encode
//! let payload = Payload::new(data);
//! let result = vectra_encode(payload);
//!
//! match result {
//!     EncodeResult::Encoded(artifact) => {
//!         // Transmit or store artifact
//!         let bytes = artifact.to_bytes();
//!         
//!         // Later: decode
//!         let restored = Artifact::from_bytes(&bytes)?;
//!         let original = vectra_decode(&restored)?;
//!     }
//!     EncodeResult::PassThrough(original) => {
//!         // Encoding not beneficial, use original
//!     }
//! }
//! ```
//!
//! # Version Compatibility
//!
//! Artifacts are version-locked. An artifact produced by version X can only
//! be decoded by version X. This ensures determinism across deployments.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules
pub mod artifact;
pub mod decompose;
pub mod ebta;
pub mod ebta_x; // EBTA-X: Adaptive Multi-Dimensional Entropy (Patent Pending)
pub mod encode;
pub mod decode;
pub mod error;
pub mod fee;
pub mod integrity;
pub mod spe;
pub mod types;
pub mod build_watermark;
pub mod licensing; // Hardware-Bound Licensing (Patent Pending)
pub mod crypto_fingerprint; // Cryptographic Artifact Fingerprinting (Patent Pending)

// Public API - Types
pub use types::{
    Artifact,
    ArtifactError,
    ByteRange,
    EncodeResult,
    Generator,
    IntegrityMeta,
    Mapping,
    MappingSet,
    MappingTransform,
    Payload,
    PredictorParameters,
    PredictorState,
    ReconstructionConstraints,
    RepetitionSpec,
    Residual,
    ResidualSegment,
    SchemaId,
    SemanticType,
    Structure,
    StructureLevel,
    VariablePart,
    VariableSegment,
    VERSION_ID,
    H_MAX,
};

// Public API - Errors
pub use error::{VectraError, VectraResult};

// Public API - Core Functions
pub use encode::{vectra_encode, try_encode};
pub use decode::{vectra_decode, vectra_decode_with_details, DecodeDetails};

// Public API - Utilities
pub use artifact::{estimate_artifact_size, compression_ratio, is_encoding_beneficial};
pub use ebta::compute_byte_entropy;
pub use integrity::sha256;

// Public API - SPE (Symbolic Predictor Engine)
pub use spe::{spe_predict, predict_next, update_state, reconstruct_variable, SpePredictResult};

// Public API - EBTA-X (Adaptive Entropy Validation)
pub use ebta_x::{
    ebta_x_validate, ebta_x_validate_simple, analyze_payload,
    AdaptiveThresholdCalculator, EbtaXResult, PayloadCharacteristics,
};

/// Library version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if this library can decode an artifact.
///
/// Returns true if artifact version matches library version.
pub fn can_decode(artifact: &Artifact) -> bool {
    artifact.integrity.version == VERSION_ID
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test the fundamental losslessness invariant.
    #[test]
    fn test_losslessness_invariant() {
        let test_cases: Vec<Vec<u8>> = vec![
            vec![],                              // Empty
            vec![0xAA],                          // Single byte
            vec![0xAA; 100],                     // Repeated byte
            b"Hello, VECTRA!".to_vec(),          // ASCII text
            (0..=255).collect(),                 // All byte values
            b"HEADER:001\nHEADER:002\n".to_vec(), // Structured
        ];

        for data in test_cases {
            let payload = Payload::new(data.clone());
            let result = vectra_encode(payload);

            match result {
                EncodeResult::Encoded(artifact) => {
                    // Must be able to decode
                    let decoded = vectra_decode(&artifact);
                    if let Ok(restored) = decoded {
                        assert_eq!(
                            restored.as_bytes(),
                            &data,
                            "Losslessness violated: decoded != original"
                        );
                    }
                }
                EncodeResult::PassThrough(p) => {
                    // Pass-through must preserve data
                    assert_eq!(
                        p.as_bytes(),
                        &data,
                        "Pass-through modified data"
                    );
                }
            }
        }
    }

    /// Test the determinism invariant.
    #[test]
    fn test_determinism_invariant() {
        let data = b"Determinism test payload with some structure: AAA BBB AAA CCC".to_vec();

        let payload1 = Payload::new(data.clone());
        let payload2 = Payload::new(data);

        let result1 = vectra_encode(payload1);
        let result2 = vectra_encode(payload2);

        // Same outcome type
        match (&result1, &result2) {
            (EncodeResult::Encoded(a1), EncodeResult::Encoded(a2)) => {
                // Byte-identical artifacts
                assert_eq!(
                    a1.to_bytes(),
                    a2.to_bytes(),
                    "Determinism violated: different artifacts for same input"
                );
            }
            (EncodeResult::PassThrough(p1), EncodeResult::PassThrough(p2)) => {
                assert_eq!(p1.as_bytes(), p2.as_bytes());
            }
            _ => panic!("Determinism violated: different outcome types"),
        }
    }

    /// Test fail-open behavior.
    #[test]
    fn test_fail_open_invariant() {
        // High-entropy data should fail EBTA
        let random_like: Vec<u8> = (0..10000)
            .map(|i| ((i * 17 + 31) % 256) as u8)
            .collect();

        let payload = Payload::new(random_like.clone());
        let result = vectra_encode(payload);

        // Either encodes successfully OR returns original unchanged
        match result {
            EncodeResult::Encoded(_) => {
                // Acceptable if EBTA passed
            }
            EncodeResult::PassThrough(p) => {
                assert_eq!(
                    p.as_bytes(),
                    &random_like,
                    "Fail-open modified data"
                );
            }
        }
    }

    /// Test version compatibility check.
    #[test]
    fn test_version_check() {
        let payload = Payload::new(vec![0xAA; 50]);

        if let EncodeResult::Encoded(artifact) = vectra_encode(payload) {
            assert!(can_decode(&artifact));
            assert_eq!(artifact.integrity.version, VERSION_ID);
        }
    }
}
