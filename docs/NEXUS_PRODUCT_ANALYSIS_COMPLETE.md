# NEXUS Repository — Complete Product Analysis

**Scope:** Entire codebase, line-by-line analysis  
**Purpose:** Exact product inventory; what runs, what integrates, what stands alone  

---

## 1. Executive Summary: What NEXUS Actually Is

The NEXUS repository is a **multi-product monorepo** under SYNTRIASS Labs. It contains **7 distinct products/systems**, not one. The README describes "AURA Protocol"; the workspace is "NEXUS"; the DPR describes "NEXUS/AGP." All coexist.

**The core product (per DPR/patent strategy):**  
**NEXUS = Governed execution substrate** — WASM PCU execution with mandatory guards + AGP OS (agents as processes) + TELOS membrane.

---

## 2. Product Inventory (7 Products)

### Product 1: NEXUS — Governed Execution Substrate (Rust)

| Attribute | Detail |
|-----------|--------|
| **What it is** | Execution governance for portable computation: PCU (WASM) execution with mandatory guard, semantic cache, execution proofs, no-proof-on-deny |
| **Entry points** | `nexus-executor` (binary `nexus-exec`), `nexus-cli`, `nexus-server` |
| **Core crates** | nexus-pcu, nexus-executor, nexus-core, nexus-sync, nexus-network, nexus-storage, nexus-compress |
| **Runs as** | CLI (`nexus-exec <wasm_file>`), HTTP server (health, benchmark, USO API), P2P node |

**Exact entry points:**
- `nexus-executor/src/main.rs` → `nexus-exec` binary: loads WASM, builds PCU, runs via `ExecutorBuilder::production()`, outputs proof
- `nexus-cli/src/main.rs` → `nexus` CLI: `init`, `node run`, `pcu submit`
- `nexus-server/src/main.rs` → HTTP server on PORT: `/health`, `/api/benchmark/*`, `/api/sync/stats`, `/api/uso` (POST)

**Deliverable:** Executable that runs WASM in a governed sandbox; blocks execution when guard denies; produces cryptographic proofs for allowed execution.

---

### Product 2: AGP-CORE — Agent Governance Platform (Python)

| Attribute | Detail |
|-----------|--------|
| **What it is** | AI agents as OS processes; BioKernel; TELOS membrane; AHES (endocrine); immunity; governance; FastAPI |
| **Entry point** | `agp-core/src/main.py` → uvicorn FastAPI app |
| **Key modules** | `src/os/` (kernel, process, scheduler, FS, IPC), `src/telos/`, `src/ahes/`, `src/immunity/`, `src/governance/` |
| **Runs as** | FastAPI server; kernel boots on startup; decay scheduler in background |

**Exact entry points:**
- `agp-core/src/main.py` → FastAPI app, `kernel.boot()`, `/api/v1/*`, `/health`, `/metrics`
- `agp-core/demo/unified_demo.py` → Demo of TELOS + AHES + immunity

**Deliverable:** API server for agent lifecycle; agents as processes; TELOS gates execution handoff; endocrine-based scheduling.

---

### Product 3: AURA Protocol — Quantum-Resistant Verification (Python)

| Attribute | Detail |
|-----------|--------|
| **What it is** | Resonant Invariant Algebra (RIA); offline verification; replaces CAs, SWIFT; quantum-resistant |
| **Entry points** | `src/core/ria.py`, `mvp/72hour_mvp.py` |
| **Key modules** | `src/core/ria.py` (ResonantInvariantAlgebra), `mvp/72hour_mvp.py` (AURAMVP) |
| **Runs as** | Library + MVP demo (`python mvp/72hour_mvp.py --demo`) |

**Exact entry points:**
- `mvp/72hour_mvp.py` → SQLite-backed transaction verification, monetization counters
- `src/core/ria.py` → `create_ria_for_device()`, `verify_transaction()`

**Deliverable:** Library for quantum-resistant verification; MVP for payments/DNS/PKI; works offline.

---

### Product 4: ASIM — ASI Alignment Orchestrator (Python)

| Attribute | Detail |
|-----------|--------|
| **What it is** | Physics-based alignment; TIH, IPE, SFA, NTP; SIL, RRE, SAM, HATE; sovereign mesh |
| **Entry point** | `src/asi/core.py` → AsiOrchestrator |
| **Key modules** | `src/asi/` (tih, ipe, reasoning, alignment, sil, rre, sam, hate) |
| **Runs as** | Library; no standalone server in codebase |

**Exact entry points:**
- `src/asi/core.py` → `AsiOrchestrator(ria).process_decision(...)`

**Deliverable:** Alignment orchestration layer; integrates with RIA; no direct execution path to nexus-executor or AGP.

---

### Product 5: SYNTRIASS Path 6 — Generative AI Preview Engine (Python)

| Attribute | Detail |
|-----------|--------|
| **What it is** | Real-time preview for diffusion models; inference tap; fast decoder; conditioning injection |
| **Entry points** | `syntriass/demos/image_preview.py`, `syntriass/front/gradio_app.py` |
| **Key modules** | `syntriass/core/`, `syntriass/preview/`, `syntriass/patch/` |
| **Runs as** | Demo scripts, Gradio UI |

**Exact entry points:**
- `syntriass/demos/image_preview.py` → Image preview demo
- `syntriass/front/gradio_app.py` → Gradio UI

**Deliverable:** Non-blocking preview during diffusion; <300ms preview target.

---

### Product 6: VECTRA — Deterministic Compression (Rust + Python + C++)

| Attribute | Detail |
|-----------|--------|
| **What it is** | Entropy-bounded tensor algebra (EBTA); lossless compression; FEE, NSGE; self-verifying artifacts |
| **Entry points** | `vectra/vectra` (Rust crate), `vectra/python/`, `vectra/cpp/` |
| **Integration** | `nexus-compress` uses VECTRA for PCU/USO compression |
| **Runs as** | Library; benchmarks; telecom_6g integration |

**Exact entry points:**
- Rust: `vectra/vectra` crate
- Python: `vectra/python/`
- Telecom: `vectra/telecom_6g/` (6G RAN, DPD, beamforming)

**Deliverable:** Compression with determinism, integrity, fail-open semantics.

---

### Product 7: CAUSALUX — Distributed Execution Fabric (Rust)

| Attribute | Detail |
|-----------|--------|
| **What it is** | Causal DAG; version vectors; conflict resolution; Morgan economy; Tesla resonance; Da Vinci atom |
| **Entry point** | `causalux` crate (as causalux-v2); used by nexus-sync, nexus-server |
| **Key modules** | `causalux/src/` (dag, sync, version_vector, economy, resonance, atom) |
| **Runs as** | Library; nexus-sync wraps it |

**Exact entry points:**
- `nexus-sync` wraps `CausalDAG` for USO sync
- `nexus-server` uses NexusSyncEngine for `/api/uso`, `/api/sync/stats`

**Deliverable:** Conflict-free distributed state; optional BFT, economy, resonance.

---

## 3. Integration Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         NEXUS EXECUTION PATH                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│  nexus-cli / nexus-exec                                                      │
│        │                                                                     │
│        ▼                                                                     │
│  nexus-executor (PcuExecutor::execute)                                       │
│        │  ◄── guard.check() [ExecutionGuard]                                 │
│        │  ◄── NervousSystemGuard, ImmuneGuard, CompositeGuard                │
│        ▼                                                                     │
│  nexus-pcu (PCU, Identity, Proof)                                            │
│        │                                                                     │
│        ▼                                                                     │
│  semantic_cache, proof, cache.put                                            │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         AGP EXECUTION PATH                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  FastAPI (main.py)                                                           │
│        │                                                                     │
│        ▼                                                                     │
│  BioKernel.schedule()                                                        │
│        │                                                                     │
│        ▼                                                                     │
│  context_switch(pcb)                                                         │
│        │  ◄── telos_membrane.request_crossing(execute:*)                     │
│        ▼                                                                     │
│  pcb.state = RUNNING (only if allowed)                                       │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                    NO CROSS-CONNECT                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  AURA (RIA)          ──►  src/core/ria.py, mvp/                              │
│  ASIM (ASI)          ──►  src/asi/                                           │
│  SYNTRIASS (preview) ──►  syntriass/                                         │
│  VECTRA              ──►  vectra/ + nexus-compress                           │
│  CAUSALUX            ──►  causalux + nexus-sync + nexus-server               │
│                                                                              │
│  nexus-executor and AGP-CORE do NOT call AURA, ASIM, or SYNTRIASS.           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Runnable Binaries / Servers

| Binary / Server | Command | Product |
|-----------------|---------|---------|
| `nexus-exec` | `nexus-exec <wasm_file> [inputs...]` | NEXUS |
| `nexus` | `nexus init \| node run \| pcu submit` | NEXUS |
| `nexus-server` | HTTP on PORT (default 8080) | NEXUS |
| `agp-core` | `uvicorn src.main:app` or `python -m src.main` | AGP-CORE |
| `72hour_mvp` | `python mvp/72hour_mvp.py --demo` | AURA |
| `image_preview` | `python syntriass/demos/image_preview.py` | SYNTRIASS |

---

## 5. What Is NOT a Product (Libraries / Subsystems)

| Component | Role |
|-----------|------|
| nexus-pcu | Library (PCU, USO, identity types) |
| nexus-core | Library (causal tensors) |
| nexus-sync | Library (wraps CAUSALUX) |
| nexus-network | Library (QUIC transport) |
| homeostasis-engine | Library (metrics, feedback) |
| autonomic-system | Library (ACT/CALM modes) |
| developmental-gates | Library (stages, capabilities) |
| nervous-system | Library (coordinator) |
| multi-asi-immune | Library (reputation, defection) |
| telos-protocol | Library (Rust TELOS; AGP uses Python membrane) |
| nexus-agp | Bridge library (AGP ↔ NEXUS) |

---

## 6. Naming Inconsistency

| Location | Name Used |
|----------|-----------|
| Root README.md | AURA Protocol |
| Cargo.toml workspace | NEXUS |
| nexus-cli | "Network-Embedded eXecution Unified Substrate" |
| DPR / patent docs | NEXUS / AGP |
| agp-core | AGP-CORE, Artificial Governance Protocol |

**Conclusion:** Repository is a SYNTRIASS monorepo. "NEXUS" in patent/DPR context = governed execution substrate (nexus-executor + AGP OS + TELOS). "AURA" = RIA/verification product. Both live in the same repo.

---

## 7. Exact Product Definition (DPR-Aligned)

**NEXUS (as product for patent/investor narrative):**

> A governed execution substrate comprising:  
> (1) A WASM executor that enforces a mandatory guard before any PCU run, with no proof or cache on deny;  
> (2) An OS kernel (AGP) that treats agents as processes and gates execution handoff through a TELOS commitment membrane;  
> (3) Developmental, autonomic, and immune guards that block execution based on maturity, risk, and reputation.  

**Delivered as:**
- `nexus-exec` (CLI)
- `nexus-server` (HTTP API for sync/benchmark)
- `agp-core` (FastAPI + kernel + TELOS)

**Not included in "NEXUS product":** AURA, ASIM, SYNTRIASS, VECTRA, CAUSALUX as standalone products (they are separate or supporting).
