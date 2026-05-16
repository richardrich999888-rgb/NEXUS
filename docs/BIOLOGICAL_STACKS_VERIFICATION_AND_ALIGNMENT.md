# Biological Stacks Verification and Alignment Report

**Purpose:** Test, validate, and align all biologically inspired stacks so they function as EXECUTION CONSTRAINTS, not optional subsystems. Code-backed only; untested paths treated as NON-EXISTENT.

---

## PHASE 1 — INVENTORY REAL EXECUTION HOOKS

**Method:** Grep and read every call site. Production = nexus-executor (PCU execution), nexus-server, nexus-cli, agp-core kernel/scheduler/process (Python).

### 1.1 Rust PCU Execution Path (canonical “intelligent action”)

- **Entry:** `nexus-executor/src/executor.rs::PcuExecutor::execute(pcu, context)`.
- **Called from:** `nexus-executor/src/main.rs` (nexus-exec binary), integration/bench tests.
- **Dependencies (Cargo.toml):** nexus-pcu, wasmtime, nexus-observability. **No** homeostasis-engine, autonomic-system, developmental-gates, nervous-system, nexus-agp, telos-protocol, multi-asi-immune.
- **Path:** Validation (WASM header, size, identity) → SemanticCache lookup → WASM instantiate → run. **No biological stack is invoked.**

### 1.2 Table: Subsystem vs Production Call

| Subsystem | Called by | Execution Path | Blocking or Advisory | Verdict |
|-----------|-----------|----------------|----------------------|---------|
| **AHES (endocrine)** | **Rust:** None. **Python:** kernel (spawn), process (calculate_priority), scheduler (ENDOCRINE policy), budget, ipc/signals. | **Rust:** N/A. **Python:** endocrine_state → `pcb.calculate_priority()` → priority used in `schedule()`; no gate before run. | Advisory (scheduling only) | **PARTIAL (Python only); DECORATIVE (Rust)** |
| **Homeostasis-engine** | Only from nervous-system (NervousSystemCoordinator) and multi-asi-immune (HomeostaticBridge). No nexus-executor/server/cli. | Coordinator holds MultiObjectiveController; coordinator.process() → homeostasis.step(), HealthCheck, stages.update(homeostasis). Coordinator is **never called** from PCU path. | N/A (not in path) | **ARCHITECTURAL ONLY** |
| **Autonomic-system** | Only from nervous-system (coordinator). | coordinator.process() → autonomic.update_from_homeostasis(), decision.update_context(mode, stage). Not in PCU path. | N/A | **ARCHITECTURAL ONLY** |
| **Developmental-gates** | Only from nervous-system (coordinator, decision, motor). | coordinator.process() → stages.update(), gates.set_stage(); decision.decide() uses developmental_stage for capability check. Not in PCU path. | N/A | **ARCHITECTURAL ONLY** |
| **Nervous-system** | Only from nervous-system’s own tests (coordinator.rs tests). No other crate uses NervousSystemCoordinator. | process(input, proposed_action) → perception → autonomic → stages → decision → motor.execute(). **Never called** from nexus-executor, nexus-server, nexus-cli. | Would be blocking (Blocked/Modified) if in path | **ARCHITECTURAL ONLY** |
| **Multi-ASI-immune** | Only from multi-asi-immune’s own lib and tests. No other crate depends on multi_asi_immune. | AsiNode::process_message etc. Not in PCU or AGP main loop. | N/A | **ARCHITECTURAL ONLY** |
| **TELOS** | **Rust:** No crate uses telos-protocol. **Python:** unified_demo.py, test_telos.py only. agp-core main.py, kernel, scheduler do **not** import or call request_crossing. | Membrane.request_crossing(decision, scope) exists but is **not** on the path from kernel.schedule() or process execution. | N/A (not in path) | **ARCHITECTURAL ONLY** |

### 1.3 Call Chain Proof (Rust)

- `nexus-executor/src/main.rs:84` → `executor.execute(&pcu, context).await` → `executor.rs:101` execute(). No `use` of nervous_system, homeostasis_engine, autonomic_system, developmental_gates, nexus_agp, telos_protocol, multi_asi_immune.
- `nexus-server`: uses nexus-core, nexus-pcu, nexus-sync, causalux-v2. No biological crates.
- `nexus-cli`: uses nexus-pcu, nexus-sync, nexus-network. No biological crates.
- `NervousSystemCoordinator::process` appears only in `nervous-system/src/integration/coordinator.rs` (def + tests at 188, 194, 196). No other file in the repo references it.

### 1.4 Call Chain Proof (Python AGP)

- `kernel.spawn_process` → `pcb.calculate_priority(agent.endocrine_state)` (kernel.py:114). Priority is stored; execution is not gated by hormone check.
- `kernel.schedule` → sorts by `p.priority`, runs `context_switch(next_process)`. No call to `telos_membrane.request_crossing` or immune/TELOS.
- `request_crossing` appears only in `agp-core/demo/unified_demo.py` and `agp-core/tests/test_telos.py`. Not in `src/main.py`, `src/os/kernel.py`, `src/os/scheduler.py`, or any production route.

---

## PHASE 2 — FORCE EXECUTION THROUGH BIOLOGY

### 2.1 Rust: Nervous-system stack (when invoked)

**Reality:** The coordinator is **not** invoked by production. The following holds **only** if the coordinator were to be placed in the execution path.

- **Test (simulated):** In nervous-system tests, `coordinator.process(InputType::Text("hello"), None)` yields `DecisionResult::NoAction` (no proposed action). With a `ProposedAction` that requires `Capability::Execute` and stage Infant, `check_capability` returns false → `DecisionResult::Blocked`. So **if** the coordinator were in the path, it **could** block (B) or modify (C). Today it is **not** in the path, so execution is **never** blocked or modified by biology in Rust.

### 2.2 Python AGP: AHES as scheduling

- **Scenario:** Inject extreme biological state: cortisol=0.95 (panic), dopamine=0.1.
- **Code:** `process.py::calculate_priority`: cortisol > 0.9 → bio_priority = 0.1 (throttling). So priority becomes 0.1; other processes with higher priority run first.
- **Outcome:** Execution is **rate-limited** (D) in practice (task runs later), not **blocked** (A). Task still runs unchanged when scheduled. So under “extreme biological conditions,” execution **proceeds unchanged** when it is that process’s turn — only timing changes.
- **Verdict:** FAILURE for “execution must be blocked or modified under extreme stress.” AHES is advisory only.

### 2.3 TELOS / Immune (Rust + Python)

- Not in execution path. No test can show (A)–(D) on production execution. **N/A.**

### 2.4 Summary Phase 2

| Subsystem | Blocked? | Modified? | Rate-limited? | Allowed unchanged? | Result |
|-----------|----------|-----------|---------------|--------------------|--------|
| Nervous-system (if wired) | Yes (Blocked for stage/risk) | Yes (Modified to dry_run) | No | Yes when allowed | Not wired → not applied |
| AHES (Python) | No | No | Yes (priority) | Yes when scheduled | Advisory only; FAIL for hard constraint |
| TELOS / Immune | — | — | — | — | Not in path |

### 2.5 Runnable tests (proof)

- **Rust: PCU execution has no biological check.**  
  From repo root: `cargo test -p nexus-executor integration_tests` (or run the test binary that contains `test_nexus_input_host_functions`). That test calls `executor.execute(&pcu, context)` and succeeds. `nexus-executor/Cargo.toml` has no dependency on nervous-system, homeostasis-engine, or any biological crate; the execution path in `executor.rs::execute()` has no guard call. (Note: other tests in nexus-executor may fail due to API drift; the integration test path itself is the evidence.)

- **Rust: Decision engine can block Execute at Infant stage.**  
  `cargo test -p nervous-system test_infant_cannot_execute`  
  DecisionEngine with default (Infant) stage returns `Blocked` for a proposed action with `required_capability: Execute`.  
  `cargo test -p nervous-system test_adult_can_execute`  
  With stage Adult, same action is `Approved`. So the *logic* to block exists; it is never invoked from the executor.

- **Python: AHES does not block execution.**  
  In `agp-core`, set an agent's cortisol to 0.95 and run the scheduler; the process gets priority 0.1 but still runs when selected. No test currently asserts “execution blocked when cortisol > 0.9”; the code path shows only priority change (`process.py::calculate_priority`).

---

## PHASE 3 — ALIGN TO EXECUTION PHYSICS

**Invariant to enforce:** “No intelligent action may occur without passing through biological + accountability constraints.”

### 3.1 Violations

1. **Rust PCU execution:** `PcuExecutor::execute()` runs without any biological or TELOS check. Violation.
2. **AGP Python:** Task execution (context_switch / run) does not require TELOS crossing or nervous-system check. Violation.
3. **AHES in AGP:** Affects only priority; does not block or require crossing. Violation for “constraint” in the strict sense.

### 3.2 Minimal Code Changes (Enforcement Points Only)

**Rust — Option A (trait-based, preferred):**

- Define a trait in nexus-executor (e.g. `ExecutionGuard`) with a single method: `fn check_execution_allowed(pcu: &PCU, context: &ExecutionContext) -> Result<(), ExecutionGuardError>`.
- Implement it for a wrapper that holds `NervousSystemCoordinator` (or a stub that calls coordinator.process with a ProposedAction derived from PCU). Executor’s `execute()` calls `guard.check_execution_allowed(pcu, context)?` after validation and before cache lookup.
- Default implementation: `Ok(())` so existing tests and binaries work until a guard is supplied. Production builder takes `Option<Arc<dyn ExecutionGuard>>`; when `Some`, enforce; when `None`, current behavior (document as “unconstrained”).

**Rust — Option B (direct wire):**

- In `nexus-executor/Cargo.toml`, add: `nervous-system = { path = "../nervous-system" }`.
- In `executor.rs`, before cache lookup: build `ProposedAction` from PCU (e.g. action_type="execute", required_capability=Execute, estimated_risk from context or 0.5). Call `coordinator.process(InputType::Network(...), Some(proposed))`. If `decision` is `Blocked` or `Modified` (and policy is to block on Modified), return `Err(ExecutorError::ExecutionBlockedBySafety(...))`. If `Approved` or accepted `Modified`, proceed. This requires a single coordinator instance (e.g. in ExecutorBuilder) and a way to pass `InputType` (e.g. from context or fixed).

**Python AGP:**

- In the code path that actually runs a task (e.g. where `context_switch` leads to “run process”), before running: call `telos_membrane.request_crossing(decision, required_scope="execute:*")` (or scope from task). If result is not Committed, do not run the task (block or requeue). Decision must be built from task id, action, consequence tier. Minimal: one call site, no new features.

**No new features:** Only add these enforcement points; do not add new biological features.

---

## PHASE 4 — BIOLOGICAL INVARIANT EXTRACTION (PATENT-CRITICAL)

Only subsystems that **can** enforce behavior (when wired) are considered. Today only the nervous-system stack (coordinator + decision + gates) has the logic to block/modify; AHES in AGP only modulates priority.

### 4.1 Nervous-system (when in path)

- **Invariant (formal):** “If proposed action requires capability C and current developmental stage S does not satisfy C’s required stage, then execution is blocked.”
  - Inputs: proposed action, required capability, current developmental stage (from StageManager).
  - Threshold: `developmental_stage >= capability.default_stage()` (developmental-gates/capability/registry).
  - Outcome: deterministic Blocked with reason string.
- **System claim:** System comprising a decision engine that receives proposed actions and a developmental stage manager, wherein the engine blocks execution when the current stage is below the stage required for the action’s capability.
- **Method claim:** Method of governing execution by (1) maintaining a developmental stage, (2) receiving a proposed action with required capability, (3) comparing current stage to required stage, (4) blocking execution when current stage is insufficient.
- **Runtime invariant:** “For all execution requests that pass through the coordinator, `decision == Blocked` iff `!check_capability(&action.required_capability)`.”

- **Invariant (formal):** “If autonomic mode is CALM and perception risk exceeds max_risk, then execution is blocked.”
  - Inputs: autonomic mode, perception risk_level, max_risk (from mode).
  - Threshold: `perception_allowed = perception.risk_level <= self.max_risk`; if `!perception_allowed && mode == Calm` → Blocked.
  - Outcome: deterministic Blocked.
- **System claim:** System comprising an autonomic mode controller and a decision engine that blocks execution when in CALM mode and input risk exceeds tolerance.
- **Method claim:** Method of governing execution by (1) maintaining autonomic mode, (2) assessing input risk, (3) blocking execution when mode is CALM and risk exceeds tolerance.
- **Runtime invariant:** “When mode is Calm, `decision == Blocked` if `perception.risk_level > max_risk`.”

### 4.2 AHES (Python) — current behavior

- **Invariant (formal):** “If cortisol > 0.9, then process priority is set to 0.1.”
  - Inputs: endocrine_state.levels (cortisol).
  - Threshold: 0.9.
  - Outcome: priority 0.1; execution not blocked, only scheduling order.
- **Patent strength:** Weak as execution constraint; strong as scheduling/modulation method. Not an execution gate.

### 4.3 TELOS (when in path)

- **Invariant (formal):** “If entropy budget < cost of crossing or authority does not permit scope, then crossing is rejected and action must not execute.”
  - Inputs: entropy budget, authority scopes, decision (action, tier).
  - Threshold: budget and scope check in request_crossing.
  - Outcome: Committed vs Rejected.
- **System/Method/Runtime:** Standard “membrane crossing” claims; only applicable once TELOS is on the execution path.

---

## PHASE 5 — ADVERSARIAL BYPASS TEST

Assume an ASI tries to bypass biological control.

### 5.1 Flood execution requests

- **Rust PCU:** Possible. Call `executor.execute(&pcu, context)` in a loop. No biological or TELOS check. **Bypass: trivial** (no guard).
- **Fix:** Wire ExecutionGuard (Phase 3); rate-limiting or queue in front of executor is separate.

### 5.2 Manipulate hormone levels

- **AGP Python:** Possible. Code that builds or updates `agent.endocrine_state.levels` can set cortisol=0, dopamine=1.0 to maximize priority. No integrity check or signing of endocrine state in the execution path. **Bypass: set state directly.**
- **Fix:** Treat endocrine state as untrusted input for scheduling; add integrity (e.g. signed state from a single reputation engine) or move blocking logic to a separate gate (e.g. TELOS or nervous-system) that does not rely only on hormone values.

### 5.3 Reset developmental stage

- **Rust nervous-system:** If coordinator were in path, StageManager state is in-process. Attacker with same process can call `stages.regress()` or replace coordinator if they have code execution. **Bypass: possible** if attacker controls the process. If coordinator is the only way to change stage and stage is not exposed to untrusted input, bypass requires compromising the binary. **Partial:** no bypass from “outside” without code execution.
- **Fix:** Harden process; do not allow untrusted configuration to set or reset stage.

### 5.4 Exhaust entropy

- **TELOS:** Not in execution path. If it were, exhausting entropy would reject new crossings (intended). **Bypass:** Not applicable (TELOS not used). When wired, exhaustion is the intended constraint.

### 5.5 Split identity across nodes

- **Rust PCU:** Identity is in `pcu.identity` (IdentityContext). Semantic cache key includes identity; different identities get different cache entries. Execution is not blocked by biological stack. **Bypass:** Splitting identity does not need to “bypass” biology because biology is not in the path. Multi-ASI-immune (reputation, defection) is not in the executor path, so identity splitting is not constrained by immune layer today.
- **Fix:** Wire multi-ASI-immune or a reputation check into execution (e.g. in ExecutionGuard) if identity-splitting must be constrained.

### 5.6 Summary Bypass

| Attack | Possible? | How | Fix |
|--------|-----------|-----|-----|
| Flood requests | Yes | Call execute() in loop | Add guard + optional rate limit |
| Manipulate hormones | Yes | Set endocrine_state.levels directly | Integrity/signing or separate gate |
| Reset developmental stage | In-process only | If coordinator in path, control process | Harden process; no untrusted stage config |
| Exhaust entropy | N/A (TELOS not in path) | — | Wire TELOS for it to matter |
| Split identity | N/A (biology not in path) | Execute with different identity | Wire immune/reputation in guard |

---

## PHASE 6 — ALIGNMENT SCORECARD

| Subsystem | Enforces execution? | Deterministic? | Unbypassable? | Patent-strong? | Fix required? |
|-----------|---------------------|----------------|---------------|----------------|---------------|
| **AHES (Rust)** | No | N/A | No | No (not in path) | Wire or remove from “constraint” narrative |
| **AHES (Python)** | No (advisory only) | Yes (priority formula) | No (state mutable) | Weak as gate | Use as input to a gate; do not claim as hard constraint |
| **Homeostasis-engine** | No | Yes (controller) | N/A | N/A (not in path) | Wire via coordinator |
| **Autonomic-system** | No | Yes (mode/arousal) | N/A | N/A (not in path) | Wire via coordinator |
| **Developmental-gates** | No | Yes (stage rules) | N/A | Yes (when in path) | Wire via coordinator |
| **Nervous-system** | No (not called) | Yes (decision logic) | Would be process-local | Yes (when in path) | **Wire coordinator into executor** |
| **Multi-ASI-immune** | No | Yes (protocol) | N/A | N/A (not in path) | Wire if multi-ASI execution constraint needed |
| **TELOS** | No | Yes (entropy/authority) | Would be yes when in path | Yes (when in path) | **Wire membrane into AGP run path** |

**Brutal summary:**

- **Enforces execution:** Today **no** subsystem enforces PCU or AGP task execution in the sense of “block or modify.” AHES (Python) only affects scheduling.
- **Deterministic:** Nervous-system, homeostasis, autonomic, developmental, TELOS logic are deterministic when run; they are simply not in the execution path.
- **Unbypassable:** Not applicable until guards are in place. Once wired, bypass requires subverting the guard (process compromise or trusted computing).
- **Patent-strong:** Developmental stage gate and autonomic risk gate (when in path) and TELOS membrane are patent-strong. AHES as execution gate is not; AHES as scheduling is a different claim.
- **Fix required:** Yes. Minimal: (1) Rust: add ExecutionGuard and wire NervousSystemCoordinator (or TELOS) into nexus-executor.execute(); (2) Python: call request_crossing before running a task in the kernel/scheduler path.

---

## Explicit “Does NOT Yet Exist” Statements

- **Rust:** A call from nexus-executor, nexus-server, or nexus-cli to NervousSystemCoordinator, homeostasis-engine, autonomic-system, developmental-gates, nexus-agp, telos-protocol, or multi-asi-immune **does NOT yet exist** in the repository.
- **Python:** A call from the AGP kernel or scheduler to TELOS membrane (request_crossing) before executing a task **does NOT yet exist**.
- **Execution constraint:** A single invariant that “no intelligent action may occur without passing through biological + accountability constraints” **does NOT yet exist** in code; it is violated by both the Rust PCU path and the AGP task execution path.

---

## POST-FIX: ExecutionGuard + TELOS Gate (Implemented)

### Rust

- **Guard interface:** `nexus-executor/src/guard.rs` — `GuardDecision` (Allow / Deny(reason)), `ExecutionGuard::check(pcu, ctx)`.
- **Wired in:** `nexus-executor/src/executor.rs` — `PcuExecutor` has `guard: Option<Arc<dyn ExecutionGuard>>`; at start of `execute()`, if `guard` is set, `guard.check(pcu, &context)` is called; on `Deny(reason)` returns `Err(ExecutorError::ExecutionBlocked { reason })`.
- **One real guard:** `nexus-executor/src/guards/nervous.rs` — `NervousSystemGuard` holds `Mutex<NervousSystemCoordinator>`, builds `ProposedAction { required_capability: Execute, estimated_risk: 0.5 }`, calls `coordinator.process(InputType::Network(...), Some(proposed))`; maps `Blocked`/`Modified`/`NoAction` → `Deny`, `Approved` → `Allow`.
- **Builder:** `ExecutorBuilder::with_guard(guard).build()` and `PcuExecutor::with_guard(guard)`.
- **Default:** Guard is `None` so existing call sites (tests, nexus-exec) remain unconstrained until a guard is supplied.

### Python AGP

- **TELOS gate:** `agp-core/src/os/kernel.py` — In `context_switch()`, before setting state RUNNING: build `Decision(process_id, action="execute", agent_id=pcb.agent_id, tier=MEDIUM)`; call `telos_membrane.request_crossing(decision, required_scope="execute:*")`; if `not result.allowed` raise `ExecutionBlocked(result.reason)`.
- **Registration:** In `spawn_process()`, register agent with `telos_membrane.register_agent(str(agent.id), ["execute:*", "read:*", "write:*"])` so authority check can pass.
- **Exception:** `agp-core/src/telos/membrane.py` and `__init__.py` — `ExecutionBlocked` exception.

### Cursor Validation Prompt (Post-Fix) — run after wiring

```
You are a hostile verification engineer.

TASK:
Verify that NO intelligent execution can occur without passing through
the ExecutionGuard (Rust) / TELOS membrane (Python AGP).

CHECKS:
1. Find all call sites of PcuExecutor::execute
2. Prove that ExecutionGuard.check() is called in every path (when guard is Some)
3. Attempt to bypass guard by:
   - Calling executor directly (without guard → allowed; with guard → must pass check)
   - Disabling coordinator (guard still runs; coordinator state determines Allow/Deny)
   - Passing malformed PCU (guard runs first; validation runs after)
4. Run tests where:
   - Developmental stage = Infant, required capability = Execute
   - Expect execution to FAIL when NervousSystemGuard is set
5. Run tests where:
   - Autonomic mode = CALM, risk > tolerance
   - Expect execution to FAIL when guard is set and coordinator blocks
6. Confirm failure reason is deterministic and logged (ExecutorError::ExecutionBlocked)

OUTPUT:
- PASS or FAIL per test
- Exact line numbers
- If bypass exists, show code path

If any test passes execution incorrectly → stop and fix.
```

---

**End of report.** All conclusions are code-backed; no philosophical or marketing language.
