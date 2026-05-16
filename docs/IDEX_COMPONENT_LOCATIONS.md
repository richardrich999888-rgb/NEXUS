# iDEX PITCH — COMPONENT LOCATION MAP

## Exact File Locations for Every Component Being Pitched

**Root:** `/Users/richardrich/Desktop/NEXUS/`  
**Total Defence-Relevant Files:** 158 source files + 40 test files

---

# COMPONENT 1: EXECUTION GUARD SYSTEM
**iDEX Pitch:** "No autonomous weapon fires without multi-layer authorization"

### Source Files (15 files)

| File | Purpose | Defence Role |
|------|---------|--------------|
| `nexus-executor/src/guard.rs` | **FROZEN** guard trait — `Allow`/`Deny` | The choke point. Every action passes through here |
| `nexus-executor/src/guards/composite.rs` | Multi-guard chaining, first-deny-wins | Layers: bio + maturity + immune + reputation |
| `nexus-executor/src/guards/nervous.rs` | Bio-safety guard | Blocks if nervous system detects danger |
| `nexus-executor/src/guards/immune.rs` | Threat-check guard | Blocks if immune system detects compromise |
| `nexus-executor/src/guards/mod.rs` | Guard module exports | — |
| `nexus-executor/src/executor.rs` | Core execution engine | Runs PCU only after all guards allow |
| `nexus-executor/src/proof.rs` | Execution proof generation | Cryptographic proof of authorized execution |
| `nexus-executor/src/cache.rs` | Semantic result cache | Cache by PCU ID — no recomputation |
| `nexus-executor/src/semantic_cache.rs` | Advanced caching | Content-addressed result storage |
| `nexus-executor/src/limits.rs` | Resource limits | CPU/memory/time bounds per execution |
| `nexus-executor/src/host_functions.rs` | WASM host functions | Sandboxed execution interface |
| `nexus-executor/src/types.rs` | Type definitions | `ExecutionContext`, `ExecutionResult` |
| `nexus-executor/src/error.rs` | Error types | `ExecutionBlocked`, `GuardDenied` |
| `nexus-executor/src/lib.rs` | Crate root | Public API |
| `nexus-executor/src/main.rs` | Binary entry | Standalone executor |

### Test Files (5 files)

| File | Tests | What It Proves |
|------|-------|----------------|
| `nexus-executor/tests/red_team_execution.rs` | Adversarial | Guard cannot be bypassed |
| `nexus-executor/tests/integration_tests.rs` | Integration | Full execution flow works |
| `nexus-executor/tests/adversarial.rs` | Attack tests | Malicious inputs blocked |
| `nexus-executor/tests/stability.rs` | Stability | No crashes under stress |
| `nexus-executor/tests/performance.rs` | Perf | Latency benchmarks |

---

# COMPONENT 2: MULTI-ASI IMMUNE SYSTEM (Rust)
**iDEX Pitch:** "Detect and neutralize rogue drones in a swarm at machine speed"

### Source Files (20 files)

| File | Purpose | Defence Role |
|------|---------|--------------|
| **Identity** | | |
| `multi-asi-immune/src/identity/keypair.rs` | Ed25519 identity | Unforgeable drone ID |
| `multi-asi-immune/src/identity/mod.rs` | Module exports | — |
| **Threat Detection** | | |
| `multi-asi-immune/src/threat/pattern.rs` | 10 threat categories | GoalDrift, Deception, CoordinatedAttack, etc. |
| `multi-asi-immune/src/threat/signature.rs` | Signed threat reports | Cryptographically verified threat intel |
| `multi-asi-immune/src/threat/memory.rs` | Threat memory | Remember past attacks |
| `multi-asi-immune/src/threat/mod.rs` | Module exports | — |
| **Reputation** | | |
| `multi-asi-immune/src/reputation/score.rs` | Trust scores | Decaying, non-transferable trust |
| `multi-asi-immune/src/reputation/aggregation.rs` | Transitive trust | Network-wide reputation |
| `multi-asi-immune/src/reputation/mod.rs` | Module exports | — |
| **Enforcement** | | |
| `multi-asi-immune/src/enforcement/defection.rs` | 6 defection types | Auto-isolation when threshold exceeded |
| `multi-asi-immune/src/enforcement/mod.rs` | Module exports | — |
| **Protocol** | | |
| `multi-asi-immune/src/protocol/message.rs` | Network messages | Handshake, gossip, heartbeat |
| `multi-asi-immune/src/protocol/mod.rs` | Module exports | — |
| **Attestation** | | |
| `multi-asi-immune/src/attestation/range_proof.rs` | Zero-knowledge proofs | Prove properties without revealing data |
| `multi-asi-immune/src/attestation/mod.rs` | Module exports | — |
| **Node** | | |
| `multi-asi-immune/src/node/state.rs` | Node state machine | Full ASI node lifecycle |
| `multi-asi-immune/src/node/mod.rs` | Module exports | — |
| **Integration** | | |
| `multi-asi-immune/src/integration/homeostasis_bridge.rs` | Homeostasis link | Connect immune to bounds checking |
| `multi-asi-immune/src/integration/mod.rs` | Module exports | — |
| `multi-asi-immune/src/lib.rs` | **Crate root** | Full architecture docs in comments |

### Test Files (5 files, 68 tests)

| File | Tests | What It Proves |
|------|-------|----------------|
| `multi-asi-immune/tests/identity_tests.rs` | 12 | Ed25519 sign/verify works |
| `multi-asi-immune/tests/reputation_tests.rs` | 15 | Trust decays, can't be faked |
| `multi-asi-immune/tests/threat_propagation_tests.rs` | 10 | Threats spread across network |
| `multi-asi-immune/tests/defection_tests.rs` | 8 | Rogue nodes get isolated |
| `multi-asi-immune/tests/integration_tests.rs` | 23 | Full system works together |

---

# COMPONENT 3: IMMUNITY SYSTEM (Python)
**iDEX Pitch:** "Bio-inspired defence with T-cells, antibodies, and vaccination"

### Source Files (24 files)

| File | Purpose | Defence Role |
|------|---------|--------------|
| **Core** | | |
| `agp-core/src/immunity/immune_system.py` | Main immune controller | Orchestrates innate + adaptive |
| `agp-core/src/immunity/innate.py` | Innate immunity | Fast, pre-programmed patterns |
| `agp-core/src/immunity/adaptive.py` | Adaptive immunity | Learns new threats |
| `agp-core/src/immunity/tcell.py` | T-cell simulation | Helper, killer, regulatory |
| `agp-core/src/immunity/antibody.py` | Antibody generation | Specific countermeasures |
| `agp-core/src/immunity/memory.py` | Immune memory | Remember past threats forever |
| `agp-core/src/immunity/unified.py` | Unified system | Combined innate + adaptive |
| `agp-core/src/immunity/reputation.py` | Agent reputation | Trust scoring |
| `agp-core/src/immunity/gossip.py` | Threat gossip | Distribute threat signatures |
| `agp-core/src/immunity/integration.py` | System integration | Connect to OS |
| `agp-core/src/immunity/governance_bridge.py` | Governance link | Immune → governance enforcement |
| **Training** | | |
| `agp-core/src/immunity/training/negative_selection.py` | Self-tolerance | Don't attack friendlies |
| `agp-core/src/immunity/training/live_training.py` | Online learning | Learn during operation |
| `agp-core/src/immunity/training/vaccination.py` | Pre-immunization | Pre-load known threats |
| **Experiments** | | |
| `agp-core/src/immunity/experiments/exp1_self_tolerance.py` | Test 1 | Verify friendly fire prevention |
| `agp-core/src/immunity/experiments/exp2_novel_threats.py` | Test 2 | Detect unknown attacks |
| `agp-core/src/immunity/experiments/exp3_memory_speed.py` | Test 3 | Response time benchmarks |
| `agp-core/src/immunity/experiments/exp4_clonal_selection.py` | Test 4 | Antibody evolution |
| **Evaluation** | | |
| `agp-core/src/immunity/evaluation/metrics.py` | Performance metrics | Detection rate, false positive rate |
| `agp-core/src/immunity/evaluation/benchmarks.py` | Benchmarks | Throughput testing |

---

# COMPONENT 4: HOMEOSTASIS ENGINE
**iDEX Pitch:** "Mathematical hard bounds that autonomous systems cannot exceed"

### Source Files (18 files)

| File | Purpose | Defence Role |
|------|---------|--------------|
| **Core** | | |
| `homeostasis-engine/src/core/bounds.rs` | `HardBounds` — clamp/violation | **Cannot be overridden by AI** |
| `homeostasis-engine/src/core/metric.rs` | Metric tracking | Speed, altitude, temperature |
| `homeostasis-engine/src/core/setpoint.rs` | Target values | Optimal operating point |
| `homeostasis-engine/src/core/feedback.rs` | Negative feedback | Auto-correction |
| **Constraints** | | |
| `homeostasis-engine/src/constraints/hard_bounds.rs` | Absolute limits | Physical safety limits |
| `homeostasis-engine/src/constraints/soft_bounds.rs` | Preferred range | Normal operating range |
| **Controller** | | |
| `homeostasis-engine/src/controller/single_metric.rs` | Single variable | One metric regulation |
| `homeostasis-engine/src/controller/multi_objective.rs` | Multi-variable | Balance competing constraints |
| **Solver** | | |
| `homeostasis-engine/src/solver/quadratic.rs` | Optimization | Find optimal correction |
| **Diagnostics** | | |
| `homeostasis-engine/src/diagnostics/health.rs` | Health check | System health assessment |
| **Integration** | | |
| `homeostasis-engine/src/integration/endocrine_bridge.rs` | AHES link | Connect to hormone system |

### Test Files (3 files, 52 tests)

| File | Tests | What It Proves |
|------|-------|----------------|
| `homeostasis-engine/tests/single_metric_tests.rs` | 20 | Bounds never violated |
| `homeostasis-engine/tests/multi_objective_tests.rs` | 6 | Multi-metric convergence |
| `homeostasis-engine/tests/integration_tests.rs` | 5 | `test_bounds_never_violated` ✅ |

---

# COMPONENT 5: TELOS ACCOUNTABILITY
**iDEX Pitch:** "Cryptographic proof of every AI decision for LAWS compliance"

### Rust Source (11 files, 50 tests)

| File | Purpose | Defence Role |
|------|---------|--------------|
| `telos-protocol/src/membrane.rs` | Commitment membrane | Decision gateway |
| `telos-protocol/src/entropy.rs` | Entropy meter | Rate-limits consequences |
| `telos-protocol/src/authority.rs` | Authority registry | Chain of command |
| `telos-protocol/src/trust.rs` | Trust accumulator | Earned behavioral trust |
| `telos-protocol/src/validator.rs` | External validation | Human-in-the-loop for CRITICAL |
| `telos-protocol/src/merkle.rs` | Merkle tree | Unforgeable audit log |
| `telos-protocol/src/ledger.rs` | Decision ledger | Immutable decision chain |
| `telos-protocol/src/vdf.rs` | Verifiable delay function | Time-locked entropy proofs |
| `telos-protocol/src/network.rs` | Validator network | Distributed attestation |
| `telos-protocol/src/error.rs` | Error types | — |
| `telos-protocol/src/lib.rs` | Crate root | — |

### Python Source (2 files)

| File | Purpose | Defence Role |
|------|---------|--------------|
| `agp-core/src/telos/membrane.py` | Python membrane | 5 consequence tiers + entropy budget |
| `agp-core/src/telos/__init__.py` | Module init | — |

---

# COMPONENT 6: AGP-OS (Robotic Operating System)
**iDEX Pitch:** "Complete OS for governed military robots with ROS2 integration"

### Source Files (35 files, 6,189 LOC)

| File | Purpose | Defence Role |
|------|---------|--------------|
| **Kernel** | | |
| `agp-core/src/os/kernel.py` | BioKernel | Process management |
| `agp-core/src/os/process.py` | Process control block | Agent lifecycle |
| `agp-core/src/os/scheduler.py` | Scheduler | Priority scheduling |
| `agp-core/src/os/syscalls.py` | System calls | Governed kernel interface |
| `agp-core/src/os/context_manager.py` | Context switching | Memory management |
| **Real-Time** | | |
| `agp-core/src/os/rtos/scheduler.py` | RTOS scheduler | 5-priority real-time scheduling |
| **Robotics** | | |
| `agp-core/src/os/hal/hal.py` | Hardware abstraction | Sensor/actuator safety interlocks |
| `agp-core/src/os/ros2/bridge.py` | ROS2 bridge | Topic/service/action integration |
| `agp-core/src/os/ros2/production.py` | Production ROS2 | Hardware deployment adapter |
| **Coordination** | | |
| `agp-core/src/os/mesh/mesh.py` | Mesh coordinator | Multi-robot consensus + mailbox |
| **Resources** | | |
| `agp-core/src/os/resources/controller.py` | Resource controller | CPU/memory/token quotas |
| `agp-core/src/os/budget.py` | Token budget | LLM cost control |
| **Communication** | | |
| `agp-core/src/os/ipc/message_queue.py` | Message queues | Inter-process communication |
| `agp-core/src/os/ipc/shared_memory.py` | Shared memory | Fast data sharing |
| `agp-core/src/os/ipc/signals.py` | Signals | SIGTERM, SIGSTOP |
| `agp-core/src/os/network/manager.py` | Network manager | Connection handling |
| `agp-core/src/os/network/websocket.py` | WebSocket | Real-time communication |
| **Resilience** | | |
| `agp-core/src/os/recovery/checkpoint.py` | Checkpoint/restore | Field recovery |
| `agp-core/src/os/resilience/circuit_breaker.py` | Circuit breaker | Failure isolation |
| **Storage** | | |
| `agp-core/src/os/persistence/database.py` | SQLite backend | Persistent state |
| `agp-core/src/os/fs/vfs.py` | Virtual filesystem | /proc, /home, /shared |
| **Observability** | | |
| `agp-core/src/os/observability/prometheus.py` | Metrics | Prometheus monitoring |
| `agp-core/src/os/logging/syslog.py` | System logging | Audit trail |
| **Security** | | |
| `agp-core/src/os/security/auth.py` | JWT auth | Access control |

---

# COMPONENT 7: SUPPORTING SYSTEMS

### Bio-Governance

| File | Purpose | Tests |
|------|---------|-------|
| `nervous-system/src/integration/coordinator.rs` | Central coordinator | 8 |
| `nervous-system/src/perception/processor.rs` | Sensor processing | — |
| `nervous-system/src/decision/engine.rs` | Decision engine | — |
| `nervous-system/src/motor/executor.rs` | Action execution | — |
| `nervous-system/src/integration/safety.rs` | Safety state | — |
| `autonomic-system/src/mode/controller.rs` | CALM/ACT/EMERGENCY | 10 |
| `autonomic-system/src/regulation/transition.rs` | Mode transitions | — |
| `autonomic-system/src/reflex/response.rs` | Reflex responses | — |
| `developmental-gates/src/stage/manager.rs` | Stage progression | 13 |
| `developmental-gates/src/gate/enforcer.rs` | Capability gating | — |
| `developmental-gates/src/capability/registry.rs` | What each stage can do | — |
| `agp-core/src/ahes/endocrine.py` | 8-hormone system | — |

### Governance

| File | Purpose |
|------|---------|
| `agp-core/src/governance/rules.py` | Policy rules engine |
| `agp-core/src/governance/alignment.py` | Alignment scoring |
| `agp-core/src/governance/anomaly.py` | Anomaly detection |
| `agp-core/src/governance/enforcer.py` | Action enforcement |
| `agp-core/src/governance/behavioral_rag.py` | Behavioral history search |
| `agp-core/src/governance/impact.py` | Impact assessment |

### Post-Quantum Crypto

| File | Purpose |
|------|---------|
| `nexus-pcu/src/pqc.rs` | Hybrid Ed25519 + ML-DSA signatures |
| `nexus-pcu/src/identity.rs` | Cryptographic identity |
| `nexus-pcu/src/crypto.rs` | Signing/verification |
| `nexus-pcu/src/pcu.rs` | Portable Computation Unit |

---

# QUICK REFERENCE: FILE COUNTS

| Component | Src Files | Test Files | Tests | LOC |
|-----------|-----------|------------|-------|-----|
| Execution Guards | 15 | 5 | 5 | ~400 |
| Multi-ASI Immune (Rust) | 20 | 5 | 68 | 2,715 |
| Immunity (Python) | 24 | 2 | 54 | 4,349 |
| Homeostasis Engine | 18 | 3 | 52 | 2,290 |
| TELOS (Rust) | 11 | — | 50 | 4,014 |
| TELOS (Python) | 2 | 1 | 3 | 336 |
| AGP-OS | 35 | 15 | 58+ | 6,189 |
| Nervous System | 10 | — | 8 | 906 |
| Autonomic System | 8 | — | 10 | 683 |
| Developmental Gates | 8 | — | 13 | 915 |
| AHES | 2 | 1 | — | 341 |
| Governance | 7 | 4 | 5+ | 1,682 |
| PQC + Identity | 10 | — | 72 | ~2,000 |
| **TOTAL** | **170** | **36** | **398+** | **26,820** |

---

**© 2026 SYNTRIASS Labs Private Limited. All rights reserved.**  
**Inventor:** Katta Naga Sri Ganesh
