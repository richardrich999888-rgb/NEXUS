# Execution Law

**Plain-language statement of what “execution” means, how it is enforced, and the non-bypassability claim.** For regulators and auditors. References code and tests.

---

## 1. What “execution” means

- **Rust (PCU execution):** Running a Portable Computation Unit (PCU) in the WASM sandbox. One invocation of `PcuExecutor::execute(pcu, context)` constitutes one execution request. Execution includes: validation, optional cache lookup, WASM run, proof generation, and cache write on success.
- **Python (AGP task execution):** Handing control to a process so it can run. The moment the kernel assigns the CPU to a process (state → RUNNING, `last_scheduled_at` set) is the execution handoff. Execution is the act of crossing from “scheduled” to “running.”

In both cases, **execution** is the protected logical resource: no execution occurs without passing the enforcement point when it is configured.

---

## 2. The single allowed execution path

### Rust (PCU)

1. Caller invokes `PcuExecutor::execute(&pcu, context)`.
2. **Enforcement point:** If a guard is set, `guard.check(pcu, context)` is called first (before cache lookup, before WASM run).  
   - **Location:** `nexus-executor/src/executor.rs` — guard check at the start of `execute()` (lines ~148–156).
3. If guard returns **Deny:** function returns `Err(ExecutorError::ExecutionBlocked { reason })` immediately. No cache read for this request, no WASM run, no proof, no cache write.
4. If guard returns **Allow** (or no guard): validation → cache lookup → on miss: WASM run → proof creation → cache put → return result.

There is **no other code path** that runs PCU WASM or writes proofs/cache. All execution goes through `PcuExecutor::execute()`, and when a guard is set it is always consulted first.

### Python (AGP)

1. Scheduler selects a process; kernel calls `context_switch(pcb)`.
2. **Enforcement point:** Before changing process state, kernel calls `telos_membrane.request_crossing(decision, required_scope="execute:*")`.  
   - **Location:** `agp-core/src/os/kernel.py` — `context_switch()` (lines ~188–190).
3. If `request_crossing` returns `allowed=False:` kernel raises `ExecutionBlocked` and does **not** set `pcb.state` to RUNNING or update `last_scheduled_at`.
4. If `allowed=True:` kernel sets `pcb.state = RUNNING` and `pcb.last_scheduled_at`; execution handoff is complete.

There is **no other path** in the kernel that sets a process to RUNNING without going through `request_crossing` in `context_switch`.

**AGP OS run loop:** The kernel’s scheduler loop is `BioKernel.schedule()` (`agp-core/src/os/kernel.py`, lines 149–177). It selects a runnable process and calls **only** `await self.context_switch(next_process)` (line 171). The kernel does **not** use `AdvancedScheduler` for execution handoff; it uses its own `schedule()` and `context_switch()`. The standalone `AdvancedScheduler` (`agp-core/src/os/scheduler.py`) has a `preempt()` method that sets `new_proc.state = ProcessState.RUNNING` without calling TELOS; **no caller in the kernel or main app** uses `preempt()` for execution handoff. If code is later wired to call `advanced_scheduler.preempt()` to hand control to a process, that would bypass TELOS; do not use `preempt()` for execution handoff — use `kernel.context_switch(pcb)` instead.

---

## 3. Enforcement points (exact locations)

| Component        | File                          | Approx. line | What is enforced                                      |
|-----------------|-------------------------------|-------------|------------------------------------------------------|
| PCU guard check | `nexus-executor/src/executor.rs` | 148–156     | Every `execute()` call checks guard first; Deny → return error. |
| Production guard| `nexus-executor/src/executor.rs` | 52–61       | `ExecutorBuilder::production()` sets a default guard. |
| TELOS gate      | `agp-core/src/os/kernel.py`   | 188–190     | `context_switch()` calls `request_crossing()`; on not allowed, raises before RUNNING. |
| Crossing logic  | `agp-core/src/telos/membrane.py` | 212–279     | `request_crossing()` implements entropy, authority, trust checks. |

---

## 4. Denial semantics

- **Rust:** On guard **Deny**, the executor returns `Err(ExecutorError::ExecutionBlocked { reason })`. No proof is generated. No cache entry is written. Identical retry with same inputs yields the same error (no cache write on block).
- **Python:** On TELOS **not allowed**, kernel raises `ExecutionBlocked`. Process state is not set to RUNNING; `last_scheduled_at` is not updated. The membrane may record the denied crossing for audit (e.g. `trust_accumulator.record`); this is not an “execution” artifact.

---

## 5. Non-bypassability claim (scope)

- **With guard set (Rust):** There is no intended code path that executes PCU WASM or writes proof/cache after a guard **Deny**. Bypass would require changing or replacing the binary, or subverting the guard implementation itself.
- **With TELOS in use (Python):** There is no intended code path in the kernel that sets a process to RUNNING without a successful `request_crossing` for the execution handoff. Bypass would require modifying the kernel or membrane.
- **Without guard (Rust):** If the executor is built with `guard: None` (e.g. via `ExecutorBuilder::new(...).build()`), execution is not constrained by the guard. Policy: use `ExecutorBuilder::production()` for deployment so a guard is always set. The CLI binary (`nexus-exec`) uses `production()` and therefore enforces the guard.
- **Determinism:** Same inputs (same PCU, same context, same guard state) produce the same allow/deny outcome. Tests enforce this (e.g. `test_no_proof_on_blocked_execution`, `red_team_composite_guard_order_invariance`).

This document does not claim that the system cannot be subverted by compromise of the host, the process, or the application code; it states that within the designed execution path, **unsafe execution is structurally impossible** without code or configuration change.

---

## 6. References

- **Guard / executor:** `nexus-executor/src/guard.rs`, `nexus-executor/src/executor.rs`, `nexus-executor/src/guards/` (nervous, immune, composite).
- **Tests:** `nexus-executor/tests/integration_tests.rs`, `nexus-executor/tests/red_team_execution.rs` (guard, no proof on block, composite order, production guard).
- **TELOS:** `agp-core/src/telos/membrane.py`, `agp-core/src/os/kernel.py`; `agp-core/tests/test_telos_gate.py`.
- **ISO/NIST mapping:** `docs/ISO_NIST_CONTROL_MAPPING.md`.
- **Regulator summary:** `docs/DETERMINISTIC_EXECUTION_CONSTRAINTS.md`.
- **Frozen interfaces:** `docs/FROZEN_INTERFACES.md`.

---

**End of document.**
