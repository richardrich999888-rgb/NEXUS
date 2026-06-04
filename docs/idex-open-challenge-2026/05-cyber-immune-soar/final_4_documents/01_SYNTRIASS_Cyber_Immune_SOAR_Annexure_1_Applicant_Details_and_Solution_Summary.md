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

Purpose: applicant details and proposed solution summary for the iDEX Open Challenge.

Contents:

- Company and applicant details.
- Challenge title.
- Intended defence end-user profile.
- Brief solution summary under 250 words.
- Key technologies used.
- Deliverables table.
- Phase-wise 12-month timeline.

## Annexure-2 Outline

Purpose: technical architecture and implementation approach for Cyber Immune SOAR.

Contents:

- Cyber event to immune signal flow.
- Governance bridge response policy.
- Quarantine, throttle, escalation, and audit actions.
- Multi-agent defection and trust-reduction handling.
- Test evidence and software readiness caveats.

## Annexure-3 Outline

Purpose: advantages, product value, commercial value, and competencies.

## Annexure-4 Outline

Purpose: supporting evidence, screenshots, test output, repository locations, artifact locations, and readiness caveats.

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
| Technology / Platform Name | Cyber Immune SOAR |
| Intended Defence End User | Indian Armed Forces cyber defence teams, DRDO cyber/AI assurance evaluators, SOC teams supporting defence networks, autonomous-system security teams, and command technology units evaluating governed response automation. |
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

Cyber Immune SOAR: Policy-Bounded Cyber Defence Response With Immune Threat Memory, Quarantine, and Audit

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Defence SOC teams | Reduce response delay from alert overload while keeping containment actions policy-bounded. |
| Cyber defence units | Convert suspicious service, endpoint, and agent behavior into scored response decisions. |
| Autonomous-system security teams | Detect compromised software agents or services before they influence wider mission systems. |
| DRDO cyber/AI assurance evaluators | Test immune-style threat memory, governance bridge response mapping, and auditability. |
| Command technology units | Review what action was taken, why it was taken, and what evidence supported the response. |
| System integrators | Add governed cyber containment to simulation environments before live network integration. |

### 3.1 Official Defence Problem Alignment and Non-Duplication

| Source / Route | Relevant Defence Demand Signal | Why Cyber Immune SOAR Fits | Boundary From Existing ADITI PS24 Submission |
| --- | --- | --- | --- |
| iDEX Open Challenge | Open-category route for prototype solutions to urgent defence gaps not fully captured by one named problem statement. | Cyber Immune SOAR defines a bounded cyber-response problem: converting events into governed observe, throttle, block, quarantine, or escalation decisions. | Submitted as a separate cyber-defence response product, not as a space-domain training or surveillance tool. |
| DISC 14 cyber problem areas | Network monitoring, secure information exchange, and cyber deception problem areas indicate demand for autonomous but bounded cyber defence workflows. | The proposal contributes immune threat memory, multi-agent defection handling, trust reduction, containment classes, and audit evidence. | The existing PS24 application is space-domain focused; this proposal targets cyber event response and containment simulation. |
| ADITI 4 EW / OSINT-adjacent problem areas | Cognitive EW and AI-assisted intelligence workflows require anomaly detection, event triage, and policy-limited response. | Cyber Immune SOAR can ingest simulated events and route them through a governance bridge before any containment action is claimed. | It does not duplicate PS24 deliverables, datasets, or space-domain operational training objectives. |

```{=typst}
#pagebreak()
```

## 4. A. Brief Summary of Proposed Solution

Defence cyber teams face alert overload, expanding attack surfaces, and response delays across endpoints, services, robotic agents, and mission software. Conventional SOAR tools automate playbooks, but they often lack behavior-aware trust scoring, immune-style threat memory, governed response constraints, and reviewable quarantine evidence.

Cyber Immune SOAR proposes a policy-bounded cyber defence prototype that converts simulated cyber events into immune threat signals, assigns severity and confidence, applies governance rules, triggers observe, throttle, block, quarantine, or escalation actions, and records the event-to-action trail for review. The system is designed as controlled cyber response infrastructure with explicit policy limits and human-escalation thresholds.

The first iDEX demo will use simulated cyber events such as suspicious process behavior, anomalous service activity, policy violation, compromised agent indicators, and multi-agent collusion signals. These events will be converted into immune signals. The governance bridge will decide whether to observe, throttle, block, quarantine, or escalate. The prototype will show reputation impact, trust reduction, containment action, and audit evidence for each response.

Current evidence is software-subsystem TRL 3-4. The iDEX exit target is TRL 5 after PQC-enabled event/audit signing, relevant-environment SOC simulation, evaluator-provided event replay, and bounded containment validation. Live SOC integration, real endpoint enforcement, classified network accreditation, and operational cyber response authority are not claimed and are proposed as later validation stages.

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Alert overload | Human analysts cannot inspect every event at machine speed. | Immune signal engine converts events into severity and confidence. |
| Unsafe automation | Automated playbooks can disrupt mission services if unconstrained. | Governance bridge maps actions to monitor, throttle, block, quarantine, or escalation. |
| Compromised agent/service | A malicious or compromised software component can move laterally or mislead systems. | ThreatSignal and DefectionSignal pathways trigger response and trust reduction. |
| Collusion or coordinated activity | Multiple compromised nodes may act together. | Multi-agent defection signal returns multi_quarantine and trust penalties. |
| Weak after-action review | Cyber actions need reason codes and reproducible evidence. | Event-to-action audit trail and screenshot-backed test evidence. |
| Poor threat memory | One-off detection does not improve future response. | Unified immune system stores known threat vectors and performs similarity checks. |

```{=typst}
#pagebreak()
```

## 6. B. Key Technologies Used

- Immune threat signalling
- Governance bridge response policy
- Multi-agent defection detection
- Trust and reputation scoring
- Hybrid Ed25519 plus ML-DSA PQC event/audit signing path
- Quarantine and throttle simulation
- Audit trail and replay evidence

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| Cyber Event Simulator | Generates controlled cyber event streams for process, service, agent, and policy-violation scenarios. |
| Immune Threat Signal Engine | Converts events into ThreatSignal and DefectionSignal records with severity and confidence. |
| Governed Response Engine | Selects observe, throttle, block, quarantine, or escalation under bounded policy. |
| Multi-Agent Defection Scenario | Demonstrates collusion detection, trust reduction, and multi_quarantine behavior in simulation. |
| PQC Event and Audit Signing Profile | Integrates the `nexus-pcu` hybrid Ed25519 plus ML-DSA path for event-to-action audit packets and identity transition testing. |
| Audit and Replay Bundle | Records event, signal, governance decision, action, result, and reviewer reason code. |
| Dashboard Prototype | Shows active threats, threat breakdown, quarantine state, trust changes, and response history. |
| Test and Validation Report | Provides executed test commands, results, artifact paths, caveats, and hardening roadmap. |
| Integration Adapter Plan | Defines future adapters for SIEM/SOAR feeds, endpoint control, and defence evaluator datasets. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Defence cyber threat model | Event classes, severity policy, response limits, and evaluator scenarios. |
| Phase 2 | Month 3 to Month 4 | Event ingestion and simulator | Controlled event generator and parser for cyber scenario feeds. |
| Phase 3 | Month 5 to Month 6 | Immune signal conversion | ThreatSignal, DefectionSignal, confidence scoring, and known-threat memory. |
| Phase 4 | Month 7 to Month 8 | Governance response engine | Observe, throttle, block, quarantine, escalation, trust reduction, and audit records. |
| Phase 5 | Month 9 | Dashboard and replay | Reviewer UI showing event-to-action chain and replayable scenarios. |
| Phase 6 | Month 10 | Adversarial simulation | False-positive, evasion, collusion, service-disruption, and recovery scenarios. |
| Phase 7 | Month 11 | Evaluator packaging | Runbooks, sample datasets, source maps, artifact maps, and validation report. |
| Phase 8 | Month 12 | Final demonstration and TRL 5 exit evidence | iDEX demo package, PQC verification report, relevant-environment SOC simulation report, test evidence, and operational hardening roadmap. |

## 9. E. Readiness Position

Fresh evidence includes the governance-immune bridge script reporting 19/19 checks passed, immune pytest suites reporting 54 passed, and a multi-agent governance simulation completing successfully.

Submission boundary: software-subsystem prototype only. TRL 5 will be claimed only after relevant-environment SOC simulation, PQC-signed audit packet validation, evaluator replay, and containment safety checks are completed. Live SOC integration, endpoint quarantine, classified network deployment, regulatory cyber response authority, and production accreditation remain proposed iDEX work packages.

## 10. F. TRL 5 Exit Criteria

| Exit Criterion | Evidence Required Before TRL 5 Claim |
| --- | --- |
| Relevant environment | Cyber events replayed through an evaluator-approved SOC/SIEM simulation, range dataset, or controlled lab event source. |
| PQC-enabled audit path | Hybrid Ed25519 plus ML-DSA event/audit signature bundle generated and verified through `nexus-pcu` CI. |
| Response safety | Observe, throttle, block, quarantine, and escalate actions validated with bounded policy and human-escalation thresholds. |
| Adversarial validation | False-positive, evasion, collusion, maintenance-suppression, and recovery scenarios executed with documented outcomes. |
| Operator review package | Evidence bundle includes event, signal, confidence, action, trust delta, signature metadata, and replay instructions. |
