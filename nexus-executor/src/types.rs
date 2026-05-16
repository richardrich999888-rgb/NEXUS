//! Core types for execution context and results.

use nexus_pcu::{ContentHash, IdentityContext};
use crate::limits::ExecutionLimits;
use crate::proof::ExecutionProof;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Context for PCU execution.
///
/// Contains all data needed to execute a PCU:
/// - Input data (already fetched)
/// - Identity for capability checking
/// - Resource limits
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Input data keyed by content hash.
    pub inputs: Vec<(ContentHash, Vec<u8>)>,

    /// Identity context for capability verification.
    pub identity: IdentityContext,

    /// Resource limits.
    pub limits: ExecutionLimits,

    /// Optional: Request ID for tracing.
    pub request_id: Option<String>,

    /// Optional: Biological / AHES-derived risk [0, 1] for guard (e.g. stress, cortisol).
    /// When set, NervousSystemGuard uses this as estimated_risk for autonomic check.
    /// Invariant: write-once per request; not exposed to executing PCU (guest has no access).
    pub biological_risk: Option<f64>,
}

impl ExecutionContext {
    /// Create new execution context.
    pub fn new(
        inputs: Vec<(ContentHash, Vec<u8>)>,
        identity: IdentityContext,
        limits: ExecutionLimits,
    ) -> Self {
        Self {
            inputs,
            identity,
            limits,
            request_id: None,
            biological_risk: None,
        }
    }

    /// Create minimal context for testing.
    pub fn minimal() -> Self {
        Self {
            inputs: Vec::new(),
            identity: IdentityContext::anonymous(),
            limits: ExecutionLimits::minimal(),
            request_id: None,
            biological_risk: None,
        }
    }

    /// Set request ID for tracing.
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Set biological/AHES-derived risk for guard (e.g. from hormone levels).
    pub fn with_biological_risk(mut self, risk: f64) -> Self {
        self.biological_risk = Some(risk.clamp(0.0, 1.0));
        self
    }

    /// Get input data by hash.
    pub fn get_input(&self, hash: &ContentHash) -> Option<&[u8]> {
        self.inputs
            .iter()
            .find(|(h, _)| h == hash)
            .map(|(_, data)| data.as_slice())
    }

    /// Get input data by index.
    pub fn get_input_by_index(&self, index: usize) -> Option<&[u8]> {
        self.inputs.get(index).map(|(_, data)| data.as_slice())
    }

    /// Number of inputs.
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Total size of all inputs.
    pub fn total_input_size(&self) -> usize {
        self.inputs.iter().map(|(_, data)| data.len()).sum()
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::minimal()
    }
}

/// Result of PCU execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Output data produced by the PCU.
    pub output: Vec<u8>,

    /// Content hash of the output.
    pub output_hash: ContentHash,

    /// Fuel (instructions) consumed.
    pub fuel_consumed: u64,

    /// Peak memory usage in bytes.
    pub peak_memory: usize,

    /// Wall-clock execution duration.
    #[serde(with = "duration_serde")]
    pub duration: Duration,
}

impl ExecutionResult {
    /// Create new result.
    pub fn new(output: Vec<u8>, fuel_consumed: u64, peak_memory: usize, duration: Duration) -> Self {
        let output_hash = ContentHash::compute(&output);
        Self {
            output,
            output_hash,
            fuel_consumed,
            peak_memory,
            duration,
        }
    }

    /// Output size in bytes.
    pub fn output_size(&self) -> usize {
        self.output.len()
    }

    /// Check if output is empty.
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            output: Vec::new(),
            output_hash: ContentHash::zero(),
            fuel_consumed: 0,
            peak_memory: 0,
            duration: Duration::ZERO,
        }
    }
}

/// Complete execution response including proof.
#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    /// The execution result.
    pub result: ExecutionResult,

    /// Cryptographic proof of execution.
    pub proof: ExecutionProof,

    /// Whether result was served from cache.
    pub cached: bool,
}

impl ExecutionResponse {
    /// Create new response.
    pub fn new(result: ExecutionResult, proof: ExecutionProof, cached: bool) -> Self {
        Self {
            result,
            proof,
            cached,
        }
    }
}

/// Serde helpers for Duration.
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_nanos().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u128::deserialize(deserializer)?;
        Ok(Duration::from_nanos(nanos as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context() {
        let hash1 = ContentHash::compute(b"input1");
        let hash2 = ContentHash::compute(b"input2");

        let ctx = ExecutionContext::new(
            vec![
                (hash1, b"input1".to_vec()),
                (hash2, b"input2".to_vec()),
            ],
            IdentityContext::anonymous(),
            ExecutionLimits::default(),
        );

        assert_eq!(ctx.input_count(), 2);
        assert_eq!(ctx.get_input(&hash1), Some(b"input1".as_slice()));
        assert_eq!(ctx.get_input_by_index(0), Some(b"input1".as_slice()));
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult::new(
            b"output".to_vec(),
            1000,
            1024,
            Duration::from_millis(10),
        );

        assert_eq!(result.output_size(), 6);
        assert!(!result.is_empty());
        assert_eq!(result.output_hash, ContentHash::compute(b"output"));
    }
}
