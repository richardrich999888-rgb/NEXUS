//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! SPE — Symbolic Predictor Engine
//!
//! Implements spec §5: Deterministic prediction of variable components.
//!
//! SPE predicts V_hat from V, producing:
//! - V_hat: Predicted variable component
//! - Θ: Predictor state (version-locked)
//!
//! Key properties:
//! - Deterministic: same input + same version → same prediction
//! - No learning at decode time
//! - Symbolic constraints enforce bounded behavior
//!
//! # Implementation
//!
//! SPE uses rule-based symbolic predictors for different semantic types:
//! - Counter: arithmetic progression (last + delta)
//! - Timestamp: linear progression (base + delta)
//! - Metric: exponential moving average
//! - Identifier/Opaque: zero prediction
//!
//! All predictions are deterministic and based on observable patterns
//! in the data, without requiring machine learning models.

use crate::error::EncodeError;
use crate::types::{
    ByteRange, PredictorParameters, PredictorState, SemanticType, VariablePart, VariableSegment,
    VERSION_ID,
};

/// Result of SPE prediction.
#[derive(Debug, Clone)]
pub struct SpePredictResult {
    /// Predicted variable component
    pub predicted: VariablePart,
    /// Predictor state (for artifact)
    pub state: PredictorState,
}

/// Compute deterministic prediction of variable component.
///
/// # Algorithm (spec §5.1-5.2)
///
/// For each variable segment:
///   1. Select predictor based on semantic type
///   2. Apply deterministic prediction model
///   3. Update predictor state
///
/// # Determinism
///
/// Prediction is fully deterministic. The predictor state Θ is
/// version-locked and produces identical results for VERSION_ID.
pub fn spe_predict(variable: &VariablePart) -> Result<SpePredictResult, EncodeError> {
    let mut state = load_predictor_state()?;
    let initial_state = state.clone();
    let mut predicted_segments = Vec::with_capacity(variable.segments.len());

    for segment in &variable.segments {
        let predicted_data = predict_segment(segment, &mut state.parameters)?;
        predicted_segments.push(VariableSegment {
            range: segment.range,
            data: predicted_data,
            semantic_type: segment.semantic_type,
        });
    }

    Ok(SpePredictResult {
        predicted: VariablePart {
            segments: predicted_segments,
        },
        state: initial_state,
    })
}

/// Load version-locked predictor state.
///
/// This state MUST be identical for the same VERSION_ID.
/// Any change to prediction behavior requires VERSION_ID bump.
fn load_predictor_state() -> Result<PredictorState, EncodeError> {
    Ok(PredictorState {
        version: VERSION_ID,
        parameters: PredictorParameters::default(),
    })
}

/// Predict next value for a variable segment (decode path).
///
/// Generates V_hat without needing original V.
pub fn predict_next(
    semantic_type: SemanticType,
    width: usize,
    params: &PredictorParameters,
) -> Vec<u8> {
    match semantic_type {
        SemanticType::Counter => {
            // Logic derived from predict_counter: last + delta
            // Use wrapping arithmetic to prevent overflow panics on extreme values
            let last = params.counter_state.last().copied().unwrap_or(0);
            let delta = if params.counter_state.len() >= 2 {
                last.wrapping_sub(params.counter_state[params.counter_state.len() - 2])
            } else {
                1
            };
            let predicted_value = if params.counter_state.is_empty() {
                0
            } else {
                last.wrapping_add(delta)
            };
            format_ascii_number(predicted_value, width)
        }
        SemanticType::Timestamp => {
            // Logic derived from predict_timestamp: base + delta
            // Use wrapping arithmetic to prevent overflow panics on extreme values
            let predicted_ts = if params.timestamp_base == 0 {
                0
            } else {
                params.timestamp_base.wrapping_add(params.timestamp_delta)
            };
            format_ascii_number(predicted_ts, width)
        }
        SemanticType::Metric => {
            // Logic derived from predict_metric: mean / 1000
            let predicted_value = params.metric_mean / 1000;
            encode_binary_le(predicted_value, width)
        }
        SemanticType::Identifier | SemanticType::Opaque => vec![0u8; width],
    }
}

/// Update predictor state with observed data (decode path).
///
/// Updates Θ using reconstructed V.
pub fn update_state(
    semantic_type: SemanticType,
    data: &[u8],
    params: &mut PredictorParameters,
) {
    match semantic_type {
        SemanticType::Counter => {
            // Parse as ASCII (per decompose assumption)
            let current_value = parse_ascii_number(data);
            params.counter_state.push(current_value);
            if params.counter_state.len() > 16 {
                params.counter_state.remove(0);
            }
        }
        SemanticType::Timestamp => {
            // Parse as ASCII
            let current_ts = parse_ascii_number(data);
            if params.timestamp_base != 0 {
                // Use wrapping arithmetic to match predict_next() behavior
                params.timestamp_delta = current_ts.wrapping_sub(params.timestamp_base);
            }
            params.timestamp_base = current_ts;
        }
        SemanticType::Metric => {
            // Parse as Binary
            let current_value = parse_binary_le(data);
            // new_mean = 0.9 * old_mean + 0.1 * current
            // Use saturating arithmetic to prevent overflow on extreme values
            let term1 = 900_i64.saturating_mul(params.metric_mean);
            let term2 = 100_i64.saturating_mul(current_value).saturating_mul(1000);
            params.metric_mean = term1.saturating_add(term2) / 1000;
        }
        SemanticType::Identifier | SemanticType::Opaque => {
            // No state update
        }
    }
}

/// Reconstruct variable component from prediction and residual.
///
/// V = V_hat + Δ (where + is XOR for bytes)
pub fn reconstruct_variable(predicted: &VariablePart, residual: &[Vec<u8>]) -> VariablePart {
    let mut segments = Vec::with_capacity(predicted.segments.len());

    for (pred_seg, res_data) in predicted.segments.iter().zip(residual.iter()) {
        let reconstructed = pred_seg
            .data
            .iter()
            .zip(res_data.iter())
            .map(|(p, r)| p ^ r)
            .collect();

        segments.push(VariableSegment {
            range: pred_seg.range,
            data: reconstructed,
            semantic_type: pred_seg.semantic_type,
        });
    }

    VariablePart { segments }
}

/// Predict a single variable segment (encode path helper).
fn predict_segment(
    segment: &VariableSegment,
    params: &mut PredictorParameters,
) -> Result<Vec<u8>, EncodeError> {
    // 1. Predict
    let prediction = predict_next(segment.semantic_type, segment.data.len(), params);

    // 2. Update
    update_state(segment.semantic_type, &segment.data, params);

    Ok(prediction)
}

// ============================================================================
// Helper functions for number parsing/formatting
// ============================================================================

/// Parse ASCII decimal number to i64.
fn parse_ascii_number(bytes: &[u8]) -> i64 {
    let s = String::from_utf8_lossy(bytes);
    s.trim().parse().unwrap_or(0)
}

/// Format i64 as ASCII decimal with padding.
fn format_ascii_number(value: i64, width: usize) -> Vec<u8> {
    let s = format!("{:0>width$}", value.max(0), width = width);
    s.into_bytes()[..width].to_vec()
}

/// Parse binary little-endian to i64.
fn parse_binary_le(bytes: &[u8]) -> i64 {
    let mut result: i64 = 0;
    for (i, &b) in bytes.iter().enumerate().take(8) {
        result |= (b as i64) << (i * 8);
    }
    result
}

/// Encode i64 as binary little-endian.
fn encode_binary_le(value: i64, width: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(width);
    let mut v = value;
    for _ in 0..width {
        result.push((v & 0xFF) as u8);
        v >>= 8;
    }
    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spe_predict_empty() {
        let variable = VariablePart { segments: vec![] };
        let result = spe_predict(&variable).unwrap();
        assert!(result.predicted.segments.is_empty());
        assert_eq!(result.state.version, VERSION_ID);
    }

    #[test]
    fn test_spe_determinism() {
        let variable = VariablePart {
            segments: vec![VariableSegment {
                range: ByteRange { start: 0, end: 4 },
                data: vec![0x01, 0x02, 0x03, 0x04],
                semantic_type: SemanticType::Counter,
            }],
        };

        let result1 = spe_predict(&variable).unwrap();
        let result2 = spe_predict(&variable).unwrap();

        assert_eq!(result1.predicted.segments[0].data, result2.predicted.segments[0].data);
    }

    #[test]
    fn test_counter_prediction() {
        let mut params = PredictorParameters::default();

        // First prediction should be 0 (formatted to width 3)
        // logic: empty state -> 0
        let pred1 = predict_next(SemanticType::Counter, 3, &params);
        assert_eq!(pred1, b"000"); // 0 padded to width 3

        // Update with observed "100"
        update_state(SemanticType::Counter, b"100", &mut params);

        // Next prediction: last (100) + delta (default 1) = 101
        let pred2 = predict_next(SemanticType::Counter, 3, &params);
        assert_eq!(pred2, b"101");
        
        // Update with "101"
        update_state(SemanticType::Counter, b"101", &mut params);
        
        // Next prediction: last (101) + delta (101-100=1) = 102
        let pred3 = predict_next(SemanticType::Counter, 3, &params);
        assert_eq!(pred3, b"102");
    }

    #[test]
    fn test_identifier_prediction_zeros() {
        let params = PredictorParameters::default();
        let pred = predict_next(SemanticType::Identifier, 4, &params);
        assert_eq!(pred, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_parse_binary_le() {
        assert_eq!(parse_binary_le(&[0x01, 0x00, 0x00, 0x00]), 1);
        assert_eq!(parse_binary_le(&[0x00, 0x01, 0x00, 0x00]), 256);
        assert_eq!(parse_binary_le(&[0xFF, 0x00]), 255);
    }

    #[test]
    fn test_encode_binary_le() {
        assert_eq!(encode_binary_le(1, 4), vec![0x01, 0x00, 0x00, 0x00]);
        assert_eq!(encode_binary_le(256, 4), vec![0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_reconstruct_variable() {
        let predicted = VariablePart {
            segments: vec![VariableSegment {
                range: ByteRange { start: 0, end: 4 },
                data: vec![0x00, 0x00, 0x00, 0x00],
                semantic_type: SemanticType::Opaque,
            }],
        };
        let residual = vec![vec![0xAA, 0xBB, 0xCC, 0xDD]];

        let reconstructed = reconstruct_variable(&predicted, &residual);

        // XOR with zeros should give original residual
        assert_eq!(reconstructed.segments[0].data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }
}
