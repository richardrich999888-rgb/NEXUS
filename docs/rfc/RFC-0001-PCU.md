# RFC-0001: Portable Computation Unit (PCU)

| Field | Value |
|-------|-------|
| RFC | 0001 |
| Title | Portable Computation Unit (PCU) |
| Status | Draft |
| Created | 2024-12 |
| Author | Katta Naga Sri Ganesh |
| Organization | SYNTRIASS Labs Private Limited |

---

## Abstract

This RFC specifies the **Portable Computation Unit (PCU)**, a fundamental primitive that enables deterministic, content-addressed, and identity-aware computation in distributed systems. PCUs carry their own execution context, route to where data lives, and generate cryptographic proofs of execution.

## Motivation

Traditional distributed computation suffers from:

1. **Data movement overhead** — Code expects data to come to it
2. **Non-determinism** — Same inputs can produce different outputs
3. **Weak identity** — No cryptographic binding between code and executor
4. **No proof of work** — Results cannot be verified without re-execution

PCUs solve these by inverting the computation model: **code travels to data**, with full identity and capability tracking.

## Specification

### 1. PCU Structure

```rust
pub struct PCU {
    pub id: ContentHash,           // Deterministic identifier
    pub code: WasmModule,          // Executable WASM bytecode
    pub inputs: Vec<ContentHash>,  // Content-addressed inputs
    pub parameters: Vec<u8>,       // Execution parameters
    pub identity: IdentityContext, // Who is executing
    pub constraints: ExecutionConstraints,
    pub created_at: Timestamp,
    pub result: Option<PCUResult>,
}
```

### 2. Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-PCU-001 | ID Determinism | `id = HASH(code.hash, inputs, parameters, identity.principal)` |
| INV-PCU-002 | Code Integrity | `code.hash == BLAKE3(code.bytecode)` |
| INV-PCU-003 | Content Addressing | All hashes are 32-byte BLAKE3 |
| INV-PCU-004 | Principal Format | Identity principal is exactly 32 bytes |
| INV-PCU-005 | Serialization Lossless | `PCU::from_bytes(pcu.to_bytes()) == pcu` |

### 3. ID Computation

The PCU ID is computed deterministically:

```
PCU_ID = BLAKE3(
    code.hash ||
    inputs[0] || inputs[1] || ... ||
    parameters ||
    identity.principal
)
```

This ensures:
- Same inputs → same ID (determinism)
- Any change → different ID (integrity)
- ID is globally unique (content addressing)

### 4. Identity Context

```rust
pub struct IdentityContext {
    pub principal: PrincipalId,    // 32-byte Ed25519 public key
    pub capabilities: CapabilitySet,
    pub delegation: Option<DelegationChain>,
    pub valid_until: Timestamp,
    pub signature: Vec<u8>,        // Ed25519 signature
}
```

### 5. Execution Constraints

```rust
pub struct ExecutionConstraints {
    pub max_fuel: u64,             // Maximum instructions
    pub max_memory_bytes: u64,     // Maximum memory
    pub max_time_ms: u64,          // Wall-clock timeout
    pub required_capabilities: Vec<String>,
}
```

### 6. Execution Result

```rust
pub struct PCUResult {
    pub output: ContentHash,       // Hash of output data
    pub fuel_consumed: u64,
    pub execution_time_ms: u64,
    pub proof: ExecutionProof,
}
```

## Routing Semantics

PCUs implement **code-to-data routing**:

1. PCU specifies input content hashes
2. Network locates nodes with required data
3. PCU routes to optimal node (has most inputs, lowest latency)
4. Execution happens at data location
5. Only results travel back

This eliminates data transfer for computation.

## Security Considerations

### Cryptographic Primitives

| Purpose | Algorithm | Size |
|---------|-----------|------|
| Content hash | BLAKE3 | 32 bytes |
| Signatures | Ed25519 | 64 bytes |
| Principal ID | Ed25519 pubkey | 32 bytes |

### Capability Model

PCUs execute with explicitly granted capabilities:
- `compute` — Execute WASM code
- `read:<hash>` — Read specific content
- `write:<hash>` — Write to specific location
- `network` — Make network calls

### Future: Post-Quantum

When ML-DSA stabilizes, PCUs will support hybrid signatures (Ed25519 + ML-DSA-65) for quantum resistance.

## Implementation

Reference implementation: `nexus-pcu/src/pcu.rs`

## References

- [BLAKE3 Hash Function](https://github.com/BLAKE3-team/BLAKE3)
- [Ed25519 Digital Signatures](https://ed25519.cr.yp.to/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)

---

**Copyright © 2025 SYNTRIASS Labs Private Limited. All rights reserved.**  
**Inventor: Katta Naga Sri Ganesh**
