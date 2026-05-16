# Deterministic Execution Constraints for Intelligent Systems

**One-page summary for regulators and auditors.** No biology language. No AGI hype. Invariants and tests only.

---

## 1. What is enforced

Execution of portable computation units (PCUs) and governed tasks is **gated** by a single enforcement point. No execution occurs without passing the gate when it is configured.

- **Rust (PCU execution):** An optional **ExecutionGuard** is invoked before any PCU run. If the guard returns **Deny**, the executor returns an error, does **not** execute, does **not** generate a proof, and does **not** write to the cache.
- **Python (AGP task execution):** Before handing control to a process, the kernel calls a **commitment membrane** (TELOS). If the crossing is **not allowed** (entropy, authority, or trust check fails), execution is blocked and an exception is raised.

---

## 2. Invariants (code-backed)

| # | Invariant | Enforced by |
|---|-----------|-------------|
| 1 | When a guard is set, every execution request is checked; on Deny, execution does not proceed. | `PcuExecutor::execute()` calls `guard.check()` before cache/run. |
| 2 | When execution is blocked, no proof is produced and no cache entry is written. | Guard check runs before cache lookup and proof generation. |
| 3 | Production executor builds include a guard. | `ExecutorBuilder::production()` sets a default guard; test `test_production_executor_requires_guard` asserts `has_guard()`. |
| 4 | Risk input to the guard is write-once per request and not exposed to executing code. | `ExecutionContext.biological_risk` is set by the caller; guest (WASM) has no access. |
| 5 | Multiple constraints are applied in a fixed order; first Deny wins. | `CompositeGuard` runs sub-guards in order; first Deny terminates and blocks execution. |
| 6 | Task execution (AGP) does not proceed without a successful membrane crossing. | `context_switch()` calls `request_crossing()`; on not allowed, raises and does not set RUNNING. |

---

## 3. Tests (reproducible)

| Test | What it proves |
|------|----------------|
| `test_guard_blocks_execute_at_infant_stage` | With guard set and maturity state “Infant,” execution is blocked; error is ExecutionBlocked. |
| `test_immune_guard_blocks_anonymous` | Anonymous principal is denied by the immune/reputation guard. |
| `test_composite_guard_first_deny_wins` | First sub-guard that denies wins; execution is blocked. |
| `test_production_executor_requires_guard` | Production build has a guard set. |
| `test_biological_risk_write_once_retained` | Risk value is set once and retained (clamped); no mutation API for guest. |
| `test_no_proof_on_blocked_execution` | Blocked execution returns error; second identical request is still blocked (no cache write). |
| TELOS gate tests (Python) | Unregistered agent or exhausted entropy → crossing denied; registered agent with sufficient entropy → crossing allowed. |

**Run:**  
Rust: `cargo test -p nexus-executor guard` (and the tests above).  
Python: `cd agp-core && .venv/bin/python tests/test_telos_gate.py`.

---

## 4. Bypass

- **Without guard:** If no guard is set, execution is not constrained by the guard. Policy: use `ExecutorBuilder::production()` for deployment so a guard is always set.
- **With guard:** Bypass requires subverting the guard (e.g. process compromise or replacing the binary). There is no intended code path that executes after a Deny.

---

## 5. Interfaces (frozen)

- **ExecutionGuard:** `check(pcu, ctx) -> Allow | Deny(reason)`. Signature and semantics are stable; regulator and patent claims refer to this contract.
- **CompositeGuard:** Ordered list of guards; first Deny wins. Semantics are stable.
- **CrossingGate (TELOS):** No task execution without successful `request_crossing` on the execution path. Contract is stable.

See `docs/FROZEN_INTERFACES.md` for the full list.

---

**End of summary.** All statements above are tied to code and tests in the repository.
