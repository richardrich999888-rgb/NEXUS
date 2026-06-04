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

Purpose: technical architecture and implementation approach for CAUSALUX Contested Sync.

Contents:

- Disconnected-node state synchronization.
- Version-vector causality and CRDT convergence.
- Snapshot-based long-partition recovery.
- Compact VECTRA-backed transfer path.
- USO state object and NEXUS sync integration.
- Test evidence and readiness caveats.

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
| Technology / Platform Name | CAUSALUX Contested Sync |
| Intended Defence End User | Indian Armed Forces C2 modernization teams, DRDO distributed-systems evaluators, contested communications teams, autonomous-systems assurance teams, tactical data-link integrators, and secure mission-state synchronization teams. |
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

CAUSALUX Contested Sync: Low-Bandwidth Disconnected Tamper-Evident State Synchronization For Defence Nodes

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Tactical C2 teams | Preserve mission-state continuity when command links are degraded or intermittent. |
| Autonomous-system operators | Allow robots, sensors, or agents to continue local updates while disconnected. |
| DRDO distributed-system evaluators | Inspect deterministic merge, vector clocks, CRDTs, and snapshot-based recovery evidence. |
| Contested communications teams | Evaluate compact transfer, synchronization deltas, and compression behavior. |
| Secure information teams | Track provenance of accepted updates during reconnect and merge. |
| Procurement and audit panels | Review test output, source paths, caveats, and artifact locations. |

```{=typst}
#pagebreak()
```

## 4. A. Brief Summary of Proposed Solution

Defence systems frequently operate under disconnected, degraded, intermittent, and low-bandwidth conditions. Command posts, sensors, autonomous vehicles, and edge agents may continue updating local mission state while separated. When connectivity returns, the state must merge deterministically, preserve provenance, reject stale or invalid updates, and avoid wasting bandwidth by replaying full state.

CAUSALUX Contested Sync proposes a software-subsystem prototype for disconnected mission-state synchronization. It combines CAUSALUX causal DAGs, version-vector causality, CRDT merge types, snapshot-based long-partition recovery, USO state objects, NEXUS sync deltas, and VECTRA-backed compression. The objective is not to claim field qualification in phase one. The objective is to demonstrate that nodes can diverge during disconnection, reconnect, exchange compact updates, converge through deterministic merge behavior, and retain a reviewer-visible audit trail for accepted updates.

The iDEX prototype will demonstrate multi-node software simulation, offline local updates, reconnect synchronization, common-snapshot selection, compressed state transfer, CRDT/USO merge, provenance export, and caveat-driven hardening for stale E2E tests and contested-network validation.

Current evidence is software-subsystem TRL 3-4. Radio hardware, EW/jamming conditions, mission-specific merge policy approval, and physical platform validation are proposed work packages.

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Command-link loss | Nodes may continue operating while disconnected from central command. | Offline-first local updates with later synchronization. |
| State divergence | Multiple nodes may update mission state independently. | Version vectors, CRDTs, and deterministic conflict policy. |
| Low-bandwidth recovery | Full-state replay can overwhelm tactical links. | Snapshot negotiation, hierarchical sync, and compressed transfer. |
| Stale or replayed updates | Old or invalid update material can corrupt mission state. | Causal metadata, operation IDs, version context, and verifier-visible caveats. |
| Unclear provenance | Reviewers need to know which updates were accepted and why. | USO history, CAUSALUX DAG ordering, Merkle-root snapshots, and evidence logs. |
| E2E drift risk | Integration tests can become stale as core APIs evolve. | iDEX milestone for updating stale E2E tests and network-in-loop scenarios. |

```{=typst}
#pagebreak()
```

## 6. B. Key Technologies Used

- CAUSALUX causal DAG
- Version vectors
- CRDT merge types
- Snapshot-based recovery
- NEXUS USO state objects
- VECTRA-backed compression

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| Disconnected Node Simulator | Runs multiple software nodes that continue local updates during communication loss. |
| Deterministic Merge Engine | Reconciles divergent state using version vectors, CRDTs, and declared conflict policy. |
| Compact Sync Transfer | Exchanges deltas and compressed state material instead of full-state replay where possible. |
| Snapshot Recovery Path | Demonstrates common-snapshot negotiation and long-partition recovery behavior. |
| USO State Adapter | Maps mission-state objects into Universal State Objects with sync/access policy and history. |
| Provenance Exporter | Records source node, causal context, operation IDs, accepted merge result, and reason code. |
| Stale Update Test Plan | Defines replay, stale vector, dependency gap, and invalid operation scenarios. |
| Evaluation Report | Provides source paths, test output, screenshots, caveats, artifact locations, and hardening roadmap. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Contested-sync threat model | Mission-state model, threat assumptions, link constraints, and evaluator-approved scope. |
| Phase 2 | Month 3 to Month 4 | Node simulator | Multi-node offline update and reconnect harness. |
| Phase 3 | Month 5 to Month 6 | Merge policy implementation | Deterministic merge, conflict traces, and provenance output. |
| Phase 4 | Month 7 to Month 8 | Compact transfer path | Snapshot negotiation, sync deltas, and compression measurements. |
| Phase 5 | Month 9 | Stale-update hardening | Replay, stale vector, missing dependency, and invalid operation tests. |
| Phase 6 | Month 10 | E2E test modernization | Update stale integration tests to current PCU and compression APIs. |
| Phase 7 | Month 11 | Network-in-loop validation | Emulated packet loss, latency, bandwidth caps, and reconnect scenarios. |
| Phase 8 | Month 12 | Final demonstration | iDEX demo package, test report, reviewer map, and field-validation roadmap. |

## 9. E. Readiness Position

Fresh selected evidence includes 90 passing checks across CAUSALUX library/integration tests, nexus-sync library tests, PCU USO selected tests, and nexus-compress tests.

Submission boundary: software-subsystem prototype only. One CAUSALUX doctest and stale nexus-sync E2E tests require modernization before full end-to-end claims. Radio hardware, EW simulation, operational data-link integration, and physical platform validation remain proposed iDEX work packages.
