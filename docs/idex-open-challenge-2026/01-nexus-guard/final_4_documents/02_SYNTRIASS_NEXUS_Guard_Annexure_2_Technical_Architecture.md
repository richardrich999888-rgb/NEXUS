# IDEX OPEN CHALLENGE SUBMISSION

# Annexure-2

Technical architecture and implementation approach

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

# Technical Architecture and Approach

## Company Identification Details

| Field | Details |
| --- | --- |
| Legal Entity Name | Syntriass Labs Private Limited |
| CIN | U62011AP2025PTC120239 |
| PAN | ABQCS7152R |
| TAN | VPNS31351F |
| Registered Office | 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India |
| Contact Email | kattanaga5555@gmail.com |
| Contact Phone | +91 88864 68060 |
| Submission Date | 17 May 2026 |

## 1. Technical Architecture and Approach

### 1.1 System Objective

NEXUS Guard provides governed execution control for autonomous defence software. It is designed to operate between autonomous clients, agents, robot-control software, mission tools, APIs, databases, MCP servers, and edge services. The architecture follows the principle:

```text
Identity + Context -> Guard Evaluation -> Consequence Budget -> Execution or Denial -> Audit Evidence
```

The system does not replace mission software. It provides a control and evidence layer that ensures high-consequence autonomous actions are policy-governed before execution, denied when unauthorized, and recorded when allowed.

### 1.2 Intended Defence End-User Environment

The intended users are Indian defence organizations that operate, evaluate, integrate, or assure autonomous and AI-enabled mission systems. This includes Indian Armed Forces units using unmanned systems, ISR workflows, autonomous logistics tools, cyber-physical platforms, and command-support software; DRDO laboratories and technical evaluators conducting validation; command-and-control assurance teams responsible for accountability; defence cyber and AI security teams responsible for tool-governance boundaries; and defence PSUs or system integrators deploying autonomous mission software.

The system is designed for environments where autonomous software may invoke tools, APIs, databases, sensors, or command-support functions and where the evaluator must know whether the action was authorized, policy-compliant, denied before consequence, and auditable after the mission.

### 1.2A Official Problem Pull and Scope Boundary

NEXUS Guard is positioned as an Open Challenge proposal because it cuts across several official autonomy problem areas rather than fitting only one named challenge. The strongest demand signals are ADITI 4 autonomy classes and DISC 14 autonomy/UAS classes where autonomous software can invoke protected action paths.

| Alignment Area | Defence Pull | NEXUS Guard Scope | Explicit Non-Duplication Boundary |
| --- | --- | --- | --- |
| ADITI 4 autonomy problem areas | Autonomous armoured vehicles, robot-team systems, and autonomous logistics require execution assurance before command paths are invoked. | Provide a reusable guard layer for protected software actions, command adapters, and evidence export. | Does not propose a space-domain training, surveillance, or operations tool under PS24. |
| DISC 14 UAS / multi-agent autonomy problem areas | AI-enabled UAS modules and robot-team autonomy need policy control around high-impact tool or mission calls. | Wrap protected actions with first-deny-wins guard checks and denial-path evidence. | Does not reuse PS24 datasets, operational scenarios, or space-domain deliverables. |
| Open Challenge route | The problem is a cross-cutting assurance gap rather than a single platform procurement item. | Deliver a 12-month software prototype, red-team denial harness, PQC-enabled audit profile, audit exporter, and evaluator runbook. | Current evidence remains software-subsystem TRL 3-4; TRL 5 is an exit target after relevant-environment validation. |

### 1.3 Core Architecture

```text
Autonomous Agent / Robot Controller / Mission Client
        |
        v
Protected Execution Request
        |
        v
NEXUS Guard Context Builder
        |
        v
ExecutionGuard Chain
        |
        +--> DENY -> Block Execution -> Denial Evidence -> Audit Store
        |
        v
TELOS Consequence Budget
        |
        v
Allowed Execution
        |
        v
ETK Audit Evidence + Replay Package
```

### 1.4 ExecutionGuard Architecture

The ExecutionGuard layer is the pre-execution enforcement point. Each protected command becomes a proof-carrying execution request with identity, context, action type, authority scope, mission policy, and consequence class.

The guard model supports layered checks:

| Guard Layer | Purpose |
| --- | --- |
| Authority Guard | Verifies whether the requesting agent has permission for the action. |
| Mission Policy Guard | Checks whether the action fits the current mission context and policy bundle. |
| Immune/Compromise Guard | Blocks agents or requests marked as suspicious or compromised. |
| Reputation Guard | Applies trust-state and prior-behavior constraints. |
| Consequence Guard | Routes high-impact actions to TELOS consequence budget checks. |

The core execution rule is first-deny-wins. If any guard denies, the action is blocked before execution.

### 1.5 Denial and No-Proof Flow

NEXUS Guard separates denial evidence from success evidence. A denied action is recorded as an attempted execution with reason codes, but it does not generate a success proof or execution-cache artifact.

Execution denial flow:

```text
1. Agent submits protected action.
2. NEXUS Guard validates request structure and context.
3. ExecutionGuard chain evaluates authority and policy.
4. Any Deny result stops execution.
5. Denial reason is recorded.
6. No protected action is forwarded.
7. No success proof or success cache is generated.
8. Denial evidence is exported for review.
```

### 1.6 TELOS Consequence-Budget Flow

TELOS adds consequence accounting to execution control. Rather than treating every action as equal, actions are classified by consequence tier. High-consequence actions consume more budget and can require stricter policy or human attestation.

| Tier | Defence Meaning | Example |
| --- | --- | --- |
| Trivial | Low-risk system observation | Read telemetry |
| Low | Routine movement or benign tool action | Navigation adjustment |
| Medium | Mission parameter change | Update route or sensor tasking |
| High | High-impact action requiring strict control | Engage non-lethal system or restricted operation |
| Critical | Highest consequence class | Requires explicit human/command attestation before any operational claim |

### 1.7 ETK Audit Evidence Flow

For allowed actions, ETK-compatible records capture the execution request, policy decision, context hash, result hash, timestamp, and event linkage. This provides a tamper-evident evidence chain for after-action review.

ETK record fields:

| Field | Purpose |
| --- | --- |
| Previous Hash | Links the event to prior execution state. |
| Current Hash | Commits to the canonical execution record. |
| Request Hash | Proves the request payload was not modified. |
| Result Hash | Proves the execution result was not modified. |
| Guard Result | Records allow, deny, or escalate decision. |
| Reason Code | Explains which policy or guard caused the result. |
| Timestamp | Provides temporal ordering. |
| Signature / Proof Metadata | Supports verification and audit. |

### 1.7A PQC Identity and Audit Hardening

NEXUS Guard will include a PQC transition profile based on the repository's `nexus-pcu` hybrid-signature path. The current production path uses classical Ed25519. The proposed hardening work enables hybrid Ed25519 plus ML-DSA signing for selected identity and audit records, verifies both paths in CI, and records signature metadata in the evidence packet.

| PQC Work Item | Implementation Target |
| --- | --- |
| Hybrid key bundle | Use `nexus-pcu::PublicKeyBundle` for classical and PQC verification metadata. |
| Hybrid audit signature | Sign selected execution/audit records through `nexus-pcu::HybridSignature`. |
| Verification mode | Verify classical-only and hybrid signatures, including tampered-classical fallback tests. |
| TRL 5 evidence | Include PQC test output and evaluator replay in the final evidence package. |

### 1.8 Runtime Governance Pipeline

```text
Request Intake
   -> Schema Validation
   -> Agent Identity and Context Binding
   -> Nonce / Replay Check
   -> Authority Scope Check
   -> Mission Policy Evaluation
   -> Risk and Consequence Classification
   -> Guard Decision
   -> Allow / Deny / Escalate
   -> ETK Audit or Denial Evidence
   -> Replay Store
```

The pipeline is deterministic and designed for auditability. Policy results are recorded as part of the execution chain and can be inspected during after-action review.

### 1.9 Critical Defence Problem-to-Control Mapping

| Defence Problem | Consequence If Uncontrolled | System Control |
| --- | --- | --- |
| Unauthorized autonomous execution | A malicious, misconfigured, or over-permissioned agent can trigger restricted tools or mission actions. | ExecutionGuard blocks protected actions before operational consequence. |
| Tool escalation | An agent with limited authority can attempt a higher-risk API, shell, database, or external connector action. | Guard policy validates allowed tools, denied tools, action class, and authority scope. |
| High-consequence action without approval | An autonomous system may attempt an action requiring command or supervisor approval. | TELOS consequence tiers route high-impact actions to stricter policy or explicit attestation. |
| Denial-path ambiguity | Reviewers may not know whether an unsafe action was blocked or simply did not occur. | Denial evidence records attempted action and reason code while avoiding success proof artifacts. |
| Mission log tampering | Event records can be edited, deleted, reordered, or selectively presented. | ETK evidence uses hash-linked records and verification metadata. |
| Contested verification | Cloud-dependent verification may fail in forward-deployed, classified, or air-gapped environments. | Local policy bundles and portable evidence packets support offline review. |
| Policy bypass through encoded requests | Dangerous actions can be hidden inside tool arguments, prompt-injected text, or malformed payloads. | Schema validation, argument inspection, and guard policy checks occur before execution. |

```{=typst}
#pagebreak()
```

## 2. Innovation

NEXUS Guard introduces a governed execution kernel for autonomous systems. The innovation is not limited to logging or dashboards. It combines mandatory pre-execution checks, layered first-deny-wins guard evaluation, consequence budgeting, and audit evidence into one deployable architecture.

| Innovation Area | Description |
| --- | --- |
| Runtime-Native Governance | Policy is enforced before execution, not only observed after the event. |
| First-Deny-Wins Guard Chain | Multiple guard layers can protect one action; any denial blocks execution. |
| Denial Evidence Without Success Proof | Denied attempts are reviewable without creating artifacts that imply successful execution. |
| Consequence-Budgeted Autonomy | TELOS limits repeated or high-impact action attempts under configured policy. |
| Replay-Ready Audit Evidence | ETK-compatible records support after-action reconstruction and independent review. |
| Sovereign Deployment Path | The prototype is designed for local, on-premise, and air-gapped evaluation. |

## 3. Implementation and Feasibility

### 3.1 Modular Deployment

| Module | Function |
| --- | --- |
| Guard Runtime | Receives protected requests and runs layered guard checks. |
| Context Builder | Binds request, identity, mission context, and policy metadata. |
| Policy Engine | Applies deterministic authority, action, risk, and mission constraints. |
| TELOS Adapter | Assigns consequence tier and updates consequence budget. |
| ETK Exporter | Produces audit records and verification artifacts. |
| Dashboard APIs | Exposes blocked actions, reason codes, and execution evidence. |

### 3.2 Edge Compatibility

The runtime is designed to operate close to the execution environment. Critical checks use deterministic policy evaluation and compact evidence records. This supports lab, on-premise, edge-node, or air-gapped evaluation environments.

### 3.3 Air-Gap Deployment

The architecture does not require external cloud access for core policy decisions. Policy bundles and public verification material can be provisioned locally. Audit and denial evidence can be exported as portable review packets.

### 3.4 API-First Architecture

The system can expose REST, JSON-RPC, MCP-compatible, and tool-router integration interfaces. Existing autonomous clients can route protected execution through NEXUS Guard without replacing the underlying mission software.

### 3.5 Scalability

Guard evaluation can be scaled horizontally where the policy state and audit store are managed through durable services. Nonce tracking, replay checks, and consequence budgets can be maintained through local stores appropriate to the deployment environment.

## 4. Challenges and Mitigation

| Challenge | Risk | Mitigation Strategy |
| --- | --- | --- |
| Runtime Latency | Inline enforcement could add delay to time-sensitive workflows. | Keep policy evaluation deterministic, benchmark mission-specific latency, and run guard services close to execution. |
| Integration Bypass | A tool or agent may attempt to call protected actions directly. | Define protected interfaces and require command routing through the guard adapter. |
| Compromised Agent | A registered agent may attempt unauthorized tool use or privilege escalation. | Enforce least-privilege scopes, revocation, reputation signals, nonce checks, and policy denial. |
| Policy Bypass Attempts | Dangerous commands may be hidden inside arguments or malformed payloads. | Apply schema validation, pattern controls, tool-category restrictions, and raw argument inspection. |
| Replay Attacks | Previously valid requests may be resubmitted. | Use nonce protection, session-bound context, timestamp checks, and replay-store validation. |
| Evidence Integrity | An adversary may attempt to fabricate or reorder audit records. | Recompute hashes, validate event linkage, and verify signatures/proof metadata. |
| Hardware Readiness | Current evidence is software-subsystem level. | Conduct hardware-in-loop validation as a later iDEX work package before higher readiness claims. |

```{=typst}
#pagebreak()
```

## 5. Visual Architecture Diagrams

### 5.1 Runtime Guard Architecture

```text
AI Agent / Autonomous Client
        |
        | protected execution request
        v
NEXUS Guard Context Builder
        |
        | identity + authority + policy + mission context
        v
ExecutionGuard Chain
        |
        +-- DENY -> Block Action -> Denial Evidence
        |
        +-- ALLOW -> TELOS Consequence Check
                         |
                         v
                    Execute Tool / Mission API
                         |
                         v
                    ETK Audit Evidence
```

### 5.2 Denial Path

```text
Unauthorized Request
   -> Guard Evaluation
   -> Deny Reason
   -> Action Not Forwarded
   -> No Success Proof
   -> No Success Cache
   -> Denial Evidence Export
```

### 5.3 Audit and Replay Flow

```text
Allowed Execution Record
   -> Request Hash
   -> Guard Decision
   -> Result Hash
   -> Previous Event Hash
   -> Current Event Hash
   -> Signature / Proof Metadata
  -> Replay Verification Result
```

```{=typst}
#pagebreak()
```

## 6. Defence Use Cases

### 6.1 Autonomous ISR Tool Control

ISR systems may trigger automated tasking, analysis, or data access. NEXUS Guard can block restricted actions before execution and record allowed actions for review.

### 6.2 Robotic Command Admission

Robotic controllers may receive commands from AI planners. NEXUS Guard can require protected motion, actuator, or mission commands to pass policy and consequence checks.

### 6.3 Command-Support Agent Governance

Command-support agents may call databases, APIs, or planning tools. NEXUS Guard can enforce role, mission, and action-level constraints.

### 6.4 Red-Team Policy Bypass Testing

Evaluators can inject unauthorized commands, malformed payloads, replay attempts, or restricted tool calls to verify denial behavior.

### 6.5 After-Action Review

For a mission anomaly or policy violation, ETK audit evidence can help reconstruct requested actions, guard decisions, denial reasons, and allowed execution results.

### 6.6 Air-Gapped Evaluation

Forward-deployed or classified environments can review local evidence packets without depending on foreign cloud services.
