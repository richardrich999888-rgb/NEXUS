# Frozen Interfaces — Execution Constraints

**Purpose:** Interfaces that must not drift. Regulator and patent claims depend on them.

---

## 1. ExecutionGuard (Rust — nexus-executor)

- **Trait:** `ExecutionGuard::check(&self, pcu: &PCU, ctx: &ExecutionContext) -> GuardDecision`
- **Enum:** `GuardDecision::Allow | Deny(String)`
- **Contract:** When a guard is set, the executor calls `check` before execution; on `Deny(reason)` it returns `Err(ExecutorError::ExecutionBlocked { reason })` and does not execute, generate a proof, or write to the cache.
- **Do not:** Change the signature of `check`, add optional parameters that change semantics, or allow `Deny` to be ignored.

---

## 2. CompositeGuard (Rust — nexus-executor)

- **Contract:** Guards are checked in order; first `Deny` wins; no aggregation or override.
- **Do not:** Change first-Deny-wins semantics or allow sub-guards to be skipped.

---

## 3. CrossingGate (Python — agp-core TELOS)

- **Entry:** `context_switch()` in `agp-core/src/os/kernel.py` calls `telos_membrane.request_crossing(decision, required_scope="execute:*")` before setting state RUNNING.
- **Contract:** If `not result.allowed`, raise `ExecutionBlocked`; do not run the process.
- **Do not:** Bypass `request_crossing` on the execution path or allow execution when `allowed` is False.

---

## 4. Production vs testing

- **Production:** `ExecutorBuilder::production(...).build()` must set a guard. `PcuExecutor::has_guard()` must be true for production builds.
- **Testing:** `ExecutorBuilder::new(...).build()` may omit the guard for baseline tests.
- **Do not:** Allow production deployment without a guard unless explicitly documented as unsafe.

---

**End.** Change these only with a documented rationale and test/regulator impact.
