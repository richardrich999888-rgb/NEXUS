# NEXUS Project — Components Audit

**Date:** 2025-01-29  
**Scope:** All components inventoried in `docs/PROJECT_COMPONENTS.md` (Rust crates, Python packages, formal specs).  
**Companion:** Core crate API/dead-code details in `docs/CORE_SRC_AUDIT.md`.

---

## 1. Rust — Safety / ASI Stack

### 1.1 homeostasis-engine

| Item | Status |
|------|--------|
| **Purpose** | Constraint enforcement for bio-inspired ASI safety (metrics, bounds, setpoints, negative feedback). |
| **Modules** | `core` (bounds, metric, setpoint, feedback), `controller` (single_metric, multi_objective), `constraints` (hard/soft bounds), `solver`, `diagnostics` (health), `integration` (endocrine_bridge). |
| **Public API** | `HardBounds`, `Metric`, `MetricId`, `SingleMetricController`, `MultiObjectiveController`, `HealthStatus`, `HealthCheck`; prelude provided. |
| **Dependencies** | thiserror, serde, serde_json, tracing; optional nalgebra (`advanced-solver`). |
| **Tests** | `single_metric_tests`, `multi_objective_tests`, `integration_tests`. |
| **Findings** | ✅ Well-documented; no NEXUS crate deps; suitable as substrate. |

### 1.2 multi-asi-immune

| Item | Status |
|------|--------|
| **Purpose** | Distributed immune protocol: Ed25519 identity, reputation, threat signatures, mutual constraints, defection detection. |
| **Modules** | `identity`, `attestation`, `threat`, `reputation`, `protocol`, `enforcement`, `node`, `integration`. |
| **Public API** | `AsiId`, `AsiNode`, `ReputationScore`, `ThreatPattern`, `SignedThreatReport`, `ProtocolMessage`, `NodeConfig`, `NetworkHealth`; prelude. |
| **Dependencies** | homeostasis-engine, ed25519-dalek, rand, sha2, serde, thiserror, tracing. |
| **Tests** | identity, reputation, threat_propagation, defection, integration. |
| **Findings** | ✅ Depends only on homeostasis-engine; no nexus-core/pcu. |

### 1.3 autonomic-system

| Item | Status |
|------|--------|
| **Purpose** | ACT (action) vs CALM (contemplation) mode; arousal, transitions, reflex responses; homeostasis integration. |
| **Modules** | `mode` (controller, state), `reflex` (response), `regulation` (transition). |
| **Public API** | `AutonomicController`, `ControllerConfig`, `AutonomicMode`, `Arousal`, `ReflexResponse`, `ReflexType`, `ModeTransition`, `TransitionTrigger`. |
| **Dependencies** | homeostasis-engine, thiserror, serde, tracing. |
| **Findings** | ✅ Thin layer over homeostasis; no dev-deps/tests in Cargo.toml (tests may be in integration elsewhere). |

### 1.4 developmental-gates

| Item | Status |
|------|--------|
| **Purpose** | Staged capability unlock (Stage 0–3); stage manager, capability registry, gate enforcer. |
| **Modules** | `stage` (manager, definition), `gate` (enforcer), `capability` (registry). |
| **Public API** | `StageManager`, `StageConfig`, `DevelopmentalStage`, `StageRequirements`, `GateEnforcer`, `AccessResult`, `Capability`, `CapabilityRegistry`. |
| **Dependencies** | homeostasis-engine, thiserror, serde, tracing. |
| **Findings** | ✅ Clear API; no NEXUS deps. |

### 1.5 nervous-system

| Item | Status |
|------|--------|
| **Purpose** | Coordinator: perception → decision engine → motor; integrates homeostasis, autonomic, developmental-gates, multi-asi-immune. |
| **Modules** | `perception`, `decision`, `motor`, `integration` (coordinator, safety). |
| **Public API** | `NervousSystemCoordinator`, `CoordinatorConfig`, `SafetyState`, `SafetySummary`. |
| **Dependencies** | homeostasis-engine, autonomic-system, developmental-gates, multi-asi-immune, thiserror, serde, tracing. |
| **Findings** | ✅ Single integration point for all four safety layers. |

---

## 2. Rust — TELOS, AGP Bridge, Edge, Secrets, CLI, Server, Runtime

### 2.1 telos-protocol

| Item | Status |
|------|--------|
| **Purpose** | Cognitive accountability: commitment membrane, entropy meter, authority registry, validator network, trust accumulator. |
| **Modules** | `membrane`, `entropy`, `authority`, `validator`, `trust`, `error`, `merkle`, `ledger`, `vdf`, `network`. |
| **Public API** | `CommitmentMembrane`, `Decision`, `CrossingResult`; `EntropyMeter`, `ConsequenceTier`, `EntropyProof`; `AuthorityRegistry`, `AgentId`; `Validator`, `Attestation`, `ValidatorNetwork`; `TrustAccumulator`, `TrustScore`, `CommitmentHistory`; `MerkleTree`, `Ledger`, `VdfGenerator`/`VdfVerifier`, `NetworkCoordinator`. |
| **Dependencies** | thiserror, serde, serde_json, sha2, uuid, hex, ed25519-dalek, rand, chrono; optional tokio. |
| **Findings** | ✅ Standalone; no NEXUS deps; Vanian Technologies; schemas in `schemas/`, docs in `docs/`. |

### 2.2 nexus-agp

| Item | Status |
|------|--------|
| **Purpose** | Bridge AGP (Python) ↔ NEXUS: identity, reputation CRDT, verification; AHES (endocrine, glands, homeostasis). |
| **Modules** | `identity`, `reputation`, `verification`, `endocrine`, `glands`, `homeostasis`. |
| **Public API** | `NexusAgentIdentity`, `AgentRegistration`; `ReputationCRDT`, `ReputationProof`; `NexusVerifier`, `VerificationResult`; `Hormone`, `HormoneLevel`, `EndocrineState`; `Gland`, `GlandularSystem`; `HomeostasisController`, `HealthStatus`, `SetPoint`. |
| **Dependencies** | nexus-pcu, causalux-v2, sha2, hex, serde, bincode, tokio; optional `pqc` (nexus-pcu/pqc). |
| **Findings** | ✅ Patent claims documented in lib.rs; single Rust entry point for AGP ↔ NEXUS. |

### 2.3 nexus-edge

| Item | Status |
|------|--------|
| **Purpose** | Cloudflare Worker: health, benchmark endpoints (causal merge, USO creation, latency percentiles, hash lookup), POST /api/uso. |
| **Crate type** | `cdylib` + `rlib`; entry is `#[event(fetch)]` in `src/lib.rs` (worker fetch handler). |
| **Dependencies** | worker, serde, serde_json, console_error_panic_hook, nexus-pcu, causalux-v2, hex, js-sys, wasm-bindgen, getrandom, uuid. |
| **API consistency** | Uses `VersionVector::merge` (causalux), `USO::new`, `PrincipalId::from_bytes` (nexus-pcu) — APIs match. |
| **Findings** | ✅ Functional; debug/benchmark routes; RNG and CPU limits noted for free tier. |

### 2.4 nexus-secrets

| Item | Status |
|------|--------|
| **Purpose** | Secret management: encrypted at rest; backends Local (encrypted file), Vault, AWS, K8s (all optional). |
| **Modules** | `backend`, `encryption`, `manager`, `error`. |
| **Public API** | `SecretBackend`, `SecretBackendType`, `SecretManager`, `SecretError`, `SecretResult`, `Secret` (zeroized on drop). |
| **Backends** | Local always; vault/aws/k8s behind features. |
| **Findings** | ✅ `Secret` type and manager API clear; optional backends correctly gated. |

### 2.5 nexus-cli

| Item | Status |
|------|--------|
| **Purpose** | CLI: init, node run/peer, pcu submit; uses nexus_pcu, nexus_sync, nexus_network (SyncNode). |
| **Entry** | `src/main.rs`; clap subcommands. |
| **Findings** | ✅ Wires core, pcu, sync, network; tracing subscriber; no stub commands found in sampled code. |

### 2.6 nexus-server

| Item | Status |
|------|--------|
| **Purpose** | HTTP API: health, benchmark (causal merge, USO creation), sync stats, POST /api/uso; axum + CORS. |
| **State** | `AppState`: NexusSyncEngine, node_id, start_time. |
| **Dependencies** | causalux_v2, nexus_pcu, nexus_sync, axum, tower_http (cors). |
| **Findings** | ✅ Functional server; no nexus-core direct use (sync/pcu only). |

### 2.7 nexus-runtime

| Item | Status |
|------|--------|
| **Contents** | Single type: `pub struct WasmExecutor;` (no methods, no impl). |
| **Findings** | 🟡 **Stub only.** Placeholder for future WASM runtime; no dependency on nexus-executor or nexus-pcu. |

---

## 3. Rust — nexus-core-v2 (Not in Workspace)

| Item | Status |
|------|--------|
| **Purpose** | Standalone binary: deterministic execution log, algebraic merge, WASM execution, replay (init, exec, replay, status). |
| **Structure** | No `lib.rs`; binary with `main.rs`; modules: core, errors, executor, hash, log, merge, op, replay, storage, sync. |
| **Cargo.toml** | name = "nexus-core" (collides with workspace crate name if ever added); deps: sha2, bincode, serde, wasmtime, anyhow, thiserror, clap. |
| **Findings** | 🟡 **Out of workspace;** name collision with `nexus-core`; separate lineage (own executor, log, storage). Recommend: document as reference/legacy or add to workspace under distinct name (e.g. `nexus-core-cli`). |

---

## 4. Python Components

### 4.1 agp/

| Item | Status |
|------|--------|
| **Purpose** | Agent Governance Protocol demo: task clustering, governance (proposals, voting), verification, reputation; adapters (memory storage, mock proof). |
| **Entry** | `main.py` (demo script); imports from `core` (types, governance, reputation, verification, task_clustering). |
| **Findings** | ✅ Self-contained demo; `sys.path.insert` in main.py is brittle for portability — consider package install or env. |

### 4.2 agp-core/

| Item | Status |
|------|--------|
| **Purpose** | Full stack: FastAPI app, OS kernel, scheduler, FS, IPC, HAL, mesh, network, observability, persistence, recovery, resilience, ROS2/RTOS bridges, security; agents (ADK), AHES, API v1, compliance, governance, immunity (adaptive, antibody, experiments, training), ML (RAG, deep learning), services (reputation, blockchain, webhooks, etc.), TELOS membrane. |
| **Entry** | `src/main.py` (FastAPI, lifespan, CORS, Prometheus, API v1 router). |
| **Packaging** | No root `pyproject.toml`; `requirements.txt`, `setup_immunity.py`, Docker, docker-compose. |
| **Findings** | ✅ Large but structured; Solidity contracts, benchmarks, deploy configs present. 🟡 Add root `pyproject.toml` or document canonical install (e.g. `pip install -e .` from a package dir) for consistency. |

### 4.3 nexus-telecom/

| Item | Status |
|------|--------|
| **Purpose** | NEXUS Telecom: Wake-up receiver (RI-WuR), Lyapunov safety controller, energy/entropy models; FYNTRAX + 6G RAN. |
| **Packaging** | `pyproject.toml`, `setup.py`; package in `src/nexus_telecom`. |
| **Modules** | `ran` (WakeUpReceiver, WuRConfig, WakeUpSignal), `control` (LyapunovController, create_identity_controller), `models` (EnergyModel, EntropyCalculator, SiteConfig). |
| **Findings** | ✅ Implementations are substantive (detection, Lyapunov V/dV/is_safe/filter_action, energy/entropy formulas). API matches `__all__` in `__init__.py`. |

### 4.4 syntriass/

| Item | Status |
|------|--------|
| **Purpose** | Real-time generative AI preview (Path 6): inference tap, fast decoder, temporal interpolation, conditioning, WebSocket API, Gradio frontend, Diffusers patch. |
| **Entry** | Demos: `demos/image_preview.py`; UI: `front/gradio_app.py`. |
| **Findings** | ✅ Architecture documented in README; Python package layout (core, preview, api, front, patch, demos). Dependency on diffusers/websockets per requirements.txt; import tests exist. |

---

## 5. Formal Specs

### 5.1 brain/formal_specs/SafetyProtocol.tla

| Item | Status |
|------|--------|
| **Purpose** | TLA+ spec of NEXUS Autonomic Safety Protocol: mode (CALM, ACT, EMERGENCY, RECOVERY), arousal, duration, reflex_queue. |
| **Actions** | IncreaseArousal, DecreaseArousal, Tick (mode transition with MIN_DURATION), TriggerReflex. |
| **Invariants** | TypeOK (mode ∈ MODES, arousal in range, duration ∈ Nat, IsSafeSequence(reflex_queue)). |
| **Properties** | EmergencySafety (high arousal + duration ⇒ EMERGENCY), NoGhostStates, ReturnsToCalm (liveness). |
| **Findings** | ✅ Aligns with autonomic-system (ACT/CALM, arousal, transitions); suitable for model-checking (TLC). |

---

## 6. Cross-Component Consistency

| Check | Status |
|-------|--------|
| **nexus-edge ↔ nexus-pcu** | Uses `USO::new`, `PrincipalId::from_bytes`; APIs match. |
| **nexus-edge ↔ causalux** | Uses `VersionVector::merge`; causalux exports `merge`. |
| **nexus-agp ↔ nexus-pcu / causalux** | Bridge only; no type leakage. |
| **Safety stack ↔ autonomic** | nervous-system depends on autonomic-system, developmental-gates, multi-asi-immune, homeostasis-engine; TLA+ SafetyProtocol matches autonomic mode/arousal. |
| **agp (Python) ↔ nexus-agp (Rust)** | No direct Python→Rust call in audited code; nexus-agp is the intended bridge. |

---

## 7. Summary Table

| Component | Type | Status | Notes |
|-----------|------|--------|--------|
| homeostasis-engine | Rust | ✅ | Substrate; tests present. |
| multi-asi-immune | Rust | ✅ | Depends on homeostasis only. |
| autonomic-system | Rust | ✅ | Thin; no tests in Cargo.toml. |
| developmental-gates | Rust | ✅ | Clear API. |
| nervous-system | Rust | ✅ | Integrates all four safety crates. |
| telos-protocol | Rust | ✅ | Standalone; no NEXUS deps. |
| nexus-agp | Rust | ✅ | Bridge; patent claims in lib. |
| nexus-edge | Rust | ✅ | Worker; APIs consistent. |
| nexus-secrets | Rust | ✅ | Optional backends gated. |
| nexus-cli | Rust | ✅ | Wires core/pcu/sync/network. |
| nexus-server | Rust | ✅ | Axum API. |
| nexus-runtime | Rust | 🟡 | Stub only (WasmExecutor). |
| nexus-core-v2 | Rust | 🟡 | Out of workspace; name collision. |
| agp (Python) | Python | ✅ | Demo; path setup brittle. |
| agp-core (Python) | Python | ✅ | Large; add pyproject or doc install. |
| nexus-telecom (Python) | Python | ✅ | Implementations complete. |
| syntriass (Python) | Python | ✅ | Path 6 preview engine. |
| brain/SafetyProtocol.tla | TLA+ | ✅ | Matches autonomic design. |

---

## 8. Recommendations

1. **nexus-runtime:** Either implement a minimal wrapper around nexus-executor (or document as reserved) or remove to avoid dead stub.
2. **nexus-core-v2:** Add to docs (e.g. PROJECT_COMPONENTS.md + this audit) as “reference/legacy”; if bringing into workspace, use a distinct crate name and resolve `nexus-core` name collision.
3. **agp main.py:** Prefer package install or PYTHONPATH over hardcoded `sys.path.insert` for portability.
4. **agp-core:** Add root `pyproject.toml` or document canonical install (e.g. `pip install -e src/` or similar) for reproducibility.
5. **Formal verification:** Run TLC on `SafetyProtocol.tla` with chosen constants (CALM_THRESHOLD, ACT_THRESHOLD, EMERGENCY_THRESHOLD, MIN_DURATION, MAX_AROUSAL) and record results.

---

*Audit complete. For core NEXUS crate API and dead-code details see `docs/CORE_SRC_AUDIT.md`; for inventory see `docs/PROJECT_COMPONENTS.md`.*
