# NEXUS / AGP — Prior-Art & Novelty Analysis

**Role:** USPTO / EPO patent examiner  
**Standard:** Do not assume novelty; combination novelty stated explicitly; cite concrete prior art  
**Date:** February 2025  

---

## Summary Table

| # | Invention | Novelty | Design-Around | Strategic Value | Novelty Type |
|---|-----------|---------|---------------|-----------------|--------------|
| 1 | Mandatory execution gate for intelligent computation | Medium | Hard | Core | Combination |
| 2 | Execution treated as a protected logical resource | Medium | Hard | Core | Combination |
| 3 | Agents modeled as OS processes with governed execution handoff | Medium | Hard | Core | Combination |
| 4 | Commitment membrane (entropy + authority + trust) | High | Very Hard | Core | Individual + Combination |
| 5 | No-proof-on-deny execution invariant | High | Very Hard | Core | Individual |
| 6 | Developmental maturity gating of AI capabilities | Medium | Medium | Supporting | Combination |

---

## 1. Mandatory Execution Gate for Intelligent Computation

### Closest Prior Art

| Source | Type | Citation | Description |
|--------|------|----------|-------------|
| GuardAgent | Paper | OpenReview 2024 | Guardrail system for LLM agents; checks actions against safety requirements; generates executable guardrail code; 98% accuracy in healthcare access control |
| Governable AI (GAI) | Paper | arXiv 2508.20411 | Rule Enforcement Module (REM) enforcing computational boundaries via cryptographic mechanisms; externally enforced structural compliance |
| AgentSpec | Paper | arXiv 2503.18666 | Runtime constraints on LLM agents; triggers, predicates, enforcement; intercepts agent outputs |
| AgentGuardian | Paper | arXiv 2601.10440 | Context-aware access control for AI agent execution; monitors execution traces; derives policies for tool calls |
| US12141268B2 | Patent | NVIDIA | Secure execution for multiple processors using trusted execution environments |
| US8646100B2 | Patent | — | Executing applications in restricted operating environments with permission-based authorization |
| US8627414B1 | Patent | — | User-verifiable execution of security-sensitive code; hypervisor/guest control |

### (a) What Is Similar

- GuardAgent, AgentSpec, GAI: runtime checks before or during agent actions
- AgentGuardian: execution-level access control, policy enforcement
- Sandbox/restricted execution patents: authorization before execution
- TEE patents: hardware-level gating of execution

### (b) What Is Missing

- **Single choke point:** Prior art uses layered checks, interceptors, or wrappers—not one mandatory gate that all computation must pass through
- **Production-configuration structural requirement:** NEXUS’s `ExecutorBuilder::production()` structurally requires a guard; prior art typically allows opt-out
- **Unified gate for PCU + kernel:** NEXUS applies the same gate to WASM PCU execution and OS task handoff

### (c) What NEXUS Adds Structurally

- Guard invoked at execution boundary (before cache lookup, before WASM run)
- `guard.check(pcu, ctx)` as single entry point when configured
- Early return on Deny; no alternate path when guard is set

### Novelty Type

**Combination-based.** Gate concepts exist individually; novelty is in structural placement + production enforcement + unified application.

### Valuation

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Novelty | Medium | Gate concepts exist; novelty is placement + structural enforcement |
| Ease of design-around | Hard | Requires rearchitecting execution model |
| Strategic value | Core | Foundational to execution-law thesis |

---

## 2. Execution Treated as a Protected Logical Resource

### Closest Prior Art

| Source | Type | Citation | Description |
|--------|------|----------|-------------|
| AgentGuardian | Paper | arXiv 2601.10440 | Treats AI agent execution as protected resource; context-aware ABAC; monitors execution traces; regulates tool invocations |
| SAGA | Paper | arXiv 2504.21034 | Security Architecture for Governing AI; user-controlled access policies; cryptographic tokens for inter-agent delegation |
| NIST AC-3 / ISO A.5.15 | Standard | — | Logical access control; typically applied to data/assets, not execution as the object |
| OAuth 2.0 / OIDC | Standard | RFC 6749, OIDC | Scope-based authorization; applied to API access, not execution invocation |

### (a) What Is Similar

- AgentGuardian: execution as controlled resource; granular access management
- SAGA: execution-level governance; user control over agent lifecycle
- Standards: logical access control, need-to-know, least privilege

### (b) What Is Missing

- **Execution as the access-control object:** Prior art often protects data, APIs, or actions; NEXUS treats *execution invocation* as the resource
- **Execution = state transition:** NEXUS frames execution as protected state transition (READY → RUNNING); prior art focuses on action/output
- **Single authorization point for execution:** NEXUS has one gate for “can this run”; prior art has multiple policy layers
- **Integration with execution substrate:** NEXUS embeds in executor/kernel; prior art is often middleware

### (c) What NEXUS Adds Structurally

- Execution is the controlled object; guard grants/denies *execution*, not merely access to data
- `PcuExecutor::execute()` and `context_switch()` are the sole entry points
- Deny → no state change, no side effects, no proof

### Novelty Type

**Combination-based.** AgentGuardian and SAGA approach execution protection; novelty is in framing execution as *the* logical resource and structural single-point enforcement.

### Valuation

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Novelty | Medium | Execution protection exists; novelty is object-of-control framing + substrate integration |
| Ease of design-around | Hard | Requires redefining access-control target |
| Strategic value | Core | Central to execution-law thesis |

---

## 3. Agents Modeled as OS Processes with Governed Execution Handoff

### Closest Prior Art

| Source | Type | Citation | Description |
|--------|------|----------|-------------|
| SchedCP / sched-agent | Paper | Linux Journal, arXiv | LLM agents optimize Linux kernel schedulers; decoupled control plane; MCP server; agents configure policy, not processes |
| Agentic OS | Article | ownyourai.com | LLM agent framework for Linux schedulers |
| AIOS | Code | GitHub agiresearch/aios | Agent-oriented OS research |
| Unix/Linux process model | Prior art | — | PCB, states, scheduling—for human/legacy workloads |

### (a) What Is Similar

- Process abstraction: PCB, state machine, scheduling
- SchedCP: AI influences scheduling policies
- Standard kernels: only kernel sets process state (RUNNING, etc.)

### (b) What Is Missing

- **Agents as first-class processes:** Prior work uses agents to *configure* or *optimize* the OS; NEXUS models each agent as a process with its own PCB
- **Governance gate on handoff:** NEXUS requires TELOS crossing before `context_switch()` sets RUNNING; prior OS work has no governance gate on state transition
- **Execution handoff = kernel-only path:** NEXUS restricts RUNNING to `context_switch()`; no alternate handoff path in kernel

### (c) What NEXUS Adds Structurally

- `ProcessControlBlock` per agent; `spawn_process()` creates PCB; `context_switch()` is sole path to RUNNING
- `request_crossing()` invoked inside `context_switch()` before state update
- `schedule()` → `context_switch()` only; no other kernel path sets RUNNING

### Novelty Type

**Combination-based.** Process model is conventional; novelty is agent-as-process + governed handoff + single kernel path.

### Valuation

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Novelty | Medium | Process model conventional; novelty is agent-as-process + governed handoff |
| Ease of design-around | Hard | Requires governed OS kernel for agents |
| Strategic value | Core | Central to AGP OS thesis |

---

## 4. Commitment Membrane Enforcing Entropy + Authority + Trust Before Execution

### Closest Prior Art

| Source | Type | Citation | Description |
|--------|------|----------|-------------|
| IBM US12235933B2 | Patent | IBM | Trust formation for autonomous agents; reduces oversight costs |
| Zero-trust identity for agentic AI | Paper | arXiv 2505.19301 | DIDs, Verifiable Credentials for agent capabilities |
| US11025626B1, US11171783 | Patents | — | Decentralized identity; machine-to-machine auth; blockchain authorization |
| OAuth 2.0 / OIDC | Standard | RFC 6749 | Scope-based authorization |
| Entropy (cryptography) | Prior art | — | Entropy as randomness; not as consumable budget for actions |

### (a) What Is Similar

- Authority/scope: OAuth-style scopes, capability checks
- Trust: accumulation, scoring, thresholds
- Identity: verification before granting access

### (b) What Is Missing

- **Entropy as execution budget:** Prior art does not use entropy as consumable budget for irreversible actions; NEXUS ties consequence tier to entropy cost
- **Commitment membrane metaphor:** No prior art presents a membrane that must be crossed before commitment, with entropy + authority + trust as joint requirements
- **Integration with execution handoff:** TELOS called from `context_switch()` before RUNNING; prior identity/auth work is separate from OS scheduling
- **Consequence-tiered costs:** NEXUS’s LOW/MEDIUM/HIGH/CRITICAL tiers with proportional entropy cost not found in prior art

### (c) What NEXUS Adds Structurally

- `CommitmentMembrane` with `request_crossing(decision, required_scope)`
- `EntropyMeter.spend(tier)`; `AuthorityRegistry.verify(agent_id, scope)`; `TrustAccumulator.get_trust()` for HIGH+ tier
- Membrane invoked from kernel `context_switch()`; denial prevents state transition

### Novelty Type

**Individual + Combination.** Entropy-as-budget for execution authorization appears individually novel; combination with authority + trust + membrane + kernel integration strengthens claim.

### Valuation

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Novelty | High | Entropy-as-budget + membrane + execution integration uncommon |
| Ease of design-around | Very Hard | Conceptual and architectural |
| Strategic value | Core | Distinct TELOS narrative |

---

## 5. No-Proof-on-Deny Execution Invariant

### Closest Prior Art

| Source | Type | Citation | Description |
|--------|------|----------|-------------|
| Proof-of-Execution | Framework | therisk.global | Cryptographic evidence of AI actions; focuses on *allowed* execution |
| Cross-Trace Verification Protocol | Paper | arXiv 2512.13821 | Detects blocked/deviant actions by analyzing execution traces |
| Immutable audit trails | Article | quantumencoding.io | Tamper-proof records; typically records *all* events including denials |
| HashiCorp Vault blocked audit | System | HashiCorp docs | Execution denied when audit device blocked; different concern |
| NIST AU-2, AU-3, AU-9 | Standard | NIST 800-53 | Audit requirements; do not specify “no proof on deny” |

### (a) What Is Similar

- Audit trails: records of events
- Immutability: tamper-evident logs
- Access control: denial produces error/response

### (b) What Is Missing

- **Explicit no-proof-on-deny:** Prior art usually logs denials; NEXUS ensures *no execution proof or cache entry* exists for denied execution
- **Audit semantics:** “Absence of proof” as invariant—denied execution cannot produce a success artifact
- **Structural guarantee:** NEXUS enforces via control flow (early return before proof/cache); prior art does not frame this as first-class invariant
- **Retry determinism:** Same inputs → same deny; no cache write on block

### (c) What NEXUS Adds Structurally

- Guard check at `executor.rs:148`; return before `cache.get`, `generate_proof()`, `cache.put()`
- Denied execution: `Err(ExecutionBlocked)` only; no proof object, no cache key written
- Test: `test_no_proof_on_blocked_execution` verifies retry yields same deny

### Novelty Type

**Individual.** Explicit no-proof-on-deny as structural invariant and audit semantic is not clearly taught in prior art.

### Valuation

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Novelty | High | Explicit no-proof-on-deny invariant uncommon |
| Ease of design-around | Very Hard | Requires changing execution and caching model |
| Strategic value | Core | Strong for compliance narrative |

---

## 6. Developmental Maturity Gating of AI Capabilities

### Closest Prior Art

| Source | Type | Citation | Description |
|--------|------|----------|-------------|
| AGI Capability Maturity Model | Paper | philarchive | Trajectories from thresholds; developmental psychology themes |
| OECD AI Capability Indicators | Report | OECD 2025 | Measures AI vs. human skills; progression levels |
| GSA AI Capability Maturity Model | Framework | coe.gsa.gov | Organizational maturity for AI adoption |
| Tufts developmental AI | Paper | Adams et al., AI Magazine | Developmental milestones for AI; not execution gating |
| US11171983B2 | Patent | Intel | Function-level isolation via capability-based security; not developmental stages |

### (a) What Is Similar

- Maturity models: staged progression (Infant/Child/Adult-like)
- Capability gating: restricting actions by capability
- Developmental analogy: human development as reference

### (b) What Is Missing

- **Execution gating by stage:** Prior work is assessment/framework; NEXUS *blocks execution* when stage < required for capability
- **Integration with execution guard:** NEXUS wires developmental stage into NervousSystemGuard; default Infant blocks Execute
- **Runtime enforcement:** NEXUS enforces at execution time; maturity models typically used for assessment, not runtime blocking

### (c) What NEXUS Adds Structurally

- `DevelopmentalStage` enum (Infant→Elder); `Capability::Execute` requires Adult
- `NervousSystemGuard` → `NervousSystemCoordinator` → `DecisionEngine.check_capability()`
- `Blocked { reason }` when stage insufficient; no execution path when guard denies

### Novelty Type

**Combination-based.** Developmental models exist; novelty is execution gating + integration with execution guard.

### Valuation

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Novelty | Medium | Developmental models exist; novelty is execution gating |
| Ease of design-around | Medium | Could implement stage checks without full NEXUS stack |
| Strategic value | Supporting | Strengthens Tier-1; dependent on execution gate |

---

## Combination Novelty Summary

| Combination | Components | Prior-Art Gap |
|-------------|------------|---------------|
| Execution-law core | (1) + (2) + (5) | No single reference teaches execution-as-resource + mandatory gate + no-proof-on-deny |
| AGP OS stack | (3) + (4) | No reference teaches agents-as-processes + TELOS membrane in kernel handoff |
| TELOS membrane | (4) alone | Entropy-as-budget + authority + trust + membrane is individually novel |
| Developmental layer | (6) + (1) | Maturity models exist; execution gating integrated with guard is novel |

**Examiner note:** Strongest claims are combination claims. Individual components (4, 5) have standalone novelty; others are strongest when combined with execution-law substrate.

---

## Examiner Recommendations

| Invention | Claim as Independent? | Claim as Dependent? | Emphasize |
|-----------|------------------------|---------------------|-----------|
| 1. Mandatory execution gate | Narrow (with structural enforcement) | Yes, on execution-as-resource | Combination |
| 2. Execution as protected resource | Narrow | Yes | Combination |
| 3. Agents as OS processes with governed handoff | Narrow | Yes | Combination |
| 4. Commitment membrane | Yes (broad) | Yes | Individual + combination |
| 5. No-proof-on-deny | Yes (broad) | Yes | Individual |
| 6. Developmental maturity gating | No | Yes (on execution gate) | Combination |

**Critical:** Avoid “mandatory” without “when configured” or “in production build.” Avoid “cryptographic non-existence proof.” Use: *“Execution is denied and no execution artifact is produced.”*
