# NEXUS Protocol Specification v1.0

This document formally specifies the core protocols and data structures of the NEXUS distributed execution substrate.

---

## 1. Core Primitives

### 1.1 ContentHash
A 32-byte SHA-256 hash used for content-addressing all data objects.
```
ContentHash := SHA256(data)
```

### 1.2 PrincipalId
A 32-byte identifier representing a cryptographic identity (Ed25519 public key).
```
PrincipalId := Ed25519.PublicKey
```

### 1.3 Timestamp
A 64-bit unsigned integer representing milliseconds since Unix epoch.

---

## 2. Portable Computation Unit (PCU)

A PCU is the fundamental unit of portable computation.

### Structure
| Field | Type | Description |
|-------|------|-------------|
| `id` | ContentHash | Hash of the PCU definition |
| `code` | WasmModule | WASM bytecode to execute |
| `inputs` | Vec<ContentHash> | References to USO inputs |
| `identity` | IdentityContext | Embedded identity and capabilities |

### Invariants
- `id = SHA256(code || inputs || identity)`
- PCU execution is deterministic: `execute(PCU, state) -> Result`

---

## 3. Universal State Object (USO)

A USO is the fundamental unit of portable state.

### Structure
| Field | Type | Description |
|-------|------|-------------|
| `id` | ContentHash | Hash of the current data |
| `data` | Vec<u8> | Raw state bytes |
| `history` | StateHistory | Causal lineage |

### StateHistory
| Field | Type | Description |
|-------|------|-------------|
| `created_at` | Timestamp | Creation time |
| `created_by` | PrincipalId | Creator identity |
| `vector_clock` | VersionVector | Causal version |
| `ancestors` | Vec<ContentHash> | Previous state hashes |

---

## 4. VersionVector

A vector clock for causal ordering.

### Structure
```
VersionVector := Map<NodeId, u64>
```

### Operations
- **Increment**: `vv[node_id] += 1`
- **Merge**: `∀k: merged[k] = max(vv1[k], vv2[k])`
- **Compare**: `vv1 ≤ vv2 iff ∀k: vv1[k] ≤ vv2[k]`

---

## 5. CausalOp

An operation in the Causal DAG.

### Structure
| Field | Type | Description |
|-------|------|-------------|
| `id` | ContentHash | Hash of the operation |
| `op_type` | String | Operation type (e.g., "uso_update") |
| `payload` | JSON | Operation-specific data |
| `deps` | BTreeSet<ContentHash> | Causal dependencies |
| `version` | VersionVector | Causal version at creation |
| `author` | NodeId | Authoring node |
| `signature` | Ed25519.Signature | Signature over canonical form |

### Canonical Form (for signing)
```
canonical(op) = CBOR.encode({
    op_type, payload, deps, version, author
})
```

---

## 6. HierarchicalSync Protocol

The sync protocol uses a 3-tier hierarchy for efficient state propagation.

### Tiers
1. **Local Cluster**: Full replication with Raft consensus.
2. **Regional**: Anti-entropy gossip with Merkle tree diffs.
3. **Global**: Snapshot-based sync with causal cuts.

### Sync Request
```json
{
  "my_version": VersionVector,
  "tier": "regional",
  "max_ops": 1000
}
```

### Sync Response
```json
{
  "your_missing_ops": [CausalOp, ...],
  "my_version": VersionVector
}
```

---

## 7. IdentityContext

Embedded authorization proof within a PCU.

### Structure
| Field | Type | Description |
|-------|------|-------------|
| `principal` | PrincipalId | Requesting identity |
| `capabilities` | CapabilitySet | Permitted actions |
| `delegation` | Option<DelegationChain> | Delegation proof |
| `valid_until` | Timestamp | Expiry time |
| `signature` | Ed25519.Signature | Proof of ownership |

### Signature Verification
```
data = principal || bincode(capabilities, delegation, valid_until)
verify(principal.as_verifying_key(), data, signature)
```

---

## 8. Conflict Resolution Policies

| Policy | Description |
|--------|-------------|
| `LastWriterWins` | Highest timestamp wins |
| `FirstWriterWins` | Lowest timestamp wins |
| `Semantic` | Use CRDT merge semantics |
| `Manual` | Return conflict for user resolution |

---

© 2025 SYNTRIASS Labs Pvt Ltd.  
Patent Pending: IN202501XXXXX
