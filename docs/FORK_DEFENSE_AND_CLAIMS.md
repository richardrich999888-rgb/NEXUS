# Fork Defense, Patent Claim Skeletons, and Narrative Hardening

**Purpose:** (1) Realistic adversary fork attempt and exact failure points. (2) Five engineer-readable patent claim skeletons. (3) Code that dilutes defensibility and recommended cuts.

---

## Part 1 — Realistic Fork Attempt and Where It Fails

### Adversary setup

- **Resources:** Unlimited funding, senior engineers, full repo access (no license).
- **Goal:** Ship a competing "verifiable distributed execution" product without infringing or depending on NEXUS branding.
- **Strategy:** Copy repo, rename crates/branding, strip SYNTRIASS references, replace hashes/signatures with "our" format, optimize or simplify where possible.

### Fork playbook (concrete steps)

1. **Clone repo; rename** `nexus-*` → `acme-*`; replace CausalTensor with "StateNode," PCU with "ComputeUnit," USO with "StateObject."
2. **Replace crypto:** BLAKE3 → SHA-256, Ed25519 → ECDSA. Update content_hash.rs, identity signing, proof signing.
3. **Simplify cache:** "Identity in cache key is redundant if we use separate auth." Drop `identity_hash` from SemanticKey; key = (code_hash, inputs_hash) only.
4. **Simplify merge:** "LWW by hash is arbitrary." Replace `merge_data` in causal.rs with wall-clock timestamp LWW or "merge by node priority."
5. **Replace identity:** "We'll use JWTs from our auth service." Remove IdentityContext embedding from PCU; executor checks JWT in HTTP header instead of pcu.identity.is_valid().
6. **Unify proof:** Use one ExecutionProof type (nexus-pcu or executor); remove the other. Change proof layout (e.g. add "version" field, reorder signing_bytes).
7. **Ship:** Deploy executor, cache, sync, storage; claim "verifiable execution with causal consistency."

### Where the fork fails (concrete)

---

#### Failure 1: Cache becomes cross-tenant

**Location:** `nexus-executor/src/semantic_cache.rs` — `SemanticKey::from_pcu` (lines 46–56).

**What adversary did:** Removed `identity_hash` from the key; key = (code_hash, inputs_hash).

**Why it fails:** Executor still has one global cache. Two principals A and B submit the same (code, inputs). First execution (A) runs and caches result. Second request (B) gets **cache hit** and is returned A’s result **with A’s proof**. B’s audit trail now shows A’s executor_node and identity_hash. Data isolation is broken; in regulated settings this is a compliance failure.

**Hidden coupling:** The executor’s `execute()` path does not take a "tenant id" or "auth token" separately from the PCU. The **only** tenant bound to the cache entry is the one inside the PCU: `identity.content_hash()`. So the cache key **must** include identity or every lookup must validate "current request identity matches cached identity" — which implies storing identity in the cache entry and checking on hit. That is exactly what SemanticKey does. Dropping identity from the key forces either (a) no cache (performance loss) or (b) cross-tenant returns (violation).

**Forces re-invention:** They must either re-introduce identity in the cache key (same idea) or design a different multi-tenant cache (e.g. separate cache per principal, or post-hit identity check with stored identity in entry). The latter is a new design and new code paths; the former admits the original design was necessary.

---

#### Failure 2: Proof verification breaks and audit trail is inconsistent

**Location:** `nexus-executor/src/proof.rs` — `signing_bytes()` (lines 154–169), `ExecutionProof::create` (117–152).

**What adversary did:** Changed proof layout (added version, reordered fields) or switched to a different ExecutionProof struct (e.g. from nexus-pcu) that has different fields (e.g. no fuel_consumed, duration_ms).

**Why it fails:**  
- **Verifiers:** Any existing verifier (audit pipeline, compliance tool) expects the exact byte sequence produced by `signing_bytes()`. Change one byte order or one field and `verify()` fails. They must either keep the exact layout (copy) or ship a new proof format and **migrate all verifiers** to the new format. Migration is costly and breaks "drop-in replacement" story.  
- **Cache:** `CacheEntry::to_result()` in semantic_cache.rs (lines 143–151) reads `self.proof.fuel_consumed`, `self.proof.peak_memory`, `self.proof.duration_ms`. If they switch to nexus-pcu’s ExecutionProof, that type has different field names/structure; CacheEntry and executor would need to be updated in lockstep. If they change field order in signing_bytes, every proof signed before the change is unverifiable by new code.

**Hidden coupling:** The proof is the **single** audit record. Its layout is the implicit schema. There is no version field in the current proof; adding one would require (a) defining versions, (b) supporting multiple verification paths, (c) migrating stored proofs. The codebase has no proof migration path — so the first layout is the contract.

**Forces re-invention:** They must either (a) keep the exact proof layout and signing_bytes() (effective copy) or (b) define a new proof format, implement new sign/verify, update executor + cache + any downstream consumer, and accept that old proofs are legacy. (b) is re-invention of the trust boundary.

---

#### Failure 3: Replicas diverge under concurrent writes

**Location:** `nexus-core/src/causal.rs` — `CausalTensor::merge` (342–387), `merge_data` (376–389).

**What adversary did:** Replaced `merge_data` with timestamp-based LWW or "pick local" or "pick remote by node rank."

**Why it fails:**  
- **Determinism:** Merge must be **deterministic** so that any two nodes that receive the same (local, remote) produce the **same** merged tensor. If merge uses wall-clock time, nodes with different clocks diverge. If merge uses "node rank," then the outcome depends on who merged first (order-dependent). The current rule: merge_data compares BLAKE3(local) and BLAKE3(remote) as bytes and returns the larger. Same inputs ⇒ same output; no clock, no node id in the comparison.  
- **Commutativity:** The merge is commutative (local, remote) and (remote, local) yield the same result because (1) clock merge is max, (2) parents are sorted (line 380), (3) merge_data is symmetric in the two inputs. If they use timestamp, (A,B) and (B,A) can differ when clocks differ.

**Hidden coupling:** Storage (ProvenanceLog) and sync assume that "merged tensor" is uniquely determined by the two input tensors. Any component that replays or re-merges (e.g. after crash) assumes the same merge result. Non-deterministic or non-commutative merge breaks replay and eventual consistency.

**Forces re-invention:** They must re-derive a **deterministic, commutative** merge rule for opaque byte payloads. Options: (1) Keep hash-based LWW (same idea). (2) Use a CRDT for the payload (requires structured payload and conflict semantics — not generic bytes). (3) Use a single "merge authority" (centralization). So for generic causal state, the design space is narrow; the current merge_data is the minimal deterministic choice.

---

#### Failure 4: PCU identity and proof/cache binding break

**Location:** `nexus-pcu/src/pcu.rs` — `compute_id` (199–221), `semantic_hash` (266–283).  
`nexus-executor/src/executor.rs` — uses SemanticKey::from_pcu(pcu, inputs, identity); proof creation uses pcu and context.identity.

**What adversary did:** Removed identity from PCU; PCU id = hash(code, inputs, parameters) only. Identity checked via JWT in request context.

**Why it fails:**  
- **Proof binding:** ExecutionProof in executor includes identity_hash (identity.content_hash()). Proof is signed over that. If PCU no longer carries identity, then (a) proof must get "identity" from somewhere else (e.g. JWT at execution time). Then the **cached** result was produced for identity A; later request with "identity B" and same (code, inputs) would hit cache — and the cached proof says identity A. So they are back to "cache hit must return proof for the same identity" — i.e. identity must be part of the cache key, and the proof must bind to that identity. So identity must be in the execution context that is part of the cache key. That is exactly "identity in PCU" or "identity in SemanticKey."  
- **PCU id:** If PCU id does not include principal, then two principals with the same (code, inputs, params) get the same PCU id. That is fine for content-addressability of "the computation," but then **semantic_hash** (used for cache) must still differ by principal, or cache is shared. So at least one of PCU.id or the cache key must include principal. The codebase puts principal in both (compute_id and semantic_hash). If they remove from PCU, they must add to cache key and to proof — and then they have recreated the same binding in a different place. The "embedding" in PCU is what keeps one source of truth.

**Hidden coupling:** Executor does not accept "PCU + separate JWT." It accepts (pcu, context) and context has identity. The cache key is derived from pcu + context.inputs + context.identity. So identity is required at the execution API. If they move identity to "HTTP layer," they still need to pass identity into the execution layer for cache and proof. The coupling is "identity is an input to execution and to cache key and to proof."

**Forces re-invention:** They must either keep identity inside the computation unit (or equivalent execution request) or redesign so that (a) cache key includes a principal/tenant from somewhere, (b) proof includes that principal, (c) verification accepts that. That is the same logical design; they are just moving where principal lives (PCU vs request envelope). If they truly remove identity from the execution path, they cannot have both cache and per-identity audit.

---

#### Failure 5: Identity validity and delegation

**Location:** `nexus-pcu/src/identity.rs` — `IdentityContext::is_valid()` (348–375), `signing_data()` (405–413), delegation chain validation.

**What adversary did:** Replaced with "check JWT signature and expiry only" or "check API key."

**Why it fails:**  
- **Delegation:** If they drop DelegationChain, they lose "acting on behalf of" and effective_principal(). If they keep it but change the validity rules (e.g. different expiry semantics), then any component that relies on is_valid() (executor) will accept or reject differently. The executor has a single check: pcu.identity.is_valid(). So the **contract** is "valid identity = not expired, delegation valid, signature valid." If they loosen this (e.g. ignore delegation), they weaken security; if they tighten, they may break existing clients.  
- **Signature binding:** signing_data() includes principal, capabilities, delegation, valid_until. So the signature binds "who, what they can do, and until when." If they replace with JWT, the JWT payload is their new "signing_data." The **invariant** is "execution is allowed iff the signed identity context is valid." Any replacement must preserve that invariant or the system no longer has "computation carries proof of authorization."

**Hidden coupling:** PCU creation (in server/CLI) builds IdentityContext and signs it. Executor only **validates**; it does not issue identity. So the issuer and the executor must agree on the validity rules. If the fork changes validity (e.g. no delegation), any issuer that uses delegation is broken. If they keep delegation, they have to implement the same chain verification (continuity, expiry, signature per link) — which is non-trivial and easy to get wrong.

**Forces re-invention:** To avoid copying, they need a different auth model that still gives (a) principal bound to execution, (b) cache key including principal, (c) proof including principal, (d) validity check at execution. That is either the same idea (signed context) or a different trust model (e.g. TEE attestation only) which is a larger re-architecture.

---

### Summary: what blocks the fork

| Fork action | Failure mode | Hidden coupling / invariant | Re-invention? |
|-------------|-------------|-----------------------------|---------------|
| Drop identity from cache key | Cross-tenant cache returns; wrong proof for principal | Cache key is the only tenant bound; executor has no separate tenant id | Yes: multi-tenant cache design |
| Change proof layout / signing_bytes | Verifiers fail; cache entry proof fields mismatch | Proof is single audit schema; no versioning | Yes: new proof format + migration |
| Change merge_data (e.g. timestamp LWW) | Non-deterministic or non-commutative merge; divergence | Storage/sync assume deterministic merge | Yes: deterministic merge for bytes |
| Remove identity from PCU | Proof and cache cannot bind to principal; or duplicate binding elsewhere | Identity is input to cache key and proof | Yes: same binding in different shape |
| Replace identity validity with JWT-only | Delegation and binding semantics change; issuer/executor contract breaks | is_valid() is the single contract | Yes: auth model + validity contract |

**Modules that force re-invention from first principles if changed:**

1. **nexus-core/causal.rs** — merge_data + sorted parents + idempotence/happens_before. Deterministic, commutative merge for opaque bytes.
2. **nexus-executor/semantic_cache.rs** — SemanticKey (code_hash, inputs_hash, identity_hash). Multi-tenant cache key.
3. **nexus-executor/proof.rs** — ExecutionProof layout and signing_bytes(). Single verifiable audit record.
4. **nexus-pcu/pcu.rs** — compute_id and semantic_hash including identity.principal. Unit of computation and cache/proof binding.
5. **nexus-pcu/identity.rs** — is_valid(), signing_data(), delegation. Definition of "authorized computation."

---

## Part 2 — Five Patent Claim Skeletons (Engineer-Readable)

Each skeleton: Title, Technical field, Problem, Core mechanism (code refs), Why existing systems cannot do this, Claim boundary (excluded). No legal prose.

---

### Claim 1 — Deterministic concurrent merge of signed causal state

**Title:** Deterministic commutative merge of causally tagged, signed state objects.

**Technical field:** Distributed systems; causal consistency; conflict resolution.

**Problem:** Concurrent updates to the same logical state on different nodes must be merged so that (1) all nodes converge to the same result, (2) the result does not depend on merge order or wall-clock time, (3) the merged state remains verifiable (signed).

**Core mechanism (code):**

- State object: `CausalTensor` (nexus-core/src/causal.rs): id (content-addressed), data, provenance (parents), vector clock, signature over (id ‖ data ‖ provenance.merkle_root).
- Merge: `CausalTensor::merge` (causal.rs:342–387). (1) If local.id == remote.id, return local (idempotence). (2) If local.clock happens_before remote.clock, return remote; if remote happens_before local, return local (causal monotonicity). (3) Else concurrent: merge clocks (max per node); merge data via `merge_data(local.data, remote.data, lca)`; parents = sorted dedup of [local.id, remote.id]; create new tensor with merged data and parents, sign with node key.
- Determinism: `merge_data` (causal.rs:376–389) compares BLAKE3(local) and BLAKE3(remote) as bytes; returns the byte-slice with the larger hash. No timestamp, no node id in comparison. Parents sorted (line 380) so order is canonical.

**Why existing systems cannot do this:**

- CRDTs: Require structured payload and type-specific merge (e.g. LWW-register, counter). This is for **opaque bytes** with a single deterministic rule.
- OT: Requires central or sequenced operations; this is peer-to-peer, order-independent.
- Timestamp LWW: Depends on synchronized clocks; this is clock-free for the data choice.
- "Merge by node priority": Not commutative; this is commutative.

**Claim boundary (excluded):**

- Application-defined or type-specific merge (e.g. CRDT per type).
- Merge that uses wall-clock time or node rank to choose payload.
- Unsigned or unkeyed state objects.
- Merge that is not deterministic for the same (local, remote) pair.

---

### Claim 2 — Computation unit with identity in its content-addressed identity and cache key

**Title:** Portable computation unit whose content-addressed identity and semantic cache key include the requesting principal.

**Technical field:** Distributed computation; sandboxed execution; multi-tenant caching.

**Problem:** Same (code, inputs) may be executed by different principals. Caching by (code, inputs) alone would share results across principals and break isolation and audit. The computation unit and cache must bind to "who requested it."

**Core mechanism (code):**

- Unit: `PCU` (nexus-pcu/src/pcu.rs). Fields: code (WasmModule), inputs (ContentHash), parameters, identity (IdentityContext), constraints.
- Content-addressed id: `compute_id` (pcu.rs:199–221). BLAKE3(code.hash ‖ inputs ‖ parameters ‖ identity.principal). So id depends on principal.
- Semantic hash (for cache): `semantic_hash` (pcu.rs:266–283). BLAKE3(code.hash ‖ sorted(inputs) ‖ parameters ‖ identity.principal). So cache key differs by principal.
- Cache key: `SemanticKey::from_pcu` (nexus-executor/src/semantic_cache.rs:46–56). code_hash = pcu.code.content_hash(); inputs_hash = combine_inputs(inputs); identity_hash = identity.content_hash(). Key = (code_hash, inputs_hash, identity_hash).

**Why existing systems cannot do this:**

- Content-addressable caches (e.g. Nix, IPFS): Key by content hash of (code, inputs) only; no principal. Results are shared; no per-principal isolation.
- Memoization: Typically key by (function, args); no notion of "principal" in key.
- "Separate auth layer": Auth checks who is calling, but cache key is still (code, inputs). To get isolation, cache must be keyed by principal or equivalent — which is "identity in key."

**Claim boundary (excluded):**

- Cache key that does not include a principal or identity-derived value.
- Computation unit id that does not include principal (e.g. code+inputs only).
- Isolation achieved only by separate caches per principal with no shared key space (different design).

---

### Claim 3 — Execution method with identity-validated unit, semantic cache lookup, and proof on every result

**Title:** Method of executing a portable computation unit with embedded identity, semantic cache keyed by identity, and verifiable proof returned for both cache hits and first execution.

**Technical field:** Verifiable computation; execution substrates; audit trails.

**Problem:** Execution must be auditable (every result has a proof) and efficient (cache repeats). Caching must not break audit (cache hit must still return a proof binding the same principal and result).

**Core mechanism (code):**

- Input: (pcu, ExecutionContext) where context has inputs and identity (nexus-executor/src/executor.rs).
- Validate: pcu.code.is_valid_header(), size ≤ MAX_MODULE_SIZE, pcu.identity.is_valid() (executor.rs:114–125).
- Cache key: SemanticKey::from_pcu(pcu, context.inputs, context.identity) (executor.rs:130).
- Lookup: self.cache.get(&semantic_key) (executor.rs:134). If Some(entry), return ExecutionResponse::new(entry.to_result(), entry.proof, true) (executor.rs:151).
- On miss: Wasmtime run; extract_output; generate_proof (ExecutionProof::create(pcu, inputs, result, identity, signing_key)); cache.put(key, result, proof); return ExecutionResponse::new(result, proof, false) (executor.rs:224–234).
- Proof: ExecutionProof (executor proof.rs) includes pcu_hash, input_hashes, output_hash, identity_hash, executor_node, duration, fuel, memory, attestation. Signed over signing_bytes().

**Why existing systems cannot do this:**

- Traditional memoization: No proof; no identity in key; no "every result verifiable."
- TEE-only attestation: Proof is "I ran in enclave," not "this principal, this code, these inputs, this output."
- "Proof only on first run": Audit gap for repeated executions; this requires proof on hit too (cached proof returned).

**Claim boundary (excluded):**

- Execution that does not validate embedded identity before run.
- Cache that does not store and return a proof with the cached result.
- Cache key that omits identity (see Claim 2).
- Proof that omits identity_hash or output_hash or code/input binding.

---

### Claim 4 — Semantic cache for computation results keyed by code, inputs, and identity

**Title:** Semantic cache for computation results whose key includes code hash, combined input hashes, and identity hash, and whose entries store an execution proof.

**Technical field:** Caching; multi-tenant execution; verifiable computation.

**Problem:** Deduplication of execution must respect principal: same (code, inputs) for different principals must not share one cache entry. Entries must be verifiable (proof stored with result).

**Core mechanism (code):**

- Key: SemanticKey (nexus-executor/src/semantic_cache.rs:34–80). code_hash, inputs_hash, identity_hash. from_pcu uses pcu.code.content_hash(), combine_inputs(inputs), identity.content_hash().
- Entry: CacheEntry (semantic_cache.rs:81–151). output, output_hash, proof (ExecutionProof), cached_at, expires_at. to_result() builds ExecutionResult from proof.fuel_consumed, peak_memory, duration_ms.
- Lookup: get(key) returns Option<CacheEntry>; on hit returns entry including proof (executor uses this at executor.rs:134–151).
- Insert: put(key, result, proof, ttl); eviction when at capacity (evict_one).

**Why existing systems cannot do this:**

- Key-value caches: Key is typically request id or (code, args); no identity. This key explicitly includes identity_hash.
- "Separate permission check": Would require storing identity with entry and checking on hit; this key makes the entry identity-specific by construction, so no post-hit check needed for isolation.

**Claim boundary (excluded):**

- Cache key that is (code, inputs) only or (code, inputs, request_id) without principal/identity.
- Cache entry that does not store a verifiable execution proof with the result.
- Isolation achieved only by "check caller after hit" without identity in key.

---

### Claim 5 — Identity context signed and validated at execution without external auth call

**Title:** Identity context signed by principal and embedded in the computation unit, validated at execution time without calling an external auth service.

**Technical field:** Authorization; distributed execution; capability-based access.

**Problem:** Execution node must decide "is this computation authorized?" without a round-trip to an auth server. The authorization must be bound to the computation unit and verifiable.

**Core mechanism (code):**

- Context: IdentityContext (nexus-pcu/src/identity.rs). principal, capabilities, delegation (Option<DelegationChain>), valid_until, signature.
- Signing: sign(signing_key) (identity.rs:395–403). signing_data = principal ‖ bincode(capabilities, delegation, valid_until). Ed25519 sign.
- Validity: is_valid() (identity.rs:348–375). now < valid_until; if delegation present, chain.is_valid(now) and principal == final_delegatee; verify() signature.
- Embedding: PCU.identity is IdentityContext (pcu.rs). Executor checks pcu.identity.is_valid() before execution (executor.rs:124–125). No HTTP call to auth service; validity is local check.

**Why existing systems cannot do this:**

- JWT in header: Server validates JWT; identity is not **inside** the computation unit. Here the unit carries the proof of authorization; the unit can be forwarded or replayed without a separate "session."
- Auth service call: Traditional "check token with auth server" adds latency and dependency; this is offline-valid (signature + expiry + delegation chain).

**Claim boundary (excluded):**

- Validation that requires a synchronous call to an external auth or token service.
- Identity that is only in request metadata (e.g. header) and not in the serialized computation unit.
- No signature binding principal and capabilities and expiry (e.g. unsigned "principal id" only).

---

## Part 3 — Complexity Without Defensibility; Narrative Dilution; Recommended Cuts

### Code that adds complexity but no defensibility

| Item | Location | Why no defensibility | Recommendation |
|------|----------|------------------------|----------------|
| WorkloadOptimizer.calculator | nexus-core/src/cost_optimizer.rs | Field never read; suggest_placement uses heuristics only. Cost-aware placement is not implemented. | **Delete** the calculator field and any CostCalculator use in WorkloadOptimizer, or implement cost in suggest_placement. Prefer delete to avoid "we do cost-based placement" claim with no implementation. |
| cost_optimizer module | nexus-core/src/cost_optimizer.rs | Cost models, ROICalculator, SalesReport: sales/ROI only. Not on execution or merge path. No patent or system dependency. | **Move** to `experimental/` or `nexus-core/sales/` (or delete if not needed). Justification: Reduces "core" surface; avoids implying cost is part of the execution invariant. |
| migration module | nexus-core/src/migration.rs | KubernetesMigrator, etc.: structs hold config but no k8s client or real migration. Stub only. | **Move** to `experimental/migration/` or feature-gate behind `migration-stub`. Justification: "Zero-downtime migration" is a strong claim; current code does not implement it. Isolating avoids diluting "causal merge + execution + proof" narrative. |
| tenancy module | nexus-core/src/tenancy.rs | TenantManager, quotas: no consumer in executor, sync, or storage in audited code. | **Keep** but do not cite in patent narrative until a consumer (e.g. server) uses it. Alternatively **move** to experimental if no near-term use. Justification: Tenancy is useful product-wise but not part of the five load-bearing modules; keeping it in core without use suggests unimplemented "multi-tenant enforcement" in core. |
| Provenance.depth | nexus-core/src/causal.rs | Set to 0 or 1 "for MVP"; not used in merge or LCA. | **Leave** as-is but document as "reserved for future DAG depth." No deletion needed; just avoid claiming "deep provenance" until used. |
| SemanticCache.route() and data_locations | nexus-executor/src/semantic_cache.rs | route() and register_data/get_data_locations not used in execute() path. | **Keep** code but **do not** claim "data locality routing" in patent until execute() uses route(). Justification: Claiming routing without using it weakens clarity; the defensible part is the key (get/put), not routing. |
| nexus-pcu proof vs executor proof | nexus-pcu/src/proof.rs vs nexus-executor/src/proof.rs | Two ExecutionProof types; executor path uses executor’s only. pcu proof has builder; executor has create(). | **Document** clearly: "Production execution proof is executor’s ExecutionProof; pcu proof is for external/builders." Consider re-exporting one or the other to avoid "we have two proof formats" confusion. Do not delete; unify narrative. |
| nexus-runtime | nexus-runtime/src/lib.rs | Only `pub struct WasmExecutor;` — no methods, no impl. | **Delete** or **implement** minimal wrapper (e.g. re-export executor). Justification: Stub suggests "runtime" is a component; it is not. Deleting removes a false narrative. |

### Modules that dilute the core narrative

| Module | Why it dilutes | Recommendation |
|--------|----------------|----------------|
| homeostasis-engine, autonomic-system, developmental-gates, nervous-system | Not on PCU execution path. Claiming "AI safety through NEXUS" implies these gate execution; they do not. | **Option A:** Wire at least one (e.g. nervous-system) into executor or a "safety gate" before execute(), then cite. **Option B:** Isolate under `experimental/safety-stack/` and **do not** claim "AI safety" in the same claim as "verifiable execution." Narrative: "Verifiable execution" = causal + PCU + executor + proof; "Safety stack" = separate, experimental. |
| telos-protocol | Not called from executor or server. "Cognitive accountability" / commitment membrane is not on the execution path. | **Isolate** as `experimental/telos/` or separate repo until integrated (e.g. "cross membrane before execute"). Do not claim "TELOS enforces accountability on execution" until there is a call path. |
| nexus-agp | Bridge to AGP (Python); not on executor path. Patent narrative is "execution + cache + proof + identity." | **Keep** as product integration. Do **not** put AGP-specific claims (e.g. "AHES") in the same claim set as the five load-bearing modules. Narrative: "NEXUS execution" vs "AGP integration via nexus-agp." |
| agp, agp-core (Python) | Separate governance/agents/OS; different codebase and narrative. | Keep as separate product. Do not mix "AGP governance" with "NEXUS verifiable execution" in one claim. |
| nexus-core-v2 | Standalone binary; name collision with nexus-core; not in workspace. | **Rename** (e.g. nexus-log-replay) and document as reference/legacy, or **move** to `experimental/nexus-core-v2/`. Do not cite in patent as "core" — the core is nexus-core (causal, merge). |

### What weakens patent clarity

- **Multiple "proof" types** (pcu vs executor): Clarify in spec and claims: "Execution proof = the proof produced by the executor (ExecutionProof with create() and signing_bytes())."
- **"Causal tensor" vs "USO"**: Both have merge and history. Claims should pick one primary (e.g. CausalTensor for merge, USO for sync/state) or state explicitly: "state object with causal history and deterministic merge."
- **Safety stack / TELOS not wired**: Any claim that says "NEXUS enforces AI safety" or "TELOS gates execution" is weak until code path exists. Either wire or remove from claim narrative.
- **Cost / migration in core**: Suggests "core" does more than causal state + merge. Moving them out keeps "core" = causal + merge only.

### Summary of recommended actions

| Action | Target | Justification |
|--------|--------|----------------|
| **Delete** | WorkloadOptimizer.calculator (or implement) | Dead code; no defensibility. |
| **Delete or implement** | nexus-runtime (WasmExecutor stub) | No runtime behavior; removes false narrative. |
| **Move to experimental/** | cost_optimizer (or sales/); migration stub; optionally tenancy | Keeps core = causal + merge; avoids unimplemented claims. |
| **Move to experimental/** | Safety stack (homeostasis, autonomic, developmental, nervous-system) unless wired | Stops "AI safety" dilution until execution path exists. |
| **Move to experimental/** | telos-protocol unless wired to execution | Stops "accountability on execution" until integration. |
| **Document / unify** | Proof: executor vs pcu; SemanticCache: get/put vs route | Single narrative: execution proof format; cache key; routing is future. |
| **Rename / relocate** | nexus-core-v2 | Avoid name collision and "two cores" confusion. |
| **Do not claim** | Data locality routing; deep provenance; cost-based placement; zero-downtime migration | Until implemented in the main path. |

---

**End of document.**  
Fork failure points and re-invention points are in Part 1; patent skeletons in Part 2; complexity and cuts in Part 3.
