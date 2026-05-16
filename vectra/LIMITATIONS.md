# VECTRA Limitations

**Version:** 0.1.0  
**Status:** Research-grade implementation

---

## Technical Facts

### Compression Algorithm

- **Type:** Deterministic, lossless, structure-aware compression
- **Invariant:** `decode(encode(x)) == x` for all supported inputs
- **Determinism:** Guaranteed — same input produces identical output

### Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Pattern detection | O(n²) | Quadratic substring search |
| Encoding | O(n² + m) | n = input size, m = patterns found |
| Decoding | O(n) | Linear reconstruction |
| Serialization | O(n) | Bincode binary format |

### Known Limitations

1. **Pattern detection is O(n²)**
   - Large files (>100KB) may be slow
   - Iteration limit (500K) prevents infinite loops
   - This is a research implementation, not production-optimized

2. **No streaming support**
   - Entire payload must fit in memory
   - Not suitable for multi-GB files

3. **Small files may expand**
   - Artifact overhead can exceed input size for small inputs
   - Fail-open behavior returns original if encoding not beneficial

4. **Single-level pattern matching**
   - Only one level of structural hierarchy detected
   - No recursive fractal patterns (yet)

5. **Binary format only**
   - Uses bincode serialization
   - Not human-readable

---

## What VECTRA Is

- A working proof-of-concept compression engine
- Deterministic and lossless
- Suitable for research and experimentation

## What VECTRA Is NOT

- A replacement for production compressors (gzip, zstd)
- Optimized for speed or compression ratio
- Feature-complete

---

## Test Coverage

| Test Suite | Status |
|------------|--------|
| Unit tests (71) | ✅ Pass |
| Edge case fuzz | ✅ Pass |
| Integration tests | ✅ Pass |
| CLI round-trip | ✅ Pass |

---

## Responsible Use

- Do not use for production data without additional validation
- Performance may vary significantly based on input characteristics
- This is research-grade software

---

*Last updated: 2025-12-18*
