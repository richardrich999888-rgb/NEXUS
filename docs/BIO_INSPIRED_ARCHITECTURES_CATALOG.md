# Biologically Inspired Architectures & Frameworks — Full NEXUS Catalog

**Purpose:** Catalog **all** biologically inspired architectures and frameworks present across the entire NEXUS project. Each entry states **where** it lives and **what** is implemented (no theory-only).

---

## Overview Map

| Category | Where | Biological analogy |
|----------|-------|--------------------|
| **1. Endocrine** | nexus-agp, agp-core/ahes | Hormones, glands, receptors, circadian, allostasis |
| **2. Nervous system** | nervous-system, autonomic-system, developmental-gates, homeostasis-engine | Perception–decision–motor, ACT/CALM, reflexes, developmental stages |
| **3. Immune** | multi-asi-immune (Rust), agp-core/immunity (Python) | Innate/adaptive immunity, antibodies, T-cells, clonal selection, vaccination |
| **4. Commitment membrane** | telos-protocol, agp-core/telos | Boundary between reasoning and action (membrane crossing) |
| **5. Homeostasis / allostasis** | homeostasis-engine, nexus-agp/homeostasis, multi-asi-immune bridge | Setpoints, bounds, negative feedback, adaptive setpoints |
| **6. Reflexes** | autonomic-system/reflex | Automatic emergency responses (brake, caution, alert) |
| **7. Developmental stages** | developmental-gates | Infant → Child → Adolescent → Adult → Elder |
| **8. Lyapunov stability** | nexus-telecom (Python) | Control-theoretic stability (BIBO) for AI actions |
| **9. Bridges** | homeostasis-engine/endocrine_bridge, multi-asi-immune/homeostasis_bridge, agp-core immunity/telos/ahes | Endocrine↔homeostasis, immune↔governance, endocrine↔immune |

---

## 1. Endocrine-like (AHES)

**Locations:** `nexus-agp/` (Rust), `agp-core/src/ahes/` (Python).

**Presented in project:**

- **8 hormones** mapped to reputation dimensions: Cortisol (Accuracy), Oxytocin (Cooperation), Serotonin (Stability), Dopamine (Uniqueness), Adrenaline (Latency), Endorphins (Ethics), Norepinephrine (Novelty), GrowthHormone (Longevity).
- **Kinetics:** First-order half-life decay, Michaelis–Menten receptor saturation, negative feedback inhibition, circadian modulation (24 h, phase per hormone), receptor down/up-regulation.
- **Virtual glands:** HypothalamicController (master regulator, stress/activity), PituitaryRouter (amplification), PerformanceGland (Cortisol, Adrenaline), CooperationGland (Oxytocin), StabilityGland (Serotonin, Endorphins), ExplorationGland (Dopamine, Norepinephrine), DevelopmentGland (GrowthHormone).
- **Stimuli:** TaskSuccess/Failure, Collaboration, NovelSolution, Urgency, EthicalCompliance, Exploration, Consistency.
- **Homeostasis controller (nexus-agp):** SetPoints, negative/positive feedback loops, CircadianController, AllostasisManager (adaptive setpoints, allostatic load), HealthStatus (Optimal/Normal/Stressed/Critical).

**Patent claims in code:** 8–12 (Bio-inspired Computational Governance, Hormone Secretion, Biological Feedback Loops, Circadian Rhythm, Allostatic Adaptation).

---

## 2. Nervous-system–like (perception–decision–motor + autonomic + developmental)

**Locations:** `nervous-system/`, `autonomic-system/`, `developmental-gates/`, `homeostasis-engine/`.

**Presented in project:**

- **Perception** (`nervous-system/perception`): InputType (Text, Command, Sensor, Network, Internal) → Perception (risk_level, required_capabilities, intent). PerceptionProcessor with history.
- **Decision** (`nervous-system/decision`): DecisionEngine uses autonomic mode and developmental stage; checks capability vs stage and risk vs risk_tolerance; returns Approved | Modified | Blocked | NoAction.
- **Motor** (`nervous-system/motor`): MotorExecutor runs approved/modified actions; records ExecutionRecord; can pause/resume.
- **Autonomic** (`autonomic-system`): Modes ACT (sympathetic-like), CALM (parasympathetic-like), Emergency, Recovery. Arousal with first-order dynamics; thresholds and min_mode_duration; reflex queue (EmergencyBrake, etc.). BehaviorModifiers: risk_tolerance, speed_factor, reflexes_enabled.
- **Developmental** (`developmental-gates`): Stages Infant → Child → Adolescent → Adult → Elder; StageManager (advance/regress from stability/violations); CapabilityRegistry; GateEnforcer (allow/deny by stage).
- **Homeostasis** (`homeostasis-engine`): Metrics with setpoints and bounds; SingleMetricController, MultiObjectiveController; used as stress/stability input for autonomic and developmental layers.
- **Coordinator** (`nervous-system/integration`): NervousSystemCoordinator.process(input, proposed_action) runs: perception → autonomic.update_from_homeostasis → stages.update → decision.decide → motor.execute. SafetyState exposes mode, stage, arousal, health, active_threats.

**Formal spec:** `brain/formal_specs/SafetyProtocol.tla` — modes, arousal, duration, reflex_queue, transitions, EmergencySafety, ReturnsToCalm.

---

## 3. Immune-like

**Locations:** `multi-asi-immune/` (Rust), `agp-core/src/immunity/` (Python).

### 3.1 Multi-ASI Immune (Rust)

**Presented:** Distributed “immune” protocol for multi-ASI coordination:

- **Identity:** Ed25519; AsiIdentity, AsiId.
- **Reputation:** ReputationScore (earned, decaying), ReputationAggregator (transitive trust).
- **Threat:** ThreatPattern (category, pattern_hash, severity), SignedThreatReport, ThreatMemory (dedup, aggregate confidence).
- **Protocol:** Handshake, gossip threats, negotiate constraints, heartbeat.
- **Enforcement:** Defection detection, isolation.
- **Bridge:** `multi-asi-immune/src/integration/homeostasis_bridge.rs` — StandardMetrics (stress, curiosity, urgency, fatigue, caution, cooperation, wellbeing, growth) mapped to protocol; mutual constraints from homeostatic state.

### 3.2 Artificial Immune System (Python — agp-core/immunity)

**Presented:** AIS-ASI — bio-inspired safety for AI:

- **Innate immunity:** PatternDetector, InnateImmuneSystem — fast, pattern-based threat scan.
- **Adaptive immunity:** Antibody, AntibodyPool (clonal selection), TCell (Helper, Killer, Regulatory), TCellPopulation, AdaptiveImmuneSystem — learned, specific responses; memory.
- **Memory:** MemoryCell, MemoryBank, MemoryMetadata — rapid recall of past threats.
- **Training:** vaccination, negative_selection, live_training — self-tolerance and threat exposure.
- **Experiments:** exp1_self_tolerance, exp2_novel_threats, exp3_memory_speed, exp4_clonal_selection.
- **Governance bridge:** `agp-core/src/immunity/governance_bridge.py` — GovernanceImmuneBridge: threat/defection signals → governance actions; immune_suppressed/restore; ThreatLevel (LOW/MEDIUM/HIGH), antibody_match, tcell_activation in signals.
- **Endocrine–immune integration:** EndocrineImmuneIntegration, IntegratedBioSafetySystem — AHES and immune system combined.

**Docstring in code:** “Multi-layered defense: Innate immunity, Adaptive immunity, Memory cells, Clonal selection, Negative selection.”

---

## 4. Commitment membrane (TELOS)

**Locations:** `telos-protocol/` (Rust), `agp-core/src/telos/` (Python).

**Presented in project:**

- **Membrane** (`telos-protocol/src/membrane.rs`, `agp-core/src/telos/membrane.py`): Boundary between “reversible reasoning” and “irreversible action.” Decision (Draft → Pending → Validating → Committed); request_crossing consumes entropy and checks authority; CrossingResult (Committed | Rejected | PendingValidation).
- **Entropy meter** (`telos-protocol/src/entropy.rs`): EntropyMeter with budget; ConsequenceTier; entropy consumed on crossing.
- **Authority** (`telos-protocol/src/authority.rs`): AuthorityRegistry, AgentId, delegation, constraints.
- **Validator network** (`telos-protocol`): Validator, Attestation, external validators, slashable stake.
- **Trust** (`telos-protocol/src/trust.rs`): TrustAccumulator, TrustScore, CommitmentHistory.

**Biological/cognitive framing in docs:** “Commitment membrane between AI reasoning and action” — crossing requires entropy, authority, attestation; creates “unforkable trust.”

---

## 5. Homeostasis & allostasis

**Locations:** `homeostasis-engine/`, `nexus-agp/src/homeostasis.rs`, bridges.

**Presented in project:**

- **homeostasis-engine:** Metric (value, setpoint, bounds, gain, weight); HardBounds; SingleMetricController (proportional + damping); MultiObjectiveController (convergence, bounds); AdaptiveSetpoint (fixed or adaptive); HealthCheck, HealthStatus. Endocrine_bridge: EndocrineMetrics (stress, curiosity, urgency, fatigue, caution, cooperation, wellbeing, growth) registered as metrics — “hormone-like metrics that modulate cognition.”
- **nexus-agp/homeostasis:** SetPoint (target, tolerance, adaptation_rate); FeedbackLoop (negative/positive); CircadianController; AllostasisManager (history, adapt_setpoints, allostatic_load, in_balance); HomeostasisController (tick, feedback_factor, health_status).
- **multi-asi-immune/homeostasis_bridge:** StandardMetrics (same 8 names) for inter-ASI protocol; HomeostaticBridge maps homeostatic state to mutual constraints.

---

## 6. Reflexes

**Locations:** `autonomic-system/src/reflex/`.

**Presented in project:**

- **ReflexType:** EmergencyBrake, ResourceConservation, HeightenedCaution, HumanAlert, SuspendNonCritical, DefensivePosture, RequestHelp — each with priority and description.
- **ReflexResponse:** reflex_type, strength, timestamp.
- **AutonomicController:** When mode is ACT or Emergency and arousal is critical, queues reflex (e.g. EmergencyBrake). take_reflex() pops from queue.
- **TLA+:** TriggerReflex action appends to reflex_queue; IsSafeSequence(reflex_queue) in invariants.

**Biological framing:** Automatic, involuntary responses to critical arousal (analogous to fight-or-flight reflexes).

---

## 7. Developmental stages

**Locations:** `developmental-gates/`.

**Presented in project:**

- **DevelopmentalStage:** Infant (observation only), Child (basic, supervised), Adolescent (extended, limited autonomy), Adult (full, self-regulated), Elder (full + mentoring). StageRequirements per stage; advancement/regression by stability and violations.
- **StageManager:** current_stage, time_at_stage, stability_history, violations, successes; assessment_interval; advance or regress with TransitionReason (Advancement, Regression, ManualOverride).
- **GateEnforcer:** Checks capability vs current_stage; AccessResult Allowed | Denied(required, current) | UnknownCapability | Suspended.

**Biological framing in comments:** “Like human cognitive development, capabilities unlock progressively.”

---

## 8. Lyapunov stability (control-theoretic, safety framing)

**Location:** `nexus-telecom/src/nexus_telecom/control/` (Python).

**Presented in project:**

- **LyapunovController:** V(x) = xᵀ P x; dV(x, x_next); is_safe(x, x_next) ⟺ V(x_next) − V(x) < −α·V(x) (decay); filter_action applies safe action or fallback. create_identity(dim), create_weighted(weights). Statistics: approval/rejection counts.
- **Framing in docstring:** “Lyapunov-based safety supervisor for AI actions”; “Guarantees V̇(x) < 0”; “BIBO stability regardless of AI model complexity.”

Not literally “biological” but **stability/regulation** in the same spirit as homeostasis and reflexes; often grouped with bio-inspired control in the project docs (e.g. ENGINEERING_AUDIT.md).

---

## 9. CAUSALUX “visionary” layers (named, optional features)

**Location:** `causalux/src/lib.rs` and feature-gated modules.

**Presented in project:**

- **Morgan Economy** (feature `economy`): Token-based metering, incentive alignment — CausalToken, TokenBalance, EconomyLedger, OperationPricing.
- **Tesla Resonance** (feature `resonance`): “Smart sync routing via data affinity patterns” — AffinityTracker, ResonantRouter, RoutingDecision.
- **Da Vinci Atom** (feature `atom`): “Unified primitive for all data types” — CausalAtom, AtomValue, AtomComposer, CompositeAtom.

Names are evocative (Morgan, Tesla, Da Vinci); “resonance” and “affinity” have a systems-biology flavor; economy/atom are more abstract. Listed here for completeness as **frameworks present in the project** that are described in bio/visionary language in the crate docs.

---

## 10. Bridges (cross-system bio-inspired integration)

| Bridge | Location | What it does |
|--------|----------|--------------|
| **Endocrine ↔ Homeostasis** | homeostasis-engine/src/integration/endocrine_bridge.rs | Registers EndocrineMetrics (stress, curiosity, urgency, fatigue, caution, cooperation, wellbeing, growth) as homeostasis-engine metrics; “hormone-like metrics that modulate cognition.” |
| **Immune ↔ Homeostasis** | multi-asi-immune/src/integration/homeostasis_bridge.rs | StandardMetrics (same 8 names); HomeostaticBridge maps homeostatic state to protocol mutual constraints for inter-ASI communication. |
| **Immune ↔ Governance** | agp-core/src/immunity/governance_bridge.py | GovernanceImmuneBridge: ThreatSignal, DefectionSignal → governance actions; immune suppression/restore; antibody_match, tcell_activation in signals. |
| **Endocrine ↔ Immune** | agp-core/src/immunity/integration.py | EndocrineImmuneIntegration, IntegratedBioSafetySystem — combined AHES + immune system. |
| **TELOS ↔ AGP demo** | agp-core/demo/unified_demo.py, agp-core/src/telos/membrane.py | Unified demo runs TELOS commitment membrane (entropy + authority + trust) as “Layer 1”; Python membrane wraps TELOS concepts for AGP. |

---

## 11. Quick reference — by crate/directory

| Crate / directory | Bio-inspired elements |
|-------------------|------------------------|
| **nexus-agp** | Hormones, glands, receptors, circadian, allostasis, negative feedback, hypothalamus/pituitary, Stimulus, GlandularSystem, HomeostasisController |
| **agp-core/ahes** | Same 8 hormones, half-life, Km, secretion, decay, circadian, receptor, EndocrineState (Python) |
| **agp-core/immunity** | Innate/adaptive immunity, Antibody, TCell, clonal selection, vaccination, negative selection, GovernanceImmuneBridge, EndocrineImmuneIntegration |
| **agp-core/telos** | Commitment membrane, entropy meter, crossing (Python wrapper) |
| **homeostasis-engine** | Metric, setpoint, bounds, SingleMetricController, MultiObjectiveController, AdaptiveSetpoint, EndocrineMetrics (endocrine_bridge) |
| **autonomic-system** | ACT/CALM/Emergency/Recovery, Arousal, reflex queue, ReflexType, BehaviorModifiers |
| **developmental-gates** | DevelopmentalStage (Infant→Elder), StageManager, GateEnforcer, CapabilityRegistry |
| **nervous-system** | PerceptionProcessor, DecisionEngine, MotorExecutor, NervousSystemCoordinator, SafetyState (perception–decision–motor pipeline) |
| **multi-asi-immune** | AsiNode, identity, reputation, threat, ThreatMemory, protocol, defection, HomeostaticBridge |
| **telos-protocol** | CommitmentMembrane, Decision, EntropyMeter, AuthorityRegistry, ValidatorNetwork, TrustAccumulator |
| **nexus-telecom** | LyapunovController (BIBO stability for AI actions) |
| **causalux** | Morgan Economy, Tesla Resonance, Da Vinci Atom (optional features) |
| **brain/formal_specs** | SafetyProtocol.tla (modes, arousal, reflex_queue, transitions) |

---

## 12. Summary count

- **Endocrine:** 2 implementations (Rust nexus-agp, Python agp-core/ahes) + homeostasis controller in nexus-agp.
- **Nervous-system–like:** 4 crates (homeostasis-engine, autonomic-system, developmental-gates, nervous-system) + TLA+ spec.
- **Immune-like:** 2 implementations (Rust multi-asi-immune, Python agp-core/immunity) + governance bridge + endocrine–immune integration.
- **Membrane:** 2 implementations (Rust telos-protocol, Python agp-core/telos).
- **Homeostasis/allostasis:** 3 places (homeostasis-engine, nexus-agp/homeostasis, bridges).
- **Reflexes:** 1 crate (autonomic-system/reflex).
- **Developmental stages:** 1 crate (developmental-gates).
- **Lyapunov:** 1 package (nexus-telecom control).
- **Bridges:** 5 (endocrine↔homeostasis, immune↔homeostasis, immune↔governance, endocrine↔immune, TELOS↔AGP demo).

---

**End of catalog.** All entries refer to code or docs present in the repository.
