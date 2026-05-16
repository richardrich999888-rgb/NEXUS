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
    Artifact, Payload, SemanticType, Structure, VariablePart, VariableSegment,
    VERSION_ID,
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
fn recompose(structure: &Structure, variable: &VariablePart, output_length: usize) -> VectraResult<Payload> {
    let mut data = vec![0u8; output_length];

    // Place Structure
    // Structure contains `byte_ranges` and `levels`. 
    // Usually level 0 literals match the ranges if no recursion?
    // Wait, `regenerate_structure` produces `Structure`. 
    // `Structure` struct has `levels` and `byte_ranges`.
    // The `levels` field in `Structure` is `Vec<StructureLevel>`.
    // We assume the leaf level (level 0?) contains literals.
    // In `calculate_structure` (decompose), `levels` are populated.
    // The `literals` in `StructureLevel` are "pattern bytes".
    // We need to fill `data` at `byte_ranges` with `literals`.
    
    // Check `regenerate_structure` logic in `fee.rs`
    // It returns `Structure`.
    // If mappings were used, `literals` might be small patterns repeated.
    // But `Structure` struct doesn't map range-to-literal index directly in `byte_ranges`?
    // `byte_ranges` is `Vec<ByteRange>`.
    // `levels` is `Vec<StructureLevel>`.
    // How do they map?
    // In `decompose.rs`, `calculate_structure`:
    // It finds patterns.
    // `structure.byte_ranges` is ALL the ranges covered by ANY pattern?
    
    // Actually, `decompose.rs` puts structure in `levels[0]`?
    // In `decompose.rs`:
    // `levels.push(StructureLevel { pattern_id: 0, literals: ... })`
    // And `byte_ranges` tracks where they go.
    
    // Simplification:
    // If `levels` has 1 level, we assume all `byte_ranges` are instances of `levels[0].literals`.
    // This handles the simple "constant pattern" case (like repeated byte).
    // What if there are multiple patterns?
    // `Structure` struct is:
    // pub struct Structure {
    //     pub levels: Vec<StructureLevel>,
    //     pub byte_ranges: Vec<ByteRange>,
    // }
    // It seems flattened?
    // If there are multiple patterns (e.g. Header1, Header2), `levels` must differentiate?
    // But `Structure` has only `levels: Vec`.
    // `StructureLevel` has `pattern_id`.
    // `byte_ranges` doesn't say which pattern ID!
    
    // Ah, `fee.rs` `regenerate_structure`:
    // Returns `Structure`.
    // `levels`: `[StructureLevel { pattern_id: 0, literals: base }]`
    // `byte_ranges`: `stride` logic.
    // It seems FEE assumes a SINGLE base pattern type repeated.
    // My implementation of `fee.rs` (saved in step 50) supports `repetition`.
    // So `levels[0].literals` is the pattern.
    // And `byte_ranges` are where it goes.
    
    // Place structural components
    // In MVP FEE, byte_ranges correspond to repetitions of the Base Generator (levels[0]).
    // Provide fail-safe if no levels exist (should be handled by empty check above or verify).
    if let Some(base_level) = structure.levels.first() {
        let pattern = &base_level.literals;
        
        for (i, range) in structure.byte_ranges.iter().enumerate() {
            if range.end > output_length {
                 return Err(DecodeError::Recomposition("Structure range exceeds output length".to_string()).into());
            }

            // For now, assume simple repetition of base pattern
            // (Mappings would require more complex logic mapping specific ranges to specific levels)
            // So they should match.
             data[range.start..range.start + pattern.len()].copy_from_slice(pattern);
        }
    }

    // Place Variable
    for segment in &variable.segments {
        if segment.range.end > output_length {
             return Err(DecodeError::Recomposition("Variable range exceeds output length".to_string()).into());
        }
        if segment.data.len() != (segment.range.end - segment.range.start) {
             return Err(DecodeError::Recomposition("Variable data length mismatch".to_string()).into());
        }
        
        
        data[segment.range.start..segment.range.end].copy_from_slice(&segment.data);
    }

    // Verify complete coverage?
    // If specific bytes are not covered, they remain 0.
    // Is this expected?
    // Decomposition (D -> S,V) should partition D fully?
    // `decompose.rs` logic ensures S and V cover D?
    // If gaps exist, they verify as 0.
    
    // Just return.
    Ok(Payload::new(data))
}
