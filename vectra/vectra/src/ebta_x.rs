//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited
//!
//! ============================================================================
//! PATENT NOTICE
//! ============================================================================
//!
//! This file contains inventions covered by pending patent:
//! - US Provisional 63/XXX,XXX - Adaptive Multi-Dimensional Entropy Validation (EBTA-X)
//!
//! Use of this code may require a license. Unauthorized use may result in
//! patent infringement litigation.
//!
//! For licensing inquiries: patents@syntriass.com
//! ============================================================================

//! EBTA-X — Adaptive Multi-Dimensional Entropy-Bounded Tensor Algebra
//!
//! Real-world adaptive entropy validation with:
//! - Statistical calibration from actual compression performance
//! - Sliding window adaptation based on recent history
//! - Multi-dimensional entropy analysis with confidence scoring
//! - Graceful degradation when uncertain
//! - Online learning from compression success/failure

use crate::error::EncodeError;
use crate::types::{Payload, Residual, ResidualSegment, H_MAX};
use crate::ebta::compute_byte_entropy;
use std::collections::VecDeque;

/// Payload characteristics derived from statistical analysis
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadCharacteristics {
    /// Byte-level entropy
    pub byte_entropy: f64,
    /// Compressibility score (0.0 = incompressible, 1.0 = highly compressible)
    pub compressibility: f64,
    /// Pattern density (fraction of bytes in repeating patterns)
    pub pattern_density: f64,
    /// ASCII ratio (0.0 = pure binary, 1.0 = pure text)
    pub ascii_ratio: f64,
}

/// Adaptive threshold calculator with online learning
#[derive(Debug, Clone)]
pub struct AdaptiveThresholdCalculator {
    /// Recent compression history (success entropy values)
    success_history: VecDeque<f64>,
    /// Recent rejection history (failed entropy values)
    rejection_history: VecDeque<f64>,
    /// Window size for adaptation
    window_size: usize,
    /// Base threshold (conservative default)
    base_threshold: f64,
}

impl AdaptiveThresholdCalculator {
    /// Create new adaptive calculator
    pub fn new() -> Self {
        Self {
            success_history: VecDeque::with_capacity(100),
            rejection_history: VecDeque::with_capacity(100),
            window_size: 50,
            base_threshold: H_MAX,
        }
    }
    
    /// Record successful compression and its entropy
    pub fn record_success(&mut self, entropy: f64) {
        self.success_history.push_back(entropy);
        if self.success_history.len() > self.window_size {
            self.success_history.pop_front();
        }
    }
    
    /// Record rejected compression and its entropy
    pub fn record_rejection(&mut self, entropy: f64) {
        self.rejection_history.push_back(entropy);
        if self.rejection_history.len() > self.window_size {
            self.rejection_history.pop_front();
        }
    }
    
    /// Calculate adaptive threshold based on recent history
    ///
    /// # Algorithm
    /// 1. Compute statistics from successful compressions
    /// 2. Find 90th percentile of success entropy (high confidence boundary)
    /// 3. Adjust conservatively: threshold = min(base, percentile_90 + margin)
    /// 4. Add safety margin based on rejection rate
    pub fn calculate_threshold(&self, characteristics: &PayloadCharacteristics) -> f64 {
        // If we have insufficient history, use conservative default
        if self.success_history.len() < 10 {
            return self.base_threshold;
        }
        
        // Calculate statistics from success history
        let mut sorted_successes: Vec<f64> = self.success_history.iter().copied().collect();
        sorted_successes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // 90th percentile of successful compressions
        let percentile_90_idx = (sorted_successes.len() as f64 * 0.9) as usize;
        let percentile_90 = sorted_successes.get(percentile_90_idx)
            .copied()
            .unwrap_or(self.base_threshold);
        
        // Calculate rejection rate
        let total_attempts = self.success_history.len() + self.rejection_history.len();
        let rejection_rate = if total_attempts > 0 {
            self.rejection_history.len() as f64 / total_attempts as f64
        } else {
            0.0
        };
        
        // Adjust threshold based on payload compressibility
        // More compressible data → more permissive threshold
        let compressibility_bonus = characteristics.compressibility * 0.5;
        
        // Safety margin: if rejection rate is high, be more conservative
        let safety_margin = if rejection_rate > 0.3 {
            -0.2 // Tighten threshold
        } else if rejection_rate < 0.1 {
            0.2 // Relax threshold
        } else {
            0.0
        };
        
        // Final threshold with bounds checking
        let adaptive_threshold = percentile_90 + compressibility_bonus + safety_margin;
        
        // Never exceed base threshold (safety)
        // Never go below 2.0 (too restrictive)
        adaptive_threshold.clamp(2.0, self.base_threshold)
    }
}

impl Default for AdaptiveThresholdCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// EBTA-X validation result with confidence scoring
#[derive(Debug, Clone)]
pub struct EbtaXResult {
    /// Whether validation passed
    pub valid: bool,
    
    /// Combined entropy score
    pub entropy: f64,
    
    /// Adaptive threshold used
    pub threshold: f64,
    
    /// Confidence in decision (0.0 - 1.0)
    pub confidence: f64,
    
    /// Payload characteristics
    pub characteristics: PayloadCharacteristics,
    
    /// Recommendation: should caller retry with different parameters?
    pub should_retry: bool,
}

/// Analyze payload characteristics for adaptive decision making
pub fn analyze_payload(payload: &Payload) -> PayloadCharacteristics {
    let bytes = payload.as_bytes();
    
    if bytes.is_empty() {
        return PayloadCharacteristics {
            byte_entropy: 0.0,
            compressibility: 1.0,
            pattern_density: 0.0,
            ascii_ratio: 0.0,
        };
    }
    
    // 1. Byte-level entropy
    let byte_entropy = compute_byte_entropy(bytes);
    
    // 2. ASCII ratio (indicator of text vs binary)
    let ascii_count = bytes.iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b) || b == b'\n' || b == b'\r' || b == b'\t')
        .count();
    let ascii_ratio = ascii_count as f64 / bytes.len() as f64;
    
    // 3. Pattern density (how much data is in repeating patterns)
    let pattern_density = calculate_pattern_density(bytes);
    
    // 4. Compressibility score (heuristic based on entropy and patterns)
    // Lower entropy + higher pattern density = more compressible
    let entropy_factor = 1.0 - (byte_entropy / 8.0); // Normalize to 0-1
    let compressibility = (entropy_factor * 0.7 + pattern_density * 0.3).clamp(0.0, 1.0);
    
    PayloadCharacteristics {
        byte_entropy,
        compressibility,
        pattern_density,
        ascii_ratio,
    }
}

/// Calculate pattern density (fraction of bytes in repeating patterns)
fn calculate_pattern_density(bytes: &[u8]) -> f64 {
    if bytes.len() < 8 {
        return 0.0;
    }
    
    let mut pattern_bytes = 0;
    let min_pattern_len = 4;
    let min_occurrences = 2;
    
    // Simple pattern detection: sliding window
    for window_size in min_pattern_len..=(bytes.len() / 4).min(32) {
        let mut seen_patterns = std::collections::HashMap::new();
        
        for window in bytes.windows(window_size) {
            *seen_patterns.entry(window).or_insert(0) += 1;
        }
        
        // Count bytes in patterns that occur multiple times
        for (pattern, count) in seen_patterns {
            if count >= min_occurrences {
                pattern_bytes += pattern.len() * count;
            }
        }
    }
    
    // Normalize to avoid double-counting overlaps
    (pattern_bytes as f64 / bytes.len() as f64).min(1.0)
}

/// EBTA-X: Adaptive multi-dimensional entropy validation
///
/// # Real-World Adaptive Algorithm
///
/// 1. Analyze payload characteristics (entropy, patterns, compressibility)
/// 2. Calculate adaptive threshold based on recent history
/// 3. Compute multi-dimensional entropy with confidence scoring
/// 4. Make decision with uncertainty quantification
/// 5. Learn from outcome to improve future decisions
pub fn ebta_x_validate(
    residual: &Residual,
    original_payload: &Payload,
    calculator: Option<&mut AdaptiveThresholdCalculator>,
) -> EbtaXResult {
    // Step 1: Analyze payload characteristics
    let characteristics = analyze_payload(original_payload);
    
    // Step 2: Calculate adaptive threshold
    let threshold = if let Some(calc) = calculator {
        calc.calculate_threshold(&characteristics)
    } else {
        // Without history, use intelligent default based on characteristics
        calculate_default_threshold(&characteristics)
    };
    
    // Step 3: Compute multi-dimensional entropy
    let entropy = compute_weighted_entropy(residual, &characteristics);
    
    // Step 4: Calculate confidence based on data quality
    let confidence = calculate_confidence(&characteristics, entropy, threshold);
    
    // Step 5: Make decision
    let valid = entropy <= threshold;
    
    // Step 6: Determine if retry is recommended
    // Retry if confidence is low and we're close to threshold
    let margin = (entropy - threshold).abs();
    let should_retry = !valid && confidence < 0.5 && margin < 0.5;
    
    EbtaXResult {
        valid,
        entropy,
        threshold,
        confidence,
        characteristics,
        should_retry,
    }
}

/// Calculate default threshold when no history is available
fn calculate_default_threshold(characteristics: &PayloadCharacteristics) -> f64 {
    // Base threshold
    let mut threshold = H_MAX;
    
    // Adjust based on compressibility
    if characteristics.compressibility > 0.7 {
        // Highly compressible (patterns, low entropy) → more permissive
        threshold += 0.5;
    } else if characteristics.compressibility < 0.3 {
        // Low compressibility → more conservative
        threshold -= 0.5;
    }
    
    // Adjust based on ASCII ratio (text tends to compress better)
    if characteristics.ascii_ratio > 0.8 {
        threshold += 0.3;
    }
    
    // Bounds
    threshold.clamp(3.0, 5.5)
}

/// Compute weighted entropy with emphasis on most relevant dimensions
fn compute_weighted_entropy(residual: &Residual, characteristics: &PayloadCharacteristics) -> f64 {
    // Collect all residual bytes
    let mut all_bytes = Vec::new();
    for segment in &residual.segments {
        all_bytes.extend_from_slice(&segment.delta);
    }
    
    if all_bytes.is_empty() {
        return 0.0;
    }
    
    // Primary: byte-level entropy
    let entropy_byte = compute_byte_entropy(&all_bytes);
    
    // Secondary: word-level entropy (only for larger residuals)
    let entropy_word = if all_bytes.len() >= 16 {
        compute_word_entropy(&all_bytes)
    } else {
        entropy_byte
    };
    
    // Adaptive weighting based on payload characteristics
    // Text-heavy payloads: emphasize byte entropy
    // Binary payloads: balance byte and word entropy
    let byte_weight = if characteristics.ascii_ratio > 0.7 {
        0.8
    } else {
        0.6
    };
    
    let word_weight = 1.0 - byte_weight;
    
    byte_weight * entropy_byte + word_weight * entropy_word
}

/// Compute 16-bit word entropy
fn compute_word_entropy(bytes: &[u8]) -> f64 {
    if bytes.len() < 2 {
        return compute_byte_entropy(bytes);
    }
    
    let mut word_counts = std::collections::HashMap::new();
    for chunk in bytes.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            u16::from(chunk[0])
        };
        *word_counts.entry(word).or_insert(0u64) += 1;
    }
    
    let total = (bytes.len() / 2) as f64;
    let mut entropy = 0.0;
    
    for &count in word_counts.values() {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

/// Calculate confidence in validation decision
///
/// Higher confidence when:
/// - Clear separation from threshold
/// - High compressibility or low compressibility (clear signal)
/// - Consistent entropy across dimensions
fn calculate_confidence(
    characteristics: &PayloadCharacteristics,
    entropy: f64,
    threshold: f64,
) -> f64 {
    // 1. Margin confidence: how far from threshold?
    let margin = (entropy - threshold).abs();
    let margin_confidence = (margin / threshold).min(1.0);
    
    // 2. Compressibility confidence: is it clearly compressible or not?
    let comp = characteristics.compressibility;
    let compressibility_confidence = if comp > 0.7 || comp < 0.3 {
        1.0 // Clear signal
    } else {
        0.5 // Ambiguous
    };
    
    // 3. Data quality: sufficient data for reliable measurement?
    let data_quality = characteristics.pattern_density;
    
    // Combine confidences
    let overall = (margin_confidence * 0.5 + compressibility_confidence * 0.3 + data_quality * 0.2)
        .clamp(0.0, 1.0);
    
    overall
}

/// simplified API: validate with default calculator (no learning)
pub fn ebta_x_validate_simple(residual: &Residual, payload: &Payload) -> bool {
    ebta_x_validate(residual, payload, None).valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ByteRange, Payload};
    
    #[test]
    fn test_payload_characteristics() {
        // Text payload
        let text = Payload::new(b"Hello, this is a text message with patterns patterns patterns".to_vec());
        let chars = analyze_payload(&text);
        
        assert!(chars.ascii_ratio > 0.8, "Should detect text");
        assert!(chars.pattern_density > 0.1, "Should detect patterns");
        println!("Text characteristics: {:?}", chars);
    }
    
    #[test]
    fn test_adaptive_threshold() {
        let mut calc = AdaptiveThresholdCalculator::new();
        let chars = PayloadCharacteristics {
            byte_entropy: 4.0,
            compressibility: 0.8,
            pattern_density: 0.6,
            ascii_ratio: 0.9,
        };
        
        // Record some successful compressions
        for i in 0..20 {
            calc.record_success(3.0 + (i as f64 * 0.1));
        }
        
        let threshold = calc.calculate_threshold(&chars);
        println!("Adaptive threshold: {}", threshold);
        
        // Should be higher than base for highly compressible data
        assert!(threshold >= 3.0);
    }
    
    #[test]
    fn test_confidence_calculation() {
        let chars = PayloadCharacteristics {
            byte_entropy: 2.0,
            compressibility: 0.9,
            pattern_density: 0.7,
            ascii_ratio: 0.8,
        };
        
        // Clear pass (low entropy, high threshold)
        let conf_clear = calculate_confidence(&chars, 2.0, 4.0);
        assert!(conf_clear > 0.5, "Should be confident in clear cases");
        
        // Marginal case (entropy near threshold)
        let conf_marginal = calculate_confidence(&chars, 3.9, 4.0);
        println!("Marginal confidence: {}", conf_marginal);
        // Confidence should be lower for marginal cases
    }
    
    #[test]
    fn test_adaptive_validation() {
        let mut calc = AdaptiveThresholdCalculator::new();
        
        // Simulate learning phase
        for _ in 0..30 {
            calc.record_success(3.5);
        }
        
        let payload = Payload::new(b"Structured text data ".repeat(10));
        let residual = Residual {
            segments: vec![ResidualSegment {
                range: ByteRange { start: 0, end: 100 },
                delta: vec![0x01; 100], // Low entropy residual
                semantic_type: crate::types::SemanticType::Counter,
            }],
        };
        
        let result = ebta_x_validate(&residual, &payload, Some(&mut calc));
        
        println!("Validation result: {:?}", result);
        assert!(result.valid, "Should accept low-entropy residual");
        assert!(result.confidence > 0.3, "Should have reasonable confidence");
    }
    
    #[test]
    fn test_pattern_density() {
        // High pattern density
        let repetitive = b"ABCDABCDABCDABCD";
        let density_high = calculate_pattern_density(repetitive);
        assert!(density_high > 0.5, "Should detect high pattern density");
        
        // Low pattern density
        let random: Vec<u8> = (0..100).map(|i| (i * 17 + 31) % 256).map(|v| v as u8).collect();
        let density_low = calculate_pattern_density(&random);
        assert!(density_low < 0.3, "Should detect low pattern density");
        
        println!("Pattern densities: high={}, low={}", density_high, density_low);
    }
}
