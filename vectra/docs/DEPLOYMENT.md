# VECTRA Deployment Guide

## Quick Start

### Rust (Recommended)

```bash
cd vectra
cargo build --release
cargo test
```

### Python

```bash
cd python
pip install -e .
pytest tests/
```

### C++

```bash
cd cpp
mkdir build && cd build
cmake ..
make
```

**Note**: C++ implementation is incomplete (40%). See `cpp/STATUS.md` for details.

---

## Production Deployment

### Requirements

- **Rust**: 1.70+ (edition 2021)
- **Python**: 3.10+ (if using Python bindings)
- **C++**: C++20 compiler (if using C++ bindings)
- **OpenSSL**: For C++ bindings (SHA-256)

### Configuration

#### Environment Variables

```bash
# Optional: Override entropy threshold (default: 4.0)
export VECTRA_H_MAX=5.0

# Optional: Override max payload size (default: 100MB)
export VECTRA_MAX_PAYLOAD_SIZE=104857600

# Optional: Override max pattern length (default: 1024)
export VECTRA_MAX_PATTERN_LEN=1024
```

#### Rust Configuration

```rust
use vectra::{Payload, vectra_encode, H_MAX};

// Custom entropy threshold
let result = vectra_encode(payload);
```

#### Python Configuration

```python
from vectra import encode, H_MAX

# H_MAX is module-level constant
# Modify in core/ebta.py if needed
result = encode(payload)
```

---

## Docker Deployment

### Dockerfile (Rust)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY vectra/ ./vectra/
WORKDIR /app/vectra
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/vectra/target/release/libvectra.so /usr/local/lib/
COPY --from=builder /app/vectra/target/release/vectra /usr/local/bin/
```

### Docker Compose

```yaml
version: '3.8'
services:
  vectra-api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - VECTRA_H_MAX=4.0
      - VECTRA_MAX_PAYLOAD_SIZE=104857600
    volumes:
      - ./data:/app/data
```

---

## Performance Tuning

### Benchmarking

```bash
# Rust benchmarks
cd vectra
cargo bench

# Results saved to target/criterion/
```

### Expected Performance

- **Encoding**: 10-100 MB/s (depends on payload structure)
- **Decoding**: 50-200 MB/s
- **Compression Ratio**: 1.5x - 10x (structured data)

### Optimization Tips

1. **Payload Size**: Keep under 10 MB for best performance
2. **Pattern Length**: Shorter patterns (4-16 bytes) compress better
3. **Structure**: More repeating patterns = better compression
4. **Entropy**: Lower entropy residuals = better compression

---

## Security Considerations

### Input Validation

- Maximum payload size: 100 MB (configurable)
- Maximum pattern length: 1024 bytes (configurable)
- DoS protection: O(n²) algorithm has input limits

### Integrity Verification

- All artifacts include SHA-256 hashes
- Version locking prevents incompatible decodes
- Tamper detection on decode

### Privacy

- **VECTRA is compression, not encryption**
- Artifacts contain payload hashes (may leak information)
- For sensitive data, add encryption layer

---

## Monitoring

### Metrics to Track

1. **Encoding Success Rate**: % of payloads that encode vs pass-through
2. **Compression Ratio**: Average artifact size / original size
3. **Encoding Latency**: P50, P95, P99
4. **Decoding Latency**: P50, P95, P99
5. **Error Rate**: Fail-open frequency, decode failures

### Logging

```rust
#[cfg(feature = "debug-logging")]
eprintln!("VECTRA encode failed: {:?}", error);
```

Enable debug logging in development only.

---

## Troubleshooting

### Common Issues

#### 1. "Payload size exceeds maximum"

**Solution**: Increase `MAX_PAYLOAD_SIZE` or split payload

#### 2. "EBTA validation failed"

**Cause**: Residual entropy too high (H > H_MAX)

**Solution**: 
- Increase H_MAX (less conservative)
- Improve NSGE prediction (lower residual entropy)
- Accept pass-through (original payload returned)

#### 3. "Version mismatch"

**Cause**: Artifact created with different library version

**Solution**: Use matching library version or migrate artifacts

#### 4. "Integrity check failed"

**Cause**: Artifact tampered with or corrupted

**Solution**: Verify artifact source, check transmission errors

---

## Migration Guide

### Version Upgrades

1. **Check VERSION_ID**: Ensure compatibility
2. **Test Decode**: Verify old artifacts still decode
3. **Gradual Rollout**: Deploy new version alongside old
4. **Monitor**: Watch for decode failures

### Artifact Migration

If VERSION_ID changes, old artifacts become undecodable. Migration strategy:

1. **Decode Before Upgrade**: Decode all artifacts with old version
2. **Re-encode**: Encode with new version
3. **Verify**: Ensure losslessness maintained

---

## Examples

### Rust Example

```rust
use vectra::{Payload, vectra_encode, vectra_decode};

let data = b"HEADER:value1:HEADER:value2".to_vec();
let payload = Payload::new(data);

let result = vectra_encode(payload);
match result {
    EncodeResult::Encoded(artifact) => {
        let decoded = vectra_decode(&artifact)?;
        assert_eq!(decoded.as_bytes(), &data);
    }
    EncodeResult::PassThrough(original) => {
        // Encoding not beneficial, use original
    }
}
```

### Python Example

```python
from vectra import encode, decode

data = b"HEADER:value1:HEADER:value2"
result = encode(data)

if isinstance(result, Artifact):
    decoded = decode(result)
    assert decoded == data
else:
    # Pass-through, use original
    pass
```

---

## Support

- **Documentation**: See `docs/ARCHITECTURE.md`
- **Issues**: GitHub Issues
- **Status**: See `cpp/STATUS.md` for C++ implementation status

---

**Last Updated**: 2025-01-27










