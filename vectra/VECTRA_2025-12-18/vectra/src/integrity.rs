//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! Integrity verification for VECTRA artifacts.
//!
//! Implements spec §9: Hash generation and verification.
//!
//! All integrity operations use SHA-256 for:
//! - Determinism across platforms
//! - Cryptographic security
//! - Wide support

use crate::error::DecodeError;
use crate::types::{
    Artifact, Generator, IntegrityMeta, MappingSet, Payload, PredictorState,
    ReconstructionConstraints, Residual, VERSION_ID,
};
use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of byte slice.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute hash of payload for integrity metadata.
pub fn hash_payload(payload: &Payload) -> [u8; 32] {
    sha256(payload.as_bytes())
}

/// Generate integrity metadata for an artifact.
///
/// This creates a verifiable binding between:
/// - Original payload (payload_hash)
/// - Artifact contents (artifact_hash)
/// - Version (must match for decode)
pub fn generate_integrity_metadata(
    payload: &Payload,
    generator: &Generator,
    mappings: &MappingSet,
    predictor_state: &PredictorState,
    residual: &Residual,
) -> IntegrityMeta {
    // Hash original payload
    let payload_hash = hash_payload(payload);

    // Hash artifact components (excluding integrity itself)
    let artifact_hash = hash_artifact_components(generator, mappings, predictor_state, residual);

    IntegrityMeta {
        payload_hash,
        artifact_hash,
        version: VERSION_ID,
        encoded_at: current_timestamp(),
    }
}

/// Hash artifact components for integrity verification.
///
/// Uses deterministic serialization to ensure consistent hashing.
fn hash_artifact_components(
    generator: &Generator,
    mappings: &MappingSet,
    predictor_state: &PredictorState,
    residual: &Residual,
) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Hash generator
    hasher.update(&generator.base);
    hasher.update(&generator.repetition.count.to_le_bytes());
    hasher.update(&generator.repetition.stride.to_le_bytes());
    hasher.update(&(generator.repetition.start_offset as u64).to_le_bytes());

    // Hash mappings count (mappings are deterministically ordered)
    hasher.update(&(mappings.mappings.len() as u64).to_le_bytes());
    for mapping in &mappings.mappings {
        hasher.update(&(mapping.from_level as u64).to_le_bytes());
        hasher.update(&(mapping.to_level as u64).to_le_bytes());
    }

    // Hash predictor state version
    hasher.update(&predictor_state.version.to_le_bytes());

    // Hash residual
    for segment in &residual.segments {
        hasher.update(&(segment.range.start as u64).to_le_bytes());
        hasher.update(&(segment.range.end as u64).to_le_bytes());
        hasher.update(&segment.delta);
    }

    hasher.finalize().into()
}

/// Generate reconstruction constraints.
///
/// These are checked at decode time to verify losslessness.
pub fn generate_reconstruction_constraints(payload: &Payload) -> ReconstructionConstraints {
    ReconstructionConstraints {
        output_length: payload.len(),
        output_hash: hash_payload(payload),
    }
}

/// Verify artifact integrity.
///
/// Checks:
/// 1. Version matches current VERSION_ID
/// 2. Artifact hash matches recomputed hash
///
/// # Errors
///
/// Returns error if verification fails. Decode must abort.
pub fn verify_integrity(artifact: &Artifact) -> Result<(), DecodeError> {
    // Check version
    if artifact.integrity.version != VERSION_ID {
        return Err(DecodeError::VersionMismatch {
            expected: VERSION_ID,
            found: artifact.integrity.version,
        });
    }

    // Recompute artifact hash
    let computed_hash = hash_artifact_components(
        &artifact.generator,
        &artifact.mappings,
        &artifact.predictor_state,
        &artifact.residual,
    );

    if computed_hash != artifact.integrity.artifact_hash {
        return Err(DecodeError::IntegrityFailed(
            "artifact hash mismatch".to_string(),
        ));
    }

    Ok(())
}

/// Verify reconstructed payload matches original.
///
/// This is the final losslessness check at decode time.
pub fn verify_reconstruction(
    reconstructed: &Payload,
    constraints: &ReconstructionConstraints,
) -> Result<(), DecodeError> {
    // Check length
    if reconstructed.len() != constraints.output_length {
        return Err(DecodeError::OutputHashMismatch);
    }

    // Check hash
    let reconstructed_hash = hash_payload(reconstructed);
    if reconstructed_hash != constraints.output_hash {
        return Err(DecodeError::OutputHashMismatch);
    }

    Ok(())
}

/// Get current timestamp in seconds since Unix epoch.
///
/// Used for `encoded_at` field. Not used for determinism
/// (encoding is deterministic regardless of timestamp).
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ByteRange, PredictorParameters, RepetitionSpec, ResidualSegment};

    #[test]
    fn test_sha256_determinism() {
        let data = b"test data for hashing";
        let hash1 = sha256(data);
        let hash2 = sha256(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_known_vector() {
        // Known SHA-256 test vector
        let data = b"";
        let hash = sha256(data);
        let expected = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_payload() {
        let payload = Payload::new(vec![1, 2, 3, 4, 5]);
        let hash1 = hash_payload(&payload);
        let hash2 = hash_payload(&payload);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_integrity_metadata_generation() {
        let payload = Payload::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let generator = Generator {
            base: vec![0xDE, 0xAD],
            repetition: RepetitionSpec { count: 1, stride: 2, start_offset: 0 },
        };
        let mappings = MappingSet { mappings: vec![] };
        let predictor_state = PredictorState {
            version: VERSION_ID,
            parameters: PredictorParameters::default(),
        };
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 2, end: 4 },
                delta: vec![0xBE, 0xEF],
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        let meta = generate_integrity_metadata(
            &payload,
            &generator,
            &mappings,
            &predictor_state,
            &residual,
        );

        assert_eq!(meta.version, VERSION_ID);
        assert_eq!(meta.payload_hash, hash_payload(&payload));
    }

    #[test]
    fn test_verify_integrity_success() {
        let generator = Generator {
            base: vec![0xAA],
            repetition: RepetitionSpec { count: 1, stride: 1, start_offset: 0 },
        };
        let mappings = MappingSet { mappings: vec![] };
        let predictor_state = PredictorState {
            version: VERSION_ID,
            parameters: PredictorParameters::default(),
        };
        let residual = Residual { segments: vec![] };

        let artifact_hash =
            hash_artifact_components(&generator, &mappings, &predictor_state, &residual);

        let artifact = Artifact {
            generator,
            mappings,
            predictor_state,
            residual,
            constraints: ReconstructionConstraints {
                output_length: 1,
                output_hash: [0u8; 32],
            },
            integrity: IntegrityMeta {
                payload_hash: [0u8; 32],
                artifact_hash,
                version: VERSION_ID,
                encoded_at: 0,
            },
        };

        assert!(verify_integrity(&artifact).is_ok());
    }

    #[test]
    fn test_verify_integrity_version_mismatch() {
        let artifact = Artifact {
            generator: Generator {
                base: vec![],
                repetition: RepetitionSpec { count: 0, stride: 0, start_offset: 0 },
            },
            mappings: MappingSet { mappings: vec![] },
            predictor_state: PredictorState {
                version: VERSION_ID,
                parameters: PredictorParameters::default(),
            },
            residual: Residual { segments: vec![] },
            constraints: ReconstructionConstraints {
                output_length: 0,
                output_hash: [0u8; 32],
            },
            integrity: IntegrityMeta {
                payload_hash: [0u8; 32],
                artifact_hash: [0u8; 32],
                version: VERSION_ID + 1, // Wrong version
                encoded_at: 0,
            },
        };

        let result = verify_integrity(&artifact);
        assert!(matches!(result, Err(DecodeError::VersionMismatch { .. })));
    }

    #[test]
    fn test_verify_reconstruction_success() {
        let payload = Payload::new(vec![1, 2, 3, 4]);
        let constraints = generate_reconstruction_constraints(&payload);

        assert!(verify_reconstruction(&payload, &constraints).is_ok());
    }

    #[test]
    fn test_verify_reconstruction_length_mismatch() {
        let payload = Payload::new(vec![1, 2, 3, 4]);
        let constraints = ReconstructionConstraints {
            output_length: 5, // Wrong length
            output_hash: hash_payload(&payload),
        };

        let result = verify_reconstruction(&payload, &constraints);
        assert!(matches!(result, Err(DecodeError::OutputHashMismatch)));
    }

    #[test]
    fn test_verify_reconstruction_hash_mismatch() {
        let payload = Payload::new(vec![1, 2, 3, 4]);
        let constraints = ReconstructionConstraints {
            output_length: 4,
            output_hash: [0xFFu8; 32], // Wrong hash
        };

        let result = verify_reconstruction(&payload, &constraints);
        assert!(matches!(result, Err(DecodeError::OutputHashMismatch)));
    }
}
