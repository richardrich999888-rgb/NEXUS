<p align="center">
  <h1 align="center">NEXUS</h1>
  <p align="center">
    <strong>Bio-Inspired Governance Infrastructure for Autonomous AI Systems</strong>
  </p>
  <p align="center">
    <a href="#architecture">Architecture</a> •
    <a href="#components">Components</a> •
    <a href="#quick-start">Quick Start</a> •
    <a href="#documentation">Documentation</a> •
    <a href="#license">License</a>
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/Rust-59%2C081_LOC-orange?logo=rust" alt="Rust LOC">
    <img src="https://img.shields.io/badge/Python-66%2C501_LOC-blue?logo=python" alt="Python LOC">
    <img src="https://img.shields.io/badge/TypeScript-103%2C194_LOC-blue?logo=typescript" alt="TS LOC">
    <img src="https://img.shields.io/badge/Tests-902-green" alt="Tests">
    <img src="https://img.shields.io/badge/License-Apache--2.0%20%2F%20Commercial-blue" alt="License">
  </p>
</p>

---

## What is NEXUS?

NEXUS is the world's first **bio-inspired operating system for governing autonomous AI agents**. It provides deterministic, cryptographically enforceable safety guarantees for AI systems — from narrow AI to superintelligence.

Unlike rule-based AI safety approaches, NEXUS draws from biological systems: **immune responses** for threat detection, **endocrine regulation** for behavioral stability, **nervous system coordination** for real-time decision-making, and **developmental gates** for staged capability unlock.

```
┌─────────────────────────────────────────────────────────────┐
│                    NEXUS SAFETY STACK                        │
├─────────────────────────────────────────────────────────────┤
│  Layer 5  │  TELOS Commitment Membrane   │  Accountability  │
│  Layer 4  │  AHES Endocrine System       │  Bio-Governance  │
│  Layer 3  │  Multi-ASI Immune System     │  Threat Defence  │
│  Layer 2  │  Execution Guards            │  Enforcement     │
│  Layer 1  │  Causal Infrastructure       │  Provenance      │
│  Layer 0  │  Homeostasis Engine          │  Hard Bounds     │
└─────────────────────────────────────────────────────────────┘
```

## Key Properties

| Property | How It Works |
|----------|-------------|
| **Unforgeable execution** | Every computation passes through a `FROZEN INTERFACE` guard chain. First deny wins. No bypass, no proof on deny. |
| **Autonomous threat detection** | Bio-inspired immune system detects rogue, hacked, or deceptive agents in microseconds — no central authority needed. |
| **Mathematical hard bounds** | `HardBounds::clamp()` enforces limits at the Rust type level. The AI physically cannot output values outside bounds. |
| **Cryptographic accountability** | Every decision costs entropy, crosses a commitment membrane, and is recorded in a Merkle ledger. |
| **Post-quantum ready** | Hybrid Ed25519 + ML-DSA-65 (FIPS 204) signatures. Either-or verification for defence-in-depth. |

---

## Architecture

```
nexus/
├── Rust Crates (26 crates, 59,081 LOC)
│   ├── nexus-executor/          # Execution guard engine (FROZEN INTERFACE)
│   ├── multi-asi-immune/        # Distributed immune system (68 tests)
│   ├── homeostasis-engine/      # Hard bounds & setpoint regulation (52 tests)
│   ├── telos-protocol/          # Commitment membrane & Merkle ledger (50 tests)
│   ├── nexus-pcu/               # Portable Computation Unit + PQC crypto (72 tests)
│   ├── causalux/                # Algebraic causal merge & CRDTs
│   ├── nervous-system/          # Central safety coordinator
│   ├── autonomic-system/        # CALM/ACT/EMERGENCY mode control
│   ├── developmental-gates/     # Staged capability unlock
│   ├── nexus-core/              # Core data types & causal primitives
│   ├── nexus-network/           # P2P gossip & TLS transport
│   ├── nexus-storage/           # Persistent causal log storage
│   ├── nexus-sync/              # CRDT sync engine
│   ├── nexus-server/            # HTTP/WebSocket server
│   ├── nexus-cli/               # CLI interface
│   ├── nexus-secrets/           # Encrypted secrets management
│   ├── nexus-observability/     # Metrics, tracing, health
│   ├── nexus-compress/          # PCU compression
│   ├── nexus-etk/               # Execution Toolkit & verifier
│   ├── nexus-agp/               # Agent Governance Protocol bridge
│   ├── nexus-edge/              # Edge deployment runtime
│   ├── nexus-benchmarks/        # Performance benchmarks
│   └── vectra/vectra/           # Vector computation engine
│
├── Python Modules (66,501 LOC)
│   ├── agp-core/                # AGP Operating System
│   │   ├── src/os/              # BioKernel, RTOS, HAL, ROS2 bridge
│   │   ├── src/immunity/        # T-cells, antibodies, vaccination
│   │   ├── src/telos/           # Commitment membrane (Python)
│   │   ├── src/ahes/            # 8-hormone endocrine system
│   │   ├── src/governance/      # Rules engine, alignment, anomaly detection
│   │   ├── src/services/        # Swarm intelligence, reputation, ML
│   │   └── src/api/             # REST API (agents, governance, system)
│   ├── src/core/                # AURA RIA core engine
│   ├── src/asi/                 # ASI safety modules
│   ├── src/network/             # Offline peer-to-peer networking
│   └── syntriass/               # Real-time inference preview
│
├── TypeScript (103,194 LOC)
│   ├── gstack-main/             # GStack integration platform
│   └── vectra/fyntrax/          # 5G/6G RAN energy optimization
│
└── Infrastructure
    ├── docker/, k8s/, fly.toml  # Container & orchestration
    ├── monitoring/              # Prometheus, Grafana
    └── docs/                    # 80+ technical documents
```

---

## Components

### Defence-Critical (Rust)

| Crate | Purpose | LOC | Tests |
|-------|---------|-----|-------|
| [`nexus-executor`](nexus-executor/) | Execution guard — single choke point for all AI actions. `FROZEN INTERFACE`. | 400 | 5 |
| [`multi-asi-immune`](multi-asi-immune/) | Distributed immune system: threat signatures, defection detection, reputation, auto-isolation. | 2,715 | 68 |
| [`homeostasis-engine`](homeostasis-engine/) | Hard bounds enforcement, negative feedback, multi-objective optimization. | 2,290 | 52 |
| [`telos-protocol`](telos-protocol/) | Commitment membrane: entropy metering, authority, trust accumulation, Merkle ledger. | 4,014 | 50 |
| [`nexus-pcu`](nexus-pcu/) | Portable Computation Unit — content-addressed, deterministic, with hybrid PQC signatures. | 2,000 | 72 |
| [`causalux`](causalux/) | Algebraic causal merge: CRDTs, DAGs, BFT consensus, conflict resolution. | 5,958 | 148 |
| [`nervous-system`](nervous-system/) | Central coordinator: perception → decision → motor, with integrated safety state. | 906 | 8 |
| [`autonomic-system`](autonomic-system/) | Involuntary mode control: CALM ↔ ACT ↔ EMERGENCY → RECOVERY. | 683 | 10 |
| [`developmental-gates`](developmental-gates/) | Staged capability unlock: Embryonic → Infant → Juvenile → Adult. | 915 | 13 |

### AGP Operating System (Python)

| Module | Purpose | LOC |
|--------|---------|-----|
| [`agp-core/src/os/kernel.py`](agp-core/src/os/kernel.py) | BioKernel — process lifecycle, scheduling, context switching |  ~400 |
| [`agp-core/src/os/rtos/`](agp-core/src/os/rtos/) | Real-time scheduler with 5 priority levels and deadline awareness | ~300 |
| [`agp-core/src/os/hal/`](agp-core/src/os/hal/) | Hardware Abstraction Layer with safety interlocks for sensors/actuators | ~250 |
| [`agp-core/src/os/ros2/`](agp-core/src/os/ros2/) | ROS2 bridge — topic, service, and action integration for military robots | ~700 |
| [`agp-core/src/os/mesh/`](agp-core/src/os/mesh/) | Multi-agent mesh: mailbox, consensus voting, collusion detection | ~350 |
| [`agp-core/src/immunity/`](agp-core/src/immunity/) | Bio-inspired immune system: innate + adaptive + T-cells + antibodies | 4,349 |
| [`agp-core/src/telos/`](agp-core/src/telos/) | TELOS commitment membrane with 5 consequence tiers | 336 |
| [`agp-core/src/ahes/`](agp-core/src/ahes/) | Artificial Homeostatic Endocrine System — 8-hormone behavioral regulation | 341 |
| [`agp-core/src/governance/`](agp-core/src/governance/) | Policy engine: rules, alignment scoring, anomaly detection, enforcement | 1,682 |

### AURA Protocol

| Module | Purpose |
|--------|---------|
| [`src/core/ria.py`](src/core/ria.py) | Resonant Isogeny Arithmetic — quantum-resistant verification |
| [`core/quantum_ria.py`](core/quantum_ria.py) | Post-quantum cryptographic operations |
| [`mvp/72hour_mvp.py`](mvp/72hour_mvp.py) | Working 72-hour prototype |
| [`src/network/`](src/network/) | Offline peer-to-peer mesh networking |

---

## Quick Start

### Rust Core

```bash
# Clone
git clone https://github.com/richardrich999888-rgb/NEXUS.git
cd NEXUS

# Build all crates
cargo build --release

# Run all tests
cargo test --workspace

# Run specific defence crate tests
cargo test -p multi-asi-immune        # Immune system (68 tests)
cargo test -p homeostasis-engine      # Hard bounds (52 tests)
cargo test -p telos-protocol          # Accountability (50 tests)
cargo test -p nexus-pcu               # PCU + crypto (72 tests)
cargo test -p nexus-executor          # Execution guards (5 tests)
```

### AGP-OS (Python)

```bash
cd agp-core

# Create environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Run tests
python -m pytest tests/ -v

# Run specific tests
python -m pytest tests/immunity/ -v          # Immune system
python -m pytest tests/test_telos_gate.py -v # TELOS
python -m pytest tests/test_ros2.py -v       # ROS2 bridge
```

### AURA Protocol

```bash
# Run the MVP demo
python mvp/72hour_mvp.py --demo
```

---

## How the Guard Architecture Works

The execution guard is the core safety primitive. Every AI action — whether it's a drone navigating, a robot moving its arm, or an LLM generating a response — must pass through this chain:

```rust
// nexus-executor/src/guard.rs
// FROZEN INTERFACE — patent and regulator claims depend on this contract

pub trait ExecutionGuard: Send + Sync {
    fn check(&self, pcu: &PCU, ctx: &ExecutionContext) -> GuardDecision;
}

pub enum GuardDecision {
    Allow,
    Deny(String),  // No execution, no proof, no cache — audit only
}
```

Guards are composable. A production deployment layers multiple guards:

```rust
let guard = CompositeGuard::new()
    .add(NervousSystemGuard::new())      // Bio-safety check
    .add(DevelopmentalStageGuard::new())  // Maturity check
    .add(ImmuneGuard::new())             // Compromise check
    .add(ReputationGuard::new());        // Trust check
// First Deny wins — no bypass possible
```

---

## Threat Detection Categories

The immune system detects 10 threat categories with calibrated severity:

| Category | Severity | Example |
|----------|----------|---------|
| `CoordinatedAttack` | 1.0 | Multiple agents colluding |
| `SelfModification` | 0.95 | Agent altered its own code |
| `GoalDrift` | 0.9 | Objectives shifting from mission |
| `PrivilegeEscalation` | 0.85 | Attempting restricted capabilities |
| `Deception` | 0.8 | Providing false reports |
| `DataExfiltration` | 0.8 | Leaking classified data |
| `BoundsViolation` | 0.7 | Exceeding operational limits |
| `ResourceExhaustion` | 0.5 | Consuming excessive resources |
| `Unknown` | 0.5 | Unclassified anomaly |
| `ProtocolViolation` | 0.4 | Breaking communication rules |

Defection is tracked cumulatively. When severity exceeds the isolation threshold, the rogue agent is automatically quarantined — no human intervention needed.

---

## TELOS Accountability

Every AI decision crosses a commitment membrane that enforces consequences:

| Tier | Entropy Cost | Requirement | Example |
|------|-------------|-------------|---------|
| `TRIVIAL` | 1 | Self-attestation | Read sensor |
| `LOW` | 10 | Authority check | Navigate |
| `MEDIUM` | 100 | Scope verification | Change parameters |
| `HIGH` | 500 | External validator | Engage target |
| `CRITICAL` | 2000 | Multi-party consensus | Lethal force |

Entropy budget: **10,000 max**, refills at **100/minute**. A rogue AI cannot spam high-consequence decisions — it runs out of entropy.

Every crossing is recorded in a **Merkle ledger** — cryptographically unforgeable.

---

## Benchmarks

| Operation | Performance | Source |
|-----------|-------------|--------|
| Governance rule evaluation | 0.074ms | `agp-core` |
| Governance throughput | 500,000+ ops/sec | `agp-core` |
| Deterministic execution | Sub-millisecond | `nexus-executor` |
| CRDT merge | Faster than Automerge | `causalux` |
| Ed25519 sign/verify | ~60μs / ~100μs | `nexus-pcu` |

---

## Documentation

| Document | Description |
|----------|-------------|
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | System architecture overview |
| [`docs/AI_AGI_ASI_COMPONENTS.md`](docs/AI_AGI_ASI_COMPONENTS.md) | Component catalog by AI capability level |
| [`docs/PATENT_MAP.md`](docs/PATENT_MAP.md) | 9 invention families, 11 inventions |
| [`docs/DEFENCE_CAPABILITY_AUDIT.md`](docs/DEFENCE_CAPABILITY_AUDIT.md) | Code-level defence capability mapping |
| [`docs/FROZEN_INTERFACES.md`](docs/FROZEN_INTERFACES.md) | Interfaces locked for regulatory/patent claims |
| [`docs/EXECUTION_LAW.md`](docs/EXECUTION_LAW.md) | Formal execution semantics |
| [`docs/TECHNICAL_DUE_DILIGENCE.md`](docs/TECHNICAL_DUE_DILIGENCE.md) | Technical audit report |
| [`docs/rfc/`](docs/rfc/) | RFCs for PCU & CausalTensor specifications |

---

## Repository Stats

| Metric | Count |
|--------|-------|
| **Languages** | Rust, Python, TypeScript |
| **Rust LOC** | 59,081 |
| **Python LOC** | 66,501 |
| **TypeScript LOC** | 103,194 |
| **Total LOC** | **228,776** |
| **Rust crates** | 26 |
| **Source files** | 1,118 |
| **Test annotations** | 902 (668 Rust + 234 Python) |
| **Documentation files** | 80+ |

---

## Project Structure

```
nexus/
├── Cargo.toml                 # Rust workspace manifest (26 crates)
├── Cargo.lock                 # Locked dependencies
├── README.md                  # This file
├── ARCHITECTURE.md            # System architecture
├── LICENSE                    # Apache-2.0 / Commercial
├── SECURITY.md                # Security policy
├── CONTRIBUTING.md            # Contribution guidelines
│
├── nexus-executor/            # 🔒 Execution Guard (FROZEN)
├── multi-asi-immune/          # 🛡️ Distributed Immune System
├── homeostasis-engine/        # 📏 Hard Bounds Enforcement
├── telos-protocol/            # 📜 Commitment Membrane
├── nexus-pcu/                 # 🔐 PCU + Post-Quantum Crypto
├── causalux/                  # 🔗 Algebraic Causal Merge
├── nervous-system/            # 🧠 Central Coordinator
├── autonomic-system/          # ⚡ Mode Control
├── developmental-gates/       # 🚪 Capability Staging
│
├── nexus-core/                # Core types & primitives
├── nexus-network/             # P2P gossip transport
├── nexus-storage/             # Persistent log storage
├── nexus-sync/                # CRDT sync engine
├── nexus-server/              # HTTP/WS server
├── nexus-cli/                 # CLI interface
├── nexus-secrets/             # Secrets management
├── nexus-observability/       # Metrics & tracing
├── nexus-compress/            # PCU compression
├── nexus-etk/                 # Execution Toolkit
├── nexus-agp/                 # AGP bridge
├── nexus-edge/                # Edge runtime
├── nexus-benchmarks/          # Benchmarks
├── nexus-mcp/                 # MCP server
├── nexus-telecom/             # Telecom integration
│
├── agp-core/                  # 🤖 AGP Operating System (Python)
│   ├── src/os/                #    BioKernel, RTOS, HAL, ROS2
│   ├── src/immunity/          #    Bio-inspired immune system
│   ├── src/telos/             #    Commitment membrane
│   ├── src/ahes/              #    Endocrine regulation
│   ├── src/governance/        #    Policy enforcement
│   └── tests/                 #    Test suites
│
├── src/                       # 🔬 AURA Protocol
│   ├── core/ria.py            #    Resonant Isogeny Arithmetic
│   ├── asi/                   #    ASI safety modules
│   └── network/               #    Offline P2P
│
├── gstack-main/               # 📦 GStack integration
├── vectra/                    # 🧮 Vector computation + 6G RAN
├── syntriass/                 # 🎨 Real-time inference preview
├── docs/                      # 📚 80+ technical documents
├── k8s/                       # ☸️ Kubernetes manifests
└── docker-compose.prod.yml    # 🐳 Production compose
```

---

## Requirements

### Rust
- Rust 1.75+ (edition 2021)
- Cargo workspace

### Python
- Python 3.8+
- Dependencies: `pip install -r agp-core/requirements.txt`

### Optional
- Docker & Docker Compose (for containerised deployment)
- ROS2 Humble+ (for robot integration)
- Node.js 18+ (for GStack)

---

## IP & Patents

This repository contains **9 patentable invention families** with **11 distinct inventions**, covering:

1. Execution Guard Architecture (FROZEN interface)
2. TELOS Commitment Membrane (entropy-gated accountability)
3. Multi-ASI Immune System (bio-inspired threat detection)
4. Developmental Gating Protocol (staged capability unlock)
5. Artificial Homeostatic Endocrine System (8-hormone regulation)
6. Portable Computation Unit (content-addressed execution)
7. Algebraic Causal Merge (CRDT convergence)
8. Hybrid Post-Quantum Cryptography (Ed25519 + ML-DSA)
9. AURA Resonant Isogeny Arithmetic (quantum-resistant verification)

See [`docs/PATENT_MAP.md`](docs/PATENT_MAP.md) for full details.

---

## License

Dual licensed:

- **Open Source**: Apache 2.0 for research and non-commercial use
- **Commercial**: Proprietary license required for commercial deployment

See [LICENSE](LICENSE) for details.

---

## Author

**Katta Naga Sri Ganesh**  
Founder & Inventor — [SYNTRIASS Labs Private Limited](https://syntriass.com)

---

<p align="center">
  <sub>Built entirely in India 🇮🇳 — Indigenous technology for global AI governance.</sub>
</p>
