# PATENT APPLICATION

## Execution Law for Governed Autonomous Computation

---

**Applicant:** SYNTRIASS Labs Private Limited  
**Inventor:** Katta Naga Sri Ganesh  
**Docket No.:** [To Be Assigned]  
**Filing Date:** [To Be Assigned]  

---

## ABSTRACT

A system and method for governed autonomous computation in which execution is treated as a protected logical resource. An execution engine receives an execution request comprising a computation unit and an execution context. When a guard is configured, the engine invokes the guard to determine whether execution is allowed before performing cache lookup, computation, or proof generation. The guard returns an allow decision or a deny decision. Upon a deny decision, the engine returns an error and produces no execution artifact, including no cache entry and no execution proof. Upon an allow decision, the engine proceeds with validation, cache lookup, execution, and proof generation. In a kernel embodiment, only the kernel may transition a process to a running state, and the kernel invokes a crossing check before such transition. Denial of the crossing check prevents the transition and produces no execution handoff. Prior art governs actions or outputs; the present invention governs execution itself at a structural boundary.

---

## CROSS-REFERENCES TO RELATED APPLICATIONS

[To Be Inserted—Patent #2 (Commitment Membrane) if filed]

---

## TECHNICAL FIELD

The present disclosure relates to autonomous computing systems, execution governance, and operating systems. More particularly, it relates to systems and methods that enforce governance at the execution boundary—when computation is invoked—rather than at training time, inference time, or via output filtering.

---

## BACKGROUND

**1. Field of Endeavor**

Autonomous software agents and artificial intelligence systems execute computations that produce side effects, including state changes, cache updates, proofs of execution, and external actions. Governing such systems requires enforcement at the point where execution is invoked. Conventional approaches—output filtering, prompt engineering, training-time alignment, guardrails, and sandboxing—operate either before or after execution, or protect the host environment, but do not treat execution invocation as a protected logical resource subject to a mandatory structural gate.

**2. Limitations of the Prior Art**

Output filtering systems act after execution has occurred. Malicious or unintended outputs may have already caused side effects before filtering is applied. Such systems do not prevent execution; they filter results.

Prompt engineering and training-time alignment (e.g., RLHF) seek to influence model behavior through inputs or training. These approaches are bypassable by adversarial inputs or distribution shift at runtime. They provide no structural guarantee that execution will be governed.

Guardrail systems and guard models (e.g., GuardAgent, AgentSpec) check actions or outputs against safety requirements. Such systems may be advisory or may be bypassed by routing execution around the checker. They do not provide a mandatory choke point through which all execution must pass.

Trusted Execution Environments (TEEs) and sandboxing protect the host from malicious code. They do not govern whether execution is permitted by policy. An authorized but policy-violating computation may still execute within a sandbox.

AgentGuardian and similar systems provide context-aware access control for tool invocations. They govern actions or tool calls rather than execution invocation itself. The present invention differs in that it treats execution—the act of running the computation—as the controlled resource, with a single structural gate invoked before any execution path proceeds.

**3. Objects of the Invention**

It is an object of the present invention to provide a system in which execution is treated as a protected logical resource.

It is a further object to provide a mandatory execution gate through which every execution request must pass when configured.

It is a further object to ensure that denial of execution produces no execution artifact, including no cache entry and no execution proof.

It is a further object to provide a kernel-controlled execution handoff wherein only the kernel may transition a process to a running state, and such transition occurs only after a crossing check.

It is a further object to structurally prevent bypass such that no alternate code path produces execution artifacts when the gate denies.

---

## SUMMARY OF THE INVENTION

In one aspect, the invention provides a system for governed autonomous computation comprising: an execution engine configured to receive an execution request; an execution guard, when configured, operatively coupled to the execution engine and invoked before cache lookup, computation, or proof generation, the execution guard configured to return an allow decision or a deny decision based on the execution request; and logic configured such that upon a deny decision, the execution engine returns an error and produces no execution artifact, and upon an allow decision, the execution engine proceeds with execution and produces an execution proof and optionally a cache entry.

In another aspect, the invention provides a method for governing execution comprising: receiving an execution request; when a guard is configured, invoking the guard to obtain a decision before performing cache lookup or computation; if the decision is deny, returning an error and producing no execution artifact; and if the decision is allow, proceeding with validation, cache lookup, execution, and proof generation.

In a further aspect, the invention provides an apparatus comprising an execution guard interface configured to receive a computation unit and an execution context and to return an allow decision or a deny decision, and a composite guard comprising a plurality of guards evaluated in sequence, wherein a first deny decision from any guard terminates evaluation and returns deny, and wherein allow is returned only when all guards return allow.

---

## BRIEF DESCRIPTION OF THE DRAWINGS

**FIG. 1** is a block diagram of a system for governed autonomous computation according to an embodiment.

**FIG. 2** is a flowchart illustrating an execution allow path.

**FIG. 3** is a flowchart illustrating an execution deny path.

**FIG. 4** is a flowchart illustrating a kernel-controlled execution handoff with crossing check.

---

## DETAILED DESCRIPTION OF THE INVENTION

### 1. Overview

With reference to **FIG. 1**, a system **100** for governed autonomous computation includes an execution engine **110**, an execution guard **120** (when configured), a cache **130**, and a proof generator **140**. A caller **102** submits an execution request **104** comprising a computation unit and an execution context. The execution engine **110** receives the request **104**. When the execution guard **120** is configured, the execution engine **110** invokes the guard **120** before performing cache lookup, computation, or proof generation. The guard **120** returns an allow decision or a deny decision. Upon a deny decision, the execution engine **110** returns an error **106** and produces no execution artifact—no cache entry, no execution proof. Upon an allow decision, the execution engine **110** proceeds with validation, cache lookup, execution, and proof generation, and returns a result **108** comprising an execution proof.

### 2. Mandatory Execution Gate

The execution gate is mandatory in the sense that when configured, there is no code path that executes the computation, performs cache lookup, or generates a proof without first invoking the guard. The guard is invoked at the earliest point in the execution flow—before validation (other than guard evaluation), before cache lookup, and before any computation. Reference implementation: `nexus-executor/src/executor.rs`, lines 147–156. The guard check occurs at the start of the execute method. If the guard returns deny, the method returns immediately with an error. No subsequent steps—cache lookup, WASM instantiation, proof creation, cache write—are performed.

### 3. Execution Guard Interface

The execution guard interface accepts a computation unit and an execution context and returns one of two decisions: allow or deny. When deny is returned, a reason string may be provided. The interface is frozen in the sense that implementations may vary (e.g., developmental gating, identity checks) but the contract—allow or deny—remains fixed. Reference implementation: `nexus-executor/src/guard.rs`, lines 13–28. The GuardDecision enum comprises Allow and Deny(String). The ExecutionGuard trait defines a check method.

### 4. Composite Guard with First-Deny Semantics

A composite guard comprises a plurality of guards evaluated in sequence. Evaluation proceeds in order. If any guard returns deny, evaluation terminates and deny is returned immediately. Allow is returned only when all guards return allow. Reference implementation: `nexus-executor/src/guards/composite.rs`, lines 41–49. The first-deny-wins ordering ensures that the most restrictive policy prevails without requiring coordination among guards.

### 5. Production Configuration

A production configuration structurally requires a guard. The builder for a production executor sets a default guard such that no unguarded production build is possible without explicit configuration change. Reference implementation: `nexus-executor/src/executor.rs`, lines 52–61. ExecutorBuilder::production() sets guard to Some(NervousSystemGuard::new()) or equivalent. The CLI binary uses production() and therefore enforces the guard.

### 6. No-Proof-on-Deny Invariant

When the guard returns deny, the execution engine returns an error before any of the following occur: cache lookup for the request, computation (e.g., WASM run), proof creation, or cache write. Control flow ensures that the return-on-deny occurs prior to the cache get (reference: `nexus-executor/src/executor.rs`, line 178), prior to proof creation (which occurs only after successful execution), and prior to cache put. A test verifies that blocked execution produces no cache hit on retry: `nexus-executor/tests/integration_tests.rs`, lines 332–356 (test_no_proof_on_blocked_execution).

### 7. Kernel-Controlled Execution Handoff

In an agent operating system embodiment, processes are represented by process control blocks. A process state includes READY, RUNNING, and others. Only the kernel may transition a process from READY to RUNNING. The kernel invokes a context_switch operation when handing execution to a process. Before setting the process state to RUNNING, the kernel invokes a crossing check (e.g., request_crossing with scope "execute:*"). If the crossing check returns not allowed, the kernel raises an exception and does not set the process state to RUNNING. If the crossing check returns allowed, the kernel sets the process state to RUNNING and updates scheduling metadata. Reference implementation: `agp-core/src/os/kernel.py`, lines 177–194. The ProcessControlBlock and ProcessState are defined in `agp-core/src/os/process.py`, lines 18–26, 64–65.

### 8. Execution Invariants

The following invariants are enforced by control flow:

| Invariant | Enforcement |
|-----------|-------------|
| If guard returns deny, execution does not proceed | Early return on deny |
| If guard returns deny, no cache read for the request | Guard invoked before cache get |
| If guard returns deny, no proof is created | Proof created only after successful run |
| If guard returns deny, no cache write | Cache put only on successful execution |
| Crossing check denied → no RUNNING transition | Exception raised before state change |
| RUNNING set only in context_switch | Single assignment point |

---

## NOVELTY, DEFENSIBILITY, NON-OBVIOUSNESS & PRIOR ART

### Novelty

**Novelty assessment:** Combination-based novelty. Prior art includes GuardAgent, AgentSpec, AgentGuardian, TEEs, sandboxing. The invention's novelty lies in: (a) execution treated as the protected logical resource (not actions, outputs, or host); (b) mandatory gate at execution boundary before cache lookup and proof generation; (c) no-proof-on-deny invariant enforced by control flow; (d) kernel as sole path to RUNNING with crossing check; (e) production configuration structurally requiring guard. No single reference discloses governance of execution invocation itself with structural denial and no artifact on deny.

### Defensibility

**Design-around difficulty:** Very High. The guard check placement (before cache, before proof) and no-proof-on-deny are structural. A competitor would need to rearchitect execution flow. CompositeGuard first-deny semantics and kernel-only RUNNING transition create additional design constraints.

### Non-Obviousness

**Inventive step:** Prior art governs actions, outputs, or host. Treating execution as the controlled object with mandatory gate before any execution path, and ensuring denial produces no proof/cache, is not obvious from guardrails, TEEs, or access control. The combination of execution-as-resource + structural placement + no-proof-on-deny is non-obvious.

### Prior Art (Closest References)

| Reference | Type | Jurisdiction | Description | Distinguishing Feature |
|-----------|------|--------------|-------------|------------------------|
| GuardAgent | Paper | — | Guardrail system for LLM agents; action checks | Governs actions/outputs; not execution invocation |
| AgentSpec | Paper | — | Runtime constraints on LLM agents; intercepts outputs | Output-level; no execution gate |
| AgentGuardian | Paper | — | Context-aware access for tool invocations | Tool calls; not execution as resource |
| MI9 (arXiv 2508.03858, Aug 2025) | Paper | — | Runtime governance: CAM, FSM conformance, graduated containment | Monitors/intervenes; no mandatory gate before cache; no no-proof-on-deny |
| AGENTSAFE | Paper | — | Dynamic authorization, interruptibility | Interrupts during; NEXUS denies before execution |
| US12141268B2 (NVIDIA) | Patent | USPTO | TEE for multi-processor execution | Host protection; not execution permission |
| US8646100B2 | Patent | USPTO | Restricted execution with permission | Authorization; no no-proof-on-deny |
| OAuth/OIDC | Standard | — | Scope-based API authorization | API access; not execution boundary |

### Jurisdiction-Specific Search Databases & Queries

| Jurisdiction | Database | URL | Suggested Search Queries |
|--------------|----------|-----|--------------------------|
| **United States** | USPTO Patent Public Search | https://www.uspto.gov/patents/search | "execution gate" AND "AI agent"; "mandatory" AND execution AND guard; "no proof" AND deny |
| **European Union** | Espacenet | https://worldwide.espacenet.com/ | execution governance autonomous; mandatory gate computation; execution boundary guard |
| **India** | IPO Patent Search | https://ipindiaservices.gov.in/ | execution gate autonomous; governed computation; AI execution control |
| **International** | PATENTSCOPE (WIPO) | https://patentscope.wipo.int/search/en/advancedSearch.jsf | FTXT:(execution gate) AND FTXT:(autonomous); FTXT:(mandatory) AND FTXT:(execution) |

---

## CLAIMS

**1.** A system for governed autonomous computation, comprising:

an execution engine configured to receive an execution request comprising a computation unit and an execution context;

an execution guard, when configured, operatively coupled to the execution engine, the execution guard invoked before any cache lookup for the execution request and before computation or proof generation, and configured to return an allow decision or a deny decision based on the computation unit and the execution context; and

logic configured such that upon a deny decision, the execution engine returns an error, produces no execution proof, writes no cache entry, and has no alternate code path that produces an execution artifact when the guard returns deny; and upon an allow decision, the execution engine proceeds with execution and produces an execution proof.

**2.** The system of claim 1, wherein the logic is configured such that upon a deny decision, no cache lookup is performed for the execution request.

**3.** The system of claim 1, wherein the logic is configured such that upon a deny decision, no execution proof is created.

**4.** The system of claim 1, wherein the logic is configured such that upon a deny decision, no cache entry is written.

**5.** The system of claim 1, wherein a production configuration of the execution engine structurally requires the execution guard to be configured.

**6.** The system of claim 1, wherein the execution guard comprises a composite guard comprising a plurality of guards evaluated in sequence, and wherein a first deny decision from any of the plurality of guards terminates evaluation and returns deny.

**7.** The system of claim 6, wherein allow is returned only when all of the plurality of guards return allow.

**8.** The system of claim 1, wherein the execution engine is further configured to perform cache lookup only after the allow decision is returned.

**9.** A method for governing execution of a computation unit, comprising:

receiving an execution request comprising the computation unit and an execution context;

when an execution guard is configured, invoking the execution guard to obtain a decision before performing cache lookup or computation;

if the decision is deny, returning an error and producing no execution artifact; and

if the decision is allow, proceeding with validation, cache lookup, execution, and proof generation.

**10.** The method of claim 9, wherein producing no execution artifact comprises performing no cache lookup, creating no execution proof, and writing no cache entry.

**11.** The method of claim 9, wherein the execution guard is invoked at a point in the execution flow prior to cache lookup and prior to any computation of the computation unit.

**12.** A system for kernel-controlled execution handoff, comprising:

a kernel configured to schedule processes and to transition a process from a ready state to a running state;

a process control block associated with the process, the process control block comprising a state field indicating one of ready and running; and

a crossing check invoked by the kernel before transitioning the process to the running state, the crossing check configured to return an allow result or a deny result;

wherein upon a deny result, the kernel does not set the state field to running and does not perform execution handoff; and

wherein upon an allow result, the kernel sets the state field to running and performs execution handoff.

**13.** The system of claim 12, wherein the crossing check is the sole enforcement point through which the kernel transitions a process to the running state.

**14.** The system of claim 12, wherein upon a deny result, the kernel raises an exception and returns without modifying the state field.

**15.** An apparatus comprising:

an execution guard interface configured to receive a computation unit and an execution context and to return an allow decision or a deny decision; and

a composite guard comprising a plurality of guards implementing the execution guard interface, the composite guard configured to evaluate the plurality of guards in sequence, wherein a first deny decision from any guard terminates evaluation and returns deny, and wherein allow is returned only when all guards return allow.

---

## FIGURE DESCRIPTIONS

### FIG. 1 — Block Diagram

**FIG. 1** shows caller **102**, execution request **104**, error **106**, result **108**, execution engine **110**, execution guard **120**, cache **130**, and proof generator **140**. Arrows indicate data flow: execution request **104** from caller **102** to execution engine **110**; when guard **120** is configured, guard check before cache **130** and proof generator **140**; upon deny, error **106** to caller **102**; upon allow, result **108** (with proof) to caller **102**.

### FIG. 2 — Execution Allow Path

**FIG. 2** illustrates: (1) Execution request received; (2) Guard invoked; (3) Guard returns allow; (4) Validation; (5) Cache lookup; (6) On cache miss, computation (e.g., WASM run); (7) Proof creation; (8) Cache put; (9) Return result with proof.

### FIG. 3 — Execution Deny Path

**FIG. 3** illustrates: (1) Execution request received; (2) Guard invoked; (3) Guard returns deny; (4) Return error immediately; (5) No cache lookup; (6) No computation; (7) No proof creation; (8) No cache write.

### FIG. 4 — Kernel-Controlled Handoff

**FIG. 4** illustrates: (1) Scheduler selects process; (2) context_switch invoked; (3) Crossing check invoked; (4) If deny, raise exception, no state change; (5) If allow, set state to RUNNING, update scheduling metadata, handoff complete.

---

## REFERENCE IMPLEMENTATION

| Component | File | Lines |
|-----------|------|-------|
| Guard check | nexus-executor/src/executor.rs | 147–156 |
| Guard trait | nexus-executor/src/guard.rs | 13–28 |
| CompositeGuard | nexus-executor/src/guards/composite.rs | 41–49 |
| production() | nexus-executor/src/executor.rs | 52–61 |
| context_switch | agp-core/src/os/kernel.py | 177–194 |
| ProcessState.RUNNING | agp-core/src/os/process.py | 20, 64 |
| test_no_proof_on_blocked | nexus-executor/tests/integration_tests.rs | 332–356 |

---

## INVENTOR & ASSIGNEE

| Field | Value |
|-------|-------|
| Inventor | Katta Naga Sri Ganesh |
| Assignee | SYNTRIASS Labs Private Limited |

---

*End of Patent Application*
