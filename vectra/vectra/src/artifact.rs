//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! Artifact construction (TDF — VECTRA Data Format)
//!
//! Implements spec §7: Self-describing artifact assembly.
//!
//! The artifact contains everything needed for reconstruction:
//! - G, Φ: Structural generators (from FEE)
//! - Θ: Predictor state (from SPE)
//! - Δ: Bounded residual (approved by EBTA)
//! - C: Reconstruction constraints
//! - I: Integrity metadata

use crate::error::EncodeError;
use crate::integrity::{generate_integrity_metadata, generate_reconstruction_constraints};
use crate::types::{
    Artifact, Generator, MappingSet, Payload, PredictorState, Residual,
};

/// Build complete artifact from components.
///
/// # Preconditions
///
/// - EBTA has validated the residual (entropy within bounds)
/// - All components are deterministically generated
///
/// # Postconditions
///
/// - Artifact is self-describing (contains all reconstruction info)
/// - Artifact is self-verifiable (contains integrity metadata)
/// - Artifact can be decoded without external context
pub fn build_artifact(
    payload: &Payload,
    generator: Generator,
    mappings: MappingSet,
    predictor_state: PredictorState,
    residual: Residual,
) -> Result<Artifact, EncodeError> {
    // Generate reconstruction constraints
    let constraints = generate_reconstruction_constraints(payload);

    // Generate integrity metadata
    let integrity = generate_integrity_metadata(
        payload,
        &generator,
        &mappings,
        &predictor_state,
        &residual,
    );

    Ok(Artifact {
        generator,
        mappings,
        predictor_state,
        residual,
        constraints,
        integrity,
    })
}

/// Estimate artifact size in bytes.
///
/// Useful for deciding whether encoding provides benefit.
pub fn estimate_artifact_size(artifact: &Artifact) -> usize {
    let mut size = 0;

    // Generator
    size += artifact.generator.base.len();
    size += 8; // repetition spec

    // Mappings (estimate)
    size += artifact.mappings.mappings.len() * 24;

    // Predictor state (estimate)
    size += 64;

    // Residual
    for segment in &artifact.residual.segments {
        size += segment.delta.len();
        size += 16; // range metadata
    }

    // Constraints
    size += 8 + 32; // length + hash

    // Integrity
    size += 32 + 32 + 8 + 8; // hashes + version + timestamp

    size
}

/// Check if encoding provides size benefit.
///
/// Returns true if artifact is smaller than original payload.
pub fn is_encoding_beneficial(payload: &Payload, artifact: &Artifact) -> bool {
    let artifact_size = estimate_artifact_size(artifact);
    artifact_size < payload.len()
}

/// Compute compression ratio.
///
/// Returns original_size / artifact_size.
/// Values > 1.0 indicate compression benefit.
pub fn compression_ratio(payload: &Payload, artifact: &Artifact) -> f64 {
    let artifact_size = estimate_artifact_size(artifact);
    if artifact_size == 0 {
        return 1.0;
    }
    payload.len() as f64 / artifact_size as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        ByteRange, PredictorParameters, RepetitionSpec, ResidualSegment, VERSION_ID,
    };

    fn create_test_payload() -> Payload {
        Payload::new(vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE])
    }

    fn create_test_components() -> (Generator, MappingSet, PredictorState, Residual) {
        let generator = Generator {
            base: vec![0xDE, 0xAD],
            repetition: RepetitionSpec { count: 4, stride: 2, start_offset: 0, byte_ranges: vec![] },
        };

        let mappings = MappingSet { mappings: vec![] };

        let predictor_state = PredictorState {
            version: VERSION_ID,
            parameters: PredictorParameters::default(),
        };

        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 8 },
                delta: vec![0x00; 8],
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        (generator, mappings, predictor_state, residual)
    }

    #[test]
    fn test_build_artifact() {
        let payload = create_test_payload();
        let (generator, mappings, predictor_state, residual) = create_test_components();

        let artifact =
            build_artifact(&payload, generator, mappings, predictor_state, residual).unwrap();

        // Verify structure
        assert_eq!(artifact.generator.base, vec![0xDE, 0xAD]);
        assert_eq!(artifact.integrity.version, VERSION_ID);
        assert_eq!(artifact.constraints.output_length, 8);
    }

    #[test]
    fn test_build_artifact_determinism() {
        let payload = create_test_payload();

        let (g1, m1, p1, r1) = create_test_components();
        let (g2, m2, p2, r2) = create_test_components();

        let artifact1 = build_artifact(&payload, g1, m1, p1, r1).unwrap();
        let artifact2 = build_artifact(&payload, g2, m2, p2, r2).unwrap();

        // Core components must match
        assert_eq!(artifact1.generator.base, artifact2.generator.base);
        assert_eq!(artifact1.constraints.output_hash, artifact2.constraints.output_hash);
        assert_eq!(artifact1.integrity.payload_hash, artifact2.integrity.payload_hash);
    }

    #[test]
    fn test_estimate_artifact_size() {
        let payload = create_test_payload();
        let (generator, mappings, predictor_state, residual) = create_test_components();

        let artifact =
            build_artifact(&payload, generator, mappings, predictor_state, residual).unwrap();

        let size = estimate_artifact_size(&artifact);
        assert!(size > 0);
    }

    #[test]
    fn test_compression_ratio() {
        // Large payload with small residual should have good ratio
        let payload = Payload::new(vec![0xAA; 1000]);

        let generator = Generator {
            base: vec![0xAA],
            repetition: RepetitionSpec {
                count: 1000,
                stride: 1,
                start_offset: 0,
                byte_ranges: vec![],
            },
        };
        let mappings = MappingSet { mappings: vec![] };
        let predictor_state = PredictorState {
            version: VERSION_ID,
            parameters: PredictorParameters::default(),
        };
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 1000 },
                delta: vec![0x00; 10], // Very small residual
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        let artifact =
            build_artifact(&payload, generator, mappings, predictor_state, residual).unwrap();

        let ratio = compression_ratio(&payload, &artifact);
        assert!(ratio > 1.0, "Should have compression benefit");
    }
}
