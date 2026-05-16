# Auditor Q&A — Execution Enforcement

**Answers cite code and tests, not intent.** For ISO/NIST assessors and risk reviewers.

---

## Q1: How do you prevent unauthorized execution?

**Rust (PCU):**  
Execution is gated by an optional **ExecutionGuard**. When the executor is built with a guard (e.g. via `ExecutorBuilder::production()`), every call to `PcuExecutor::execute()` runs `guard.check(pcu, context)` first (`nexus-executor/src/executor.rs`, lines ~148–156). If the guard returns **Deny**, the function returns `Err(ExecutorError::ExecutionBlocked { reason })` and does not run the PCU, generate a proof, or write to the cache. Guards can enforce identity (e.g. ImmuneGuard denies anonymous principals), developmental stage (e.g. NervousSystemGuard blocks Execute at Infant), risk tolerance, and reputation.  
**Evidence:** `test_guard_blocks_execute_at_infant_stage`, `test_immune_guard_blocks_anonymous`, `red_team_production_blocks_infant`; single call site for PCU execution is `PcuExecutor::execute()`.

**Python (AGP):**  
Task execution handoff is gated by the TELOS membrane. Before setting a process to RUNNING, the kernel calls `telos_membrane.request_crossing(decision, required_scope="execute:*")` in `context_switch()` (`agp-core/src/os/kernel.py`, lines ~188–190). If the result is not allowed, the kernel raises `ExecutionBlocked` and does not set `pcb.state` to RUNNING or update `last_scheduled_at`. The membrane checks entropy budget, authority scope, and (for high-consequence actions) trust.  
**Evidence:** `agp-core/tests/test_telos_gate.py` (unregistered / exhausted entropy → denied; registered with scope and entropy → allowed).

---

## Q2: What happens when execution is denied?

**Rust:**  
The executor returns `Err(ExecutorError::ExecutionBlocked { reason })`. No WASM is run. No proof is created. No cache entry is written. A second identical request receives the same error (no cache write on block).  
**Evidence:** `test_no_proof_on_blocked_execution`, `red_team_no_cache_after_block`; guard check and early return in `executor.rs` (lines ~150–156) occur before cache lookup (line ~179) and proof/cache write (lines ~259–269).

**Python:**  
The kernel raises `ExecutionBlocked` with the membrane’s reason. Process state is not changed to RUNNING; `last_scheduled_at` is not set. The membrane may record the denied crossing (e.g. in `trust_accumulator`) for audit; this is not an execution artifact.  
**Evidence:** `context_switch()` raises before updating `pcb.state` or `pcb.last_scheduled_at` (`kernel.py` lines ~188–193).

---

## Q3: Can execution occur without audit?

**Rust:**  
No. When execution is **allowed**, it proceeds to run WASM and then produces an `ExecutionProof` and (on cache miss) a cache entry. There is no path that runs WASM and then skips proof creation or cache write on success. When execution is **denied**, no proof or cache entry is created, so there is no “success” record to audit; the denial is observable via the returned error.  
**Evidence:** Control flow in `executor.rs`: guard → validation → cache get → on miss: run → `generate_proof()` → `cache.put()`; `test_no_proof_on_blocked_execution`.

**Python:**  
Execution handoff (state → RUNNING) only occurs after a successful `request_crossing`. Denied crossings can be recorded by the membrane (e.g. `trust_accumulator.record`) for audit. The kernel does not advance process state on deny, so there is no “execution” without a prior allowed crossing.  
**Evidence:** `context_switch()`; `test_telos_gate.py`.

---

## Q4: What would it take to bypass this system?

**Rust:**  
- **Without a guard:** Build the executor with `ExecutorBuilder::new(...).build()` (guard `None`). Policy is to use `ExecutorBuilder::production()` for deployment; the CLI binary (`nexus-exec`) uses `production()` so it always has a guard.  
- **With a guard:** Bypass would require one of: (1) replacing or modifying the binary so that the guard is not called or Deny is ignored, (2) changing the guard implementation to always return Allow, or (3) subverting the process (e.g. memory corruption). There is no intended API or code path that executes after a Deny.  
**Evidence:** Single execution entry point `PcuExecutor::execute()`; guard check is the first conditional; `red_team_composite_first_deny_wins`, `red_team_composite_second_guard_blocks_when_first_allows`, `red_team_production_blocks_infant`.

**Python:**  
Bypass would require: (1) changing the kernel so that `context_switch()` does not call `request_crossing()` or ignores the result, (2) modifying the membrane to always return allowed, or (3) subverting the process. There is no intended path that sets RUNNING without a successful crossing in `context_switch`.  
**Evidence:** Single call to `request_crossing` in `context_switch`; exception raised when not allowed before state update.

---

## Q5: How do you ensure production deployments use the guard?

**Rust:**  
`ExecutorBuilder::production(...)` sets a default guard (e.g. `NervousSystemGuard`). The CLI binary uses `ExecutorBuilder::production(...).build()` so the resulting executor always has a guard. The test `test_production_executor_requires_guard` asserts that an executor built with `production().build()` has `has_guard() == true`.  
**Evidence:** `nexus-executor/src/executor.rs` (lines 52–61, 123–124); `nexus-executor/src/main.rs` (builder uses `production()`); `test_production_executor_requires_guard`.

---

## Q6: Is the guard order fixed? What if multiple guards are used?

**Rust:**  
`CompositeGuard` runs sub-guards in the order they were added. The first guard that returns **Deny** causes an immediate return; later guards are not evaluated. The first that allows is not sufficient if a later one denies.  
**Evidence:** `nexus-executor/src/guards/composite.rs` (iterate, return on first Deny); `test_composite_guard_first_deny_wins`, `red_team_composite_guard_order_invariance`, `red_team_composite_guard_order_second_evaluated_when_first_allows`.

---

**End of Q&A.**
