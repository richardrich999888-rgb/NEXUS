# NEXUS Full Source Audit

**Role:** Principal systems architect + patent attorney + hostile adversary.  
**Scope:** All executable source (Rust workspace + non-workspace, Python, WASM/edge, formal specs, glue).  
**Rules:** Brutal, precise; no hype; separate engineering value vs patent value; call out weak/fake novelty.

---

## Part 1 — Critical Path: File-by-File

### nexus-core/src/causal.rs — **(A) Core logic**

**Classification:** (A) Core logic. Production path: tensor creation, merge, verify, serialization.

**Types and runtime behavior:**

| Item | What it does at runtime | Data transformed | Invariants | Assumptions |
|------|-------------------------|------------------|------------|-------------|
| `CausalId` | 32-byte content-addressed ID; `from_hash` = BLAKE3 of data; `genesis()` = all zeros. | Bytes → fixed 32-byte ID. | None beyond hash determinism. | BLAKE3 collision resistance. |
| `VectorClock` | BTreeMap<u64, u64>: node_id → logical time. `tick(node)` increments; `merge(other)` takes max per node; `happens_before` / `concurrent` for ordering. | Local events + remote clocks → merged clock. | Monotonicity per node; merge is commutative. | Node IDs are u64 (not nexus_pcu::NodeId). |
| `Provenance` | Parents (Vec<CausalId>), merkle_root = BLAKE3(parents), depth. `lca`, `diff_since`, `contains_ancestor` for DAG queries. | Parent set → root hash. | depth "simplified for MVP" (currently 0 or 1). | Parents ordered/deduped by caller. |
| `CausalTensor` | Bundles id, data, provenance, clock, signature, metadata. `new()`: 100MB limit, clock tick, provenance from parents, id = BLAKE3(data\|provenance\|clock), Ed25519 sign(id\|data\|merkle_root). `merge(local, remote)`: idempotent if same id; if one happens_before other return that; else **concurrent merge**: merge clocks, LCA of provenances, **merge_data** = deterministic LWW by BLAKE3 hash comparison (local_hash > remote_hash ? local : remote), parents = [local.id, remote.id] sorted. | Bytes + parents + node_id + key → signed tensor; two tensors → one merged tensor. | 100MB max; id content-addressed; signature over id+data+merkle_root; concurrent merge deterministic (same inputs → same output). | Signing key matches node; chrono::Utc::now() for timestamp. |
| `merge_data` (private) | Last-write-wins by hash: compare BLAKE3(local) and BLAKE3(remote) as bytes; return larger. | Two byte slices → one. | Deterministic; no semantic merge (no CRDT, no OT). | Application-specific merge not implemented here. |

**Production vs test:** All of `CausalId`, `VectorClock`, `Provenance`, `CausalTensor::new`, `merge`, `verify`, `to_bytes`/`from_bytes` are production. Tests use `generate_signing_key` from nexus_core::crypto.

**Novelty (honest):** Causal tensor with content-addressed id + vector clock + signature + **deterministic concurrent merge** (LWW by hash). The merge is **not** novel per se (LWW is standard); the **bundling** (content id + provenance + clock + signature in one type) and the **single shared algorithm** across the stack are the integration point. **Weak:** merge_data is trivial LWW; no application-defined merge strategy.

**Defensibility:** The core **invariant** is: "concurrent merge is deterministic and commutative." That is enforced by fixed merge_data (hash comparison) and sorted parents. Replacing merge_data with a different strategy (e.g. CRDT) would require changing this file and any verifier that assumes this rule. **This is defensible** as a system/method claim for "causal tensor with signed, content-addressed identity and deterministic concurrent merge" **only if** the rest of the system (PCU, USO, executor) depends on this exact shape; otherwise it's a small layer that could be swapped.

---

### nexus-pcu/src/pcu.rs — **(A) Core logic**

**Classification:** (A) Core logic. Production: PCU creation, id/semantic_hash, serialization; execution is in nexus-executor.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `WasmModule` | Holds bytecode, BLAKE3 hash, optional name, memory_pages (default 16). `is_valid_header`: first 4 bytes = WASM magic. | Bytes → module with hash. | None. | Caller provides valid WASM. |
| `ExecutionConstraints` | Default 30s, 256MB, no capabilities. | — | — | Used by executor for limits. |
| `PCU::new` | id = compute_id(code, inputs, parameters, identity) = BLAKE3(code.hash\|inputs\|parameters\|identity.principal). No signature on PCU itself. | Code + inputs + params + identity → PCU. | id deterministic from (code, inputs, parameters, principal). | Identity already signed (caller). |
| `compute_id` | Hashes code hash, input hashes, parameters, identity.principal. | Same. | No identity in id beyond principal (delegation/caps not in id). | — |
| `semantic_hash` | BLAKE3(code_hash \| sorted input hashes \| parameters \| identity.principal). | PCU → ContentHash. | Deterministic; includes identity so same code+inputs, different principal → different semantic hash. | — |

**Production vs test:** `new`, `with_constraints`, `compute_id`, `content_hash`, `semantic_hash`, `to_bytes`/`from_bytes`, `is_executed`/`is_valid` are production. Tests use signed identity.

**Novelty:** PCU as a **single unit**: code + content-addressed inputs + **identity embedded** (principal in id and semantic_hash). "Computation carries identity" is the claim. **Engineering value:** Clear contract for executor (WASM + inputs + identity). **Patent value:** Identity-in-id is a design choice; prior art (capability-based compute, signed requests) exists. **Weak:** No cryptographic binding of PCU to identity inside this file (binding is in IdentityContext.sign).

**Defensibility:** Executor and semantic cache depend on `PCU.id` and `semantic_hash`. Changing how id/semantic_hash are computed would break cache and proof binding. **This is defensible** because the **invariant** "PCU id and semantic hash are deterministic and include identity" is load-bearing for cache and proofs.

---

### nexus-pcu/src/uso.rs — **(A) Core logic**

**Classification:** (A) Core logic. Production: USO creation, update, merge, access checks, serialization.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `USO::new` | id = ContentHash::compute(&data); history empty; access owner_only; sync OnDemand. | Bytes + PrincipalId → USO. | id content-addressed. | — |
| `update` | Replaces data, recomputes id, modified_at = now(), history.add_operation(Set, principal, node_id). | (data, principal, node_id) → updated USO. | id = hash(data). | — |
| `merge` | history.merge(other.history); if other.modified_at > self.modified_at then replace data/id/modified_at; push other.id to history.parents if not present. | Two USOs → one. | Wall-clock LWW for data; history is merged (vector clock max + op merge). | modified_at comparable across nodes (clock skew). |
| `CausalHistory` | Vector clock (HashMap<String,u64>), operations list, parents. `merge`: max clock; merge ops by (lamport, principal) dedup; sort by lamport. | Operations + remote history → merged history. | Lamport total order per (node, principal). | — |

**Production vs test:** All of USO/CausalHistory above are production. Sync engine and edge/server create and merge USOs.

**Novelty:** One type (USO) with **sync policy + access policy + causal history** in one place. Merge is **wall-clock LWW + history merge**. **Weak:** Data conflict resolution is pure LWW (modified_at); no CRDT of the payload. **Engineering value:** Single abstraction for "state object that syncs and has access control."

**Defensibility:** NexusSyncEngine and network layer depend on USO.id, history, merge. **This is defensible** because USO is the **shared state primitive** for sync and storage; replacing it would require a compatible history and id scheme.

---

### nexus-pcu/src/identity.rs — **(A) Core logic**

**Classification:** (A) Core logic. Production: principal, capabilities, delegation, IdentityContext validity and signing.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `IdentityContext::is_valid` | now < valid_until; if delegation, chain.is_valid(now) and principal == final_delegatee; verify() signature. | — | Valid only if not expired, delegation chain valid, signature ok. | Clock monotonic. |
| `sign` / `verify` | sign: Ed25519 over (principal \| bincode(capabilities, delegation, valid_until)). verify: same bytes, check signature. | Context → signed context. | Signature binds principal + caps + delegation + expiry. | Principal bytes are valid Ed25519 pubkey. |
| `permits` | is_valid() && capabilities.permits(resource, action). | — | Access check. | — |
| `DelegationChain::is_valid` | Expiry per link; chain continuity (link.from == prev.to); each link.verify() signature. | — | Chain is unbroken and signed. | — |

**Production vs test:** All used in executor (identity check) and PCU creation. Tests use real SigningKey.

**Novelty:** Identity **embedded in PCU** and checked at execution time (no separate auth service call). **Prior art:** JWT, capability tokens, mTLS. **Patent angle:** "Identity context signed and embedded in computation unit, validated at execution" is a method claim; non-obviousness is marginal.

**Defensibility:** Executor rejects PCU if !identity.is_valid(). **This is defensible** because the whole "computation carries proof of authorization" depends on this type and its validity rules.

---

### nexus-pcu/src/content_hash.rs — **(A) Core logic**

**Classification:** (A) Core logic. Pure BLAKE3 content addressing; used everywhere.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `ContentHash::compute` | BLAKE3(data). | Bytes → 32 bytes. | Deterministic. | — |
| `ContentHasher` | Incremental BLAKE3. | Stream → hash. | Same. | — |

**Production vs test:** All production. No test-only branches.

**Novelty:** None. Standard content addressing.

**Defensibility:** Not a differentiator. **Replaceable** by any 32-byte hash. Lock-in is **everywhere** (PCU, USO, proof, cache key); swapping hash would require coordinated change.

---

### nexus-pcu/src/proof.rs (nexus-pcu) — **(A) Core logic**

**Classification:** (A) Core logic. Defines ExecutionProof, NodeAttestation for **nexus-pcu** (builder pattern). **Note:** nexus-executor has its **own** proof.rs with ExecutionProof::create used in production execution.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `ExecutionProof` (pcu) | pcu_hash, input_hashes, code_hash, output_hash, duration, memory, attestation. content_hash() for signing; sign/verify. | Execution result → proof. | Attestation signs proof content. | — |
| `NodeAttestation` (pcu) | node_id, executed_at, security_level, signature over (node_id\|executed_at\|level\|proof_contents). | — | 64-byte signature. | NodeId is VerifyingKey bytes. |

**Production vs test:** In nexus-executor, the **executor’s** ExecutionProof (executor/src/proof.rs) is used, not the pcu one. So **nexus-pcu proof** is used where proofs are built from pcu types (e.g. builder); executor redefines its own proof with create().

**Defensibility:** Two proof types exist (pcu vs executor). Executor path uses executor’s proof. **Potential confusion** for fork: they must keep executor’s proof in sync with pcu’s contract (pcu_hash, inputs, output_hash, etc.).

---

### nexus-executor/src/executor.rs — **(A) Core logic**

**Classification:** (A) Core logic. **Main production path:** execute(pcu, context) → validate → cache lookup → Wasmtime run → proof → cache put → response.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `PcuExecutor::execute` | 1) Validate WASM header, size ≤ MAX_MODULE_SIZE, identity.is_valid(). 2) SemanticKey from pcu + context inputs + identity; cache.get(key); if hit return cached result + proof. 3) Store with context; set_fuel; Module::new(bytecode); register_host_functions; instantiate; find entry in [_start, main, execute, run]; timeout func.call(). 4) extract_output (__nexus_output_len + memory); generate_proof; cache.put; return. | PCU + ExecutionContext → ExecutionResponse (result + proof + from_cache). | Fuel and timeout enforced; output size ≤ MAX_OUTPUT_SIZE; proof created and cached on success. | Host implements NexusHost; context.inputs match pcu.inputs. |
| `extract_output` | Optional __nexus_output_len(); then memory slice. | WASM memory → Vec<u8>. | Length ≤ MAX_OUTPUT_SIZE. | Guest exports __nexus_output_len and "memory". |
| `generate_proof` | ExecutionProof::create(pcu, inputs, result, identity, signing_key). | — | Proof binds pcu, inputs, output, identity, node. | — |

**Production vs test:** execute(), extract_output(), generate_proof() are production. Tests and benches call execute.

**Novelty:** **Semantic cache before execution** (key = code + inputs + identity): same computation + same identity → cache hit, no re-run. Combined with **proof on every execution** (including cache hit return). **Engineering value:** Reduces redundant work; proof makes cache results verifiable. **Patent value:** "Semantic cache key including identity" + "proof returned even for cache hit" is a plausible method claim; prior art (memoization, content-addressable cache) exists but identity-in-key for execution is less common.

**Defensibility:** **This is defensible** because the **invariant** "every execution path (cold or cached) returns a verifiable proof" and "cache key includes identity" is what makes the system auditable and shareable under policy. Fork that omits identity from cache key breaks multi-tenant semantics.

---

### nexus-executor/src/semantic_cache.rs — **(A) Core logic**

**Classification:** (A) Core logic. Production: get/put by SemanticKey (code_hash, inputs_hash, identity_hash); routing decision (UseCached, ExecuteLocally, ForwardTo, FetchInputsFirst).

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `SemanticKey::from_pcu` | code_hash = pcu.code.content_hash(); inputs_hash = combine_inputs(inputs); identity_hash = identity.content_hash(). | PCU + inputs + identity → key. | Key is deterministic. | — |
| `SemanticCache::get` | Lookup; if expired remove and return None; else record_hit, return entry. | Key → Option<CacheEntry>. | Expiry by time. | — |
| `put` | If at capacity evict_one (LFU-like: lowest hit_count or expired); insert CacheEntry with proof. | (key, result, proof) → stored. | max_entries enforced. | — |
| `route` | get(key); if hit UseCached; else data_locations for each input; if missing inputs FetchInputsFirst; else best node by count; if best == local ExecuteLocally else if score > len/2 ForwardTo(best). | PCU + inputs + identity → RoutingDecision. | — | register_data called elsewhere for locality. |

**Production vs test:** get/put are used in executor.execute(); route() is available but executor currently uses get() only (no route() in execute path in the read code). So **production path uses get/put**; route() is for future routing.

**Novelty:** **Semantic key** (code + inputs + identity) for execution cache, plus **data locality** map for routing. **Weak:** route() and data_locations are not used in the current execute() path; only get/put are. So "routing to node with data" is **not** in the critical path yet.

**Defensibility:** **This is defensible** because the cache key definition (including identity_hash) is what enforces "same principal, same computation → same cache entry." Removing identity from key would break isolation.

---

### nexus-executor/src/proof.rs (executor) — **(A) Core logic**

**Classification:** (A) Core logic. ExecutionProof with create(), signing_bytes(), verify(), verify_output(). Used by executor after each run.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `ExecutionProof::create` | pcu_hash, input_hashes, output_hash, identity_hash, executor_node, executed_at, duration_ms, fuel_consumed, peak_memory; attestation = NodeAttestation(node_pubkey); sign(signing_bytes()). | PCU + inputs + result + identity + node_key → proof. | Signature over canonical bytes. | — |
| `signing_bytes` | pcu_hash, input_hashes.len(), input_hashes, output_hash, identity_hash, executor_node, executed_at, duration_ms, fuel_consumed, peak_memory, node_pubkey. | — | Deterministic serialization. | — |

**Production vs test:** create() and verify() are production. CacheEntry.to_result() uses proof.fuel_consumed, peak_memory, duration_ms (executor’s ExecutionProof has these).

**Defensibility:** **This is defensible** because the **proof format** is the audit record; changing it breaks verification and any downstream audit. Fork must either keep format or re-establish trust boundary.

---

### nexus-sync/src/sync_engine.rs — **(B) Glue / orchestration**

**Classification:** (B) Glue. Wraps causalux CausalDAG; holds uso_registry (HashMap ContentHash → USO); update_uso creates CausalOp and inserts into DAG; get_sync_delta computes ops since peer version.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `NexusSyncEngine` | dag: CausalDAG; uso_registry. register_uso; update_uso mutates USO and inserts CausalOp (uso_update, json uso_id/data_hash) with VersionVector. get_operations_since(since_lamport); merge_remote(ops); get_sync_delta(peer_vv). | USO updates → CausalOps in DAG; peer VV → SyncDelta (ops). | DAG and USO registry kept in sync by caller. | CausalDAG has get_version_vector(), get_operations_after(); VersionVector has .versions (BTreeMap String→u64). |

**Production vs test:** Used by nexus-server (AppState has NexusSyncEngine). Tests register USO and update_uso.

**Novelty:** **Adapter** between NEXUS USO and CAUSALUX DAG. No new algorithm; maps USO updates to causal ops. **Patent value:** Low; "bridge between state object and causal DAG" is obvious integration.

**Defensibility:** **Replaceable** by another adapter that produces/consumes the same CausalOp shape and VersionVector. The **load-bearing** part is the **USO + CausalHistory** contract, not this file.

---

### nexus-storage/src/log.rs — **(A) Core logic**

**Classification:** (A) Core logic. ProvenanceLog: RocksDB; append(tensor) keyed by tensor.id; get(id); append_batch; exists; count_approximate.

| Item | Runtime | Data | Invariants | Assumptions |
|------|---------|------|------------|-------------|
| `append` | key = tensor.id.as_bytes(); value = bincode(tensor). put. | CausalTensor → persisted. | Key = content id. | — |
| `get` | get(id.as_bytes()); deserialize. | CausalId → Option<CausalTensor>. | — | — |

**Production vs test:** Production path for any component that persists causal tensors (e.g. after merge). No execution path in the audited code **writes** to ProvenanceLog from executor; executor does not use nexus-storage. So **storage is used by sync/network/CLI** for log, not by executor directly.

**Novelty:** Standard key-value log keyed by CausalId. **Engineering value:** Durable causal history.

**Defensibility:** **Replaceable** by any store that maps CausalId → CausalTensor. The **invariant** is "log is append-only and keyed by content id"; that’s standard.

---

### Remaining critical-path and glue files (classification only)

| File | Classification | Note |
|------|----------------|------|
| nexus-core/error.rs | (A) | NexusError, Result; used everywhere. |
| nexus-core/crypto.rs | (A) | generate_signing_key; used by tests/benches. |
| nexus-core/cost_optimizer.rs | (C) | Stub/sales; WorkloadOptimizer.calculator never read. |
| nexus-core/migration.rs | (C) | Stub; KubernetesMigrator fields never read. |
| nexus-core/tenancy.rs | (B) | Glue; no current consumer in execution path. |
| nexus-pcu/routing.rs | (A) | DataLocator; public API. |
| nexus-pcu/invariants.rs | (A) | Invariant checks for PCU/USO. |
| nexus-pcu/crypto.rs | (A) | PCU-level crypto. |
| nexus-network/message.rs | (A) | CausalMessage (e.g. Tensor(CausalTensor)); production wire format. |
| nexus-network/transport.rs | (A) | QuicTransport; production. |
| nexus-network/gossip.rs, sync.rs, node.rs | (B) | Glue; propagate messages. |
| nexus-executor/host_functions.rs, limits.rs, types.rs | (A) | Host API, ExecutionLimits, ExecutionContext/Result; production. |
| nexus-executor/cache.rs | (A) | ResultCache type; used. |
| nexus-sync/crdt_uso.rs, adapters.rs | (B) | CRDT-USO adapter; glue. |
| nexus-storage/index.rs, query.rs, backup.rs | (A) | Index, query, backup; not on hot execution path. |
| homeostasis-engine/* | (A) | Core logic for safety; not on PCU path. |
| autonomic-system/*, developmental-gates/*, nervous-system/* | (B) | Orchestration over homeostasis; not on PCU path. |
| multi-asi-immune/* | (A) | Identity, reputation, threat; not on PCU path. |
| telos-protocol/* | (A) | Membrane, entropy, authority; not wired to executor. |
| nexus-agp/* | (B) | Bridge AGP ↔ NEXUS; not on executor path. |
| nexus-edge/src/lib.rs | (A) | Worker: health, benchmarks, POST /api/uso; production edge. |
| nexus-secrets/* | (B) | Secret backends; optional. |
| nexus-cli/main.rs | (B) | Glue; subcommands. |
| nexus-server/main.rs | (B) | Glue; axum + sync engine. |
| nexus-runtime/lib.rs | (D) | Stub: `pub struct WasmExecutor;` only. |
| causalux/* | (A) | External; version vectors, DAG, CRDTs; used by sync. |
| vectra/vectra/* | (A) | External; compression; used by nexus-compress. |
| agp/*.py | (B) | Demo/governance; not in executor path. |
| agp-core/* | (B) | Full stack; not in NEXUS Rust path. |
| nexus-telecom/* | (B) | Python; Lyapunov, WuR; not in executor path. |
| syntriass/* | (B) | Preview engine; not in NEXUS path. |
| brain/formal_specs/SafetyProtocol.tla | (A) | Formal spec; maps to autonomic mode transitions. |

Line-by-line treatment above is reserved for the **critical path** (causal, pcu, uso, identity, content_hash, proof, executor, semantic_cache, sync_engine, storage log). All other files are classified; full line-by-line for every function in every file would exceed a single audit document.

---

## Part 2 — Execution Path Traces

### Path 1: PCU creation → causal merge → sync → storage → execution → audit

1. **PCU creation:** Client builds PCU (WasmModule, inputs, parameters, IdentityContext). PCU.id = BLAKE3(code_hash\|inputs\|parameters\|principal). Identity must be signed and valid. **Production:** nexus-server create USO; nexus-cli submit PCU; nexus-edge POST /api/uso (USO only).
2. **Causal merge:** When two nodes have different CausalTensors (or USOs), merge uses nexus-core CausalTensor::merge (idempotent / happens_before / concurrent LWW) or USO.merge (history merge + modified_at LWW). **Production:** Sync engine and network propagate tensors/ops; merge happens on receive.
3. **Sync:** NexusSyncEngine holds USOs; update_uso writes CausalOp into CausalDAG; get_sync_delta(peer_vv) returns ops; merge_remote(ops) applies remote ops. **Production:** Server uses sync engine; network layer would send/receive deltas (not fully traced in this audit).
4. **Storage:** ProvenanceLog.append(tensor) persists CausalTensor by id. **Production:** Used when causal state is persisted (e.g. from sync or CLI).
5. **Execution:** PcuExecutor.execute(pcu, context): validate → semantic cache get(SemanticKey) → if miss: Wasmtime run → ExecutionProof::create → cache put → return result + proof. **Production:** Executor is the only component that runs WASM; server/CLI would call executor (exact wiring in server main not fully traced here).
6. **Audit:** ExecutionProof (pcu_hash, inputs, output_hash, identity_hash, executor_node, duration, fuel, memory, attestation) is the audit record. Anyone can verify(proof) and verify_output(output).

**Gap:** No direct trace from "network receives merge" → "ProvenanceLog.append" in the read code; that would be in network handler or sync handler. Executor does not write to ProvenanceLog.

### Path 2: Safety stack — homeostasis → autonomic → developmental → nervous-system

- **homeostasis-engine:** Metric (value, setpoint, bounds); SingleMetricController.step(metric) applies correction, Metric.update clamps to bounds. **Production:** Used by multi-asi-immune, autonomic-system, developmental-gates, nervous-system.
- **autonomic-system:** ACT/CALM mode, arousal, transitions. **Production:** Nervous-system integrates it.
- **developmental-gates:** Stage manager, capability registry, gate enforcer. **Production:** Nervous-system integrates it.
- **multi-asi-immune:** Identity, reputation, threat, protocol. **Production:** Nervous-system integrates it.
- **nervous-system:** Coordinator over all four; perception → decision → motor. **Production:** Entry point for "safety layer" but **no call from nexus-executor or nexus-server** to nervous-system in the read code. So the safety stack is **not on the PCU execution path** in the current codebase; it’s a separate subsystem.

**Conclusion:** Safety stack is **orchestration** for ASI agents; it does not currently gate PCU execution or TELOS crossings in the same process.

### Path 3: TELOS crossings

- **telos-protocol:** CommitmentMembrane, Decision (Draft→Pending→Validating→Committed), EntropyMeter, AuthorityRegistry. request_crossing(decision, entropy, authority) consumes entropy and checks authority. **Production:** No call from nexus-executor or nexus-server to telos in the read code. TELOS is **standalone**; integration would be in a higher-level orchestrator (e.g. agp-core or a future service).

**Conclusion:** TELOS is **not** on the critical execution path of PCU → executor → proof. It’s a separate accountability layer for "commitment crossing."

---

## Part 3 — Patent Analysis and Defensibility

### Real execution path (PCU → executor → proof)

- **Novel:** (1) **PCU with identity embedded in id and semantic hash** (computation carries identity). (2) **Semantic cache key = code + inputs + identity** (cache hit is identity-aware). (3) **Every execution path returns a single proof type** (cold and cache hit). (4) **Deterministic concurrent merge** of causal tensors (LWW by hash) with signed, content-addressed identity.
- **Non-obvious:** A senior engineer might design (1) and (2) for multi-tenant execution and audit; the **combination** (identity in both PCU id and cache key + proof on cache hit) is less obvious. (4) is standard distributed systems; the **bundling** with signatures is the claim.
- **Hard to fork:** If you remove identity from PCU id or cache key, you break multi-tenant isolation and audit. If you change merge to non-deterministic, you break convergence. If you drop proof on cache hit, you break "every result verifiable."
- **Claim mapping:** (a) System: "Execution substrate comprising PCU with embedded signed identity, semantic cache keyed by code+inputs+identity, and execution proof returned for every result." (b) Method: "Method of executing computation comprising validating embedded identity, looking up cache by semantic key including identity, on miss executing WASM and producing proof, on hit returning cached result and proof." (c) Invariant: "All execution outcomes (cache hit or miss) produce a single verifiable proof format."

### What can be copied easily

- Content hashing (BLAKE3), vector clocks, RocksDB log, Wasmtime execution, Ed25519 signatures. These are standard.
- USO merge (LWW + history merge) and CausalHistory. Straightforward CRDT-style design.
- Sync engine as "adapter": any team can write another adapter to a DAG.

### What breaks if copied without the rest

- **Semantic cache** without **identity in key**: cache would be shared across principals incorrectly.
- **Executor** without **proof on cache hit**: audit trail would have gaps.
- **Causal merge** with a **different** merge_data (e.g. non-deterministic): replicas would diverge.
- **PCU** without **identity in id**: proof and cache would not bind to principal.

### Lock-in

- **Proof format** (executor): Any verifier or auditor depends on signing_bytes() layout; change breaks verification.
- **SemanticKey** shape: Cache and executor are coupled to (code_hash, inputs_hash, identity_hash).
- **CausalTensor** merge rule: Any node that merges must use the same merge_data rule or consistency is lost.

### Weak / fake novelty

- "Causal tensor" as a name is marketing; the math is vector clocks + LWW. The **signature and content-addressed id** are the real addition.
- "Universal State Object" is a branded CRDT-style object; the **sync policy + access policy in one type** is the integration point, not novel CRDT theory.
- Safety stack (homeostasis, autonomic, developmental, nervous-system) is **not** in the PCU execution path; claiming "AI safety through NEXUS" requires actually **calling** that stack from execution or TELOS.

---

## Part 4 — Future Problems This Code Already Addresses

| Problem | Addressed? | Where |
|---------|-------------|------|
| Multi-agent determinism | Partially | Causal merge is deterministic; same inputs → same merge result. Replay requires same merge order. |
| AI safety & regulation | Stub | Safety stack exists but is not wired to executor or TELOS. |
| Causal consistency at scale | Yes | Vector clocks + merge + sync engine + storage log. |
| Cost & bandwidth collapse | Partially | Semantic cache reduces redundant execution; data locality in cache is present but route() not used in execute path. |
| Auditable AI accountability | Yes | ExecutionProof on every result; proof binds pcu, inputs, output, identity, node. |
| Edge + cloud + governance | Partially | nexus-edge (Worker) runs USO/merge; no TELOS or safety stack on edge in read code. |

---

## Part 5 — TOP 5 Load-Bearing Files/Modules

**Rule:** Without these, the NEXUS system fundamentally collapses. Exactly 5.

---

### 1. **nexus-core/src/causal.rs** — CausalTensor and merge

**Why load-bearing:** Defines the **only** causal merge used by the stack: idempotent, happens_before, and **deterministic concurrent merge** (LWW by hash). All causal state (and any storage/sync that stores CausalTensor) depends on this algorithm. If this file is removed or the merge rule is changed, convergence and verification guarantees break.

**Invariant:** Concurrent merge is deterministic and commutative; merged tensor is signed and content-addressed.

**Replaceable by existing tech?** Vector clocks and LWW exist elsewhere; the **exact bundle** (CausalTensor + signature + this merge_data) is what the rest of the system assumes. Replacing with another library would require that library to provide the same merge semantics and serialization.

**Center of patent claims?** Yes. **System/method claim:** "Causal tensor with content-addressed identity, vector clock, and deterministic concurrent merge producing a single signed tensor."

---

### 2. **nexus-pcu/src/pcu.rs** — PCU and id/semantic_hash

**Why load-bearing:** PCU is the **unit of computation** for the executor and cache. PCU.id and semantic_hash are used by executor (cache key, proof binding). If PCU did not embed identity in id and semantic_hash, the semantic cache and proof would not be identity-aware, and the "computation carries identity" story collapses.

**Invariant:** PCU id is deterministic from (code, inputs, parameters, principal); semantic_hash includes identity so same code+inputs, different principal → different cache entry.

**Replaceable?** Conceptually yes (another "compute unit" type), but every consumer (executor, cache, proof) expects this struct and these hashes. Replacing PCU means rewriting executor and cache.

**Center of patent claims?** Yes. **System/method claim:** "Portable computation unit whose identity is determined by code, inputs, parameters, and principal, and whose semantic cache key includes said identity."

---

### 3. **nexus-executor/src/executor.rs** — PcuExecutor.execute

**Why load-bearing:** The **only** production path that runs a PCU: validate → semantic cache lookup → Wasmtime execute → proof creation → cache put. Without this file, there is no execution, no proof, and no cache integration. The entire "verifiable execution" depends on this flow.

**Invariant:** Every successful execution (including cache hit) returns an ExecutionResponse with a proof; identity is validated before execution; cache key includes identity.

**Replaceable?** Wasmtime could be swapped, but the **orchestration** (validation, cache, proof, host) is NEXUS-specific. Replacing this file is re-implementing the execution substrate.

**Center of patent claims?** Yes. **Method claim:** "Method of executing a portable computation unit comprising validating embedded identity, looking up a semantic cache keyed by code, inputs, and identity, and returning a verifiable proof for both cache hits and first-time execution."

---

### 4. **nexus-executor/src/semantic_cache.rs** — SemanticCache and SemanticKey

**Why load-bearing:** Defines the **cache key** (code_hash, inputs_hash, identity_hash) and the get/put used by the executor. If the cache key did not include identity_hash, multi-tenant isolation and "same identity → same cache entry" would be wrong. The executor depends on this module for every execute().

**Invariant:** Cache key is deterministic from (PCU code, input hashes, identity); get/put are the single source of cached results and proofs.

**Replaceable?** Another cache could implement the same key and interface, but the **key shape** is load-bearing. Changing the key (e.g. dropping identity) breaks the system’s guarantees.

**Center of patent claims?** Yes. **System claim:** "Semantic cache for computation results keyed by code hash, combined input hashes, and identity hash, and storing execution proof with each entry."

---

### 5. **nexus-pcu/src/identity.rs** — IdentityContext validity and signing

**Why load-bearing:** Executor rejects PCU if !pcu.identity.is_valid(). Validity is expiry, delegation chain, and signature. PCU creation (and thus id/semantic_hash) uses this identity. Without a single, enforced notion of "valid identity," the claims "computation carries proof of authorization" and "cache is identity-aware" are undefined.

**Invariant:** Identity is valid iff not expired, delegation chain valid, and signature verifies; signature binds principal, capabilities, delegation, expiry.

**Replaceable?** Another auth scheme could be used, but then PCU id/semantic_hash and executor validation would have to change in lockstep. This file is the **definition** of "valid identity" for the execution path.

**Center of patent claims?** Yes. **Method claim:** "Identity context signed by principal and embedded in computation unit, validated at execution time without external auth service."

---

## Summary Table

| # | Module | Why load-bearing | Invariant | Replaceable? | Patent center? |
|---|--------|------------------|-----------|--------------|----------------|
| 1 | nexus-core/causal.rs | Only causal merge; convergence & verification depend on it | Deterministic commutative merge; signed content-addressed tensor | Only with same merge semantics | Yes |
| 2 | nexus-pcu/pcu.rs | Unit of computation; id/semantic_hash drive cache & proof | id and semantic_hash include identity | No without rewriting executor/cache | Yes |
| 3 | nexus-executor/executor.rs | Only execution path; proof and cache integration | Every result has proof; identity validated | No without re-implementing substrate | Yes |
| 4 | nexus-executor/semantic_cache.rs | Cache key definition; executor uses get/put | Key = code + inputs + identity | Only with same key shape | Yes |
| 5 | nexus-pcu/identity.rs | Definition of valid identity; gates execution | Valid = not expired, chain ok, signature ok | No without changing PCU and executor | Yes |

---

**End of audit.**  
For component inventory see `docs/PROJECT_COMPONENTS.md`.  
For component-level audit see `docs/COMPONENTS_AUDIT.md`.  
For core crate API and dead code see `docs/CORE_SRC_AUDIT.md`.
