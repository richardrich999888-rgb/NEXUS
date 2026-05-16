# NEXUS Core Source & Components Audit

**Date:** 2025-01-29  
**Scope:** Workspace core crates (`nexus-core`, `nexus-pcu`, `nexus-sync`, `nexus-compress`, `nexus-storage`, `nexus-network`, `nexus-executor`, `nexus-observability`) and related components.

---

## 1. Workspace & Crate Inventory

### 1.1 Workspace Members (root `Cargo.toml`)

| Crate | Role |
|-------|------|
| `nexus-core` | Causal tensor algebra, IDs, tenancy, cost/migration (optional) |
| `nexus-pcu` | PCU, USO, identity, proof, routing, content hash |
| `nexus-sync` | CAUSALUX integration, sync engine, CRDT-USO adapters |
| `nexus-compress` | VECTRA integration, PCU/USO compression |
| `nexus-storage` | Provenance log, index, query, backup |
| `nexus-network` | Messages, transport (QUIC), gossip, sync, TLS |
| `nexus-executor` | WASM execution, semantic cache, proof, host API |
| `nexus-observability` | Metrics, health, logging, optional tracing |
| `nexus-runtime` | Runtime glue |
| `nexus-cli` | CLI (core, pcu, sync, network, storage) |
| `nexus-server` | Server (core, pcu, sync, causalux) |
| `nexus-edge` | Edge (pcu, causalux) |
| `nexus-agp` | AGP (pcu, causalux) |
| `nexus-benchmarks` | Benchmarks (pcu, causalux) |
| `nexus-secrets` | Secrets management |
| `causalux` | CRDT/DAG sync (version vectors, conflict, sync) |
| `vectra/vectra` | Deterministic compression |
| + homeostasis, multi-asi-immune, autonomic-system, developmental-gates, nervous-system, telos-protocol | Other protocol/engine crates |

**Not in workspace:** `nexus-core-v2` (see §3).

---

## 2. Core Crate Audits

### 2.1 nexus-core

**Path:** `nexus-core/src/`  
**Public modules:** `causal`, `cost_optimizer`, `crypto`, `error`, `tenancy`, `migration` (feature-gated).

| Module | Purpose | Status |
|--------|---------|--------|
| `causal` | `CausalId`, `CausalTensor`, `VectorClock`, `Provenance`, merge, serialization | ✅ Used by storage, network, pcu |
| `error` | `NexusError`, `Result` | ✅ Used everywhere |
| `crypto` | `generate_signing_key` | ✅ Used by pcu tests, network benches/tests, storage tests, core benches |
| `tenancy` | `TenantManager`, `TenantId`, quotas | 🟡 No current consumers in workspace |
| `cost_optimizer` | Cost models, `CostCalculator`, `WorkloadOptimizer`, ROI | 🟡 No consumers; internal dead field |
| `migration` | `MigrationSource`, `MigrationPlan`, `KubernetesMigrator`, etc. | 🟡 Feature-gated; stub impls |

**Findings**

- **Dead / unused field:** `WorkloadOptimizer::calculator: CostCalculator` is set in `new()` but never read. `suggest_placement()` uses only heuristics (data size, deps, content type). **Recommendation:** Either use `self.calculator` for cost-aware placement or remove the field.
- **Migration:** `KubernetesMigrator` stores `kubeconfig` and `namespace` but no method reads them (no real k8s client). Other `MigrationSource` variant fields (e.g. `connection_string`, `brokers`) are only used in serialization and help text. **Recommendation:** Document as stub; implement or gate behind “migration-impl” feature.
- **Dependencies:** No internal nexus deps. Uses serde, bincode, ed25519-dalek, blake3, thiserror, anyhow, chrono, uuid, hex, tracing, dashmap, parking_lot, bytes; optional tokio for `migration`.

---

### 2.2 nexus-pcu

**Path:** `nexus-pcu/src/`  
**Public modules:** `pcu`, `identity`, `proof`, `uso`, `routing`, `content_hash`, `invariants`, `crypto`, `pqc` (optional).

| Module | Purpose | Status |
|--------|---------|--------|
| `pcu` | `PCU`, `WasmModule`, `ExecutionConstraints` | ✅ Central; used by executor, edge, server, agp |
| `identity` | `IdentityContext`, `PrincipalId`, `Capability`, `DelegationChain` | ✅ Used by executor and tests |
| `proof` | `ExecutionProof`, `NodeAttestation` | ✅ Used by executor |
| `uso` | `USO`, `SyncPolicy`, `CausalHistory`, etc. | ✅ Used by sync, storage, executor (via host) |
| `routing` | `DataLocator` | ✅ Public API |
| `content_hash` | `ContentHash`, `ContentHasher` | ✅ Used by executor, storage, sync |
| `invariants` | Invariant checks | ✅ Used internally |
| `crypto` | PCU-level crypto | ✅ Used |
| `pqc` | PQC (optional) | 🟡 Optional feature |

**Findings**

- **NodeId:** Defined in `lib.rs`; `NodeId::local()` is the canonical “local” constructor (replaces deprecated `zero()`).
- **Dependency:** Only `nexus-core` (default-features = false, features = ["std"]). No circular dependency.

---

### 2.3 nexus-sync

**Path:** `nexus-sync/src/`  
**Public modules:** `sync_engine`, `crdt_uso`, `adapters`.

- Re-exports from `causalux_v2`: `VersionVector`, `CausalOp`, `CausalDAG`, `ConflictPolicy`, `ConflictResolution`, CRDTs (`RGAText`, `GCounter`, …), `Snapshot`/`SnapshotManager`, `HierarchicalSync`, `AdaptiveSync`, `SyncStrategy`, `SyncStats`, `SyncRequest`, `SyncResponse`.
- **Verified:** causalux `lib.rs` exports match these re-exports (including `ConflictResolution` from `conflict` module).

**Findings**

- **Dependencies:** `causalux-v2` (path `../causalux`), `nexus-pcu`. No nexus-core dependency.
- **Adapters:** `ContentHashAdapter`, `CrdtUSO`, `NexusSyncEngine`, `SyncDelta` bridge NEXUS ↔ CAUSALUX.

---

### 2.4 nexus-storage

**Path:** `nexus-storage/src/`  
**Public modules:** `log`, `index`, `query`, `error`, `backup`.

| Module | Purpose | Status |
|--------|---------|--------|
| `log` | `ProvenanceLog` (causal log over RocksDB) | ✅ Uses `nexus_core::causal::{CausalTensor, CausalId}` |
| `index` | `AlgebraicIndex` | ✅ |
| `query` | `QueryPattern` | ✅ |
| `error` | `NexusStorageError` | ✅ |
| `backup` | `BackupManager`, `BackupMetadata`, `BackupType` | ✅ |

**Dependencies:** `nexus-core`, `nexus-pcu`. No circular dependency.

---

### 2.5 nexus-network

**Path:** `nexus-network/src/`  
**Public modules:** `message`, `transport`, `tls`, `gossip`, `sync`, `node`, `error`, `rate_limit`.

- **message:** `CausalMessage::Tensor(nexus_core::causal::CausalTensor)` — correct use of core causal types.
- **transport:** `QuicTransport::new_dev()` for dev TLS; production uses full TLS.
- **Dependencies:** `nexus-core`, `nexus-pcu`, `nexus-sync`, `nexus-observability`.

---

### 2.6 nexus-executor

**Path:** `nexus-executor/src/`  
**Public modules:** `cache`, `error`, `executor`, `host_functions`, `limits`, `proof`, `semantic_cache`, `types`.

| Component | Purpose | Status |
|-----------|---------|--------|
| `PcuExecutor` | WASM execution, fuel/memory limits | ✅ |
| `SemanticCache` | Content-addressed memoization; `stats()` for metrics | ✅ |
| `NexusHost` | Host API (uso_get/put, spawn_pcu, log, get_time) | ✅ |
| `NoopHost` | Default no-op host | ✅ |

**Findings**

- **No nexus-core dependency:** Uses only `nexus-pcu` and `nexus-observability`. Causal types come from PCU/USO/content-hash layer; appropriate.
- **Re-exports:** PCU, identity, proof, content hash from `nexus_pcu`; `NodeId` from `nexus_pcu`.

---

### 2.7 nexus-compress

**Path:** `nexus-compress/src/`  
**Public modules:** `pcu_compress`, `uso_compress`.

- Re-exports from `vectra`: `vectra_encode`, `vectra_decode`, `try_encode`, `EncodeResult`, `Artifact`, `Payload`, `VectraError`, `VectraResult`, utilities, version.
- **Verified:** vectra crate exposes these symbols; workspace uses `vectra/vectra`.

**Dependencies:** `vectra` (path `../vectra/vectra`), `nexus-pcu`.

---

### 2.8 nexus-observability

**Path:** `nexus-observability/src/`  
**Public modules:** `metrics`, `health`, `logging`, `tracing` (optional, feature `otel`).

- **metrics:** `NexusMetrics` (PCU, Network, Sync, Storage, Resource).
- **health:** `HealthCheck`, `HealthStatus`, `ComponentHealth`.
- **Dependencies:** `nexus-core`, `nexus-pcu` (for types in metrics).

---

## 3. nexus-core-v2 (Not in Workspace)

**Path:** `nexus-core-v2/`  
**Status:** Standalone crate; **not** listed in root workspace `members`.

- **Purpose (from README):** “Deterministic execution log with algebraic merge.” CLI: `init`, `exec <wasm> <input>`, `replay`, `status`.
- **Contents:** Own `Cargo.toml` (name `nexus-core`, deps: sha2, bincode, serde, wasmtime, anyhow, thiserror, clap). Modules: `core`, `errors`, `executor`, `hash`, `log`, `merge`, `op`, `replay`, `storage`, `sync`, `main`.
- **Difference from workspace `nexus-core`:** Self-contained executable with WASM execution and log/replay; workspace `nexus-core` is a library (causal tensors, tenancy, no WASM).

**Recommendation:** Decide explicitly: (a) treat as **reference/legacy** and document in README, or (b) add to workspace under a distinct name (e.g. `nexus-core-cli` or `nexus-core-v2`) and wire deps so it doesn’t shadow `nexus-core`.

---

## 4. Dependency Graph (Core Crates)

```
nexus-core          (no internal nexus deps)
    ↑
nexus-pcu           (nexus-core)
    ↑
nexus-sync          (causalux-v2, nexus-pcu)
nexus-storage       (nexus-core, nexus-pcu)
nexus-compress      (vectra, nexus-pcu)
nexus-observability (nexus-core, nexus-pcu)
    ↑
nexus-network       (nexus-core, nexus-pcu, nexus-sync, nexus-observability)
nexus-executor      (nexus-pcu, nexus-observability)
```

- **No circular dependencies** among these crates.
- **causalux** and **vectra** are external to the nexus-core/pcu/sync/compress/storage/network/executor/observability set; sync and compress re-export their APIs.

---

## 5. Cross-Crate API Consistency

| Concept | Source | Consumers | Notes |
|---------|--------|-----------|--------|
| `CausalId` / `CausalTensor` / `VectorClock` | nexus-core | storage, network, (pcu indirectly) | Consistent |
| `NodeId` | nexus-pcu | executor, network, benches/tests | `NodeId::local()` used; no `zero()` |
| `ContentHash` / `ContentHasher` | nexus-pcu | executor, storage, sync | Single source of truth |
| `NexusError` / `Result` | nexus-core | core, pcu (via core), storage, network | Consistent |
| `generate_signing_key` | nexus-core::crypto | pcu, network, storage tests/benches | Single place for keygen |

No conflicts or duplicate definitions found in audited code.

---

## 6. Component Summary Table

| Crate | Src modules | Deps (nexus) | Dead / stub code | Notes |
|-------|-------------|--------------|-------------------|------|
| nexus-core | causal, cost_optimizer, crypto, error, tenancy, migration | — | WorkloadOptimizer.calculator; migration stubs | Optional migration feature |
| nexus-pcu | pcu, identity, proof, uso, routing, content_hash, invariants, crypto, pqc | core | — | PQC optional |
| nexus-sync | sync_engine, crdt_uso, adapters | pcu, causalux | — | Re-exports causalux |
| nexus-storage | log, index, query, error, backup | core, pcu | — | RocksDB in log |
| nexus-network | message, transport, tls, gossip, sync, node, error, rate_limit | core, pcu, sync, observability | — | QUIC + TLS |
| nexus-executor | cache, error, executor, host_functions, limits, proof, semantic_cache, types | pcu, observability | — | WASM + semantic cache |
| nexus-compress | pcu_compress, uso_compress | pcu, vectra | — | Re-exports vectra |
| nexus-observability | metrics, health, logging, tracing (otel) | core, pcu | — | Optional otel |

---

## 7. Recommendations

1. **nexus-core:** Use or remove `WorkloadOptimizer::calculator`; document or implement migration stubs (and consider feature-gating implementation).
2. **nexus-core-v2:** Add to docs (e.g. README or ARCHITECTURE) and either keep as out-of-workspace reference or add to workspace with a distinct crate name.
3. **Tenancy / cost_optimizer:** No current consumers. Either document as “reserved for server/billing” or add a single consumer (e.g. nexus-server) to validate the API.
4. **Build:** Run `cargo build --workspace` and `cargo test --workspace` (and any feature combos used in production) to confirm no regressions; this audit is static.

---

**Audit complete.** For benchmarking status see `docs/BENCHMARKING.md`; for protocol and contracts see `docs/PROTOCOL_SPEC.md` and `docs/CORE_CONTRACTS.md`.
