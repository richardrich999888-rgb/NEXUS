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

Purpose: Technical architecture and implementation approach for BioShield Swarm.

Contents:

- Swarm integrity architecture.
- Threat category and defection scoring model.
- Identity, reputation, and threat-memory flow.
- Quarantine and graded response workflow.
- Simulation and validation plan.

## Annexure-3 Outline

Purpose: Advantages, product value, commercial value, and competencies.

## Annexure-4 Outline

Purpose: Supporting evidence, screenshots, test output, repository locations, artifact locations, and readiness caveats.

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
| Technology / Platform Name | BioShield Swarm |
| Intended Defence End User | Indian Armed Forces, DRDO laboratories, unmanned systems evaluators, swarm autonomy teams, ISR operators, defence cyber/AI security teams, and defence system integrators evaluating multi-agent robotic or software swarms. |
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

BioShield Swarm: Immune-System Inspired Rogue Agent Detection for Defence Swarms

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Unmanned aerial, ground, surface, and sensor swarm units | Detect degraded, spoofed, compromised, or rogue nodes before they corrupt collective behavior. |
| DRDO autonomy and swarm laboratories | Evaluate software-level swarm integrity, threat propagation, reputation decay, and quarantine logic in simulation. |
| Command-and-control assurance teams | Obtain audit evidence explaining why a node was trusted, downgraded, isolated, or allowed to rejoin. |
| Defence cyber and AI security teams | Detect identity abuse, false threat reports, contradictory messages, signature failures, and behavioral drift. |
| ISR and battlefield sensing teams | Maintain integrity of distributed sensing and reporting when one or more nodes are unreliable. |
| Defence PSUs and mission integrators | Integrate a swarm-health layer without replacing the complete robotics or command stack. |

### 3.1 Official Defence Problem Alignment and Non-Duplication

| Source / Route | Relevant Defence Demand Signal | Why BioShield Swarm Fits | Boundary From Existing ADITI PS24 Submission |
| --- | --- | --- | --- |
| iDEX Open Challenge | Open-category route for focused defence prototypes with applicant-defined problem statements. | BioShield Swarm defines a narrow compromised-node and swarm-integrity problem for drone, robot, and sensor-team simulations. | Submitted as a separate swarm-integrity product, not as a space-domain training or surveillance tool. |
| ADITI 4 counter-UAS / autonomy problem areas | Counter-UAS and autonomous-system problem areas require detection of spoofed, compromised, or behaviorally unreliable participants. | The proposal demonstrates identity checks, defection scoring, threat memory, reputation decay, and quarantine/revalidation logic. | The existing PS24 application is space-domain focused; this proposal targets swarm participant trust and compromised-node response. |
| DISC 14 drone-management and multi-agent UAS problem areas | Integrated drone management, C-UAS, and multi-agent UAS problem areas require trust scoring and local response under contested conditions. | BioShield Swarm supports signed reports, peer threat gossip, isolation thresholds, and reviewer-visible evidence for suspicious nodes. | It does not duplicate PS24 deliverables, datasets, or space-domain operational training objectives. |

```{=typst}
#pagebreak()
```

## 4. A. Brief Summary of Proposed Solution

Drone swarms and distributed autonomous teams create a defence problem that perimeter security does not solve. A node may be spoofed, degraded by electronic warfare, compromised by malware, manipulated by false commands, or behaviorally unreliable while still appearing to participate in the swarm. One rogue node can poison threat reports, mislead routing, corrupt consensus, or trigger unsafe collective behavior.

BioShield Swarm proposes an immune-system inspired integrity layer for multi-agent defence swarms. The system evaluates cryptographic identity, signed threat reports, heartbeat/liveness behavior, reputation decay, defection severity, threat memory, and quarantine thresholds. Nodes that behave suspiciously can be observed, downgraded, constrained, isolated, or revalidated according to policy.

The 12-month iDEX prototype will demonstrate a software swarm simulation where compromised-node behavior is injected into a multi-agent network. BioShield Swarm will score the behavior, record threat memory, decay reputation, propagate verified threat reports, and isolate or reduce trust for the suspicious node. Current evidence is software-subsystem TRL 3-4. The iDEX exit target is TRL 5 after PQC-enabled node identity/signature integration, relevant-environment swarm simulation, contested-link testing, and evaluator-witnessed hardware-in-loop or drone-simulator validation.

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Rogue swarm participant | A compromised node can mislead peers or act against mission intent. | Defection scoring, reputation decay, and isolation thresholds. |
| Spoofed or forged identity | Adversary may impersonate a trusted node. | Ed25519 public-key identity and signature verification. |
| False threat reporting | Malicious node may poison swarm perception. | Signed threat reports, reputation-filtered threat memory, duplicate rejection. |
| Coordinated malicious behavior | Multiple nodes may collude or amplify false information. | Threat categories, pattern confirmation, and multi-reporter aggregation. |
| Lost command link | Central command may be unavailable in contested environments. | Peer-to-peer gossip and local swarm-health assessment. |
| Benign degradation vs compromise ambiguity | EW or degraded hardware may look like attack behavior. | Graded response: observe, reduce trust, constrain, isolate, and revalidate. |

```{=typst}
#pagebreak()
```

## 6. B. Key Technologies Used

- Multi-agent immune protocol
- Ed25519 identity and signed reports
- Hybrid Ed25519 plus ML-DSA PQC node identity path
- Reputation decay and transitive trust
- Defection scoring and isolation
- Threat memory and pattern confirmation
- Swarm gossip and heartbeat monitoring

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| BioShield Swarm Simulator | Software swarm environment with normal, degraded, spoofed, and compromised-node scenarios. |
| Threat Category Engine | Maps behavior into goal drift, deception, self-modification, coordinated attack, protocol violation, and related threat classes. |
| Defection Scorer | Tracks missed heartbeats, contradictory messages, invalid signatures, false reports, and identity abuse. |
| Reputation Engine | Applies bounded, decaying trust scores and reduces influence of unreliable nodes. |
| Threat Memory | Stores signed threat reports, rejects duplicates, aggregates confirmations, and expires stale threat records. |
| PQC Node Identity Profile | Integrates the `nexus-pcu` hybrid Ed25519 plus ML-DSA path for node identity bundles and signed threat-report transition testing. |
| Quarantine Controller | Demonstrates observe, reduce cooperation, broadcast warning, isolate, and revalidation workflows. |
| Evidence Dashboard Prototype | Shows swarm health, suspicious nodes, isolation reasons, active threats, and audit trail. |
| Validation Report | Provides test output, scenario results, false-positive analysis, latency measurements, and integration caveats. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Defence Swarm Threat Model | Scenario library for spoofing, false reporting, liveness loss, collusion, and compromised-node behavior. |
| Phase 2 | Month 3 to Month 4 | Simulation Harness | Multi-node software simulation with configurable swarm size, compromised-node injection, and telemetry export. |
| Phase 3 | Month 5 to Month 6 | Reputation and Defection Integration | Scoring logic, threshold policy, and isolation/revalidation rules. |
| Phase 4 | Month 7 to Month 8 | Threat Memory and Gossip | Signed threat report propagation, duplicate rejection, confidence aggregation, and stale-threat expiry. |
| Phase 5 | Month 9 | Operator Review Dashboard | Swarm health summary, active threats, isolated nodes, reason codes, and audit trail. |
| Phase 6 | Month 10 | Adversarial Simulation | Rogue-node tests, false-report tests, collusion scenarios, and degradation-vs-compromise checks. |
| Phase 7 | Month 11 | Integration Packaging | SDK/API wrapper, deployment guide, test dataset, evaluator runbook, and evidence exporter. |
| Phase 8 | Month 12 | Final Demonstration and TRL 5 Exit Evidence | iDEX demo package, PQC verification report, relevant-environment validation report, readiness statement, and hardware-in-loop validation evidence or evaluator-approved simulator evidence. |

## 9. E. Readiness Position

| Area | Current Evidence | Caveat |
| --- | --- | --- |
| Rust immune modules | 68 passing Rust tests from `multi-asi-immune` | Software subsystem evidence only. |
| Threat categories | Implemented in source and tests | Operational calibration required. |
| Defection and isolation | Implemented with threshold tests | Hardware/sensor-level validation pending. |
| Threat propagation | Implemented in integration tests | Network latency/jitter testing pending. |
| PQC node identity | `nexus-pcu` PQC feature path available and tested separately | Integration into swarm identity/protocol path is proposed iDEX work. |
| Drone hardware | Not claimed | Proposed iDEX validation pathway. |

## 10. F. TRL 5 Exit Criteria

| Exit Criterion | Evidence Required Before TRL 5 Claim |
| --- | --- |
| Relevant environment | Swarm integrity tested in evaluator-approved drone simulator, contested-link lab setup, or hardware-in-loop testbed. |
| PQC-enabled identity path | Hybrid Ed25519 plus ML-DSA identity/signature bundle integrated into signed threat reports or node-attestation records and verified in CI. |
| Compromised-node scenarios | Spoofing, invalid signature, false report, collusion, liveness loss, and benign degradation scenarios executed with expected responses. |
| Response calibration | Isolation and revalidation thresholds measured for false-positive and false-negative behavior. |
| Operator review package | Evidence bundle includes node identity, report signature metadata, defection score, reputation delta, quarantine decision, and replay instructions. |
