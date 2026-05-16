# Endocrine and Neuromorphic-Like Artificial Systems — Project Report

**Purpose:** Report on what has been **implemented and presented** in the NEXUS project for (1) human endocrine-like artificial systems and (2) neuromorphic-like artificial systems. All references are to actual code and modules in the repo.

---

## Part 1 — Artificial Human Endocrine System (AHES)

### 1.1 Where it lives

| Location | Language | Role |
|----------|----------|------|
| **nexus-agp/** | Rust | Core AHES: hormones, glands, homeostasis controller |
| **agp-core/src/ahes/** | Python | AHES bridge: hormone kinetics, reputation mapping |

Patent claims cited in code: **Claims 8–12** (Bio-inspired Computational Governance, Hormone Secretion, Biological Feedback Loops, Circadian Rhythm, Allostatic Adaptation).

---

### 1.2 What is implemented (Rust — nexus-agp)

#### A. Hormone layer (`nexus-agp/src/endocrine.rs`)

**8 hormones**, each mapped to a **reputation dimension** with biological-style parameters:

| Hormone | Reputation dimension | Half-life (seconds) | Km (affinity) | Max secretion |
|---------|----------------------|---------------------|---------------|---------------|
| Cortisol | Accuracy | 90×60 | 0.3 | 0.4 |
| Oxytocin | Cooperation | 3×60 | 0.1 | 0.6 |
| Serotonin | Stability | 24×3600 | 0.5 | 0.1 |
| Dopamine | Uniqueness | 5×60 | 0.2 | 0.5 |
| Adrenaline | Latency | 2×60 | 0.05 | 0.8 |
| Endorphins | Ethics | 20×60 | 0.4 | 0.3 |
| Norepinephrine | Novelty | 1.5×60 | 0.15 | 0.5 |
| GrowthHormone | Longevity | 15×60 | 0.6 | 0.05 |

**Implemented mechanisms:**

- **First-order decay:** `HormoneLevel::decay(delta_time, half_life)` — level decays toward baseline 0.5 with factor `0.5^(Δt / half_life)`.
- **Receptor saturation (Michaelis–Menten):** `HormoneReceptor::response(level)` = `(Vmax × [H] × density × downreg) / (Km + [H])`. Km and max secretion per hormone are fixed in code.
- **Negative feedback:** `EndocrineState::apply_negative_feedback(hormone)` — high level reduces further secretion (quadratic inhibition above baseline 0.5).
- **Circadian modulation:** `HormoneLevel::circadian_factor(time_of_day)` — 24 h cycle, 20% amplitude; `effective_level(time_of_day)` returns level × circadian factor.
- **Receptor down/up-regulation:** `HormoneReceptor::downregulate(exposure_duration, hormone_level)` and `upregulate(recovery_time)` — prolonged high exposure reduces sensitivity; recovery over time.

**EndocrineState:** Per-agent state: `levels` (HashMap Hormone → HormoneLevel), `receptors` (HashMap Hormone → HormoneReceptor), `system_time`, `agent_id`. Methods: `tick(delta_time)` (decay all), `secrete(hormone, stimulus_strength)`, `privilege(hormone)` (receptor-mediated response), `alignment()` (deviation from baseline), `dominant_hormone()`, `to_reputation_vector()` / `from_reputation_vector()` (8-D ↔ hormone levels).

---

#### B. Virtual gland system (`nexus-agp/src/glands.rs`)

**Stimulus types** (events that drive secretion):

- TaskSuccess, TaskFailure → performance/stress
- Collaboration → social
- NovelSolution → reward/novelty
- Urgency → speed/deadline
- EthicalCompliance → ethics
- Exploration → risk/alertness
- Consistency → long-term stability

**Gland trait:** `process(stimulus, state) -> Vec<(Hormone, f64)>`, `produces() -> Vec<Hormone>`.

**Implemented glands:**

1. **HypothalamicController** — Master regulator: evaluates stress from cortisol level; if cortisol > stress_threshold, reduces `activity` (like HPA dampening); `releasing_signals(stimulus)` returns primary hormone + strength and cross-talk (e.g. TaskSuccess + fast latency → also Adrenaline; Collaboration + many partners → also Serotonin).
2. **PituitaryRouter** — Amplification per hormone; `route(signals)` applies amplification; `adjust_amplification(hormone, factor)` for feedback.
3. **PerformanceGland** — Cortisol, Adrenaline; fatigue state; TaskSuccess/TaskFailure/Urgency → secretions with negative feedback at high adrenaline.
4. **CooperationGland** — Oxytocin; trust_bank; Collaboration → oxytocin scaled by success_rate, partner_count, trust_bank.
5. **StabilityGland** — Serotonin, Endorphins; Consistency → serotonin; EthicalCompliance → endorphins.
6. **ExplorationGland** — Dopamine, Norepinephrine; novelty_tolerance and habituation; NovelSolution → dopamine + norepinephrine; Exploration → norepinephrine.
7. **DevelopmentGland** — GrowthHormone; experience counter; Consistency → pulsatile growth (weekly pulse + long-term base).

**GlandularSystem:** Holds hypothalamus, pituitary, and the five glands. `process(stimulus, state)` pipeline: (1) hypothalamus evaluates stress and releasing signals, (2) pituitary routes/amplifies, (3) all glands produce secretions, (4) for each (hormone, strength) apply negative feedback then `state.secrete(hormone, adjusted)`.

---

#### C. Homeostasis controller (`nexus-agp/src/homeostasis.rs`)

**SetPoint:** target, tolerance, adaptation_rate. `in_tolerance(level)`, `error(level)`, `adapt(current_level)` — allostasis: target slowly moves toward chronically high/low level (clamped 0.2–0.8).

**FeedbackLoop:** Negative or Positive; sensitivity, delay, max_effect. `calculate(current_level, set_point)` — negative: high level → inhibition factor &lt; 1; low level → stimulation &gt; 1.

**CircadianController:** Per-hormone phase offsets (e.g. Cortisol peak morning, Serotonin night, GrowthHormone sleep); 24 h cycle; `modulation(hormone)`, `apply(state)` — gently pulls levels toward circadian-adjusted baseline.

**AllostasisManager:** Per-hormone SetPoints and history (e.g. 100 samples). `record(state)`, `average(hormone)`, `adapt_setpoints()` (move set-points toward chronic average), `calculate_load(state)` (allostatic load from deviation), `in_balance(state)`.

**HomeostasisController:** Per-hormone FeedbackLoops + CircadianController + AllostasisManager. `tick(delta_time, state)`: advance circadian, apply circadian modulation, record history, compute allostatic load. `feedback_factor(hormone, state)`, `health_status(state)` → Optimal | Normal | Stressed | Critical from load and balance.

---

### 1.3 What is implemented (Python — agp-core/ahes)

**agp-core/src/ahes/endocrine.py:** Python counterpart of the hormone model:

- Same 8 hormones with same half-lives, Km, max secretion (documented in docstring and Enum).
- HormoneLevel with decay, secretion, circadian factor.
- HormoneReceptor with Michaelis–Menten response, down/up-regulation.
- EndocrineState with levels, receptors, tick, secrete, privilege, alignment, to/from reputation vector.
- Documented as “AHES Bridge” and “PATENT CLAIMS 8–12.”

So the **same biological kinetics** (half-life decay, receptor saturation, negative feedback, circadian) are presented in both Rust (nexus-agp) and Python (agp-core); Rust adds the full gland hierarchy and homeostasis controller; Python provides the bridge for the rest of agp-core.

---

### 1.4 Summary — AHES as presented

- **8 hormones** with biologically inspired half-lives and receptor parameters, mapped to **8 reputation dimensions**.
- **First-order decay**, **Michaelis–Menten receptors**, **negative feedback**, **circadian modulation**, **receptor down/up-regulation** implemented in code.
- **Virtual glands:** HypothalamicController, PituitaryRouter, Performance, Cooperation, Stability, Exploration, Development — with stimuli (task success/failure, collaboration, novelty, urgency, ethics, exploration, consistency) driving secretions and cross-talk.
- **Homeostasis:** SetPoints, negative/positive feedback loops, circadian controller, allostasis (adaptive set-points and allostatic load), and a single health status (Optimal/Normal/Stressed/Critical).

---

## Part 2 — Neuromorphic-Like Artificial Systems (Safety Stack)

### 2.1 Where it lives

| Crate | Role |
|-------|------|
| **homeostasis-engine** | Low-level: metrics, bounds, setpoints, single/multi-objective controllers |
| **autonomic-system** | ACT/CALM/Emergency/Recovery modes; arousal; reflexes |
| **developmental-gates** | Stages (Infant→Elder); capability registry; gate enforcer |
| **nervous-system** | Coordinator: perception → decision → motor; integrates all three |
| **multi-asi-immune** | Identity, reputation, threat (separate; not covered in this report) |

The **formal spec** that matches the autonomic behavior is in **brain/formal_specs/SafetyProtocol.tla** (modes, arousal, duration, reflex_queue, transitions).

---

### 2.2 What is implemented

#### A. Homeostasis engine (`homeostasis-engine`)

- **Metric:** value, setpoint, bounds (HardBounds), gain, weight. Value clamped to bounds; correction signal proportional to (setpoint − value); `update(correction)`.
- **SingleMetricController:** proportional control with max correction and optional damping; `step(metric)` applies correction.
- **MultiObjectiveController:** multiple metrics; convergence toward setpoints under bounds; CorrectionResult, BoundsViolation, MultiObjectiveResult, ConvergenceResult, SystemHealth.
- **Diagnostics:** HealthCheck, HealthStatus.

Presented as the **constraint-enforcement substrate**: “setpoint + bounds + negative feedback,” used by the autonomic and developmental layers as the “stress/stability” signal.

---

#### B. Autonomic system (`autonomic-system`)

**Modes (biological analogy in comments):**

- **ACT** — “Sympathetic”: high arousal (0.7), higher risk tolerance (0.6), speed factor 2.0, reflexes enabled.
- **CALM** — “Parasympathetic”: low arousal (0.3), low risk tolerance (0.2), speed 0.5, no reflexes.
- **Emergency** — arousal 1.0, very low risk tolerance (0.1), speed 3.0, reflexes enabled.
- **Recovery** — arousal 0.4, transitioning from Emergency toward CALM.

**Arousal:** level, velocity, target; first-order dynamics toward target (time constant TAU=10); `stimulate(amount)`, `update(dt)`; `is_high`, `is_low`, `is_critical`.

**AutonomicController:** Holds mode, arousal, mode_duration, config (act_threshold, calm_threshold, emergency_threshold, min_mode_duration), reflex_queue, transitions.  
- `tick(dt)` or `update_from_homeostasis(homeostasis, dt)`: stress metric from homeostasis drives arousal target; arousal updates; then **mode transition** if thresholds crossed and min_mode_duration met (with hysteresis: same band keeps current mode).  
- **Reflexes:** if mode is ACT or Emergency and arousal is critical, queues EmergencyBrake reflex.  
- **BehaviorModifiers:** risk_tolerance, speed_factor, reflexes_enabled, arousal_level — exposed for downstream (decision layer).

So the project **presents** an autonomic layer that behaves like a simplified sympathetic/parasympathetic switch (ACT/CALM) plus emergency and recovery, driven by a homeostatic “stress” metric and arousal dynamics, with optional reflex queue.

---

#### C. Developmental gates (`developmental-gates`)

**Stages (biological analogy in comments):**

- **Infant (0)** — Observation only.
- **Child (1)** — Basic actions, supervised.
- **Adolescent (2)** — Extended actions, limited autonomy.
- **Adult (3)** — Full, self-regulated.
- **Elder (4)** — Full + mentoring.

**StageManager:** current_stage, time_at_stage, stability_history, violations, successes. Periodically (assessment_interval) checks StageRequirements (e.g. stability score, success count); can **advance** or **regress** (if allow_regression and violations exceed threshold). Transition history and reasons (Advancement, Regression, ManualOverride) recorded.

**CapabilityRegistry:** capabilities mapped to minimum stage required.

**GateEnforcer:** For a capability, checks current_stage vs required stage; returns Allowed or Denied(required, current) or UnknownCapability or Suspended. Access attempts logged.

So the project **presents** staged capability unlock: capabilities gated by developmental stage, with advancement/regression based on stability and violations — “developmental gates” in the sense of staged unlock and possible regression.

---

#### D. Nervous system coordinator (`nervous-system`)

**Pipeline (single entry point):** `NervousSystemCoordinator::process(input, proposed_action)`:

1. **Perception** — `PerceptionProcessor::process(input)`: input type (Text, Command, Sensor, Network, Internal) → Perception with risk_level, required_capabilities, intent.
2. **Autonomic** — `autonomic.update_from_homeostasis(homeostasis, 1.0)`; mode and arousal updated from homeostasis.
3. **Developmental** — `stages.update(homeostasis)`; current stage may advance/regress; `gates.set_stage(current_stage)`.
4. **Decision** — `decision.update_context(autonomic.mode(), stages.current_stage())`; `decision.decide(perception, proposed_action)`: checks capability vs stage and risk vs autonomic risk_tolerance; returns Approved | Modified | Blocked | NoAction.
5. **Motor** — `motor.execute(decision)`: if Approved/Modified, executes action and records ExecutionRecord; if Blocked, no execution.
6. **Recording** — success/violation fed back to stages (record_success etc.).

**SafetyState:** autonomic_mode, developmental_stage, arousal, health_score, active_threats, active_constraints, is_healthy.

So the project **presents** a **neuromorphic-like** flow: perception → autonomic + developmental context → decision (with risk and capability checks) → motor execution, with homeostasis driving autonomic state and developmental stage driving capability gates. The “nervous system” is the coordinator that wires perception, decision, and motor to the homeostasis, autonomic, and developmental layers.

---

### 2.3 Formal spec (TLA+)

**brain/formal_specs/SafetyProtocol.tla:** Specifies modes (CALM, ACT, EMERGENCY, RECOVERY), arousal, duration, reflex_queue. Actions: IncreaseArousal, DecreaseArousal, Tick (mode transition when duration ≥ MIN_DURATION), TriggerReflex. Invariants: TypeOK; EmergencySafety (high arousal + duration ⇒ mode = EMERGENCY); NoGhostStates; liveness ReturnsToCalm. This aligns with the autonomic controller’s transition logic (thresholds, min_mode_duration) and reflex queue.

---

## Part 3 — Integration and Presentation Summary

### 3.1 How the two “bio” stacks relate in the repo

- **AHES (nexus-agp + agp-core):** Endocrine-like governance for **agents**: hormones ↔ reputation dimensions, glands driven by task/collaboration/novelty/ethics/etc., homeostasis (setpoints, feedback, circadian, allostasis). Used in AGP (Agent Governance Protocol) context; not wired into the PCU execution path in the audited code.
- **Neuromorphic safety stack (homeostasis-engine → autonomic → developmental-gates → nervous-system):** Generic **safety** substrate: metrics/setpoints → ACT/CALM/emergency modes and arousal → developmental stages and capability gates → perception–decision–motor pipeline. TLA+ spec matches the autonomic part. Not wired into nexus-executor or nexus-server in the audited code; it is a separate subsystem that can be driven by `process(input, proposed_action)`.

So in the project as presented:

- **Endocrine-like** = AHES (hormones, glands, homeostasis controller) for agent-level governance and reputation.
- **Neuromorphic-like** = safety stack (homeostasis engine + autonomic modes + developmental stages + perception–decision–motor) for mode-dependent risk and capability gating.

### 3.2 What is explicitly presented in code/comments

- **Patent claims 8–12** in nexus-agp (endocrine, glands, homeostasis) and agp-core ahes.
- **Biological analogies** in comments: cortisol/stress, oxytocin/social, dopamine/reward, sympathetic/parasympathetic, hypothalamic/pituitary, developmental stages (Infant→Elder), perception–decision–motor.
- **Concrete mechanisms:** half-life decay, Michaelis–Menten, negative feedback, circadian phase offsets, allostasis, arousal dynamics, mode thresholds, reflex queue, stage advancement/regression, capability gates.

### 3.3 Gaps (as presented)

- No call from **nexus-executor** or **nexus-server** to AHES or to the nervous-system coordinator in the audited code; so “endocrine” and “neuromorphic” are not yet on the PCU execution path.
- **multi-asi-immune** (identity, reputation, threat) is a separate crate; nervous-system does not reference it in the coordinator code read; integration (e.g. threat → arousal) would be an extension.

---

**End of report.** All descriptions above are tied to existing modules and functions in the repository.
