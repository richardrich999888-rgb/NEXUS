# NEXUS Project — Component Inventory

**Scope:** All components under `/Users/richardrich/Desktop/NEXUS` (desktop project root).  
**Purpose:** Discovery and audit of every major component (Rust crates, Python packages, formal specs, and related assets).

---

## 1. Rust Workspace Members (root `Cargo.toml`)

These are built with `cargo build --workspace`:

| Crate | Description |
|-------|-------------|
| **nexus-core** | Causal tensor algebra, IDs, tenancy, cost optimizer, migration (optional) |
| **nexus-pcu** | Portable Computation Units, USO, identity, proof, routing, content hash |
| **nexus-sync** | CAUSALUX integration, sync engine, CRDT-USO adapters |
| **nexus-compress** | VECTRA integration, PCU/USO compression |
| **nexus-storage** | Provenance log, algebraic index, query, backup |
| **nexus-network** | Messages, QUIC transport, TLS, gossip, sync, rate limit |
| **nexus-runtime** | Runtime glue |
| **nexus-cli** | CLI (core, pcu, sync, network, storage) |
| **nexus-server** | Server (core, pcu, sync, causalux) |
| **nexus-observability** | Metrics, health, logging, optional OpenTelemetry |
| **nexus-secrets** | Secret management (Vault, AWS, K8s optional) |
| **causalux** | CRDT/DAG sync, version vectors, conflict resolution, economy, resonance, atom (optional features) |
| **vectra/vectra** | Deterministic lossless compression (Rust) |
| **nexus-executor** | WASM executor, semantic cache, proof, host API |
| **nexus-edge** | Edge/WASM target (cdylib + rlib; worker, wasm-bindgen) |
| **nexus-benchmarks** | Competitive benchmark suite (vs Redis, Automerge) |
| **nexus-agp** | Bridge between AGP and NEXUS (pcu, causalux) |
| **homeostasis-engine** | Constraint enforcement for bio-inspired ASI safety |
| **multi-asi-immune** | Distributed immune protocol for multi-ASI coordination (depends on homeostasis-engine) |
| **autonomic-system** | Autonomic nervous system — ACT/CALM mode (depends on homeostasis-engine) |
| **developmental-gates** | Developmental gating for staged capability unlock (depends on homeostasis-engine) |
| **nervous-system** | Nervous system coordinator (depends on homeostasis-engine, autonomic-system, developmental-gates, multi-asi-immune) |
| **telos-protocol** | TELOS: Cognitive Accountability Protocol for ASI (Vanian Technologies) |

---

## 2. Rust Crates Not in Workspace

| Path | Description |
|------|-------------|
| **nexus-core-v2/** | Standalone “nexus-core” CLI: deterministic execution log, algebraic merge, WASM exec, replay. Own `Cargo.toml` (name `nexus-core`); **not** in workspace. See `docs/CORE_SRC_AUDIT.md` §3. |
| **vectra/VECTRA_2025-12-18/vectra/** | Snapshot/archive of VECTRA Rust crate; not workspace member. |

---

## 3. Python Components

| Path | Description |
|------|-------------|
| **agp/** | AGP (Agent Governance Protocol) — Python: governance, reputation, task clustering, verification; adapters (memory storage, mock proof). Entry: `main.py`. |
| **agp-core/** | Full AGP stack: agents (ADK), AHES (endocrine), API v1, compliance, governance (alignment, anomaly, behavioral RAG, enforcer, impact, rules), immunity (adaptive, antibody, experiments, training), ML (deep learning, RAG), OS (kernel, scheduler, FS, IPC, HAL, mesh, network, observability, recovery, resilience, ROS2, RTOS, security), services (reputation, blockchain, bridge, webhooks, etc.), TELOS membrane. Solidity contracts, benchmarks, deploy (Docker, systemd), dashboard. No root `pyproject.toml`; uses `requirements.txt`, `setup_immunity.py`. |
| **nexus-telecom/** | NEXUS Telecom — Python package: FYNTRAX + 6G RAN integration. `pyproject.toml`, `setup.py`. Modules: `control`, `models`, `ran`. Deps: numpy. |
| **syntriass/** | SYNTRIASS Path 6 — Real-time generative AI preview engine: inference tap, fast decoder, temporal interpolation, conditioning, scheduler, WebSocket API, Gradio frontend, Diffusers patch, demos (image/video/audio). Python; see `syntriass/README.md`. |
| **vectra/python/** | VECTRA Python bindings: encode, decode, FEE, EBTA, artifact. `pyproject.toml`, `setup.py`. |
| **vectra/fyntrax/** | FYNTRAX (Python). |
| **vectra/VECTRA_2025-12-18/python/** | Snapshot of VECTRA Python. |
| **vectra/telecom_6g/** | 6G telecom research: digital DPD, beamforming, VECTRA integration, benchmarks, examples. |

*(Excluding `agp-core/.venv` and other virtualenv/site-packages.)*

---

## 4. Formal / Specs

| Path | Description |
|------|-------------|
| **brain/formal_specs/SafetyProtocol.tla** | TLA+ formal specification (safety protocol). |

---

## 5. Root-Level and Cross-Cutting

| Item | Description |
|------|-------------|
| **tests/integration_e2e.rs** | Root-level integration test (not inside a crate). |
| **docs/** | Documentation (e.g. `BENCHMARKING.md`, `CORE_SRC_AUDIT.md`, `PROTOCOL_SPEC.md`, `CORE_CONTRACTS.md`). |
| **fly.toml** | Fly.io deployment config. |
| **GAPS_IMPLEMENTATION.md**, **WORLD_CLASS_UPGRADE_ROADMAP.md**, **TEST_RESULTS_SUMMARY.md** | Project notes and summaries. |

---

## 6. VECTRA Ecosystem (under `vectra/`)

- **vectra/vectra** — Workspace Rust crate (compression).
- **vectra/cpp/** — C++ implementation (encode, decode, FEE, EBTA, etc.).
- **vectra/python/** — Python API.
- **vectra/fyntrax/** — FYNTRAX Python.
- **vectra/telecom_6g/** — 6G use cases, DPD, beamforming, VECTRA integration.
- **vectra/VECTRA_2025-12-18/** — Dated snapshot (Rust, Python, fyntrax, telecom_6g).
- **vectra/docs/** — Architecture, API, patent/novelty, deployment, etc.

---

## 7. Dependency Overview (Rust)

- **Core chain:** `nexus-core` → `nexus-pcu` → (nexus-sync | nexus-storage | nexus-compress | nexus-observability) → nexus-network, nexus-executor, etc.
- **External libs in workspace:** `causalux`, `vectra/vectra`.
- **Safety/ASI chain:** `homeostasis-engine` → `multi-asi-immune`, `autonomic-system`, `developmental-gates` → `nervous-system`.
- **telos-protocol:** No NEXUS crate deps; standalone.
- **nexus-agp:** `nexus-pcu`, `causalux-v2`.
- **nexus-edge:** `nexus-pcu`, `causalux-v2`, `worker`, `wasm-bindgen` (WASM target).

---

## 8. Quick Reference — Where Things Live

| Concern | Location |
|--------|----------|
| Causal tensors, tenancy, migration | `nexus-core` |
| PCU, USO, identity, proof | `nexus-pcu` |
| CRDT/sync (CAUSALUX) | `causalux`, `nexus-sync` |
| Compression (VECTRA) | `vectra/vectra`, `nexus-compress` |
| Storage, provenance log | `nexus-storage` |
| Network, gossip, QUIC | `nexus-network` |
| WASM execution | `nexus-executor`, `nexus-edge` |
| Observability, health | `nexus-observability` |
| Secrets | `nexus-secrets` |
| AGP ↔ NEXUS bridge (Rust) | `nexus-agp` |
| AGP agents, OS, immunity (Python) | `agp`, `agp-core` |
| Telecom / 6G (Python) | `nexus-telecom`, `vectra/telecom_6g` |
| Real-time AI preview (Python) | `syntriass` |
| ASI safety / homeostasis | `homeostasis-engine`, `multi-asi-immune`, `autonomic-system`, `developmental-gates`, `nervous-system` |
| Cognitive accountability | `telos-protocol` |
| Formal safety spec | `brain/formal_specs/SafetyProtocol.tla` |

---

*Generated for NEXUS project desktop tree. For core crate API and dead-code details see `docs/CORE_SRC_AUDIT.md`. For audit of these components see `docs/COMPONENTS_AUDIT.md`.*
