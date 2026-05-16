//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! Top-level decoding orchestration.
//!
//! Implements spec §8: R : 𝒜 → 𝒟
//!
//! This module orchestrates the decoding pipeline:
//! 1. Verify integrity
//! 2. Regenerate structure: (G, Φ) → S
//! 3. Reconstruct variable part: (Δ, Θ) → V
//! 4. Recompose payload: (S, V) → D
//! 5. Verify final output hash

use crate::error::{DecodeError, VectraResult};
use crate::fee::regenerate_structure;
use crate::integrity::{verify_integrity, verify_reconstruction};
use crate::spe::{predict_next, update_state};
use crate::types::{
    Artifact, Payload, Structure, VariablePart, VariableSegment,
};

/// Decode an artifact into the original payload.
pub fn vectra_decode(artifact: &Artifact) -> VectraResult<Payload> {
    // 1. Verify Integrity
    verify_integrity(artifact)?;

    // 2. Regenerate Structure S
    let structure = regenerate_structure(&artifact.generator, &artifact.mappings);

    // 3. Reconstruct Variable Part V
    // V = V_hat XOR Δ
    // We must predict V_hat using the same logic as the encoder.
    // However, the predictor needs the *actual* previous values match the encoder's state.
    // The encoder updated state using V (actual).
    // The decoder must update state using V (reconstructed).
    
    let mut params = artifact.predictor_state.parameters.clone();
    let mut variable_segments = Vec::with_capacity(artifact.residual.segments.len());

    for residual_segment in &artifact.residual.segments {
        let len = residual_segment.range.end - residual_segment.range.start;
        let semantic_type = residual_segment.semantic_type;

        // A. Predict V_hat using current state
        let predicted_bytes = predict_next(semantic_type, len, &params);

        // B. Reconstruct V = V_hat XOR Δ
        if predicted_bytes.len() != residual_segment.delta.len() {
             return Err(DecodeError::VariableReconstruction(format!(
                "Prediction length mismatch: expected {}, got {}",
                residual_segment.delta.len(),
                predicted_bytes.len()
            )).into());
        }

        let reconstructed_data: Vec<u8> = predicted_bytes
            .iter()
            .zip(residual_segment.delta.iter())
            .map(|(p, d)| p ^ d)
            .collect();

        // C. Update state using reconstructed V
        update_state(semantic_type, &reconstructed_data, &mut params);

        variable_segments.push(VariableSegment {
            range: residual_segment.range,
            data: reconstructed_data,
            semantic_type,
        });
    }

    let variable_part = VariablePart {
        segments: variable_segments,
    };

    // 4. Recompose Payload D
    let payload = recompose(&structure, &variable_part, artifact.constraints.output_length)?;

    // 5. Final Verification
    verify_reconstruction(&payload, &artifact.constraints)?;

    Ok(payload)
}

/// Additional details about decoding (optional, for debug/analysis).
#[derive(Debug, Clone)]
pub struct DecodeDetails {
    pub total_bytes: usize,
    pub structure_bytes: usize,
    pub variable_bytes: usize,
}

/// Decode with details.
pub fn vectra_decode_with_details(artifact: &Artifact) -> VectraResult<(Payload, DecodeDetails)> {
    let payload = vectra_decode(artifact)?;

    // Calculate details (basic)
    let total_bytes = payload.len();
    // Rough estimation
    let structure_bytes = artifact.generator.base.len(); // Simplified
    let variable_bytes: usize = artifact.residual.segments.iter().map(|s| s.delta.len()).sum();

    Ok((payload, DecodeDetails {
        total_bytes,
        structure_bytes,
        variable_bytes,
    }))
}

/// Recompose payload from Structure and VariablePart.
/// 
/// Implements decode algorithm:
/// - Phase 0: Allocate output buffer (zeros)
/// - Phase 1: Apply structural patterns in deterministic order
/// - Phase 2: Apply variable segments (residuals) to fill gaps
///
/// In VECTRA's design, patterns and variable segments are NON-OVERLAPPING:
/// - Variable segments cover GAPS between structural ranges
/// - So application order doesn't cause overwriting
fn recompose(structure: &Structure, variable: &VariablePart, output_length: usize) -> VectraResult<Payload> {
    // Phase 0: Allocate output buffer
    let mut data = vec![0u8; output_length];

    // Phase 1: Apply structural patterns in deterministic order
    // Structure: levels contains patterns, byte_ranges contains where they go
    // In current design, byte_ranges are only for the first pattern (levels[0])
    if !structure.levels.is_empty() && !structure.byte_ranges.is_empty() {
        // Build list of (level_index, start_position, pattern_bytes)
        let mut segments: Vec<(usize, usize, Vec<u8>)> = Vec::new();
        
        // For MVP: byte_ranges correspond to levels[0] only
        if let Some(base_level) = structure.levels.first() {
            let pattern = &base_level.literals;
            for range in &structure.byte_ranges {
                if range.end > output_length {
                    return Err(DecodeError::Recomposition(
                        "Structure range exceeds output length".to_string()
                    ).into());
                }
                // Store level 0, position, and pattern
                segments.push((0, range.start, pattern.clone()));
            }
        }
        
        // Sort by level (lower first), then by position (deterministic order)
        segments.sort_by_key(|(level, start, _)| (*level, *start));
        
        // Apply patterns in sorted order
        for (_, start, pattern) in segments {
            let len = pattern.len().min(output_length.saturating_sub(start));
            if len > 0 {
                data[start..start + len].copy_from_slice(&pattern[..len]);
            }
        }
    }

    // Phase 2: Apply variable segments (residuals) to fill gaps between structural ranges
    for segment in &variable.segments {
        if segment.range.end > output_length {
            return Err(DecodeError::Recomposition(
                "Variable range exceeds output length".to_string()
            ).into());
        }
        if segment.data.len() != (segment.range.end - segment.range.start) {
            return Err(DecodeError::Recomposition(
                "Variable data length mismatch".to_string()
            ).into());
        }
        data[segment.range.start..segment.range.end].copy_from_slice(&segment.data);
    }

    Ok(Payload::new(data))
}
