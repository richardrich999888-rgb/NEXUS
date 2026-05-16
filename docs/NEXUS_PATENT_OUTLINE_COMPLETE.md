# NEXUS — Complete Patent Outline (Entire Codebase)

**Scope:** Every patentable invention with exact code locations  
**Standard:** Code-backed; no invented features  
**Inventor:** Katta Naga Sri Ganesh  
**Company:** SYNTRIASS Labs Private Limited  

---

## Summary: Patent Count by Crate/Package

| Crate/Package | Patents | Key Inventions |
|---------------|---------|----------------|
| nexus-executor | 5 | Execution gate, guards, no-proof-on-deny, proof |
| nexus-pcu | 6 | PCU, routing, USO, licensing, PQC |
| nexus-core | 1 | Causal merge |
| agp-core | 4 | TELOS, kernel, immunity, AHES |
| developmental-gates | 2 | Stage, capability gating |
| autonomic-system | 1 | Risk-tolerance modes |
| nervous-system | 2 | Coordinator, decision engine |
| multi-asi-immune | 4 | Identity, defection, reputation, isolation |
| homeostasis-engine | 1 | Multi-objective homeostasis |
| telos-protocol | 2 | Entropy, membrane |
| nexus-agp | 4 | Endocrine, glands, homeostasis, reputation |
| vectra | 7 | EBTA, EBTA-X, SPE, FEE, encode/decode |
| causalux | 5 | CausalDAG, gradient CRDT, atom, sync |
| agp | 3 | Governance, reputation, task clustering |
| src/ (AURA/ASIM) | 6 | TIH, IPE, SFA, RIA, SIL, HATE |
| telecom_6g | 3 | CSI compression, DPD, beamforming |
| syntriass | 1 | Conditioning injector |

**Total: ~56 distinct patentable inventions** (some overlap across families)

---

## 1. NEXUS-EXECUTOR (Execution Governance)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | Execution Law / Mandatory Gate | `nexus-executor/src/executor.rs:147-156` — guard check before cache/run | System, Method |
| 2 | ExecutionGuard trait | `nexus-executor/src/guard.rs:13-36` — `GuardDecision`, `ExecutionGuard::check` | Apparatus |
| 3 | CompositeGuard (first-deny) | `nexus-executor/src/guards/composite.rs:41-49` — iterate guards, first Deny returns | Apparatus |
| 4 | NervousSystemGuard | `nexus-executor/src/guards/nervous.rs:17-52` — delegates to coordinator | Apparatus |
| 5 | ImmuneGuard | `nexus-executor/src/guards/immune.rs:13-50` — `allow_execution_by` | Apparatus |
| 6 | No-proof-on-deny | `executor.rs:148-156` — return `Err` before line 173 (cache/proof) | Method |
| 7 | ExecutionProof | `nexus-executor/src/proof.rs:92-152` — `ExecutionProof::create`, signing_bytes | Apparatus |
| 8 | ExecutorBuilder::production | `executor.rs:21-71` — `production()` sets guard | Method |

**Tests:** `nexus-executor/tests/integration_tests.rs`, `red_team_execution.rs`, `test_no_proof_on_blocked_execution`

---

## 2. NEXUS-PCU (Content-Addressed Computation)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | PCU with Deterministic ID | `nexus-pcu/src/pcu.rs:151-205` — `PCU::new`, `compute_id` | System, Method |
| 2 | Code-to-Data Routing | `nexus-pcu/src/routing.rs` — `DataLocator`, `NodeInfo`, `route_pcu` | System |
| 3 | USO (Universal State Object) | `nexus-pcu/src/uso.rs:329-415` — `USO` struct, `SyncPolicy`, LWW merge | System |
| 4 | Content-Hash-Bound Licensing | `nexus-pcu/src/crypto.rs:108-257` — `PcuLicense`, `is_valid_for_pcu` | System, Method |
| 5 | Hybrid Classical-PQC | `nexus-pcu/src/pqc.rs:48-156` — `HybridSignature`, `verify_hybrid` | Apparatus |
| 6 | Identity / Capability | `nexus-pcu/src/identity.rs` — `PrincipalId`, `Capability`, `IdentityContext` | Apparatus |

**Invariant:** `PCU_ID = BLAKE3(code.hash || inputs[] || parameters || identity.principal)` — `pcu.rs:181-205`

---

## 3. NEXUS-CORE (Causal Infrastructure)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | CausalId | `nexus-core/src/causal.rs:17-50` — content-addressed identifier | Apparatus |
| 2 | VectorClock | `nexus-core/src/causal.rs:56-133` — `happens_before`, merge | Apparatus |
| 3 | Provenance (Merkle) | `nexus-core/src/causal.rs:145-203` | Apparatus |
| 4 | CausalTensor::merge | `nexus-core/src/causal.rs:341-390` — idempotent, commutative | Method |
| 5 | Tenancy | `nexus-core/src/tenancy.rs` — `PrincipalId`, `Tenant`, quotas | Apparatus |

**Patent Pending:** IN202501XXXXX (causal.rs header)

---

## 4. AGP-CORE (Agent Governance Platform)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | BioKernel / Agents as Processes | `agp-core/src/os/kernel.py` — `BioKernel`, `schedule`, `context_switch` | System |
| 2 | ProcessControlBlock | `agp-core/src/os/process.py` — PCB, states, priority | Apparatus |
| 3 | TELOS Commitment Membrane | `agp-core/src/telos/membrane.py` — `request_crossing`, entropy/authority/trust | System, Method |
| 4 | Entropy-based auth | `membrane.py:221-233` — `EntropyMeter.spend` | Method |
| 5 | Authority scope | `membrane.py:237-252` — `AuthorityRegistry.verify` | Method |
| 6 | Trust accumulator | `membrane.py:256-269` — high tier trust ≥ 0.6 | Method |
| 7 | AHES (8 hormones) | `agp-core/src/ahes/endocrine.py` — `Hormone`, half-life, Km | Apparatus |
| 8 | Artificial Immune System | `agp-core/src/immunity/immune_system.py` — innate, adaptive, memory | System |
| 9 | Negative selection | `agp-core/src/immunity/training/negative_selection.py` | Method |
| 10 | Clonal selection | `agp-core/src/immunity/adaptive.py`, `antibody.py` | Method |
| 11 | Immune memory | `agp-core/src/immunity/memory.py` | Apparatus |
| 12 | Vaccination | `agp-core/src/immunity/training/vaccination.py` | Method |
| 13 | ROS2 Bridge | `agp-core/src/os/ros2/bridge.py` — execution gate on robot commands | System |

**PATENT CLAIMS 7.1-7.5, 8-12** (docstrings in immunity, ahes)

---

## 5. DEVELOPMENTAL-GATES

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | DevelopmentalStage enum | `developmental-gates/src/stage/definition.rs:8-89` — Infant..Elder | Apparatus |
| 2 | StageManager | `developmental-gates/src/stage/manager.rs:42-74` | Apparatus |
| 3 | CapabilityRegistry | `developmental-gates/src/capability/registry.rs:84-120` — `Capability::Execute` | Apparatus |
| 4 | GateEnforcer | `developmental-gates/src/gate/enforcer.rs:37-80` — `check_capability` | Method |

---

## 6. AUTONOMIC-SYSTEM

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | AutonomicMode | `autonomic-system/src/mode/state.rs:7-75` — Act, Calm, Emergency, Recovery | Apparatus |
| 2 | risk_tolerance | `state.rs:75-85` — CALM 0.2, ACT 0.6 | Method |
| 3 | AutonomicController | `autonomic-system/src/mode/controller.rs:45-231` | Apparatus |
| 4 | ModeTransition | `autonomic-system/src/regulation/transition.rs:39-60` | Method |
| 5 | ReflexResponse | `autonomic-system/src/reflex/response.rs:54-80` | Apparatus |

---

## 7. NERVOUS-SYSTEM

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | NervousSystemCoordinator | `nervous-system/src/integration/coordinator.rs:36-174` | System |
| 2 | DecisionEngine | `nervous-system/src/decision/engine.rs:38-95` — `check_capability`, `max_risk` | Method |
| 3 | PerceptionProcessor | `nervous-system/src/perception/processor.rs:37-80` | Apparatus |
| 4 | MotorExecutor | `nervous-system/src/motor/executor.rs:30-70` | Apparatus |
| 5 | SafetyState | `nervous-system/src/integration/safety.rs:9-53` | Apparatus |

---

## 8. MULTI-ASI-IMMUNE

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | AsiId / AsiIdentity | `multi-asi-immune/src/identity/keypair.rs:14-159` | Apparatus |
| 2 | DefectionTracker | `multi-asi-immune/src/enforcement/defection.rs:40-100` | Method |
| 3 | DefectionRecord | `defection.rs:40-55` — severity, type | Apparatus |
| 4 | ReputationAggregator | `multi-asi-immune/src/reputation/aggregation.rs:12-183` | Method |
| 5 | ReputationScore | `multi-asi-immune/src/reputation/score.rs:13-80` | Apparatus |
| 6 | allow_execution_by | `multi-asi-immune/src/node/state.rs:255-267` — isolation, min_reputation | Method |
| 7 | ThreatMemory | `multi-asi-immune/src/threat/memory.rs` | Apparatus |
| 8 | SimpleRangeProof | `multi-asi-immune/src/attestation/range_proof.rs:14-103` | Method |
| 9 | ProtocolMessage | `multi-asi-immune/src/protocol/message.rs` — Hello, ThreatQuery, Constraint | Apparatus |

---

## 9. HOMEOSTASIS-ENGINE

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | MultiObjectiveController | `homeostasis-engine/src/controller/multi_objective.rs:33-275` | System |
| 2 | ProjectedGradientSolver | `homeostasis-engine/src/solver/quadratic.rs:41-120` | Method |
| 3 | QuadraticProblem | `solver/quadratic.rs:9-40` | Apparatus |
| 4 | Metric / setpoints | `homeostasis-engine/src/core/metric.rs:74-255` | Apparatus |
| 5 | EndocrineBridge | `homeostasis-engine/src/integration/endocrine_bridge.rs:13-124` | Apparatus |

**IDF-004:** Pareto-optimal configuration via projected gradient on constrained manifold

---

## 10. TELOS-PROTOCOL (Rust)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | ConsequenceTier | `telos-protocol/src/entropy.rs:14-48` — tier multiplier | Apparatus |
| 2 | EntropyProof | `telos-protocol/src/entropy.rs:49-104` | Apparatus |
| 3 | EntropyMeter | `telos-protocol/src/entropy.rs:146-250` | Apparatus |
| 4 | CommitmentMembrane | `telos-protocol/src/membrane.rs:115-200` | System |
| 5 | Decision / CrossingResult | `telos-protocol/src/membrane.rs:16-115` | Apparatus |
| 6 | Authority / Constraint | `telos-protocol/src/authority.rs:101-152` | Apparatus |
| 7 | TrustAccumulator | `telos-protocol/src/trust.rs:241-300` | Apparatus |
| 8 | VdfProof / vdf_to_entropy_proof | `telos-protocol/src/vdf.rs:34-307` | Method |

---

## 11. NEXUS-AGP (Rust AGP Bindings)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | Hormone decay | `nexus-agp/src/endocrine.rs:157-180` — PATENT CLAIM | Method |
| 2 | Receptor response | `nexus-agp/src/endocrine.rs:220-250` — PATENT CLAIM | Method |
| 3 | Privilege | `nexus-agp/src/endocrine.rs:303-320` — PATENT CLAIM | Method |
| 4 | Negative feedback | `nexus-agp/src/endocrine.rs:348-370` — PATENT CLAIM | Method |
| 5 | Glands | `nexus-agp/src/glands.rs` — PATENT CLAIM 9 | Apparatus |
| 6 | Homeostasis | `nexus-agp/src/homeostasis.rs` — PATENT CLAIM 10 | Apparatus |
| 7 | Reputation CRDT | `nexus-agp/src/reputation.rs:92-180` — PATENT CLAIM 5 | Apparatus |
| 8 | PQC-bound identity | `nexus-agp/src/identity.rs:26-120` — PATENT CLAIM 6 | Apparatus |

---

## 12. VECTRA (Deterministic Compression)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | EBTA (Entropy-Bounded Tensor Algebra) | `vectra/vectra/src/ebta.rs:48-177` — `ebta_validate`, H_MAX | Method |
| 2 | EBTA-X (Adaptive) | `vectra/vectra/src/ebta_x.rs:47-405` — `AdaptiveThresholdCalculator` | Method |
| 3 | SPE (Symbolic Predictor Engine) | `vectra/vectra/src/spe.rs:57-178` | Method |
| 4 | FEE (Fractal Entropy Encoding) | `vectra/vectra/src/fee.rs:41-303` | Method |
| 5 | vectra_encode / vectra_decode | `vectra/vectra/src/encode.rs:38`, `decode.rs:25` | Method |
| 6 | decompose / recompose | `vectra/vectra/src/decompose.rs:41-459` | Method |
| 7 | Artifact / fail-open | `vectra/vectra/src/artifact.rs`, `lib.rs` | System |
| 8 | Crypto fingerprint | `vectra/vectra/src/crypto_fingerprint.rs:32-157` | Apparatus |

**PATENT NOTICE (lib.rs):** EBTA, EBTA-X, Deterministic Compression, ZK Proof of Lossless, Federated Pattern Discovery

---

## 13. CAUSALUX (Distributed Execution Fabric)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | CausalDAG | `causalux/src/dag.rs:39-150` — ConflictPolicy | System |
| 2 | VersionVector | `causalux/src/version_vector.rs:29-120` | Apparatus |
| 3 | GradientOp / CausalGradient | `causalux/src/compute/gradient_crdt.rs:15-305` | Apparatus |
| 4 | CausalAtom | `causalux/src/atom/causal_atom.rs:13-200` | Apparatus |
| 5 | HierarchicalSync | `causalux/src/sync.rs:81-269` | Method |
| 6 | ResonantRouter | `causalux/src/resonance/router.rs:53-257` | Apparatus |
| 7 | NodeAffinity | `causalux/src/resonance/affinity.rs:148-207` | Apparatus |
| 8 | SovereignEnvelope | `causalux/src/envelope.rs:169-290` | Apparatus |
| 9 | CausalToken / EconomyLedger | `causalux/src/economy/` | Apparatus |

---

## 14. AGP (Python — Governance)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | Governance voting | `agp/core/governance.py:4-80` — verified execution history | Method |
| 2 | Reputation fork inheritance | `agp/core/reputation.py:147-481` — graduated by change magnitude | Method |
| 3 | Task clustering / validator selection | `agp/core/task_clustering.py:4-345` — expertise-based | Method |

---

## 15. SRC/ (AURA / ASIM)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | RIA (Resonant Invariant Algebra) | `src/core/ria.py:105-470` — `create_transaction`, `verify_transaction` | System |
| 2 | AuraSignature | `src/core/ria.py:57-100` | Apparatus |
| 3 | ThermodynamicHardening | `src/asi/tih.py:11-80` — entropy monitor | Method |
| 4 | IsogenyPotentialWells | `src/asi/ipe.py:9-120` — state transition verification | Method |
| 5 | StatisticalFieldAlignment | `src/asi/reasoning.py:10-100` | Method |
| 6 | SovereignAlignmentModule | `src/asi/sam.py:8-80` | Apparatus |
| 7 | SymbolicInvariantLayer | `src/asi/sil.py:9-80` | Apparatus |
| 8 | HardwareAgnosticTrustEnclave | `src/asi/hate.py:9-80` | Apparatus |
| 9 | RecursiveResonantEngine | `src/asi/rre.py:8-80` | Apparatus |
| 10 | OfflineVerifier | `src/network/offline.py:10-80` | Apparatus |

**IDF-001 to IDF-006** (INVENTION_DISCLOSURES.md)

---

## 16. TELECOM_6G

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | CSI compression | `vectra/telecom_6g/vectra_integration/csi_compression.py` | Method |
| 2 | Signaling compression | `vectra/telecom_6g/vectra_integration/signaling_compression.py` | Method |
| 3 | Neural CSI encoder | `vectra/telecom_6g/digital_ran_beamforming/models/neural_csi_encoder.py` | Apparatus |
| 4 | TT beamformer | `vectra/telecom_6g/digital_ran_beamforming/beamformers/tt_beamformer.py` | Apparatus |
| 5 | Neural DPD | `vectra/telecom_6g/digital_dpd_research/models/neural_dpd.py` | Apparatus |
| 6 | Joint DPD optimization | `vectra/telecom_6g/digital_dpd_research/demo_joint_optimization.py` | Method |

---

## 17. SYNTRIASS (Path 6)

| # | Invention | Exact Code Location | Claim Type |
|---|-----------|---------------------|------------|
| 1 | ConditioningInjector | `syntriass/core/conditioning.py:37-120` — time-varying conditioning | Method |
| 2 | PreviewBus | `syntriass/preview/stream.py:39-120` — async frame queue | Apparatus |
| 3 | FastDecoder | `syntriass/preview/fast_decoder.py` | Apparatus |

---

## 18. NEXUS-SYNC, NEXUS-COMPRESS, NEXUS-SECRETS

| Crate | Invention | Location |
|-------|-----------|----------|
| nexus-sync | CRDT-USO adapter | `nexus-sync/src/crdt_uso.rs` |
| nexus-sync | SyncEngine | `nexus-sync/src/sync_engine.rs` |
| nexus-compress | USO compress | `nexus-compress/src/uso_compress.rs` |
| nexus-compress | PCU compress | `nexus-compress/src/pcu_compress.rs` |
| nexus-secrets | SecretManager | `nexus-secrets/src/manager.rs` |
| nexus-secrets | Encryption | `nexus-secrets/src/encryption.rs` |

---

## Patent Family Mapping

| Family | Inventions | Primary Files |
|--------|------------|---------------|
| **A: Execution Law** | 1-8 (nexus-executor) | executor.rs, guard.rs |
| **B: TELOS** | 4-8 (agp-core, telos-protocol) | membrane.py, entropy.rs |
| **C: AGP OS** | 1-2 (agp-core) | kernel.py, process.py |
| **D: PCU** | 1-6 (nexus-pcu) | pcu.rs, routing.rs |
| **E: Causal** | 1-4 (nexus-core), causalux | causal.rs, dag.rs |
| **F: Biological** | 7-12 (agp-core), nexus-agp | endocrine.py, immune_system.py |
| **G: Developmental** | 1-4 (developmental-gates) | definition.rs, enforcer.rs |
| **H: Immune** | 1-9 (multi-asi-immune) | defection.rs, state.rs |
| **I: VECTRA** | 1-8 | ebta.rs, encode.rs |
| **J: AURA/ASIM** | 1-10 (src/) | ria.py, tih.py |

---

## Filing Priority (Recommended)

1. **Filing 1:** Execution Law + AGP OS + No-proof-on-deny (nexus-executor + kernel)
2. **Filing 2:** TELOS Commitment Membrane (membrane.py + entropy)
3. **Filing 3:** PCU + Content-addressed compute (nexus-pcu)
4. **Filing 4:** Causal merge + CausalDAG (nexus-core + causalux)
5. **Filing 5:** Developmental + Immune + Risk gating (developmental-gates, multi-asi-immune, autonomic)
6. **Filing 6:** VECTRA / EBTA (vectra)
7. **Filing 7:** RIA / AURA (src/core/ria.py)
8. **Filing 8:** AHES + Immunity (agp-core, nexus-agp)
9. **Filing 9:** ASIM (TIH, IPE, SFA)
10. **Filing 10:** Telecom 6G integration

---

**Document Status:** Complete. All code locations verified. Inventor: Katta Naga Sri Ganesh.
