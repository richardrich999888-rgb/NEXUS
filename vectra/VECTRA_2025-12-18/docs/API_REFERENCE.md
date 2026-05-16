# VECTRA API Reference

## Rust API

### Core Types

#### `Payload`

Raw input bytes for encoding.

```rust
pub struct Payload {
    // ...
}

impl Payload {
    pub fn new(data: Vec<u8>) -> Self;
    pub fn with_schema(data: Vec<u8>, schema_id: SchemaId) -> Self;
    pub fn as_bytes(&self) -> &[u8];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

#### `Artifact`

Encoded output containing all reconstruction information.

```rust
pub struct Artifact {
    pub generator: Generator,
    pub mappings: MappingSet,
    pub predictor_state: PredictorState,
    pub residual: Residual,
    pub constraints: ReconstructionConstraints,
    pub integrity: IntegrityMeta,
}
```

#### `EncodeResult`

Result of encoding operation.

```rust
pub enum EncodeResult {
    Encoded(Artifact),
    PassThrough(Payload),
}
```

### Core Functions

#### `vectra_encode`

Encode a payload using VECTRA.

```rust
pub fn vectra_encode(payload: Payload) -> EncodeResult
```

**Returns**:
- `EncodeResult::Encoded(artifact)` if encoding succeeds
- `EncodeResult::PassThrough(payload)` if encoding cannot be safely performed

**Guarantees**:
- Determinism: same input → same output
- Losslessness: `decode(encode(D)) == D`
- Fail-open: uncertainty → return original

#### `vectra_decode`

Decode an artifact back to the original payload.

```rust
pub fn vectra_decode(artifact: &Artifact) -> VectraResult<Payload>
```

**Returns**: `Ok(Payload)` if decode succeeds, `Err(VectraError)` otherwise

**Guarantees**:
- Determinism: same artifact → same payload
- Losslessness: output matches original exactly

#### `can_decode`

Check if this library can decode an artifact.

```rust
pub fn can_decode(artifact: &Artifact) -> bool
```

**Returns**: `true` if artifact version matches library version

### Utility Functions

#### `compute_byte_entropy`

Compute Shannon entropy of byte sequence.

```rust
pub fn compute_byte_entropy(bytes: &[u8]) -> f64
```

**Returns**: Entropy in bits (0.0 to 8.0)

#### `estimate_artifact_size`

Estimate artifact size in bytes.

```rust
pub fn estimate_artifact_size(artifact: &Artifact) -> usize
```

#### `compression_ratio`

Compute compression ratio.

```rust
pub fn compression_ratio(original_size: usize, artifact: &Artifact) -> f64
```

**Returns**: Ratio (>1.0 indicates compression benefit)

#### `is_encoding_beneficial`

Check if encoding provides size benefit.

```rust
pub fn is_encoding_beneficial(payload: &Payload, artifact: &Artifact) -> bool
```

### Constants

```rust
pub const VERSION_ID: u64 = 0x0001_0000_0000_0001;
pub const H_MAX: f64 = 4.0;
pub const MAX_PAYLOAD_SIZE: usize = 100 * 1024 * 1024;
pub const MAX_PATTERN_LEN: usize = 1024;
```

### Error Types

#### `VectraError`

Top-level error type.

```rust
pub enum VectraError {
    DecompositionFailed { reason: String },
    FeeEncodingFailed { reason: String },
    NsgePredictionFailed { reason: String },
    EbtaValidationFailed { entropy: f64, max: f64 },
    ArtifactConstructionFailed { reason: String },
    Artifact(ArtifactError),
    DecodeFailed { reason: String },
    IntegrityFailed { reason: String },
    InvalidInput { reason: String },
    InternalError { reason: String },
}
```

---

## Python API

### Core Functions

#### `encode`

Encode a payload using VECTRA.

```python
def encode(payload: bytes) -> Union[Artifact, bytes]
```

**Returns**:
- `Artifact` if encoding succeeds
- `bytes` (original payload) if encoding cannot be proven safe

#### `decode`

Decode an artifact back to the original payload.

```python
def decode(artifact: Artifact) -> bytes
```

**Raises**: `ValueError` if artifact integrity check fails

#### `encode_with_diagnostics`

Encode with detailed diagnostics.

```python
def encode_with_diagnostics(payload: bytes) -> dict
```

**Returns**: Dictionary with encoding details:
- `result`: Artifact or original payload
- `decomposition`: DecompositionResult details
- `fee`: FEEResult details
- `validation`: ValidationResult details
- `encoded`: True if artifact was produced

#### `decode_with_diagnostics`

Decode with detailed diagnostics.

```python
def decode_with_diagnostics(artifact: Artifact) -> dict
```

**Returns**: Dictionary with reconstruction details:
- `integrity_verified`: bool
- `reconstruction_success`: bool
- `payload_hash_match`: bool
- `result`: Decoded payload or None
- `error`: Error message or None

### Types

#### `Artifact`

```python
@dataclass(frozen=True)
class Artifact:
    inventor: str
    organization: str
    version: str
    generator: str  # hex-encoded
    structure_mappings: Tuple[Tuple[int, str], ...]
    structure_hash: str
    variable_segments: Tuple[Tuple[int, str], ...]
    total_segments: int
    delimiter: str  # hex-encoded
    original_hash: str
    artifact_hash: str
```

### Constants

```python
H_MAX: float = 5.0  # Entropy threshold
```

---

## C++ API

### Core Functions

#### `vectra_encode`

```cpp
[[nodiscard]] EncodeResult vectra_encode(Payload payload);
```

**Status**: ⚠️ Partially implemented

#### `vectra_decode`

```cpp
[[nodiscard]] Result<Payload> vectra_decode(const Artifact& artifact);
```

**Status**: ❌ Not implemented

#### `can_decode`

```cpp
[[nodiscard]] bool can_decode(const Artifact& artifact) noexcept;
```

**Status**: ✅ Implemented

### Utility Functions

#### `sha256`

```cpp
[[nodiscard]] Hash256 sha256(const std::vector<uint8_t>& data);
```

**Status**: ✅ Implemented

#### `compute_byte_entropy`

```cpp
[[nodiscard]] double compute_byte_entropy(const std::vector<uint8_t>& data);
```

**Status**: ❌ Not implemented

#### `estimate_artifact_size`

```cpp
[[nodiscard]] size_t estimate_artifact_size(const Artifact& artifact);
```

**Status**: ✅ Implemented

#### `compression_ratio`

```cpp
[[nodiscard]] double compression_ratio(size_t original_size, const Artifact& artifact);
```

**Status**: ✅ Implemented

---

## Error Handling

### Rust

All functions return `Result<T, VectraError>` or `EncodeResult` (which handles fail-open).

### Python

Functions raise `ValueError` or `TypeError` on errors.

### C++

Functions return `Result<T, Error>` using `std::expected` (C++23) or custom error types.

---

## Examples

See `docs/DEPLOYMENT.md` for complete examples.

---

**Last Updated**: 2025-01-27








