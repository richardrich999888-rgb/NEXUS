# NEXUS Core Contract Specification

## Status: FROZEN

This document defines the **immutable contracts** for NEXUS core types.
Any change that violates these contracts is a **breaking protocol change**.

> **Inventor**: Katta Naga Sri Ganesh  
> **Copyright**: © 2025 SYNTRIASS Labs Private Limited  
> **Document Version**: 1.0.0

---

## Contract-Frozen Types

### 1. PCU (Portable Computation Unit)

**Location**: `nexus-pcu/src/pcu.rs`

#### Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-PCU-001 | ID Determinism | Same (code, inputs, params, principal) → same ID |
| INV-PCU-002 | Code Hash Integrity | `code.hash == BLAKE3(code.bytecode)` |
| INV-PCU-003 | Content Addressing | All input hashes are 32-byte BLAKE3 hashes |
| INV-PCU-004 | Principal Format | Identity principal is exactly 32 bytes |
| INV-PCU-005 | Serialization Lossless | `PCU::from_bytes(pcu.to_bytes()) == pcu` |

#### Frozen Fields
- `id: ContentHash`
- `code: WasmModule`
- `inputs: Vec<ContentHash>`
- `parameters: Vec<u8>`
- `identity: IdentityContext`

#### May Evolve
- `constraints: ExecutionConstraints` (new fields allowed, not removed)
- `result: Option<PCUResult>` (structure may extend)

---

### 2. USO (Universal State Object)

**Location**: `nexus-pcu/src/uso.rs`

#### Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-USO-001 | Content ID | `uso.id == BLAKE3(uso.data)` |
| INV-USO-002 | Clock Monotonicity | Lamport ≥ all node clocks |
| INV-USO-003 | Owner Access | Owner always has read+write |
| INV-USO-004 | Merge Determinism | Same inputs → same merge result (LWW) |

#### Frozen Fields
- `id: ContentHash`
- `data: Vec<u8>`
- `history: CausalHistory`
- `access: AccessPolicy`

#### May Evolve
- `schema: SchemaRef`
- `sync: SyncPolicy`
- Additional metadata fields

---

### 3. CausalTensor

**Location**: `nexus-core/src/causal.rs`

#### Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-CT-001 | ID Derivation | ID derived from (data, provenance, clock) |
| INV-CT-002 | Signature Valid | Ed25519 signature covers (id, data, merkle_root) |
| INV-CT-003 | Merge Idempotent | `merge(A, A) == A` |
| INV-CT-004 | Merge Commutative | `merge(A, B).data == merge(B, A).data` |
| INV-CT-005 | Merge Deterministic | Same inputs → same output |

#### Frozen Fields
- `id: CausalId`
- `data: Vec<u8>`
- `provenance: Provenance`
- `clock: VectorClock`
- `signature: Vec<u8>`

---

### 4. ExecutionResult

**Location**: `nexus-executor/src/types.rs`

#### Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-ER-001 | Output Hash | `output_hash == BLAKE3(output)` |
| INV-ER-002 | Non-negative Metrics | `fuel_consumed >= 0`, `peak_memory >= 0` |

#### Frozen Fields
- `output: Vec<u8>`
- `output_hash: ContentHash`
- `fuel_consumed: u64`

#### May Evolve
- Additional performance metrics
- Trace/debug information

---

## Verification

### Property Tests

All invariants are verified by property-based tests in:
- `nexus-pcu/src/invariants.rs`
- `nexus-pcu/tests/property_tests.rs`
- `nexus-core/src/causal.rs` (test module)

### Running Tests

```bash
# All invariant tests
cargo test -p nexus-pcu invariants

# All property tests
cargo test -p nexus-pcu --lib

# Causal tensor tests
cargo test -p nexus-core causal
```

---

## Change Policy

| Change Type | Allowed? | Process |
|-------------|----------|---------|
| Add optional field | ✅ Yes | Document in "May Evolve" |
| Remove field | ❌ No | Protocol version bump required |
| Change field type | ❌ No | Protocol version bump required |
| Add new invariant | ✅ Yes | Must pass for existing data |
| Weaken invariant | ❌ No | Protocol version bump required |
