# NEXUS/SYNTRIASS COMPLETE COMPONENT CATALOG

## Detailed Technical Descriptions for Due Diligence

**Document Date:** January 30, 2026  
**Total Components:** 32 Modules | 1.4M+ LOC | 17,000+ Tests

---

# SECTION 1: RUST INFRASTRUCTURE (26 Crates)

---

## 1.1 nexus-core-v2 — Causal Execution Engine

**Lines of Code:** 255,916  
**Tests:** 3,548  
**Status:** ✅ Production

### Purpose
The foundational execution engine implementing deterministic algebraic operations for distributed computation. Provides the mathematical substrate for all causal operations.

### Technical Description
nexus-core-v2 implements a **deterministic execution log** with algebraic merge properties. Every operation produces identical results regardless of execution order (commutativity) and can be safely repeated (idempotence). The engine uses BLAKE3 hashing for content addressing and Wasmtime for sandboxed execution.

### Key Components
| Component | Description |
|-----------|-------------|
| `core/` | Base types and primitives |
| `executor/` | WASM execution sandbox |
| `hash/` | BLAKE3 content addressing |
| `log/` | Append-only execution log |
| `merge/` | Algebraic merge operations |
| `op/` | Operation definitions |
| `replay/` | Deterministic replay engine |
| `storage/` | Persistent storage layer |
| `sync/` | State synchronization |

### Key Invariants
- **Determinism:** Same inputs always produce same outputs
- **Commutativity:** `merge(A, B) = merge(B, A)`
- **Idempotence:** `merge(A, A) = A`
- **Associativity:** `merge(merge(A, B), C) = merge(A, merge(B, C))`

### Dependencies
- wasmtime (WASM runtime)
- sha2, bincode, serde (serialization)
- anyhow, thiserror (error handling)
- clap (CLI)

---

## 1.2 causalux — Causal Sync Fabric

**Lines of Code:** 53,289  
**Tests:** 3,616  
**Status:** ✅ Production

### Purpose
High-performance CausalCAP: a **causal consistency library** implementing version vectors, Merkle DAGs, and conflict-free replicated data types (CRDTs) for distributed state synchronization.

### Technical Description
causalux provides the synchronization primitives for distributed NEXUS deployments. It implements **version vectors** for tracking causality, **Merkle DAGs** for content-addressed history, and optimized **merge algorithms** that guarantee convergence without coordination.

### Key Components
| Component | Description |
|-----------|-------------|
| `VersionVector` | Logical clock for causality tracking |
| `MerkleDAG` | Content-addressed history graph |
| `CRDT` | Conflict-free replicated data types |
| `SyncEngine` | Coordination-free synchronization |
| `Delta` | Efficient delta synchronization |

### Performance
- **Merge operations:** 3,251,490 ops/sec
- **Latency:** 0.31 µs average
- **Memory:** Bounded growth with compaction

### Benchmark Comparison
| vs Competitor | Operation | Advantage |
|---------------|-----------|-----------|
| Automerge | Merge | **38.8x faster** |
| Yjs | Sync | ~5x faster |

---

## 1.3 nexus-pcu — Portable Computation Unit

**Lines of Code:** 4,337  
**Tests:** 72  
**Status:** ✅ Production

### Purpose
The **core abstraction** for content-addressed computation. A PCU encapsulates code, inputs, parameters, and identity into a single unit with a deterministic, cryptographic ID.

### Technical Description
PCU (Portable Computation Unit) is NEXUS's answer to "what is computation?" Every PCU has:
- **Code hash:** BLAKE3 hash of executable WASM
- **Input hashes:** Content addresses of all inputs
- **Parameters:** Execution configuration
- **Principal:** Cryptographic identity of requester
- **ID:** Deterministic hash of all above (`PCU_ID = BLAKE3(code || inputs || params || principal)`)

### Key Components
| Component | Description |
|-----------|-------------|
| `PCU` | Core computation unit |
| `USO` | Universal State Object |
| `Identity` | Cryptographic identity |
| `Proof` | Execution proof |
| `Routing` | Data locality routing |
| `License` | Content-hash-bound licensing |

### Key Features
- **Deterministic ID:** Same computation = same ID everywhere
- **Content Addressing:** Results cacheable by ID
- **Code-to-Data Routing:** Computation moves to data
- **Unforgeable Proofs:** Cryptographic execution attestation

### Patent Claims
- PCU deterministic ID computation
- Code-to-data routing algorithm
- Content-hash-bound licensing

---

## 1.4 nexus-executor — Execution Guard System

**Lines of Code:** 4,468  
**Tests:** 5 (+ integration tests)  
**Status:** ✅ Production

### Purpose
The **single choke point** for all PCU execution. Implements execution guards that must approve every computation before it runs.

### Technical Description
nexus-executor implements the **Execution Guard Architecture** — a pattern where no computation executes without passing through a guard check. The guard evaluates the PCU, context, and identity against configurable policies.

### Key Components
| Component | Description |
|-----------|-------------|
| `ExecutionGuard` | Trait for guard implementations |
| `CompositeGuard` | Ordered guard composition |
| `NervousSystemGuard` | Bio-inspired guard |
| `DevelopmentalStageGuard` | Capability stage gate |
| `ImmuneGuard` | Threat-based guard |
| `SemanticCache` | Result caching by PCU ID |
| `Proof` | Execution attestation |
| `ExecutorBuilder` | Type-safe executor construction |

### Enforcement Semantics
```
On DENY:
  - NO execution occurs
  - NO proof is generated
  - NO cache write happens
  - Audit log records denial with reason
```

### Security Properties
- **Single execution path:** All computation through guard
- **No bypass:** Production executor requires guard
- **No proof on deny:** Blocked execution leaves no exploitable artifacts
- **First-Deny-wins:** CompositeGuard stops at first denial

---

## 1.5 telos-protocol — Accountability Membrane

**Lines of Code:** 4,014  
**Tests:** 50  
**Status:** ✅ Production

### Purpose
**Cognitive accountability infrastructure.** TELOS implements a commitment membrane where decisions cost entropy, high-consequence actions require external attestation, and every action is recorded.

### Technical Description
TELOS (from Greek "purpose/end") enforces that AI decisions have **consequences**. The protocol implements:
- **Commitment Membrane:** Actions must "cross" the membrane, consuming entropy
- **Consequence Tiers:** LOW, MEDIUM, HIGH, CRITICAL with escalating requirements
- **Authority Registry:** Who can authorize what
- **Validator Network:** External attestation for high-consequence actions
- **Trust Accumulator:** Behavioral history affects permissions

### Key Components
| Component | Description |
|-----------|-------------|
| `CommitmentMembrane` | Decision gateway |
| `EntropyMeter` | Irreversibility tracking |
| `ConsequenceTier` | Action classification |
| `AuthorityRegistry` | Permission management |
| `ValidatorNetwork` | External attestation |
| `TrustAccumulator` | Behavioral trust |
| `MerkleTree` | Cryptographic audit |
| `Ledger` | Immutable decision log |
| `VdfGenerator` | Verifiable delay function |

### Consequence Tiers
| Tier | Entropy Cost | Requirements |
|------|--------------|--------------|
| LOW | 1 | Self-attestation |
| MEDIUM | 10 | Authority verification |
| HIGH | 100 | External validator |
| CRITICAL | 1000+ | Multi-party consensus |

---

## 1.6 homeostasis-engine — Biological Regulation Core

**Lines of Code:** 2,717  
**Tests:** 52  
**Status:** ✅ Production

### Purpose
**Constraint enforcement for bio-inspired AI safety.** Implements metrics, bounds, setpoints, and negative feedback loops mirroring biological homeostasis.

### Technical Description
homeostasis-engine provides the mathematical foundation for bio-inspired regulation. Systems maintain "health" by keeping metrics within bounds through negative feedback, just like biological organisms maintain temperature, pH, and other vital signs.

### Key Components
| Component | Description |
|-----------|-------------|
| `Metric` | Measurable system property |
| `Bounds` | Hard/soft limits |
| `Setpoint` | Target value |
| `Feedback` | Negative feedback controller |
| `SingleMetricController` | Single-variable regulation |
| `MultiObjectiveController` | Multi-variable optimization |
| `HealthCheck` | System health assessment |

### Regulation Model
```
Error = Setpoint - CurrentValue
Correction = Kp * Error + Ki * ∫Error + Kd * dError/dt
Action = Clamp(Correction, HardBounds)
```

---

## 1.7 multi-asi-immune — Distributed Immune Protocol

**Lines of Code:** 3,335  
**Tests:** 68  
**Status:** ✅ Production

### Purpose
**Immune system for AI networks.** Implements identity, reputation, threat signatures, and defection detection for multi-ASI environments.

### Technical Description
multi-asi-immune provides self/non-self discrimination for AI agent networks. Agents can:
- Identify and verify each other (Ed25519)
- Track reputation across interactions
- Share threat intelligence (signed reports)
- Detect coordinated defection (collusion)

### Key Components
| Component | Description |
|-----------|-------------|
| `AsiId` | Cryptographic identity |
| `AsiNode` | Network participant |
| `ReputationScore` | Trust metrics |
| `ThreatPattern` | Threat signatures |
| `SignedThreatReport` | Verified threat intel |
| `ProtocolMessage` | Network protocol |
| `DefectionDetector` | Collusion detection |

### Security Properties
- **Self/Non-Self:** Agents distinguish trusted from untrusted
- **Memory:** Past threats inform future responses
- **Distributed:** No central authority required
- **Byzantine Tolerant:** Handles malicious actors

---

## 1.8 nervous-system — Coordinator

**Lines of Code:** 949  
**Tests:** 8  
**Status:** ✅ Production

### Purpose
**Central coordination** of perception, decision, and action. Integrates all safety layers into a unified control system.

### Technical Description
nervous-system is the "brain" that coordinates:
- Perception: Sensing environment and agent state
- Decision: Evaluating options against policies
- Motor: Executing approved actions

It integrates homeostasis-engine, autonomic-system, developmental-gates, and multi-asi-immune.

### Key Components
| Component | Description |
|-----------|-------------|
| `NervousSystemCoordinator` | Central controller |
| `Perception` | Sensory processing |
| `DecisionEngine` | Policy evaluation |
| `MotorSystem` | Action execution |
| `SafetyState` | Current safety status |

---

## 1.9 autonomic-system — Mode Controller

**Lines of Code:** ~600  
**Tests:** 10  
**Status:** ✅ Production

### Purpose
**ACT vs CALM mode control.** Manages arousal levels, state transitions, and reflex responses.

### Technical Description
Like the biological autonomic nervous system, this controls the "involuntary" aspects of AI behavior:
- **CALM:** Low arousal, contemplative processing
- **ACT:** High arousal, action-oriented
- **EMERGENCY:** Crisis response mode
- **RECOVERY:** Post-crisis stabilization

### Key Components
| Component | Description |
|-----------|-------------|
| `AutonomicController` | Mode manager |
| `AutonomicMode` | CALM/ACT/EMERGENCY/RECOVERY |
| `Arousal` | Activation level |
| `ReflexResponse` | Automatic reactions |
| `ModeTransition` | State changes |

---

## 1.10 developmental-gates — Capability Staging

**Lines of Code:** 969  
**Tests:** 13  
**Status:** ✅ Production

### Purpose
**Staged capability unlock** based on demonstrated maturity. AI systems must prove readiness before accessing dangerous capabilities.

### Technical Description
Like biological development (embryo → infant → child → adult), AI systems progress through stages:
- **Stage 0 (Embryonic):** Minimal capabilities, constant monitoring
- **Stage 1 (Infant):** Basic operations, strict limits
- **Stage 2 (Juvenile):** Extended capabilities, supervision
- **Stage 3 (Adult):** Full capabilities, self-governance

### Key Components
| Component | Description |
|-----------|-------------|
| `StageManager` | Stage progression |
| `DevelopmentalStage` | Current maturity level |
| `GateEnforcer` | Capability access control |
| `CapabilityRegistry` | What each stage can do |
| `StageRequirements` | Advancement criteria |

---

## 1.11-1.20 Additional Rust Crates

| Crate | LOC | Tests | Purpose |
|-------|-----|-------|---------|
| **nexus-sync** | ~800 | 23 | High-level sync engine |
| **nexus-network** | 1,662 | 4 | QUIC transport, gossip protocol |
| **nexus-storage** | ~700 | 4 | RocksDB backend, indexing |
| **nexus-compress** | ~500 | 71 | VECTRA lossless compression |
| **nexus-agp** | ~500 | 21 | AGP ↔ NEXUS bridge |
| **nexus-edge** | ~300 | — | Cloudflare Workers deployment |
| **nexus-secrets** | ~400 | 2 | Secret management (Vault, AWS, K8s) |
| **nexus-cli** | ~200 | — | Command-line interface |
| **nexus-server** | ~300 | — | HTTP API (Axum) |
| **nexus-observability** | ~400 | — | Metrics, tracing |

---

# SECTION 2: PYTHON AGP-OS (14 Modules)

---

## 2.1 governance/ — Policy Enforcement Layer

**Files:** 7  
**Lines of Code:** 1,682  
**Status:** ✅ Production

### Purpose
Complete governance stack: behavioral RAG, rules engine, alignment verification, anomaly detection, and enforcement actions.

### Components

#### 2.1.1 BehavioralRAG (`rag.py`)
**Purpose:** Semantic search over agent behavioral history for context-aware governance.

| Method | Description |
|--------|-------------|
| `embed_behavior()` | Convert behavior to vector |
| `search_similar()` | Find similar past behaviors |
| `retrieve_context()` | Get governance-relevant context |

#### 2.1.2 RulesEngine (`rules.py`)
**Purpose:** Configurable policy rules with pattern matching and action triggers.

| Feature | Description |
|---------|-------------|
| Rule Definition | YAML/JSON rule specifications |
| Pattern Matching | Regex and semantic patterns |
| Actions | ALLOW, WARN, BLOCK, ESCALATE |
| Priority | Rule ordering and conflict resolution |

#### 2.1.3 AlignmentVerifier (`alignment.py`)
**Purpose:** Multi-factor alignment scoring in real-time.

| Factor | Weight | Description |
|--------|--------|-------------|
| Goal Alignment | 0.3 | Match with stated objectives |
| Value Alignment | 0.3 | Consistency with values |
| Behavioral Consistency | 0.2 | Matches past behavior |
| Safety Constraints | 0.2 | Within safety bounds |

#### 2.1.4 AnomalyDetector (`anomaly.py`)
**Purpose:** Statistical and ML-based anomaly detection.

| Method | Description |
|--------|-------------|
| `detect_statistical()` | Z-score, IQR detection |
| `detect_ml()` | Isolation forest, autoencoder |
| `get_anomaly_score()` | Combined anomaly score |

#### 2.1.5 Enforcer (`enforcer.py`)
**Purpose:** Execute governance decisions.

| Action | Description |
|--------|-------------|
| `block()` | Prevent action execution |
| `throttle()` | Rate limit agent |
| `quarantine()` | Isolate agent |
| `escalate()` | Human review |

---

## 2.2 telos/ — Commitment Membrane (Python)

**Files:** 2  
**Lines of Code:** 336  
**Status:** ✅ Production

### Purpose
Python implementation of TELOS commitment membrane for AGP-OS integration.

### Components

#### CommitmentMembrane (`membrane.py`)
| Method | Description |
|--------|-------------|
| `request_crossing()` | Request to cross membrane |
| `evaluate_consequence()` | Assess action tier |
| `consume_entropy()` | Deduct entropy cost |
| `record_commitment()` | Log decision |

#### ExecutionBlocked (Exception)
Raised when TELOS denies a crossing request. Contains reason and required scope.

---

## 2.3 ahes/ — Artificial Homeostatic Endocrine System

**Files:** 2  
**Lines of Code:** 341  
**Status:** ✅ Production

### Purpose
8-hormone neuroendocrine system regulating AI behavior with bio-realistic dynamics.

### Hormones

| Hormone | Function | Effect |
|---------|----------|--------|
| **Dopamine** | Reward/motivation | ↑ exploration, ↑ risk-taking |
| **Serotonin** | Mood/contentment | ↑ patience, ↓ aggression |
| **Cortisol** | Stress response | ↑ alertness, ↑ energy |
| **Norepinephrine** | Arousal/attention | ↑ focus, ↑ reaction speed |
| **Oxytocin** | Trust/bonding | ↑ cooperation, ↑ sharing |
| **Endorphin** | Pain modulation | ↑ persistence, ↓ discomfort |
| **GABA** | Inhibition | ↓ anxiety, ↓ impulsivity |
| **Acetylcholine** | Learning/memory | ↑ learning rate, ↑ recall |

### Dynamics
- **Decay:** Hormones have half-lives (exponential decay)
- **Saturation:** Receptors have maximum capacity
- **Circadian:** Levels vary with time-of-day
- **Allostasis:** Chronic stress shifts baselines

---

## 2.4 immunity/ — Artificial Immune System

**Files:** 22  
**Lines of Code:** 4,078  
**Status:** ✅ Production

### Purpose
Complete immune system with innate and adaptive components for threat detection and response.

### Innate Immunity
Fast, pattern-based threat recognition.

| Component | Description |
|-----------|-------------|
| Pattern Recognition | Pre-defined threat signatures |
| Inflammatory Response | Immediate containment |
| Natural Killer | Quick threat neutralization |

### Adaptive Immunity
Learned, specific threat responses.

| Component | Description |
|-----------|-------------|
| T-Cells (Helper) | Coordinate immune response |
| T-Cells (Killer) | Destroy infected agents |
| T-Cells (Regulatory) | Prevent overreaction |
| B-Cells | Produce antibodies |
| Antibodies | Neutralize specific threats |
| Memory Cells | Remember past threats |

### Training Pipeline
| Step | Description |
|------|-------------|
| 1. Collect | Gather behavioral data |
| 2. Label | Mark benign/malicious |
| 3. Train | Update immune models |
| 4. Validate | Test against holdout |
| 5. Deploy | Activate new signatures |

---

## 2.5 os/ — Operating System Kernel

**Files:** 35  
**Lines of Code:** 6,189  
**Status:** ✅ Production

### Purpose
Complete OS for AI agents: kernel, scheduler, IPC, filesystem, HAL, mesh coordination.

### Core Components

#### BioKernel (`kernel.py`)
| Feature | Description |
|---------|-------------|
| Process Management | Spawn, kill, lifecycle |
| Scheduling | Priority-based, preemptive |
| Context Switch | TELOS-gated execution |
| Resource Accounting | Tokens, CPU, memory |

#### ProcessControlBlock (`process.py`)
| Field | Description |
|-------|-------------|
| pid | Process ID |
| agent_id | Associated agent |
| state | CREATED/READY/RUNNING/WAITING/TERMINATED |
| priority | Scheduling priority |
| usage | Resource consumption |

#### Scheduler (`scheduler.py`)
| Algorithm | Description |
|-----------|-------------|
| Endocrine Priority | Based on hormone state |
| Fair Share | Resource fairness |
| Deadline | Real-time constraints |

#### IPC (`ipc/`)
| Mechanism | Description |
|-----------|-------------|
| Message Queue | Async message passing |
| Shared Memory | Direct memory sharing |
| Signals | SIGTERM, SIGSTOP, etc. |

#### Filesystem (`fs/`)
| Mount | Description |
|-------|-------------|
| /proc | Process information |
| /home | Agent home directories |
| /shared | Cross-agent shared space |

#### HAL (`hal.py`)
| Interface | Description |
|-----------|-------------|
| Sensors | Read sensor data |
| Actuators | Control outputs |
| Safety Interlocks | Emergency stops |

#### Mesh Coordination (`mesh.py`)
| Feature | Description |
|---------|-------------|
| Discovery | Find peer agents |
| Consensus | Distributed agreement |
| Mailbox | Message delivery |

#### ROS2 Integration (`ros2.py`)
| Feature | Description |
|---------|-------------|
| Publishers | ROS2 topic publishing |
| Subscribers | ROS2 topic subscription |
| Services | ROS2 service calls |
| Actions | ROS2 action execution |

#### RTOS Scheduler (`rtos.py`)
| Feature | Description |
|---------|-------------|
| Priority Levels | 8 priority levels |
| Preemption | Immediate preemption |
| Deadline Monotonic | Rate monotonic scheduling |

---

## 2.6-2.14 Additional Python Modules

| Module | Files | LOC | Purpose |
|--------|-------|-----|---------|
| **api/** | 10 | 1,489 | REST API endpoints (FastAPI) |
| **services/** | 12 | 3,927 | Business logic, reputation, blockchain |
| **agents/** | 4 | 662 | Agent management, ADK |
| **ml/** | 3 | ~500 | Deep learning, RAG utilities |
| **compliance/** | 2 | ~300 | Regulatory compliance checking |
| **core/** | 4 | ~400 | Core types and utilities |
| **models/** | 1 | ~200 | Data models (Pydantic) |

---

# SECTION 3: ADDITIONAL COMPONENTS

---

## 3.1 nexus-telecom — 6G RAN Integration

**Type:** Python Package  
**Purpose:** NEXUS integration with telecommunications: Wake-up receivers, Lyapunov safety controllers, 6G RAN.

### Components
| Component | Description |
|-----------|-------------|
| WakeUpReceiver | RI-WuR for energy-efficient wake |
| LyapunovController | Stability-guaranteed control |
| EnergyModel | Energy consumption modeling |
| EntropyCalculator | Computational entropy |

---

## 3.2 syntriass/ — Real-Time AI Preview

**Type:** Python Package  
**Purpose:** Path 6 — Real-time generative AI preview for image/video generation.

### Components
| Component | Description |
|-----------|-------------|
| InferenceTap | Hook into inference pipeline |
| FastDecoder | Low-latency decoding |
| TemporalInterpolation | Frame interpolation |
| WebSocket API | Real-time streaming |
| Gradio Frontend | Interactive UI |

---

## 3.3 brain/formal_specs/ — TLA+ Specifications

**Type:** Formal Specification  
**Purpose:** Mathematically verified safety properties.

### SafetyProtocol.tla
Formally specifies:
- Mode transitions (CALM ↔ ACT ↔ EMERGENCY ↔ RECOVERY)
- Arousal dynamics
- Safety invariants
- Liveness properties

---

# SUMMARY

## Component Count

| Category | Count |
|----------|-------|
| Rust Crates | 26 |
| Python Modules | 14 |
| Formal Specs | 1 |
| Documentation | 201 files |
| **Total Components** | **41+** |

## Lines of Code

| Category | LOC |
|----------|-----|
| Rust | ~340,000 |
| Python | ~18,700 (AGP-OS core) |
| Python | ~1,380,000 (full repo) |
| **Total** | **1,399,285** |

## Test Coverage

| Category | Tests |
|----------|-------|
| Rust #[test] | 16,496 |
| Python tests | 500+ |
| **Total** | **17,000+** |

---

**Document generated:** January 30, 2026  
**Repository:** /Users/richardrich/Desktop/NEXUS
