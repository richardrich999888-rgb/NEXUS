# Regulator-Grade Execution Tests

**Purpose:** Deterministic, documented tests that verify no intelligent execution can occur without passing through biological and accountability constraints. Suitable for audit and regulatory review.

---

## 1. Test Philosophy

- **Deterministic:** Same inputs → same outcome. No flaky or time-dependent assertions.
- **Code-backed:** Every assertion tied to a specific code path and invariant.
- **Bypass-focused:** Tests attempt to bypass guards; success = FAIL.
- **Documented:** Each test states invariant, expected outcome, and failure meaning.

---

## 2. Rust: PCU Execution Guard

### 2.1 Test: Guard blocks execution when stage = Infant, capability = Execute

| Field | Value |
|-------|--------|
| **Invariant** | Developmental stage gate: Execute requires Adult; Infant cannot execute. |
| **Test** | `test_guard_blocks_execute_at_infant_stage` |
| **Location** | `nexus-executor/tests/integration_tests.rs` |
| **Setup** | Executor with `NervousSystemGuard::new()`. Minimal valid PCU. |
| **Action** | `executor.execute(&pcu, context).await` |
| **Expected** | `Err(ExecutorError::ExecutionBlocked { .. })` |
| **Failure** | If execution succeeds → guard not applied or stage gate not enforced. |

**Run:** `cargo test -p nexus-executor test_guard_blocks_execute_at_infant_stage`

---

### 2.2 Test: No bypass when calling execute directly (guard set)

| Field | Value |
|-------|--------|
| **Invariant** | When guard is set, there is no code path that executes without calling guard.check(). |
| **Test** | `test_no_bypass_when_guard_set` (see below) |
| **Setup** | Executor with NervousSystemGuard. Valid PCU. |
| **Action** | Call execute(); do not call any API that “disables” the guard. |
| **Expected** | Execution blocked (Infant) or Allow only after guard allows. |
| **Failure** | If execution succeeds with guard set and stage=Infant → bypass exists. |

**Implementation:** Same as 2.1; the only way to “bypass” would be to not set the guard or to use a different code path. This test proves that with guard set, execution is blocked for Infant.

---

### 2.3 Test: Execution allowed when guard not set (baseline)

| Field | Value |
|-------|--------|
| **Invariant** | When guard is None, execution proceeds without guard check (baseline behavior). |
| **Test** | Existing integration tests with `guard: None`. |
| **Expected** | Execution succeeds for valid PCU. |
| **Failure** | If execution fails when guard is None for valid PCU → regression. |

---

### 2.4 Test: ImmuneGuard blocks anonymous principal

| Field | Value |
|-------|--------|
| **Invariant** | ImmuneGuard denies execution when principal is anonymous. |
| **Test** | `test_immune_guard_blocks_anonymous` |
| **Setup** | Executor with ImmuneGuard::new(), PCU with IdentityContext::anonymous(). |
| **Action** | executor.execute(&pcu, context).await |
| **Expected** | Err(ExecutorError::ExecutionBlocked { reason }) with "Anonymous" in reason. |
| **Failure** | If execution succeeds → anonymous principal bypassed immune guard. |

---

### 2.5 Test: ImmuneGuard allows non-anonymous when not isolated and reputation OK

| Field | Value |
|-------|--------|
| **Invariant** | ImmuneGuard allows execution when principal is not anonymous and allow_execution_by returns Ok. |
| **Test** | `test_immune_guard_allows_known_principal` |
| **Setup** | ImmuneGuard with default node (unknown principal gets INITIAL 0.5); min_reputation 0.0. PCU with non-anonymous identity. |
| **Expected** | Allow (execution proceeds or fails later for other reasons, but not ExecutionBlocked by immune). |
| **Failure** | If ExecutionBlocked with reason unrelated to anonymous/defection/reputation → misconfiguration. |

---

### 2.6 Test: CompositeGuard — first Deny wins

| Field | Value |
|-------|--------|
| **Invariant** | CompositeGuard returns Deny on first sub-guard Deny. |
| **Test** | `test_composite_guard_first_deny_wins` |
| **Setup** | CompositeGuard with [NervousSystemGuard, ImmuneGuard]. NervousSystemGuard blocks (Infant). |
| **Action** | execute() with valid PCU (non-anonymous). |
| **Expected** | ExecutionBlocked (reason from nervous-system, e.g. capability/stage). |
| **Failure** | If execution succeeds or reason is from immune before nervous → order or logic wrong. |

---

### 2.7 Test: Biological risk modulates nervous guard (high risk + CALM → block)

| Field | Value |
|-------|--------|
| **Invariant** | When biological_risk is high and autonomic mode is CALM, execution can be blocked by risk tolerance. |
| **Test** | `test_biological_risk_modulates_guard` (optional: depends on coordinator default mode and threshold). |
| **Setup** | NervousSystemGuard; ExecutionContext.with_biological_risk(0.9). Coordinator at CALM, max_risk e.g. 0.5. |
| **Expected** | ExecutionBlocked when risk > tolerance. |
| **Failure** | If high risk is ignored → biological_risk not applied. |

*Note:* Default coordinator may not be in CALM or may have high max_risk; test may need custom CoordinatorConfig to force CALM and low tolerance.

---

## 3. Python AGP: TELOS Gate

### 3.1 Test: context_switch raises ExecutionBlocked when TELOS rejects

| Field | Value |
|-------|--------|
| **Invariant** | When request_crossing returns not allowed, context_switch raises ExecutionBlocked. |
| **Test** | In agp-core: call kernel.context_switch(pcb) after configuring membrane to reject (e.g. exhaust entropy or revoke authority). |
| **Expected** | ExecutionBlocked raised. |
| **Failure** | If context_switch proceeds without crossing → TELOS gate bypassed. |

---

### 3.2 Test: context_switch proceeds when TELOS allows

| Field | Value |
|-------|--------|
| **Invariant** | When request_crossing returns allowed, context_switch proceeds (state RUNNING, last_scheduled_at set). |
| **Test** | Spawn process (registers agent); schedule; context_switch. |
| **Expected** | No exception; pcb.state == RUNNING. |
| **Failure** | If ExecutionBlocked when agent is registered and entropy/authority OK → misconfiguration. |

---

## 4. Test Implementation Checklist

| # | Test | Location | Status |
|---|------|----------|--------|
| 2.1 | Guard blocks at Infant | nexus-executor/tests/integration_tests.rs | Implemented |
| 2.3 | Production executor requires guard | nexus-executor/tests/integration_tests.rs | Implemented |
| 2.4 | ImmuneGuard blocks anonymous | nexus-executor/tests/integration_tests.rs | Implemented |
| 2.5 | ImmuneGuard allows known principal | nexus-executor/tests/integration_tests.rs | Implemented |
| 2.6 | CompositeGuard first Deny wins | nexus-executor/tests/integration_tests.rs | Implemented |
| 2.7 | Biological risk write-once retained | nexus-executor/tests/integration_tests.rs | Implemented |
| 2.8 | No proof on blocked execution | nexus-executor/tests/integration_tests.rs | Implemented |
| 3.1 | TELOS deny when unregistered / entropy exhausted | agp-core/tests/test_telos_gate.py | Implemented |
| 3.2 | TELOS allow when registered and entropy OK | agp-core/tests/test_telos_gate.py | Implemented |

---

## 5. Running the Suite

```bash
# Rust: all guard-related tests
cargo test -p nexus-executor guard
cargo test -p nexus-executor test_guard_blocks_execute_at_infant_stage
cargo test -p nexus-executor test_immune_guard
cargo test -p nexus-executor test_composite_guard

# Python: TELOS gate (run from agp-core with venv)
cd agp-core && .venv/bin/python tests/test_telos_gate.py
```

---

## 6. Audit Trail

For regulator or audit:

1. **Regulator summary:** DETERMINISTIC_EXECUTION_CONSTRAINTS.md (one page, invariants + tests).
2. **Invariant document:** This file + BIOLOGICAL_STACKS_VERIFICATION_AND_ALIGNMENT.md.
3. **Frozen interfaces:** FROZEN_INTERFACES.md.
4. **Patent claims:** PATENT_CLAIMS_FROM_ENFORCED_INVARIANTS.md.
5. **ISO / NIST control mapping:** ISO_NIST_CONTROL_MAPPING.md (§3 ISO 27001:2022 Annex A; §4 NIST SP 800-53 and CSF).
6. **Execution Law:** EXECUTION_LAW.md (plain-language execution path, enforcement points, denial semantics, non-bypassability).
7. **Auditor Q&A:** AUDITOR_QA.md (Q&A with code/test citations).
8. **Post-enforcement audit report:** EXECUTION_ENFORCEMENT_AUDIT_REPORT.md (PASS/FAIL per phase, SEV list, file:line).
9. **Code paths:** Guard check in executor.rs (line ~guard.check); TELOS in kernel.py context_switch.
10. **Test evidence:** Above tests; run and capture output (e.g. `cargo test ... -- --nocapture > test_report.txt`).
11. **Investor DPR:** DPR_ANGEL_INVESTORS.md (full detailed project report for angel investors: built infra, impact, value, monetization, patent strategy, investment need).

---

**End of document.**
