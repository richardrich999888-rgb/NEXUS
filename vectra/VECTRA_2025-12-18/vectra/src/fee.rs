//! Author / Inventor: Katta Naga Sri Ganesh
//! Organization: SYNTRIASS Labs Private Limited
//! Copyright © 2025 SYNTRIASS Labs Private Limited

//! FEE — Fractal Entropy Encoding
//!
//! Implements spec §4: Structural encoding via recursive generators.
//!
//! FEE encodes structure as:
//! - G: Base generator (the fundamental pattern)
//! - Φ: Set of recursive mapping functions
//!
//! Key insight: We encode the *generative process*, not instances.
//! This is fundamentally different from classical compression.

use crate::error::EncodeError;
use crate::types::{
    ByteRange, Generator, Mapping, MappingSet, MappingTransform, RepetitionSpec, Structure,
    StructureLevel,
};

/// Result of FEE encoding.
#[derive(Debug, Clone)]
pub struct FeeEncodeResult {
    pub generator: Generator,
    pub mappings: MappingSet,
}

/// Encode structural component using FEE.
///
/// # Algorithm (spec §4.1-4.2)
///
/// 1. Extract base generator from structure
/// 2. Identify self-similar patterns at multiple levels
/// 3. Build recursive mappings between levels
/// 4. Return (G, Φ) tuple
///
/// # Determinism
///
/// All operations are deterministic. No randomness.
pub fn fee_encode(structure: &Structure) -> Result<FeeEncodeResult, EncodeError> {
    if structure.levels.is_empty() {
        // Empty structure - return trivial generator
        return Ok(FeeEncodeResult {
            generator: Generator {
                base: vec![],
                repetition: RepetitionSpec { count: 0, stride: 0, start_offset: 0 },
            },
            mappings: MappingSet { mappings: vec![] },
        });
    }

    // Step 1: Extract base generator from the first (most frequent) pattern
    let generator = extract_base_generator(structure)?;

    // Step 2: Build recursive mappings
    let mappings = build_recursive_mappings(structure)?;

    Ok(FeeEncodeResult {
        generator,
        mappings,
    })
}

/// Extract base generator from structure.
///
/// The base generator captures the fundamental repeating unit.
fn extract_base_generator(structure: &Structure) -> Result<Generator, EncodeError> {
    // Find the most frequently occurring pattern
    // (patterns are already sorted by frequency in decompose)
    let base_level = structure
        .levels
        .first()
        .ok_or_else(|| EncodeError::Fee("No structural levels found".to_string()))?;

    let pattern_len = base_level.literals.len();
    
    // Filter byte_ranges to only include ranges that exactly match the base pattern length.
    // The structure.byte_ranges may contain merged ranges from multiple patterns,
    // but we only want ranges that correspond to the base pattern.
    let matching_ranges: Vec<_> = structure
        .byte_ranges
        .iter()
        .filter(|r| r.end - r.start == pattern_len)
        .cloned()
        .collect();

    // Calculate repetition spec from filtered byte ranges
    let repetition = calculate_repetition(&matching_ranges, pattern_len);

    Ok(Generator {
        base: base_level.literals.clone(),
        repetition,
    })
}

/// Calculate repetition specification from byte ranges.
fn calculate_repetition(ranges: &[ByteRange], pattern_len: usize) -> RepetitionSpec {
    if ranges.is_empty() {
        return RepetitionSpec { count: 0, stride: 0, start_offset: 0 };
    }

    if ranges.len() == 1 {
        return RepetitionSpec {
            count: 1,
            stride: pattern_len as u32,
            start_offset: ranges[0].start,
        };
    }

    // Calculate stride (distance between consecutive occurrences)
    let stride = if ranges.len() >= 2 {
        ranges[1].start.saturating_sub(ranges[0].start)
    } else {
        pattern_len
    };

    RepetitionSpec {
        count: ranges.len() as u32,
        stride: stride as u32,
        start_offset: ranges[0].start,
    }
}

/// Build recursive mappings between structural levels.
///
/// # Algorithm
///
/// For each level i > 0:
///   - Check if level i can be derived from level i-1
///   - If so, create mapping φ: level(i-1) → level(i)
///   - Mappings capture transformations (offset, concat, etc.)
fn build_recursive_mappings(structure: &Structure) -> Result<MappingSet, EncodeError> {
    let mut mappings = Vec::new();

    for i in 1..structure.levels.len() {
        let prev_level = &structure.levels[i - 1];
        let curr_level = &structure.levels[i];

        // Check for self-similarity between levels
        if let Some(transform) = find_transformation(prev_level, curr_level) {
            mappings.push(Mapping {
                from_level: i - 1,
                to_level: i,
                transform,
            });
        }
    }

    Ok(MappingSet { mappings })
}

/// Find transformation between two structural levels.
///
/// Returns None if no simple transformation exists.
fn find_transformation(from: &StructureLevel, to: &StructureLevel) -> Option<MappingTransform> {
    // Check for identity (exact match)
    if from.literals == to.literals {
        return Some(MappingTransform::Identity);
    }

    // Check for offset relationship
    if let Some(offset) = find_offset_relationship(&from.literals, &to.literals) {
        return Some(MappingTransform::Offset(offset));
    }

    // Check for concatenation
    if is_concatenation(&from.literals, &to.literals) {
        // Simple case: to is multiple copies of from
        let copies = to.literals.len() / from.literals.len();
        return Some(MappingTransform::Concat(vec![0; copies]));
    }

    None
}

/// Check if `to` is `from` with a byte offset applied.
fn find_offset_relationship(from: &[u8], to: &[u8]) -> Option<i32> {
    if from.len() != to.len() || from.is_empty() {
        return None;
    }

    // Calculate offset from first byte
    let offset = to[0] as i32 - from[0] as i32;

    // Verify all bytes have same offset
    for (f, t) in from.iter().zip(to.iter()) {
        let diff = *t as i32 - *f as i32;
        if diff != offset {
            return None;
        }
    }

    Some(offset)
}

/// Check if `to` is concatenation of `from` (multiple copies).
fn is_concatenation(from: &[u8], to: &[u8]) -> bool {
    if from.is_empty() || to.is_empty() {
        return false;
    }

    if to.len() % from.len() != 0 {
        return false;
    }

    let copies = to.len() / from.len();
    for i in 0..copies {
        let start = i * from.len();
        if &to[start..start + from.len()] != from {
            return false;
        }
    }

    true
}

/// Regenerate structure from generator and mappings.
///
/// This is the decode-time operation. Must be deterministic.
pub fn regenerate_structure(generator: &Generator, mappings: &MappingSet) -> Structure {
    let mut levels = Vec::new();
    let mut byte_ranges = Vec::new();

    // Generate base level from generator
    let base_level = StructureLevel {
        pattern_id: 0,
        children: vec![],
        literals: generator.base.clone(),
    };
    levels.push(base_level);

    // Generate byte ranges from repetition spec
    let pattern_len = generator.base.len();
    for i in 0..generator.repetition.count {
        let start = generator.repetition.start_offset + (i as usize * generator.repetition.stride as usize);
        byte_ranges.push(ByteRange {
            start,
            end: start + pattern_len,
        });
    }

    // Apply mappings to generate additional levels
    for mapping in &mappings.mappings {
        if mapping.from_level < levels.len() {
            let source = &levels[mapping.from_level];
            let derived = apply_mapping_transform(source, &mapping.transform);
            levels.push(derived);
        }
    }

    Structure {
        levels,
        byte_ranges,
    }
}

/// Apply a mapping transform to a structural level.
fn apply_mapping_transform(source: &StructureLevel, transform: &MappingTransform) -> StructureLevel {
    let literals = match transform {
        MappingTransform::Identity => source.literals.clone(),
        MappingTransform::Offset(offset) => {
            source
                .literals
                .iter()
                .map(|&b| (b as i32 + offset) as u8)
                .collect()
        }
        MappingTransform::Concat(indices) => {
            let mut result = Vec::new();
            for _ in indices {
                result.extend_from_slice(&source.literals);
            }
            result
        }
    };

    StructureLevel {
        pattern_id: source.pattern_id + 1,
        children: vec![],
        literals,
    }
}

/// Check if structure has self-similar properties.
///
/// Used to determine if FEE can provide compression benefit.
pub fn has_self_similarity(structure: &Structure) -> bool {
    if structure.levels.len() < 2 {
        return false;
    }

    // Check if any mapping transforms exist between levels
    for i in 1..structure.levels.len() {
        if find_transformation(&structure.levels[i - 1], &structure.levels[i]).is_some() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_encode_empty() {
        let structure = Structure {
            levels: vec![],
            byte_ranges: vec![],
        };
        let result = fee_encode(&structure).unwrap();
        assert!(result.generator.base.is_empty());
        assert!(result.mappings.mappings.is_empty());
    }

    #[test]
    fn test_fee_encode_single_level() {
        let structure = Structure {
            levels: vec![StructureLevel {
                pattern_id: 0,
                children: vec![],
                literals: vec![0xAA, 0xBB, 0xCC, 0xDD],
            }],
            byte_ranges: vec![
                ByteRange { start: 0, end: 4 },
                ByteRange { start: 10, end: 14 },
                ByteRange { start: 20, end: 24 },
            ],
        };

        let result = fee_encode(&structure).unwrap();

        assert_eq!(result.generator.base, vec![0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(result.generator.repetition.count, 3);
        assert_eq!(result.generator.repetition.stride, 10);
    }

    #[test]
    fn test_fee_determinism() {
        let structure = Structure {
            levels: vec![StructureLevel {
                pattern_id: 0,
                children: vec![],
                literals: vec![1, 2, 3, 4],
            }],
            byte_ranges: vec![ByteRange { start: 0, end: 4 }],
        };

        let result1 = fee_encode(&structure).unwrap();
        let result2 = fee_encode(&structure).unwrap();

        assert_eq!(result1.generator.base, result2.generator.base);
        assert_eq!(
            result1.generator.repetition.count,
            result2.generator.repetition.count
        );
    }

    #[test]
    fn test_regenerate_structure() {
        let generator = Generator {
            base: vec![0xAA, 0xBB],
            repetition: RepetitionSpec { count: 3, stride: 4, start_offset: 0 },
        };
        let mappings = MappingSet { mappings: vec![] };

        let structure = regenerate_structure(&generator, &mappings);

        assert_eq!(structure.levels.len(), 1);
        assert_eq!(structure.byte_ranges.len(), 3);
        assert_eq!(structure.byte_ranges[0], ByteRange { start: 0, end: 2 });
        assert_eq!(structure.byte_ranges[1], ByteRange { start: 4, end: 6 });
        assert_eq!(structure.byte_ranges[2], ByteRange { start: 8, end: 10 });
    }

    #[test]
    fn test_offset_relationship() {
        let from = vec![0x10, 0x20, 0x30];
        let to = vec![0x11, 0x21, 0x31];

        assert_eq!(find_offset_relationship(&from, &to), Some(1));
    }

    #[test]
    fn test_concatenation_detection() {
        let from = vec![0xAA, 0xBB];
        let to = vec![0xAA, 0xBB, 0xAA, 0xBB, 0xAA, 0xBB];

        assert!(is_concatenation(&from, &to));
    }

    #[test]
    fn test_no_concatenation() {
        let from = vec![0xAA, 0xBB];
        let to = vec![0xAA, 0xBB, 0xCC, 0xDD];

        assert!(!is_concatenation(&from, &to));
    }
}
