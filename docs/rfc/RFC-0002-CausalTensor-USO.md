# RFC-0002: Causal Tensor & Universal State Object (USO)

| Field | Value |
|-------|-------|
| RFC | 0002 |
| Title | Causal Tensor & Universal State Object |
| Status | Draft |
| Created | 2024-12 |
| Author | Katta Naga Sri Ganesh |
| Organization | SYNTRIASS Labs Private Limited |

---

## Abstract

This RFC specifies two complementary primitives:

1. **Causal Tensor** — A signed, causally-ordered data structure for distributed consensus
2. **Universal State Object (USO)** — A unified abstraction for all state (replaces databases, caches, queues, files)

Together, these enable conflict-free distributed state management with cryptographic integrity.

## Part I: Causal Tensor

### 1.1 Motivation

Distributed systems need to track causality (what-happened-before) and merge concurrent updates deterministically. Causal Tensors provide:

- **Causal ordering** via vector clocks
- **Cryptographic integrity** via Ed25519 signatures
- **Content addressing** via BLAKE3 hashes
- **Deterministic merge** via algebraic properties

### 1.2 Structure

```rust
pub struct CausalTensor {
    pub id: CausalId,
    pub data: Vec<u8>,
    pub provenance: Provenance,
    pub clock: VectorClock,
    pub signature: Vec<u8>,
    pub metadata: TensorMetadata,
}
```

### 1.3 Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-CT-001 | ID Derivation | `id = BLAKE3(data, provenance, clock)` |
| INV-CT-002 | Signature Valid | Ed25519 signature over `(id, data, merkle_root)` |
| INV-CT-003 | Merge Idempotent | `merge(A, A) = A` |
| INV-CT-004 | Merge Commutative | `merge(A, B).data = merge(B, A).data` |
| INV-CT-005 | Merge Deterministic | Same inputs → same output |

### 1.4 Vector Clock

```rust
pub struct VectorClock {
    clocks: BTreeMap<u64, u64>,  // node_id → logical_time
}
```

Operations:
- `tick(node_id)` — Increment local clock
- `merge(other)` — Take max of each node's clock
- `happens_before(other)` — Check causal ordering
- `concurrent(other)` — Check if events are concurrent

### 1.5 Provenance

```rust
pub struct Provenance {
    pub parents: Vec<CausalId>,
    pub merkle_root: [u8; 32],
    pub depth: u64,
}
```

The Merkle root enables efficient ancestry verification.

### 1.6 Three-Way Merge Algorithm

```
function merge(local, remote):
    // 1. IDEMPOTENCE
    if local.id == remote.id:
        return local
    
    // 2. CAUSAL ORDERING
    if local.clock.happens_before(remote.clock):
        return remote  // Remote is newer
    if remote.clock.happens_before(local.clock):
        return local   // Local is newer
    
    // 3. CONCURRENT MERGE
    merged_clock = merge_clocks(local.clock, remote.clock)
    merged_data = deterministic_merge(local.data, remote.data)
    
    return new_tensor(merged_data, [local.id, remote.id], merged_clock)
```

## Part II: Universal State Object (USO)

### 2.1 Motivation

Modern applications use many state abstractions:
- Databases (relational, NoSQL)
- Caches (Redis, Memcached)
- Queues (Kafka, RabbitMQ)
- File systems
- Key-value stores

USO **unifies all state** into a single primitive with configurable sync behavior.

### 2.2 Structure

```rust
pub struct USO {
    pub id: ContentHash,
    pub data: Vec<u8>,
    pub schema: SchemaRef,
    pub history: CausalHistory,
    pub access: AccessPolicy,
    pub sync: SyncPolicy,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
}
```

### 2.3 Invariants

| ID | Invariant | Description |
|----|-----------|-------------|
| INV-USO-001 | Content ID | `id = BLAKE3(data)` |
| INV-USO-002 | Clock Monotonicity | Lamport ≥ all node clocks |
| INV-USO-003 | Owner Access | Owner always has read+write |
| INV-USO-004 | Merge Determinism | Same inputs → same result (LWW) |

### 2.4 Sync Policies

```rust
pub enum SyncPolicy {
    Global { max_latency_ms: u32 },  // Sync everywhere
    Regional { regions: Vec<Region> }, // Sync to specific regions
    OnDemand,                        // Pull-based (lazy)
    Local,                           // Single node only
}
```

### 2.5 Access Control

```rust
pub struct AccessPolicy {
    pub owner: PrincipalId,
    pub readers: Vec<PrincipalId>,
    pub writers: Vec<PrincipalId>,
    pub public_read: bool,
    pub public_write: bool,
}
```

### 2.6 Merge Semantics

USO uses **Last-Writer-Wins (LWW)** merge:

```
function merge(self, other):
    self.history.merge(other.history)
    
    if other.modified_at > self.modified_at:
        self.data = other.data
        self.id = other.id
        self.modified_at = other.modified_at
    
    self.history.parents.push(other.id)
```

## Security Considerations

### Cryptographic Primitives

| Purpose | Algorithm | Size |
|---------|-----------|------|
| Content hash | BLAKE3 | 32 bytes |
| Signatures | Ed25519 | 64 bytes |
| Merkle root | BLAKE3 | 32 bytes |

### Byzantine Fault Tolerance

Causal Tensors support BFT through:
1. Signed data — Tampering is detectable
2. Causality tracking — Fork detection
3. Deterministic merge — Convergence guaranteed

## Implementation

Reference implementations:
- Causal Tensor: `nexus-core/src/causal.rs`
- USO: `nexus-pcu/src/uso.rs`

## References

- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree)
- [Vector Clocks](https://en.wikipedia.org/wiki/Vector_clock)
- [CRDTs](https://crdt.tech/)
- [BLAKE3](https://github.com/BLAKE3-team/BLAKE3)

---

**Copyright © 2025 SYNTRIASS Labs Private Limited. All rights reserved.**  
**Inventor: Katta Naga Sri Ganesh**
