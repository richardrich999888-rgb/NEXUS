# IDEX OPEN CHALLENGE SUBMISSION

# Annexure Outline

Company identification and section outline

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

## Company Identification

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

## Annexure-1 Outline

Purpose: Applicant details and proposed solution summary for the iDEX Open Challenge.

Contents:

- Company and applicant details.
- Challenge title.
- Intended defence end-user profile.
- Brief solution summary under 250 words.
- Key technologies used.
- Deliverables table.
- Phase-wise 12-month timeline.

## Annexure-2 Outline

Purpose: Technical architecture and implementation approach.

Contents:

- NEXUS Guard execution-governance architecture.
- ExecutionGuard first-deny-wins flow.
- TELOS consequence-budget flow.
- ETK audit and denial-evidence flow.
- Innovation and feasibility.
- Challenges and mitigation.
- Architecture diagrams and defence use cases.

## Annexure-3 Outline

Purpose: Advantages, product value, commercial value, and competencies.

Contents:

- Technology advantages.
- Product advantages.
- Commercial advantages.
- Capabilities and competencies.

## Annexure-4 Outline

Purpose: Supporting document structure and evidence checklist.

Contents:

- Founder resume format.
- Prototype evidence list.
- Architecture screenshots list.
- Demo evidence list.
- Technical validation material.
- Future deployment scope.

```{=typst}
#pagebreak()
```

# Annexure-1

Application and proposed solution summary

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

# Applicant Details and Proposed Solution Summary

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

## 1. Applicant Details

| Field | Details |
| --- | --- |
| Applicant Startup Name | Syntriass Labs Private Limited |
| Technology / Platform Name | NEXUS Guard |
| Intended Defence End User | Indian Armed Forces, DRDO laboratories, defence AI/autonomy program offices, command-and-control assurance teams, unmanned systems operators, ISR workflow teams, and defence system integrators evaluating autonomous mission software. |
| Applicant Name | K. Naga Sri Ganesh |
| Contact Email | kattanaga5555@gmail.com |
| Contact Number | +91 88864 68060 |
| Registered Office Address | 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India |
| CIN / Incorporation Number | U62011AP2025PTC120239 |
| PAN | ABQCS7152R |
| TAN | VPNS31351F |
| DPIIT, Certificate No. | DIPP215355 |
| Proposed Project Duration | 12 months |
| Submission Date | 17 May 2026 |

## 2. Final Challenge Title

NEXUS Guard: Governed Execution Layer for Unauthorized Autonomous Action Prevention

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Indian Army, Navy, and Air Force autonomous systems units | Pre-execution control for unmanned, ISR, logistics, command-support, and AI-enabled mission workflows. |
| DRDO laboratories and defence R&D evaluators | Technical assurance layer for testing autonomous agents and verifying execution-denial behavior under controlled evaluation. |
| Command-and-control assurance teams | Evidence showing which autonomous system attempted an action, which policy gate was applied, and whether the action was approved, denied, or escalated. |
| ISR and intelligence workflow operators | Controlled execution of automated tasking, sensor-analysis chains, data access, and downstream autonomous tool invocation. |
| Defence cyber and AI security teams | Prevention of unauthorized tool escalation, policy bypass attempts, replay attempts, and unaccountable non-human execution paths. |
| Defence PSUs and mission system integrators | Deployable guard layer that can wrap existing agents, APIs, tools, and edge services without replacing mission software. |

### 3.1 Official Defence Problem Alignment and Non-Duplication

| Source / Route | Relevant Defence Demand Signal | Why NEXUS Guard Fits | Boundary From Existing ADITI PS24 Submission |
| --- | --- | --- | --- |
| iDEX Open Challenge | Open-category route for defence prototypes where the applicant defines the problem and solution. | NEXUS Guard defines a narrow governed-execution problem: preventing unauthorized high-consequence software action before execution. | Submitted as a separate execution-governance product, not as a space-domain training or surveillance tool. |
| ADITI 4 autonomy problem areas | Autonomous armoured vehicle, robotic combat-support, autonomous logistics, and AI-enabled autonomy classes require stronger execution assurance. | NEXUS Guard provides a reusable policy gate for autonomous command paths, robot-command adapters, mission software, and tool-routing systems. | The existing PS24 application is space-domain focused; this proposal addresses pre-execution authorization across autonomous systems. |
| DISC 14 autonomy/UAS problem areas | UAS recognition/targeting support, robot-team autonomy, and multi-agent UAS modules need guardrails before protected actions are invoked. | The proposal adds first-deny-wins execution control, denial evidence, and ETK-compatible audit records around protected action calls. | It does not duplicate PS24 deliverables, datasets, or mission-training objectives. |

## 4. A. Brief Summary of Proposed Solution

Autonomous defence systems can move from sensing to action faster than conventional command review, logging, or after-action audit can control. The core assurance problem is whether any high-consequence autonomous command can execute without authorization, mission policy approval, and reviewable evidence.

NEXUS Guard is a governed execution layer for autonomous software agents, robotic control stacks, and mission decision services. It places a mandatory ExecutionGuard in front of protected actions. A command that does not satisfy authorization, policy, context, and consequence constraints is denied before execution. Denied commands do not generate success proofs or execution-cache artifacts. Approved commands generate ETK-compatible audit evidence and TELOS consequence accounting so reviewers can reconstruct why execution was allowed.

The proposed 12-month prototype will demonstrate a red-team scenario where unauthorized autonomous execution is blocked, no proof/cache is generated for the denied path, and an authorized action produces a verifiable execution record. Current evidence is software subsystem TRL 3-4. The iDEX exit target is TRL 5 after PQC-enabled identity/audit integration, relevant-environment simulation, latency characterization, and evaluator-witnessed hardware-in-loop or mission-software-in-loop validation.

```{=typst}
#pagebreak()
```

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Unauthorized high-consequence execution | Autonomous agents, tools, or mission software may attempt restricted actions before a human or command authority can intervene. | NEXUS Guard intercepts protected execution requests and applies first-deny-wins guard evaluation before connected tools or mission actions are invoked. |
| Policy bypass through tool/API calls | AI agents may route dangerous actions through APIs, tool arguments, command paths, or encoded requests. | ExecutionGuard evaluates request structure, action type, policy context, and restricted command classes before allowing execution. |
| No reliable denial evidence | Many systems log only completed activity, leaving blocked or attempted actions poorly documented. | Denied actions produce denial evidence while avoiding success proof/cache artifacts. |
| High-consequence action without consequence budget | Autonomous systems may repeatedly attempt high-impact actions if no consequence limiter exists. | TELOS consequence budgeting limits repeated high-impact requests under configured policy. |
| Mutable mission logs | Conventional logs can be edited, deleted, reordered, or challenged during after-action review. | ETK-compatible audit records capture hash-linked execution evidence for allowed actions and denial traces. |
| Delegated authority ambiguity | Multi-agent workflows may obscure whether the acting agent had authority for a specific command. | Guard context records identity, authority scope, policy result, and reason codes for review. |
| Contested or disconnected verification | Forward or air-gapped environments may not have continuous cloud validation. | The prototype supports local policy bundles and portable audit evidence for offline review. |

## 6. B. Key Technologies Used

- ExecutionGuard runtime governance
- TELOS consequence budgeting
- ETK audit evidence
- Hybrid Ed25519 plus ML-DSA PQC signing path
- Proof-carrying unit patterns
- Rust policy enforcement
- Autonomous systems security

```{=typst}
#pagebreak()
```

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| NEXUS Guard Runtime | Inline runtime layer that intercepts autonomous execution requests before protected tool, API, or mission-system action. |
| ExecutionGuard Policy Engine | Deterministic policy engine for authority checks, restricted actions, risk thresholds, command boundaries, and mission constraints. |
| Denial-Path Evidence Module | Demonstrates that unauthorized actions are denied before execution and do not produce success proof/cache artifacts. |
| ETK Audit Exporter | Produces tamper-evident execution records for allowed actions and structured denial evidence for blocked attempts. |
| PQC Identity and Audit Profile | Integrates the `nexus-pcu` hybrid Ed25519 plus ML-DSA path for signing execution evidence and validating post-quantum transition readiness. |
| TELOS Consequence Trace | Tracks consequence budgets for repeated or high-impact autonomous requests. |
| Red-Team Demo Harness | Simulated mission workflow showing unauthorized action denial, allowed action execution, and audit replay. |
| Operational Dashboard Prototype | Monitoring interface for live execution status, blocked actions, policy decisions, and audit history. |
| Deployment Artifacts | Docker and local deployment assets for lab, on-premise, and sovereign infrastructure evaluation. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Architecture Hardening and Requirements Alignment | Finalized defence use cases, threat model, protected-action schema, policy assumptions, and API contract. |
| Phase 2 | Month 3 to Month 4 | ExecutionGuard Runtime Packaging | Guard runtime, context model, authority inputs, reason-code model, and policy rule templates. |
| Phase 3 | Month 5 to Month 6 | Denial-Path and Policy Enforcement | Inline enforcement, unauthorized-action blocking, denial evidence, and no-proof/no-cache validation. |
| Phase 4 | Month 7 to Month 8 | ETK Audit and TELOS Integration | Audit evidence export, consequence trace, hash-linked execution record, and replay-ready evidence packet. |
| Phase 5 | Month 9 | Sovereign Deployment Testing | Local deployment packaging, offline evidence review workflow, and secure configuration documentation. |
| Phase 6 | Month 10 | Field Simulation and Red-Team Validation | Simulated autonomous mission workflow, policy-bypass tests, denial checks, and latency measurement. |
| Phase 7 | Month 11 | Pilot Integration Preparation | API hardening, dashboard refinement, operational runbook, evaluator documentation, and training material. |
| Phase 8 | Month 12 | Pilot Demonstration and TRL 5 Exit Evidence | Demonstration package, PQC verification report, relevant-environment validation report, technical handover, deployment guide, and iDEX evaluation evidence set. |

## 9. F. TRL 5 Exit Criteria

| Exit Criterion | Evidence Required Before TRL 5 Claim |
| --- | --- |
| Relevant environment | Guarded execution tested against evaluator-approved mission software, robot-command simulator, or hardware-in-loop testbed. |
| PQC-enabled evidence path | `nexus-pcu` hybrid Ed25519 plus ML-DSA signing path enabled for audit or identity records and verified in CI. |
| Red-team denial validation | Unauthorized action, replay, malformed payload, and policy-bypass attempts blocked before execution. |
| Performance characterization | Latency and overhead measured under representative request volume and documented with acceptance thresholds. |
| Operator review package | Evidence bundle includes request, policy result, denial/allow result, signature metadata, and replay instructions. |
