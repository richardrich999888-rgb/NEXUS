# Patent Claims from Enforced Biological Invariants

**Purpose:** Engineer-readable patent claim skeletons derived from code-enforced invariants. Each claim is tied to a specific invariant and code path. Not legal advice.

---

## 1. Execution Gate (Single Choke Point)

### Invariant (formal)
“No intelligent action may occur without passing through a configurable execution guard when one is set.”

- **Inputs:** PCU, ExecutionContext, optional Guard.
- **Threshold:** If `guard.is_some()`, then `guard.check(pcu, ctx)` must return Allow before execution proceeds.
- **Outcome:** Deterministic Allow or Deny(reason); on Deny, execution returns `ExecutorError::ExecutionBlocked`.

### System claim (skeleton)
A system for governing execution of portable computation units (PCUs), comprising: (a) an execution engine that executes a PCU only after a guard check; (b) a guard interface that accepts the PCU and execution context and returns Allow or Deny; (c) wherein when a guard is configured, the engine does not execute the PCU if the guard returns Deny.

### Method claim (skeleton)
A method of governing execution, comprising: (1) receiving a PCU and an execution context; (2) when an execution guard is configured, invoking the guard with the PCU and context; (3) if the guard returns Deny, returning an execution-blocked error and not executing the PCU; (4) if the guard returns Allow or no guard is configured, proceeding with execution.

### Runtime invariant
For all call sites of `PcuExecutor::execute`, when `self.guard.is_some()`, execution of the PCU occurs only after `guard.check(pcu, &context)` returns `GuardDecision::Allow`.

---

## 2. Developmental Stage Gate

### Invariant (formal)
“If the proposed action requires capability C and the current developmental stage S does not satisfy C’s required stage, then execution is blocked.”

- **Inputs:** Proposed action, required capability (e.g. Execute), current developmental stage (from StageManager).
- **Threshold:** `developmental_stage >= capability.default_stage()` (e.g. Execute requires Adult).
- **Outcome:** Deterministic Blocked with reason string.

### System claim (skeleton)
A system comprising: (a) a developmental stage manager that maintains a current developmental stage (e.g. Infant through Elder); (b) a decision engine that receives a proposed action with a required capability; (c) wherein the decision engine blocks execution when the current stage is below the stage required for that capability.

### Method claim (skeleton)
A method of governing execution by developmental stage, comprising: (1) maintaining a current developmental stage; (2) receiving a proposed action with a required capability; (3) comparing the current stage to the required stage for that capability; (4) blocking execution when the current stage is insufficient.

### Runtime invariant
For all execution requests that pass through the nervous-system coordinator, `decision == Blocked` iff `!check_capability(&action.required_capability)`.

---

## 3. Autonomic Risk Gate

### Invariant (formal)
“If autonomic mode is CALM and perception risk exceeds max_risk, then execution is blocked.”

- **Inputs:** Autonomic mode, perception risk_level, max_risk (from mode).
- **Threshold:** When mode is Calm, `perception.risk_level <= max_risk`; otherwise Blocked.
- **Outcome:** Deterministic Blocked.

### System claim (skeleton)
A system comprising: (a) an autonomic mode controller that maintains a mode (e.g. ACT, CALM, Emergency, Recovery); (b) a decision engine that assesses input risk and blocks execution when in CALM mode and input risk exceeds a tolerance.

### Method claim (skeleton)
A method of governing execution by autonomic state, comprising: (1) maintaining an autonomic mode; (2) assessing input or proposed-action risk; (3) blocking execution when the mode is CALM and the risk exceeds the mode’s tolerance.

### Runtime invariant
When mode is Calm, `decision == Blocked` if `perception.risk_level > max_risk`.

---

## 4. Biological / AHES Risk Modulation (Input to Gate)

### Invariant (formal)
“Estimated risk for the guard may be supplied by the execution context (e.g. from AHES or stress signals).”

- **Inputs:** ExecutionContext.biological_risk (optional).
- **Threshold:** NervousSystemGuard uses `ctx.biological_risk.unwrap_or(0.5)` as estimated_risk for the proposed action.
- **Outcome:** Higher context risk increases likelihood of Blocked when autonomic tolerance is exceeded.

### System claim (skeleton)
A system comprising: (a) an execution context that may carry a biological or stress-derived risk value; (b) an execution guard that uses that value as the estimated risk for a proposed action; (c) wherein the guard’s decision (Allow/Block) depends on that risk relative to autonomic or other tolerances.

### Method claim (skeleton)
A method of modulating execution governance by biological state, comprising: (1) providing an execution context that optionally includes a biological or stress-derived risk value; (2) passing the context to an execution guard; (3) the guard using that value as the estimated risk for the proposed action when deciding Allow or Deny.

### Runtime invariant
When `ExecutionContext.biological_risk` is set, NervousSystemGuard uses it as `ProposedAction.estimated_risk` in the coordinator’s decision.

---

## 5. Immune and Reputation Gate

### Invariant (formal)
“If the principal is isolated due to defection or aggregated reputation is below threshold, then execution is blocked.”

- **Inputs:** Principal id (32 bytes), defection tracker, reputation aggregator, min_reputation threshold.
- **Threshold:** `!defections.should_isolate(asi_id)` and `reputation >= min_reputation`.
- **Outcome:** Deterministic Allow or Deny(reason).

### System claim (skeleton)
A system comprising: (a) an immune/reputation node that maintains defection records and aggregated reputation for principals; (b) an execution guard that, before execution, checks the PCU’s principal against defection isolation and a minimum reputation threshold; (c) wherein execution is blocked if the principal is isolated or below threshold.

### Method claim (skeleton)
A method of governing execution by reputation and defection, comprising: (1) maintaining defection records and aggregated reputation for principals; (2) upon an execution request, mapping the request’s principal to an identity; (3) blocking execution if the principal is isolated due to defection or if aggregated reputation is below a configured threshold.

### Runtime invariant
When ImmuneGuard is set, execution is allowed only if `node.allow_execution_by(principal_bytes, min_reputation)` returns `Ok(())`.

---

## 6. Composite Guard (Multiple Gates in Sequence)

### Invariant (formal)
“Execution is allowed only if every guard in a configured sequence returns Allow.”

- **Inputs:** Ordered list of guards, PCU, ExecutionContext.
- **Threshold:** For each guard, `guard.check(pcu, ctx)` must return Allow; first Deny terminates and returns Deny.
- **Outcome:** Allow iff all guards Allow.

### System claim (skeleton)
A system comprising: (a) a composite execution guard that holds an ordered list of sub-guards; (b) wherein the composite guard invokes each sub-guard in order and returns Deny on the first Deny, and Allow only if all sub-guards return Allow.

### Method claim (skeleton)
A method of governing execution by multiple constraints in sequence, comprising: (1) configuring an ordered list of execution guards; (2) for each execution request, invoking each guard in order; (3) if any guard returns Deny, blocking execution and returning that reason; (4) if all guards return Allow, permitting execution.

### Runtime invariant
For CompositeGuard, `check` returns `Allow` iff for all `g in guards`, `g.check(pcu, ctx)` returns `Allow`.

---

## 7. TELOS Commitment Membrane (AGP)

### Invariant (formal)
“If entropy budget is insufficient, or authority does not permit the scope, or trust is insufficient for the tier, then crossing is rejected and the action must not execute.”

- **Inputs:** Decision (action, agent_id, tier), entropy budget, authority scopes, trust score.
- **Threshold:** request_crossing checks entropy, authority, and (for high tier) trust.
- **Outcome:** CrossingResult.allowed true iff crossing committed; otherwise execution must not proceed.

### System claim (skeleton)
A system comprising: (a) a commitment membrane that separates reversible reasoning from irreversible action; (b) an entropy meter, an authority registry, and a trust accumulator; (c) wherein a request to cross the membrane is granted only when entropy, authority, and (where applicable) trust checks pass; (d) wherein execution of the action occurs only after a granted crossing.

### Method claim (skeleton)
A method of governing irreversible actions, comprising: (1) receiving a decision to perform an action with a consequence tier; (2) checking entropy budget, authority scope, and optionally trust; (3) rejecting the crossing if any check fails; (4) permitting the action to execute only when the crossing is granted.

### Runtime invariant
In AGP kernel, `context_switch` raises `ExecutionBlocked` unless `telos_membrane.request_crossing(decision, required_scope="execute:*")` returns `result.allowed == True`.

---

## 8. Summary Table

| # | Invariant (short) | System claim focus | Method claim focus |
|---|-------------------|--------------------|--------------------|
| 1 | Guard choke point | Engine + guard interface; no execution without Allow | Invoke guard; block on Deny |
| 2 | Developmental stage | Stage manager + decision engine; block when stage &lt; required | Maintain stage; compare; block |
| 3 | Autonomic risk | Mode controller + decision; block in CALM when risk &gt; tolerance | Maintain mode; assess risk; block |
| 4 | Biological risk input | Context carries risk; guard uses it for proposed action | Provide risk in context; guard uses it |
| 5 | Immune/reputation | Node + guard; block when isolated or reputation &lt; threshold | Maintain defection/reputation; block |
| 6 | Composite guard | Ordered list of guards; first Deny wins | Invoke in order; block on first Deny |
| 7 | TELOS membrane | Membrane + entropy + authority + trust; execute only after crossing | Check entropy/authority/trust; execute only if allowed |

---

**End of document.** All claims are derived from implemented or specified behavior in the repository.
