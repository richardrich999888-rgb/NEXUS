//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! Core type definitions for VECTRA.
//!
//! These types map directly to the formal specification:
//! - Payload: D ∈ 𝒟
//! - Artifact: A ∈ 𝒜
//! - Structure: S (stable structural components)
//! - VariablePart: V (time-evolving components)
//!
//! All types enforce determinism through:
//! - No interior mutability without explicit synchronization
//! - No floating-point (use fixed-point or rational)
//! - No random number generation
//! - Version-locked serialization

use serde::{Deserialize, Serialize};
use std::fmt;

/// System version identifier. All artifacts are version-locked.
/// Changing this constant breaks backward compatibility by design.
pub const VERSION_ID: u64 = 0x0001_0000_0000_0001;

/// Maximum allowed Shannon entropy for residuals (H_MAX from spec §6).
/// Value is in bits. Residuals exceeding this trigger fail-open.
/// This is a conservative default; tune per deployment.
pub const H_MAX: f64 = 4.0;

/// Maximum payload size in bytes to prevent DoS attacks.
/// Payloads exceeding this limit will be rejected.
/// Default: 100 MB (conservative for most use cases).
pub const MAX_PAYLOAD_SIZE: usize = 100 * 1024 * 1024;

/// Maximum pattern length in bytes during decomposition.
/// Prevents excessive memory usage in pattern matching.
pub const MAX_PATTERN_LEN: usize = 1024;

/// Raw payload bytes. Represents D ∈ 𝒟.
///
/// Payloads are treated as opaque byte sequences at the system boundary.
/// Internal decomposition extracts structure.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Payload {
    /// Raw byte content
    data: Vec<u8>,
    /// Schema identifier for structured interpretation (optional)
    schema_id: Option<SchemaId>,
}

impl Payload {
    /// Create a new payload from raw bytes.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            schema_id: None,
        }
    }

    /// Create a payload with an associated schema.
    #[must_use]
    pub fn with_schema(data: Vec<u8>, schema_id: SchemaId) -> Self {
        Self {
            data,
            schema_id: Some(schema_id),
        }
    }

    /// Access raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Consume and return owned bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Get schema identifier if present.
    #[must_use]
    pub fn schema_id(&self) -> Option<&SchemaId> {
        self.schema_id.as_ref()
    }

    /// Byte length of payload.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if payload is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl fmt::Debug for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Payload")
            .field("len", &self.data.len())
            .field("schema_id", &self.schema_id)
            .finish()
    }
}

/// Schema identifier for typed payload interpretation.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct SchemaId {
    /// Namespace (e.g., "telecom.oss", "observability.otel")
    pub namespace: String,
    /// Schema name within namespace
    pub name: String,
    /// Schema version (semver-compatible)
    pub version: (u16, u16, u16),
}

/// Structural component extracted from payload (S from spec §3).
///
/// Represents stable, repeatable patterns in the data.
/// FEE operates on this to produce generators.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Structure {
    /// Hierarchical structure representation.
    /// Each level represents a self-similar pattern.
    pub levels: Vec<StructureLevel>,
    /// Original byte ranges this structure covers.
    pub byte_ranges: Vec<ByteRange>,
}

/// A single level in the structural hierarchy.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct StructureLevel {
    /// Pattern identifier at this level
    pub pattern_id: u64,
    /// Child indices (references to sub-patterns)
    pub children: Vec<usize>,
    /// Literal bytes at this level (leaf nodes)
    pub literals: Vec<u8>,
}

/// Byte range in original payload.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

/// Variable component extracted from payload (V from spec §3).
///
/// Represents time-evolving or non-repeatable data.
/// SPE predicts this component.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VariablePart {
    /// Variable data segments
    pub segments: Vec<VariableSegment>,
}

/// A segment of variable data.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VariableSegment {
    /// Byte range in original payload
    pub range: ByteRange,
    /// Actual variable bytes
    pub data: Vec<u8>,
    /// Semantic type hint for prediction
    pub semantic_type: SemanticType,
}

/// Semantic type hints for variable data.
/// Guides SPE prediction model selection.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum SemanticType {
    /// Monotonic counter (e.g., sequence numbers)
    Counter,
    /// Timestamp (various formats)
    Timestamp,
    /// Metric value (bounded numeric)
    Metric,
    /// Identifier (UUIDs, hashes)
    Identifier,
    /// Unknown/opaque
    Opaque,
}

/// Structural generator produced by FEE (G from spec §4).
///
/// Encodes the base pattern from which structure can be regenerated.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Generator {
    /// Base pattern bytes
    pub base: Vec<u8>,
    /// Pattern repetition metadata
    pub repetition: RepetitionSpec,
}

/// Specification for how a pattern repeats.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct RepetitionSpec {
    /// Number of repetitions
    pub count: u32,
    /// Byte offset between repetitions (kept for compression optimization)
    pub stride: u32,
    /// Start offset of the first occurrence
    pub start_offset: usize,
    /// Actual byte ranges where the pattern occurs.
    /// Used for exact reconstruction when patterns have non-uniform spacing.
    pub byte_ranges: Vec<ByteRange>,
}

/// Recursive mapping function (φ from spec §4).
///
/// Maps one structural level to the next.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Mapping {
    /// Source level index
    pub from_level: usize,
    /// Target level index
    pub to_level: usize,
    /// Transformation parameters
    pub transform: MappingTransform,
}

/// Transformation applied by a mapping.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum MappingTransform {
    /// Direct reference (no transformation)
    Identity,
    /// Byte offset within pattern
    Offset(i32),
    /// Concatenation of sub-patterns
    Concat(Vec<usize>),
}

/// Set of mappings Φ = {φ₀, φ₁, ..., φₖ} from spec §4.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct MappingSet {
    pub mappings: Vec<Mapping>,
}

/// Predictor state (Θ from spec §5).
///
/// Version-locked state for deterministic prediction.
/// MUST be identical for same VERSION_ID.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PredictorState {
    /// Version this state is locked to
    pub version: u64,
    /// Model parameters (deterministic, no learning at decode)
    pub parameters: PredictorParameters,
}

/// Predictor model parameters.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PredictorParameters {
    /// Counter prediction: last seen values
    pub counter_state: Vec<i64>,
    /// Timestamp prediction: base and delta
    pub timestamp_base: i64,
    pub timestamp_delta: i64,
    /// Metric prediction: running statistics (fixed-point)
    pub metric_mean: i64,  // Fixed-point, scale factor 1000
    pub metric_variance: i64,
}

impl Default for PredictorParameters {
    fn default() -> Self {
        Self {
            counter_state: Vec::new(),
            timestamp_base: 0,
            timestamp_delta: 0,
            metric_mean: 0,
            metric_variance: 0,
        }
    }
}

/// Residual Δ = V - V̂ from spec §5.
///
/// The difference between actual and predicted variable data.
/// Subject to entropy bounds enforced by EBTA.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Residual {
    /// Residual bytes per segment
    pub segments: Vec<ResidualSegment>,
}

/// Residual for a single variable segment.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ResidualSegment {
    /// Byte range in original payload
    pub range: ByteRange,
    /// Delta bytes (XOR with prediction)
    pub delta: Vec<u8>,
    /// Semantic type hint for correct prediction at decode time
    pub semantic_type: SemanticType,
}

/// Integrity metadata (I from spec §7).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct IntegrityMeta {
    /// SHA-256 hash of original payload
    pub payload_hash: [u8; 32],
    /// SHA-256 hash of artifact content (excluding this field)
    pub artifact_hash: [u8; 32],
    /// Version identifier
    pub version: u64,
    /// Timestamp of encoding (deterministic: seconds since epoch)
    pub encoded_at: u64,
}

/// Reconstruction constraints (C from spec §7).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ReconstructionConstraints {
    /// Expected output length
    pub output_length: usize,
    /// Expected output hash
    pub output_hash: [u8; 32],
}

/// Complete VECTRA artifact (A from spec §7).
///
/// Self-describing, self-verifiable encoded representation.
/// Contains everything needed for reconstruction.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Artifact {
    /// Structural generator (from FEE)
    pub generator: Generator,
    /// Recursive mappings (from FEE)
    pub mappings: MappingSet,
    /// Predictor state (from SPE)
    pub predictor_state: PredictorState,
    /// Bounded residual (approved by EBTA)
    pub residual: Residual,
    /// Reconstruction constraints
    pub constraints: ReconstructionConstraints,
    /// Integrity metadata
    pub integrity: IntegrityMeta,
}

impl Artifact {
    /// Serialize artifact to bytes (deterministic binary format).
    ///
    /// Uses bincode for compact, fast serialization.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Artifact serialization cannot fail")
    }

    /// Deserialize artifact from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ArtifactError> {
        bincode::deserialize(bytes).map_err(|e| ArtifactError::DeserializationFailed {
            reason: e.to_string(),
        })
    }
}

/// Artifact-specific errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtifactError {
    DeserializationFailed { reason: String },
    IntegrityCheckFailed,
    VersionMismatch { expected: u64, found: u64 },
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializationFailed { reason } => {
                write!(f, "artifact deserialization failed: {}", reason)
            }
            Self::IntegrityCheckFailed => write!(f, "artifact integrity check failed"),
            Self::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "artifact version mismatch: expected {:#x}, found {:#x}",
                    expected, found
                )
            }
        }
    }
}

impl std::error::Error for ArtifactError {}

/// Result of encoding: either an artifact or the original payload (fail-open).
#[derive(Clone, Debug)]
pub enum EncodeResult {
    /// Successfully encoded to artifact
    Encoded(Artifact),
    /// Failed to encode safely, returning original payload
    PassThrough(Payload),
}

impl EncodeResult {
    /// Check if encoding succeeded.
    #[must_use]
    pub fn is_encoded(&self) -> bool {
        matches!(self, Self::Encoded(_))
    }

    /// Check if encoding failed open.
    #[must_use]
    pub fn is_pass_through(&self) -> bool {
        matches!(self, Self::PassThrough(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let payload = Payload::new(data.clone());
        assert_eq!(payload.as_bytes(), &data);
        assert_eq!(payload.len(), 5);
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_artifact_serialization_determinism() {
        // Create two identical artifacts
        let artifact1 = create_test_artifact();
        let artifact2 = create_test_artifact();

        // Serialize both
        let bytes1 = artifact1.to_bytes();
        let bytes2 = artifact2.to_bytes();

        // Must be byte-identical
        assert_eq!(bytes1, bytes2, "Artifact serialization must be deterministic");
    }

    #[test]
    fn test_artifact_round_trip() {
        let original = create_test_artifact();
        let bytes = original.to_bytes();
        let restored = Artifact::from_bytes(&bytes).expect("deserialization should succeed");
        assert_eq!(original, restored);
    }

    fn create_test_artifact() -> Artifact {
        Artifact {
            generator: Generator {
                base: vec![0x00, 0x01],
                repetition: RepetitionSpec { count: 1, stride: 2, start_offset: 0, byte_ranges: vec![] },
            },
            mappings: MappingSet {
                mappings: vec![],
            },
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
}
