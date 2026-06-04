# IDEX OPEN CHALLENGE SUBMISSION

# Annexure-3

Advantages, capabilities, and competencies

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

# Advantages, Capabilities, and Competencies

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

## Mandatory Defence Problem Fit

- Addresses rogue, spoofed, degraded, or compromised participants in defence swarms.
- Provides machine-speed trust scoring without requiring continuous central command connectivity.
- Uses identity verification, signed threat reports, reputation decay, defection severity, and threat memory.
- Supports peer-to-peer threat propagation through gossip-style message handling.
- Provides graded response: observe, reduce cooperation, increase caution, warn, isolate, and revalidate.
- Produces source-level and test-level evidence for evaluator review.
- Fits drone swarms, ground robot teams, unmanned sensor meshes, ISR agent networks, and simulation-based autonomy assurance.

## A. Technology Advantages

- Combines signed identity, reputation, threat memory, defection scoring, and quarantine logic in one swarm-integrity layer.
- Uses bounded reputation scores that decay over time instead of permanent trust.
- Uses signed threat reports to bind reporter, pattern, confidence, timestamp, and signature.
- Detects multiple defection classes: missed heartbeats, contradiction, invalid signatures, false threat reports, constraint violation, and identity abuse.
- Supports distributed threat sharing when command links are degraded or unavailable.
- Provides deterministic test evidence for identity, reputation, defection, threat propagation, and integration behavior.

## B. Product Advantages

- Can be delivered as a swarm simulation tool, Rust SDK, command dashboard, or integration layer.
- Allows defence evaluators to inject compromised-node scenarios and observe isolation decisions.
- Provides a clear reviewer trail from problem statement to source files to test output.
- Supports scenario-specific thresholds for mission type, swarm size, environment, and acceptable false-positive rate.
- Can integrate with robotics middleware, simulation environments, and NEXUS Guard protected-action control.

## C. Commercial Advantages

- Dual-use relevance for industrial robotics, autonomous inspection fleets, warehouse robots, critical-infrastructure sensor meshes, telecom edge networks, and multi-agent cybersecurity systems.
- Product lines can include swarm-integrity SDK, fleet trust dashboard, compromised-node simulation toolkit, and audit evidence exporter.
- Sovereign deployment is possible because core detection and evidence workflows can run locally.
- Supports customers that need continuous trust scoring across many autonomous or semi-autonomous nodes.
- Creates reusable trust infrastructure rather than a single hard-coded drone application.

## D. Capabilities and Competencies

- Rust implementation of distributed immune protocol components under `multi-asi-immune`.
- Code-level implementation of node identity, signed threat reports, threat memory, reputation scoring, defection tracking, protocol messages, and node health.
- Test coverage across identity, reputation, defection, threat propagation, and integration flows.
- Ability to convert defence problem statements into evidence-mapped architecture documents.
- Ability to produce iDEX-ready evidence annexures with repository locations, artifact locations, screenshots, and fresh test output.
- Experience preparing conservative readiness language for defence evaluation: software subsystem TRL 3-4, simulation evidence now, hardware-in-loop validation proposed.

## E. Evidence Summary

| Evidence Item | Repository Location |
| --- | --- |
| Core source module | `multi-asi-immune/src/` |
| Identity tests | `multi-asi-immune/tests/identity_tests.rs` |
| Reputation tests | `multi-asi-immune/tests/reputation_tests.rs` |
| Defection tests | `multi-asi-immune/tests/defection_tests.rs` |
| Threat propagation tests | `multi-asi-immune/tests/threat_propagation_tests.rs` |
| Integration tests | `multi-asi-immune/tests/integration_tests.rs` |
| Fresh test output | `docs/idex-open-challenge-2026/02-bioshield-swarm/final_4_documents/evidence_assets/bioshield_swarm_test_output.txt` |

## F. Readiness Caveat

BioShield Swarm is submitted as a software-subsystem prototype. It has source and test evidence, but does not claim physical drone hardware validation, tactical radio validation, EW range testing, or operational deployment. Those are proposed iDEX prototype validation activities.
