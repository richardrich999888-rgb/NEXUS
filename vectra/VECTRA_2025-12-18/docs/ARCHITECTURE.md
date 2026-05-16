# VECTRA Architecture Documentation

## High-Level Design (HLD)

### System Overview

VECTRA is a deterministic, lossless data compression system designed for structured payloads. It operates transparently beneath existing protocols and produces self-describing artifacts that guarantee exact reconstruction or safe pass-through.

### Core Principles

1. **Determinism**: Same input + same version → identical output
2. **Losslessness**: `decode(encode(D)) == D` always
3. **Fail-Open**: Uncertainty → return original payload unchanged
4. **Self-Describing**: Artifacts contain all reconstruction information

### Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                    │
│  (Python API, C++ API, Rust API)                       │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                  Encoding Pipeline                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │Decompose │→ │   FEE   │→ │   NSGE   │→ │   EBTA   ││
│  │   D→S,V  │  │  S→G,Φ  │  │  V→V̂,Θ  │  │  H(Δ)≤H  ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
│                                                          │
│  ┌────────────────────────────────────────────────────┐│
│  │           Artifact Construction (TDF)               ││
│  │  A = { G, Φ, Θ, Δ, C, I }                         ││
│  └────────────────────────────────────────────────────┘│
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                  Decoding Pipeline                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │Integrity │→ │Regenerate│→ │Reconstruct│→ │Recompose ││
│  │  Verify  │  │Structure │  │ Variable │  │  S+V→D   ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
└──────────────────────────────────────────────────────────┘
```

### Data Flow

#### Encoding Path: D → A ∪ D

1. **Input**: Payload D (raw bytes)
2. **Decomposition**: D → (S, V)
   - S: Structural components (stable patterns)
   - V: Variable components (time-evolving data)
3. **FEE Encoding**: S → (G, Φ)
   - G: Generator (base pattern)
   - Φ: Mappings (recursive transformations)
4. **NSGE Prediction**: V → (V̂, Θ)
   - V̂: Predicted variable component
   - Θ: Predictor state
5. **Residual Computation**: Δ = V ⊕ V̂ (XOR)
6. **EBTA Validation**: H(Δ) ≤ H_MAX?
   - If valid: Build artifact A
   - If invalid: Return D (fail-open)
7. **Output**: Artifact A or original D

#### Decoding Path: A → D

1. **Input**: Artifact A
2. **Integrity Verification**: Check version, hashes
3. **Structure Regeneration**: (G, Φ) → S
4. **Variable Reconstruction**: (V̂, Θ, Δ) → V
   - Predict V̂ using Θ
   - Reconstruct: V = V̂ ⊕ Δ
5. **Recomposition**: (S, V) → D
6. **Verification**: Hash(D) == expected?
7. **Output**: Original payload D

---

## Low-Level Design (LLD)

### Module Structure

#### 1. Decomposition (`decompose.rs`)

**Purpose**: Separate payload into structural and variable components.

**Algorithm**:
- Pattern detection: O(n²) scan for repeating byte sequences
- Minimum pattern length: 4 bytes
- Minimum occurrences: 2
- Semantic type inference: Counter, Timestamp, Identifier, Metric, Opaque

**Security**:
- Maximum payload size: 100 MB (configurable)
- Maximum pattern length: 1024 bytes

**Key Functions**:
- `decompose(payload)` → `DecompositionResult`
- `recompose(structure, variable)` → `Payload`

#### 2. FEE - Fractal Entropy Encoding (`fee.rs`)

**Purpose**: Encode structural patterns as generators + mappings.

**Current Implementation (MVP)**:
- Common prefix extraction
- Suffix mappings per segment
- Transform types: Identity, Offset, Concat

**Future Enhancement**:
- Multi-level recursive patterns
- Advanced transforms (rotation, scaling)

**Key Functions**:
- `fee_encode(structure)` → `FeeEncodeResult`
- `regenerate_structure(generator, mappings)` → `Structure`

#### 3. NSGE - Neural-Symbolic Gradient Engine (`nsge.rs`)

**Purpose**: Predict variable components deterministically.

**Current Implementation**:
- Rule-based predictors per semantic type
- State management (counters, timestamps, metrics)
- Deterministic prediction logic

**Predictor Types**:
- Counter: `last + delta`
- Timestamp: `base + delta`
- Metric: `mean / 1000`
- Identifier/Opaque: Zero prediction

**Note**: Despite "Neural" in name, current implementation is symbolic/rule-based.

**Key Functions**:
- `nsge_predict(variable)` → `NsgePredictResult`
- `predict_next(semantic_type, width, params)` → `Vec<u8>`
- `reconstruct_variable(predicted, residual)` → `VariablePart`

#### 4. EBTA - Entropy-Bounded Tensor Algebra (`ebta.rs`)

**Purpose**: Validate residual entropy to ensure compression safety.

**Algorithm**:
- Compute Shannon entropy: H(X) = -Σ p(x) log₂ p(x)
- Compare against threshold: H_MAX = 4.0 bits
- Hard gate: H(Δ) ≤ H_MAX → proceed, else fail-open

**Key Functions**:
- `ebta_validate(residual)` → `EbtaResult`
- `compute_byte_entropy(bytes)` → `f64`
- `compute_residual(actual, predicted, range, type)` → `ResidualSegment`

#### 5. Artifact Construction (`artifact.rs`)

**Purpose**: Assemble self-describing artifact from components.

**Artifact Structure (TDF)**:
```rust
struct Artifact {
    generator: Generator,           // G: FEE generator
    mappings: MappingSet,           // Φ: FEE mappings
    predictor_state: PredictorState, // Θ: NSGE state
    residual: Residual,             // Δ: Bounded residual
    constraints: ReconstructionConstraints, // C: Output constraints
    integrity: IntegrityMeta,       // I: Verification metadata
}
```

**Key Functions**:
- `build_artifact(...)` → `Artifact`
- `estimate_artifact_size(artifact)` → `usize`
- `is_encoding_beneficial(payload, artifact)` → `bool`

#### 6. Integrity Verification (`integrity.rs`)

**Purpose**: Cryptographic verification of artifacts.

**Hash Functions**:
- SHA-256 for payload hash
- SHA-256 for artifact component hash
- Version-locked encoding

**Key Functions**:
- `verify_integrity(artifact)` → `Result<(), DecodeError>`
- `verify_reconstruction(payload, constraints)` → `Result<(), DecodeError>`
- `generate_integrity_metadata(...)` → `IntegrityMeta`

---

## Type System

### Core Types

- **Payload**: Raw input bytes D ∈ 𝒟
- **Artifact**: Encoded output A ∈ 𝒜
- **Structure**: Stable patterns S
- **VariablePart**: Time-evolving data V
- **Residual**: Prediction error Δ = V ⊕ V̂
- **Generator**: Base pattern G
- **MappingSet**: Transformations Φ
- **PredictorState**: Prediction parameters Θ

### Error Types

- **VectraError**: Top-level error enum
- **EncodeError**: Encoding-specific (triggers fail-open)
- **DecodeError**: Decoding-specific (aborts on failure)

---

## Security Considerations

### DoS Protection

- **Input Size Limits**: MAX_PAYLOAD_SIZE = 100 MB
- **Pattern Length Limits**: MAX_PATTERN_LEN = 1024 bytes
- **O(n²) Algorithm**: Acceptable for structured data, but monitor in production

### Integrity

- **Cryptographic Hashing**: SHA-256 for all integrity checks
- **Version Locking**: Artifacts are version-specific
- **Tamper Detection**: Artifact hash verification on decode

### Privacy

- **No Encryption**: VECTRA is compression, not encryption
- **Hash Leakage**: Payload hash in artifact may leak information
- **Recommendation**: Add optional encryption layer for sensitive data

---

## Performance Characteristics

### Time Complexity

- **Decomposition**: O(n²) worst case (pattern matching)
- **FEE Encoding**: O(n) for common prefix
- **NSGE Prediction**: O(n) per segment
- **EBTA Validation**: O(n) for entropy calculation
- **Overall**: O(n²) dominated by decomposition

### Space Complexity

- **Payload Storage**: O(n)
- **Residual Storage**: O(n)
- **Structure Storage**: O(k) where k = number of patterns

### Optimization Opportunities

1. **Suffix Trees/Arrays**: Reduce decomposition to O(n log n)
2. **Parallel Pattern Matching**: Multi-threaded decomposition
3. **Caching**: Cache common patterns across payloads

---

## Multi-Language Architecture

### Rust Core (`vectra/`)

- **Status**: 100% complete, production-ready
- **Role**: Core library with all algorithms
- **API**: Public functions in `lib.rs`

### Python Bindings (`python/`)

- **Status**: 80% complete (MVP implementation)
- **Current**: Parallel Python implementation
- **Future**: PyO3 bindings to Rust core (recommended)

### C++ Bindings (`cpp/`)

- **Status**: 40% complete (API defined, partial implementation)
- **Current**: Only `encode.cpp` partially implemented
- **Future**: Complete implementation or remove from public API

---

## Versioning & Compatibility

### Version Locking

- **VERSION_ID**: `0x0001_0000_0000_0001`
- **Artifact Version**: Embedded in integrity metadata
- **Decode Requirement**: Artifact version must match library version

### Breaking Changes

- Changing VERSION_ID breaks backward compatibility
- Old artifacts become undecodable (by design)
- Migration strategy needed for production deployments

---

## Testing Strategy

### Unit Tests

- Each module has comprehensive unit tests
- Test coverage: ~40% of codebase
- Focus on core invariants

### Integration Tests

- Losslessness: `decode(encode(D)) == D`
- Determinism: `encode(D) == encode(D)`
- Fail-open: High entropy → original unchanged

### Benchmarks

- Encoding throughput (MB/s)
- Decoding throughput (MB/s)
- Compression ratio
- Entropy calculation performance

### Future Testing

- Property-based testing (proptest available)
- Fuzz testing for edge cases
- Performance regression testing

---

## Deployment Considerations

### Build Requirements

- **Rust**: 1.70+ (edition 2021)
- **C++**: C++20 compiler (GCC 10+, Clang 12+)
- **Python**: 3.10+
- **OpenSSL**: For C++ bindings (SHA-256)

### Dependencies

- `sha2 = 0.10.8` (Rust)
- `serde = 1.0.193` (Rust)
- `thiserror = 1.0.50` (Rust)
- OpenSSL (C++)

### Configuration

- `H_MAX`: Entropy threshold (default: 4.0 bits)
- `MAX_PAYLOAD_SIZE`: Input size limit (default: 100 MB)
- `MAX_PATTERN_LEN`: Pattern length limit (default: 1024 bytes)

---

## Future Enhancements

1. **True Fractal FEE**: Multi-level recursive pattern detection
2. **Neural NSGE**: Actual ML-based predictors
3. **Schema Registry**: Schema-aware decomposition
4. **Performance Optimization**: Suffix trees for O(n log n) decomposition
5. **Cross-Language Integration**: PyO3 bindings, complete C++ implementation

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-27








