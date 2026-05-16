# NEXUS Post-Quantum Cryptography Status

## Current Status: CLASSICAL-ONLY

> **Version**: 1.0.0  
> **Last Updated**: December 2024  
> **Status**: Ed25519 (classical) implemented; ML-DSA (PQC) awaiting ecosystem stabilization

---

## Implementation Summary

| Component | Classical (Ed25519) | Post-Quantum (ML-DSA-65) |
|-----------|---------------------|--------------------------|
| Key Generation | ✅ Implemented | ⏳ Pending |
| Signing | ✅ Implemented | ⏳ Pending |
| Verification | ✅ Implemented | ⏳ Pending |
| Hybrid Mode | ✅ Types Ready | ⏳ Pending |

## PQC Feature Gate

The `pqc` feature in `nexus-pcu` enables PQC-ready types but does NOT yet provide actual post-quantum signatures.

```toml
# Cargo.toml
[dependencies]
nexus-pcu = { version = "0.1", features = ["pqc"] }
```

When enabled:
- `HybridSignature` type supports both classical and PQC signatures
- `PublicKeyBundle` reserves space for PQC public keys
- Actual PQC operations return placeholders or errors

## Blocking Issues

### 1. ml-dsa Crate Stabilization

The `ml-dsa` crate (NIST FIPS 204 implementation) requires `rand_core 0.9`:

```
ml-dsa v0.1.0-rc.2 requires rand_core ^0.9
nexus workspace uses rand 0.8 (rand_core 0.6)
```

**Resolution**: Wait for either:
- `ml-dsa` to support `rand_core 0.6`, OR
- Upgrade entire workspace to `rand 0.9` when stable

### 2. ml-kem Crate Compatibility

Similar issue with `ml-kem` (FIPS 203 for key encapsulation).

## Security Guarantees

### Current (Classical)

| Property | Guarantee |
|----------|-----------|
| Key Size | 32 bytes (Ed25519) |
| Signature Size | 64 bytes |
| Security Level | 128-bit classical |
| Quantum Resistance | ❌ None |

### Future (Hybrid)

| Property | Guarantee |
|----------|-----------|
| Classical Key | 32 bytes (Ed25519) |
| PQC Key | ~1,952 bytes (ML-DSA-65) |
| Classical Signature | 64 bytes |
| PQC Signature | ~3,293 bytes |
| Security Level | 128-bit classical + NIST Level 3 PQC |
| Quantum Resistance | ✅ Yes (defense-in-depth) |

## Migration Path

### Phase 1: Current (2024-2025)
- Classical-only signatures
- PQC-ready types in place
- No additional dependencies

### Phase 2: Transition (2025-2026)
- Enable hybrid signatures when `ml-dsa` stabilizes
- Both classical and PQC signatures computed
- EITHER signature validates (defense-in-depth)

### Phase 3: PQC-Primary (2027+)
- PQC becomes primary signature
- Classical maintained for backward compatibility
- Gradual deprecation plan

## Relevant Files

| File | Purpose |
|------|---------|
| `nexus-pcu/src/pqc.rs` | Hybrid signature types |
| `nexus-pcu/src/crypto.rs` | Classical crypto utilities |
| `nexus-pcu/Cargo.toml` | Feature flags |

## Testing PQC-Readiness

```bash
# Run all crypto tests
cargo test -p nexus-pcu pqc
cargo test -p nexus-pcu crypto

# Check PQC feature compiles
cargo check -p nexus-pcu --features pqc
```

## Contact

For PQC integration questions, contact:
- **Inventor**: Katta Naga Sri Ganesh
- **Organization**: SYNTRIASS Labs Private Limited
