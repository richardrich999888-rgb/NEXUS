# VECTRA Architecture

**Version:** 1.0  
**Classification:** Product Architecture Document

---

## Asset Layers

VECTRA is structured into three distinct layers with clear boundaries:

```
┌─────────────────────────────────────────────────┐
│           VECTRA-Acceleration (Commercial)       │
│   Parallel/SIMD • Heuristics • Performance       │
├─────────────────────────────────────────────────┤
│           VECTRA-Formats (Commercial)            │
│   Binary Schema • Versioning • Compatibility     │
├─────────────────────────────────────────────────┤
│           VECTRA-Core (Reference)                │
│   Encode/Decode • Invariants • Determinism       │
└─────────────────────────────────────────────────┘
```

---

## Layer A: VECTRA-Core (Reference Implementation)

**Status:** Open / Reference  
**License:** Research use permitted; commercial requires license

### Components
| Module | Purpose |
|--------|---------|
| `encode.rs` | Compression pipeline orchestration |
| `decode.rs` | Reconstruction pipeline |
| `decompose.rs` | Structural pattern detection |
| `fee.rs` | Fractal Entropy Encoding |
| `spe.rs` | Symbolic Predictor Engine |
| `ebta.rs` | Entropy-Bounded validation |
| `integrity.rs` | Hash verification |

### Guarantees
- Deterministic: Same input → identical output
- Lossless: `decode(encode(x)) == x`
- Testable: Full test suite included

### Limitations
- O(n²) pattern detection
- Not optimized for speed
- Research-grade implementation

---

## Layer B: VECTRA-Formats (Commercial)

**Status:** Commercial / Licensed  
**Value:** Stability, compatibility, trust

### Components
| Component | Purpose |
|-----------|---------|
| Artifact Schema | Binary format specification |
| Version Protocol | Backward compatibility rules |
| Magic Headers | Format identification (VCTR/PASS) |
| Integrity Metadata | SHA-256 hashes, version stamps |

### Guarantees
- v1.x artifacts decode forever
- Version mismatch detection
- No silent corruption

### Commercial Value
- Long-term support contracts
- Certified format compliance
- Migration tooling

---

## Layer C: VECTRA-Acceleration (Commercial)

**Status:** Commercial / Premium  
**Value:** Performance, scale, enterprise needs

### Potential Components (Not Yet Implemented)
| Component | Purpose |
|-----------|---------|
| Parallel Pattern Detection | Multi-threaded decomposition |
| SIMD Encoding | Vectorized byte operations |
| Adaptive Heuristics | Skip unlikely candidates |
| Caching Layer | Memoize recurring patterns |

### Commercial Value
- 10-100x performance improvement
- Enterprise throughput requirements
- Premium licensing tier

---

## Boundary Rules

### Core → Formats
- Core produces artifacts conforming to Format spec
- Format changes require Core compatibility

### Formats → Acceleration
- Acceleration must produce Format-compliant artifacts
- Performance optimizations must preserve invariants

### Never Mix Layers
- Core logic must not depend on Acceleration
- Format spec must not assume performance characteristics
- Commercial features must not break open/reference behavior

---

## IP Mapping

| Layer | IP Type | Protection |
|-------|---------|------------|
| Core | Trade Secret + Patent | Algorithm design |
| Formats | Copyright + Standard | Schema ownership |
| Acceleration | Patent | Performance techniques |

---

## Deployment Models

### Model 1: Open Core
```
Core: Open source (reference)
Formats: Licensed (commercial)
Acceleration: Licensed (premium)
```

### Model 2: Full Commercial
```
All layers: Licensed
Reference implementation: Separate
```

### Model 3: Vertical Integration
```
Core + Formats: Bundled for specific industry
Acceleration: Optional add-on
```

---

*This architecture document defines the asset structure for VECTRA*
