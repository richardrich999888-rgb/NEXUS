//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! EBTA — Entropy-Bounded Tensor Algebra
//!
//! Implements spec §6: Entropy constraint enforcement.
//!
//! EBTA is the safety gate. It validates:
//! - H(Δ) ≤ H_MAX (Shannon entropy bound)
//!
//! If validation fails, encoding MUST NOT proceed.
//! This enforces the fail-open invariant.
//!
//! Key property: EBTA makes NO transformation.
//! It is a pure decision function: Δ → Boolean.

use crate::error::EncodeError;
use crate::types::{Residual, ResidualSegment, H_MAX};

/// Result of EBTA validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EbtaResult {
    /// Whether residual passed validation
    pub valid: bool,
    /// Computed entropy (bits)
    pub entropy: f64,
    /// Maximum allowed entropy (bits)
    pub max_entropy: f64,
}

/// Validate residual against entropy bounds.
///
/// # Algorithm (spec §6.1-6.2)
///
/// 1. Compute Shannon entropy H(Δ)
/// 2. Compare against H_MAX
/// 3. Return validation result
///
/// # Decision Rule (spec §6.2)
///
/// ```text
/// E(D) = A   if H(Δ) ≤ H_MAX
/// E(D) = D   otherwise
/// ```
///
/// This is a HARD gate. No soft thresholds. No retries.
pub fn ebta_validate(residual: &Residual) -> EbtaResult {
    let entropy = compute_residual_entropy(residual);

    EbtaResult {
        valid: entropy <= H_MAX,
        entropy,
        max_entropy: H_MAX,
    }
}

/// Validate with custom entropy threshold.
///
/// Used for testing and deployment-specific tuning.
pub fn ebta_validate_with_threshold(residual: &Residual, h_max: f64) -> EbtaResult {
    let entropy = compute_residual_entropy(residual);

    EbtaResult {
        valid: entropy <= h_max,
        entropy,
        max_entropy: h_max,
    }
}

/// Compute Shannon entropy of residual.
///
/// H(X) = -Σ p(x) log₂ p(x)
///
/// where p(x) is the probability of byte value x.
fn compute_residual_entropy(residual: &Residual) -> f64 {
    // Collect all residual bytes
    let mut all_bytes = Vec::new();
    for segment in &residual.segments {
        all_bytes.extend_from_slice(&segment.delta);
    }

    if all_bytes.is_empty() {
        return 0.0;
    }

    compute_byte_entropy(&all_bytes)
}

/// Compute Shannon entropy of byte sequence.
///
/// # Algorithm
///
/// 1. Count frequency of each byte value (0-255)
/// 2. Compute probability p(x) = count(x) / total
/// 3. Compute H = -Σ p(x) log₂ p(x)
///
/// # Properties
///
/// - H = 0 for constant sequence (all bytes same)
/// - H = 8 for uniform random bytes (maximum)
/// - Higher H means less predictable (worse for compression)
pub fn compute_byte_entropy(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }

    // Count byte frequencies
    let mut counts = [0u64; 256];
    for &b in bytes {
        counts[b as usize] += 1;
    }

    let total = bytes.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Check if residual is compressible (low entropy).
///
/// A residual with entropy significantly below H_MAX
/// indicates good prediction quality.
pub fn is_highly_compressible(residual: &Residual) -> bool {
    let entropy = compute_residual_entropy(residual);
    entropy <= H_MAX / 2.0
}

/// Compute compression potential from entropy.
///
/// Returns estimated bits per byte after ideal compression.
/// Lower is better.
pub fn compression_potential(residual: &Residual) -> f64 {
    compute_residual_entropy(residual)
}

/// Compute residual from actual and predicted values.
///
/// Δ = V - V_hat (using XOR for byte-level difference)
///
/// XOR is used because:
/// - It's reversible: V = V_hat XOR Δ
/// - It's deterministic
/// - It preserves byte boundaries
pub fn compute_residual(
    actual: &[u8],
    predicted: &[u8],
    range: crate::types::ByteRange,
    semantic_type: crate::types::SemanticType,
) -> Result<ResidualSegment, EncodeError> {
    if actual.len() != predicted.len() {
        return Err(EncodeError::Ebta {
            entropy: f64::INFINITY,
            max: H_MAX,
        });
    }

    let delta: Vec<u8> = actual.iter().zip(predicted.iter()).map(|(a, p)| a ^ p).collect();

    Ok(ResidualSegment {
        range,
        delta,
        semantic_type,
    })
}

/// Reconstruct actual value from prediction and residual.
///
/// V = V_hat XOR Δ
pub fn apply_residual(predicted: &[u8], delta: &[u8]) -> Vec<u8> {
    predicted.iter().zip(delta.iter()).map(|(p, d)| p ^ d).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ByteRange;

    #[test]
    fn test_entropy_constant_sequence() {
        // All same bytes - entropy should be 0
        let bytes = vec![0xAA; 100];
        let entropy = compute_byte_entropy(&bytes);
        assert!((entropy - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_two_values() {
        // Two values, equal probability - entropy should be 1
        let bytes: Vec<u8> = (0..100).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
        let entropy = compute_byte_entropy(&bytes);
        assert!((entropy - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_uniform_random() {
        // Approximate uniform distribution - entropy should approach 8
        let bytes: Vec<u8> = (0..=255).collect();
        let entropy = compute_byte_entropy(&bytes);
        assert!((entropy - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_ebta_validate_low_entropy() {
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 100 },
                delta: vec![0x00; 100], // All zeros - entropy 0
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        let result = ebta_validate(&residual);
        assert!(result.valid);
        assert!(result.entropy < 0.001);
    }

    #[test]
    fn test_ebta_validate_high_entropy() {
        // Create high-entropy residual (uniform distribution)
        let delta: Vec<u8> = (0..=255).cycle().take(1024).collect();
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 1024 },
                delta,
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        let result = ebta_validate(&residual);
        // Default H_MAX is 4.0, uniform has ~8.0
        assert!(!result.valid);
        assert!(result.entropy > 7.9);
    }

    #[test]
    fn test_compute_residual_xor() {
        let actual = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let predicted = vec![0x00, 0x00, 0x00, 0x00];
        let range = ByteRange { start: 0, end: 4 };

        let segment = compute_residual(&actual, &predicted, range, crate::types::SemanticType::Opaque).unwrap();

        // XOR with zeros should give original
        assert_eq!(segment.delta, actual);
    }

    #[test]
    fn test_apply_residual_roundtrip() {
        let original = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let predicted = vec![0xAA, 0xBB, 0xCC, 0xDD];

        // Compute residual
        let delta: Vec<u8> = original.iter().zip(predicted.iter()).map(|(a, p)| a ^ p).collect();

        // Reconstruct
        let reconstructed = apply_residual(&predicted, &delta);

        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_ebta_determinism() {
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 50 },
                delta: vec![0x01, 0x02, 0x03, 0x04, 0x05].repeat(10),
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        let result1 = ebta_validate(&residual);
        let result2 = ebta_validate(&residual);

        assert_eq!(result1.entropy, result2.entropy);
        assert_eq!(result1.valid, result2.valid);
    }

    #[test]
    fn test_empty_residual() {
        let residual = Residual { segments: vec![] };
        let result = ebta_validate(&residual);

        assert!(result.valid);
        assert_eq!(result.entropy, 0.0);
    }

    #[test]
    fn test_custom_threshold() {
        let delta: Vec<u8> = (0..16).collect(); // 4 bits of entropy
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 16 },
                delta,
                semantic_type: crate::types::SemanticType::Opaque,
            }],
        };

        // With high threshold - should pass
        let result_high = ebta_validate_with_threshold(&residual, 8.0);
        assert!(result_high.valid);

        // With low threshold - should fail
        let result_low = ebta_validate_with_threshold(&residual, 1.0);
        assert!(!result_low.valid);
    }
}
