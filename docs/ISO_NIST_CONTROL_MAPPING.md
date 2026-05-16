# ISO / NIST Control Mapping — Execution Constraints

**Purpose:** Map NEXUS execution guard and TELOS gate mechanisms to standard control language for regulator and auditor alignment. No normative claim that NEXUS “implements” these frameworks; this document states how the implemented invariants **align with** or **support satisfaction of** the following controls.

**References:**  
- ISO/IEC 27001:2022, Annex A  
- NIST SP 800-53 Rev. 5  
- NIST Cybersecurity Framework (CSF) v1.1 / v2.0  

---

## 1. Scope of mapping

In this context, **execution** (PCU run or task run) is treated as the **protected logical resource** subject to access control. The primary threat addressed is unauthorized or unsafe execution of intelligent actions, including bypass of safety or accountability mechanisms.

| NEXUS mechanism | Description |
|----------------|-------------|
| **ExecutionGuard** (Rust) | Single choke point before PCU execution; on Deny: no run, no proof, no cache write. |
| **CompositeGuard** | Ordered sub-guards; first Deny wins. |
| **Production guard** | `ExecutorBuilder::production()` sets a guard; tests assert `has_guard()`. |
| **Risk input** | `ExecutionContext.biological_risk` write-once per request; not exposed to guest. |
| **TELOS CrossingGate** (Python) | `context_switch()` calls `request_crossing()`; on not allowed, raises and does not set RUNNING. |

---

## 2. How to use this mapping

This mapping does not assert organizational compliance or certification; it documents how specific code-level execution controls align with the intent of selected ISO/NIST controls.

- **For ISO 27001:** Use the Annex A control references as evidence that execution authorization and operations security are addressed in code and tests.
- **For NIST 800-53:** Use the AC/AU/CM references to show access enforcement, audit behavior, and configuration of production vs test.
- **For NIST CSF:** Use the PR/DE subcategories to show Protect (access control, protective technology) and Detect (audit of blocked execution).

---

## 3. ISO/IEC 27001:2022 Annex A — Control language alignment

### 3.1 Access control (A.5.15)

| Control objective (summary) | ISO 27001:2022 language (aligned) | NEXUS implementation |
|-----------------------------|------------------------------------|------------------------|
| Rules to control logical access | *Establish and implement rules to control logical access to information and other associated assets based on business and information security requirements.* | ExecutionGuard and TELOS gate enforce a single authorization point. Access to execution (PCU run, task run) is granted only when the guard/membrane allows. |
| Authorized access only | *Ensure authorised access and prevent unauthorised access.* | When guard returns Deny or `request_crossing` returns not allowed, execution does not proceed. No code path executes after Deny. |
| Need to know / least privilege | *Access limited to what is necessary (need to know, least privilege).* | Guard checks identity, developmental stage, risk tolerance, and (in ImmuneGuard) reputation; TELOS checks authority scope and entropy. Execution is the controlled “object.” |

**Evidence:** `PcuExecutor::execute()` calls `guard.check()` before cache/run; `context_switch()` calls `request_crossing()` before setting RUNNING. Tests: `test_guard_blocks_execute_at_infant_stage`, `test_immune_guard_blocks_anonymous`, TELOS gate tests.

---

### 3.2 Operations security (A.5.24)

| Control objective (summary) | ISO 27001:2022 language (aligned) | NEXUS implementation |
|-----------------------------|------------------------------------|------------------------|
| Operational procedures | *Ensure correct and secure operations of information processing facilities.* | Execution is only performed after a defined check (guard / crossing). Blocked requests do not alter system state (no proof, no cache write). |
| Change / execution control | *Changes to assets and operations are authorised and documented.* | Production builds are required to use a guard (`ExecutorBuilder::production()`); test builds may omit it. No “authorised” execution without passing the gate when configured. |

**Evidence:** `test_no_proof_on_blocked_execution`, `test_production_executor_requires_guard`, FROZEN_INTERFACES.md.

---

### 3.3 Secure development and application security (A.5.25 / A.5.26)

| Control objective (summary) | ISO 27001:2022 language (aligned) | NEXUS implementation |
|-----------------------------|------------------------------------|------------------------|
| Security in development | *Rules for the development of software and systems.* | ExecutionGuard and CrossingGate are defined, frozen interfaces; no execution path bypasses them when configured. |
| Application access control | *Applications enforce access control and do not expose security-relevant information to unauthorised parties.* | Risk input (`biological_risk`) is set by the caller; guest (WASM) has no access. Blocked execution does not produce proofs or cache entries. |

**Evidence:** DETERMINISTIC_EXECUTION_CONSTRAINTS.md § 2 (invariants 2, 4); FROZEN_INTERFACES.md; `test_biological_risk_write_once_retained`.

---

## 4. NIST SP 800-53 Rev. 5 — Control language alignment

### 4.1 AC-3: Access Enforcement

| Control | NIST 800-53 statement | NEXUS implementation |
|---------|----------------------|----------------------|
| **AC-3** | *Enforce approved authorizations for logical access to information and system resources in accordance with applicable access control policies.* | ExecutionGuard and TELOS gate are the enforcement points for “logical access” to execution. Policies are implemented by NervousSystemGuard (stage, capability, risk), ImmuneGuard (identity, reputation), and TELOS (authority, entropy). Deny → no execution. |

**Evidence:** Same as § 3.1. AC-3 is required at all baselines (Low, Moderate, High).

---

### 4.2 AC-3(5): Security-relevant Information (enhancement)

| Control | NIST 800-53 statement | NEXUS implementation |
|---------|----------------------|----------------------|
| **AC-3(5)** | *Prevent access to [security-relevant information] except during secure, non-operable system states.* | Risk input to the guard (`biological_risk`) is write-once per request and not exposed to the executing guest (WASM). Security-relevant decision inputs are not passed into the execution environment. |

**Evidence:** `ExecutionContext.biological_risk`; `test_biological_risk_write_once_retained`.

---

### 4.3 AC-3(12): Assert and Enforce Application Access (enhancement)

| Control | NIST 800-53 statement | NEXUS implementation |
|---------|----------------------|----------------------|
| **AC-3(12)** | *Require applications to assert, as part of the installation process, the access needed… Provide an enforcement mechanism to prevent unauthorized access.* | Production executor is built with a guard (`ExecutorBuilder::production()`); the guard is the enforcement mechanism. Applications (PCU execution path) cannot run without passing the guard when it is set. |

**Evidence:** `test_production_executor_requires_guard`, FROZEN_INTERFACES.md § 4.

---

### 4.4 AC-6: Least Privilege

| Control | NIST 800-53 statement | NEXUS implementation |
|---------|----------------------|----------------------|
| **AC-6** | *Employ the principle of least privilege, allowing only authorized accesses.* | Guard and TELOS restrict execution to principals and contexts that satisfy stage, capability, risk, identity, and authority. Anonymous or insufficiently trusted principals are denied (ImmuneGuard); immature stage denies Execute (NervousSystemGuard). |

**Evidence:** `test_immune_guard_blocks_anonymous`, `test_guard_blocks_execute_at_infant_stage`.

---

### 4.5 AU-2 / AU-3 / AU-9: Audit (no proof on blocked execution)

| Control | NIST 800-53 statement | NEXUS implementation |
|---------|----------------------|----------------------|
| **AU-2** | *Ensure that the events that are audited are sufficient to support after-the-fact investigations.* | Blocked execution returns `ExecutorError::ExecutionBlocked { reason }`; no proof or cache entry is created. Thus “success” cannot be falsely claimed; only allowed executions produce audit records (proofs). |
| **AU-3** | *Content of audit records.* | When execution is blocked, the only record is the error (reason); no execution proof is generated. |
| **AU-9** | *Protect audit information from unauthorized access, modification, and deletion.* | By not creating a proof or cache entry on Deny, the system does not create audit-like artifacts that could be misused for blocked attempts. |

**Evidence:** `test_no_proof_on_blocked_execution`; guard check occurs before cache lookup and proof generation (executor.rs).

---

### 4.6 CM-6 / CM-7: Configuration / Least Functionality

| Control | NIST 800-53 statement | NEXUS implementation |
|---------|----------------------|----------------------|
| **CM-6** | *Establish and document configuration settings.* | Production vs test is explicit: `ExecutorBuilder::production()` sets a guard; test builds may use `ExecutorBuilder::new()` with no guard. |
| **CM-7** | *Configure the system to provide only essential capabilities.* | When a guard is set, the only path to execution is through the guard; no alternate “essential” path that bypasses the guard is provided. |

**Evidence:** FROZEN_INTERFACES.md § 4; `test_production_executor_requires_guard`.

---

### 4.7 NIST Cybersecurity Framework (CSF) — Subcategory mapping

| CSF (v1.1) | CSF (v2.0) | Statement | NEXUS implementation |
|------------|------------|-----------|------------------------|
| **PR.AC-4** | **PR.AA-05** | Access permissions are managed; principle of least privilege. | Guard and TELOS enforce who/what can execute; least privilege via stage, identity, reputation, authority. |
| **PR.PT-3** | **PR.PT-3** | Principle of least functionality; only necessary capabilities. | Execution is the capability; it is only granted when the guard/membrane allows. |
| **DE.CM** | **DE.CM** | Network/system monitored to detect cybersecurity events. | Blocked execution is detectable via `ExecutionBlocked` error and absence of proof; supports detection of unauthorized execution attempts. |

**Evidence:** Same code paths and tests as above; regulator summary in DETERMINISTIC_EXECUTION_CONSTRAINTS.md.

---

## 5. Summary table

| Framework | Control(s) | NEXUS mechanism |
|-----------|------------|-----------------|
| ISO 27001:2022 | A.5.15 (Access control) | ExecutionGuard; TELOS CrossingGate |
| ISO 27001:2022 | A.5.24 (Operations security) | No proof/cache on block; production guard |
| ISO 27001:2022 | A.5.25 / A.5.26 (Development / application security) | Frozen interfaces; risk not exposed to guest |
| NIST 800-53 | AC-3 (Access enforcement) | Guard and TELOS as single enforcement point |
| NIST 800-53 | AC-3(5), AC-3(12), AC-6 | Risk write-once; production guard; least privilege |
| NIST 800-53 | AU-2, AU-3, AU-9 | No proof on block |
| NIST 800-53 | CM-6, CM-7 | Production vs test configuration; no bypass path |
| NIST CSF | PR.AC-4, PR.PT-3, DE.CM | Access/permissions; least functionality; detect blocked attempts |

---

## 6. Audit trail

- **Regulator summary:** DETERMINISTIC_EXECUTION_CONSTRAINTS.md  
- **Invariants and tests:** REGULATOR_GRADE_EXECUTION_TESTS.md, BIOLOGICAL_STACKS_VERIFICATION_AND_ALIGNMENT.md  
- **Frozen interfaces:** FROZEN_INTERFACES.md  
- **This mapping:** ISO_NIST_CONTROL_MAPPING.md  

**End of document.**
