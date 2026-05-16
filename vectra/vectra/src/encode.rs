//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! Top-level encoding orchestration.
//!
//! Implements spec §1: E : 𝒟 → 𝒜 ∪ 𝒟
//!
//! This module orchestrates the encoding pipeline:
//! 1. Decompose payload into (S, V)
//! 2. FEE encode structure: S → (G, Φ)
//! 3. SPE predict variable: V → (V̂, Θ)
//! 4. Compute residual: Δ = V - V̂
//! 5. EBTA validate: H(Δ) ≤ H_MAX?
//! 6. If valid: build artifact
//! 7. If invalid: return original (fail-open)

use crate::artifact::build_artifact;
use crate::decompose::decompose;
use crate::ebta::{compute_residual, ebta_validate};
use crate::error::{EncodeError, VectraError, VectraResult};
use crate::fee::fee_encode;
use crate::spe::spe_predict;
use crate::types::{Artifact, EncodeResult, Payload, Residual};

/// Encode a payload using VECTRA.
///
/// # Returns
///
/// - `EncodeResult::Encoded(artifact)` if encoding succeeds
/// - `EncodeResult::PassThrough(payload)` if encoding cannot be safely performed
///
/// # Guarantees
///
/// - Determinism: same input → same output
/// - Losslessness: decode(encode(D)) == D
/// - Fail-open: uncertainty → return original
pub fn vectra_encode(payload: Payload) -> EncodeResult {
    match encode_internal(&payload) {
        Ok(artifact) => EncodeResult::Encoded(artifact),
        Err(_e) => {
            #[cfg(feature = "debug-logging")]
            eprintln!("VECTRA encode failed: {:?}", _e);
            EncodeResult::PassThrough(payload)
        }
    }
}

/// Internal encoding implementation.
fn encode_internal(payload: &Payload) -> Result<Artifact, EncodeError> {
    // Step 1: Decompose payload
    let decomposition = decompose(payload)?;

    // Step 2: FEE encode structure
    let fee_result = fee_encode(&decomposition.structure)?;

    // Step 3: SPE predict variable component
    let spe_result = spe_predict(&decomposition.variable)?;

    // Step 4: Compute residual (Δ = V - V̂)
    let residual = compute_residual_from_parts(
        &decomposition.variable,
        &spe_result.predicted,
    )?;

    // Step 5: EBTA validate entropy bound
    let ebta_result = ebta_validate(&residual);
    if !ebta_result.valid {
        return Err(EncodeError::Ebta {
            entropy: ebta_result.entropy,
            max: ebta_result.max_entropy,
        });
    }

    // Step 6: Build artifact
    build_artifact(
        payload,
        fee_result.generator,
        fee_result.mappings,
        spe_result.state,
        residual,
    )
}

/// Compute residual from actual and predicted variable parts.
fn compute_residual_from_parts(
    actual: &crate::types::VariablePart,
    predicted: &crate::types::VariablePart,
) -> Result<Residual, EncodeError> {
    if actual.segments.len() != predicted.segments.len() {
        return Err(EncodeError::Spe(
            "segment count mismatch between actual and predicted".to_string(),
        ));
    }

    let mut segments = Vec::with_capacity(actual.segments.len());

    for (actual_seg, predicted_seg) in actual.segments.iter().zip(predicted.segments.iter()) {
        let segment = compute_residual(
            &actual_seg.data,
            &predicted_seg.data,
            actual_seg.range,
            actual_seg.semantic_type,
        )?;
        segments.push(segment);
    }

    Ok(Residual { segments })
}

/// Try to encode, returning error instead of fail-open.
pub fn try_encode(payload: &Payload) -> VectraResult<Artifact> {
    encode_internal(payload).map_err(VectraError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty_payload() {
        let payload = Payload::new(vec![]);
        let result = vectra_encode(payload);
        assert!(result.is_encoded() || result.is_pass_through());
    }

    #[test]
    fn test_encode_determinism() {
        let data = b"HEADER:12345:HEADER:67890".to_vec();
        let payload1 = Payload::new(data.clone());
        let payload2 = Payload::new(data);

        let result1 = vectra_encode(payload1);
        let result2 = vectra_encode(payload2);

        assert_eq!(result1.is_encoded(), result2.is_encoded());

        if let (EncodeResult::Encoded(a1), EncodeResult::Encoded(a2)) = (&result1, &result2) {
            assert_eq!(a1.to_bytes(), a2.to_bytes());
        }
    }
}
