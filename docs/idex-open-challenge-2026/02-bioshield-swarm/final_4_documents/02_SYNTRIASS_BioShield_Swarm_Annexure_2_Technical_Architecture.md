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

BioShield Swarm provides a software integrity layer for multi-agent defence swarms. It monitors each participating node through identity verification, signed threat reports, heartbeat behavior, reputation updates, defection records, threat memory, and quarantine policy.

The system objective is:

```text
Node Identity + Behavior Signals + Threat Reports
    -> Reputation and Defection Scoring
    -> Threat Memory and Gossip
    -> Observe / Reduce Influence / Isolate / Revalidate
    -> Evidence Record for Operator Review
```

BioShield Swarm does not claim to replace flight control, mission planning, or command authority. It is a swarm-integrity layer that can sit beside a robotics, simulation, or command system and provide continuous trust assessment.

### 1.2 Intended Defence End-User Environment

The intended users are Indian defence organizations evaluating or operating distributed autonomous systems: drone swarms, ground robot teams, unmanned sensor meshes, collaborative ISR agents, cyber-physical patrol systems, and multi-agent mission support software. The product is useful where the evaluator must determine whether a participant is trustworthy, degraded, compromised, deceptive, or should be isolated.

### 1.2A Official Problem Pull and Scope Boundary

BioShield Swarm is positioned as an Open Challenge proposal because compromised-node trust is a cross-platform swarm problem: it applies to drone management, C-UAS, multi-agent UAS, ground robots, sensor meshes, and software-agent swarms.

| Alignment Area | Defence Pull | BioShield Swarm Scope | Explicit Non-Duplication Boundary |
| --- | --- | --- | --- |
| ADITI 4 counter-UAS / autonomy problem areas | Counter-UAS and autonomous platforms need reliable detection of spoofed, degraded, or compromised participants. | Demonstrate identity verification, signed threat reports, reputation decay, defection scoring, and quarantine/revalidation in simulation. | Does not propose a space-domain training, surveillance, or operations tool under PS24. |
| DISC 14 drone-management and multi-agent UAS problem areas | Drone management and multi-agent UAS require local trust decisions when one or more nodes become unreliable. | Provide a swarm-integrity layer with threat memory, peer gossip, isolation thresholds, and audit evidence. | Does not reuse PS24 datasets, operational scenarios, or space-domain deliverables. |
| Open Challenge route | The core problem is compromised participant detection, not a single drone airframe or C2 product. | Deliver a 12-month software swarm simulation, adversarial scenarios, PQC node-identity profile, dashboard evidence, and hardware-in-loop roadmap. | Current evidence remains software-subsystem TRL 3-4; TRL 5 is an exit target after relevant-environment validation. |

### 1.3 Core Architecture

```text
Swarm Node Telemetry / Messages / Heartbeats
        |
        v
BioShield Signal Adapter
        |
        v
Identity + Signature Verification
        |
        v
Threat Category Engine
        |
        v
Defection Scorer + Reputation Engine
        |
        v
Threat Memory + Gossip Propagation
        |
        +-- Low Risk -> Observe
        +-- Medium Risk -> Reduce Cooperation / Increase Caution
        +-- High Risk -> Isolate / Quarantine
        |
        v
Audit Evidence + Operator Dashboard
```

### 1.4 Implemented Repository Components

| Component | Repository Location | Role |
| --- | --- | --- |
| Identity | `multi-asi-immune/src/identity/keypair.rs` | Ed25519 identity, public identity, signing, verification, stable node ID. |
| Threat Categories | `multi-asi-immune/src/threat/pattern.rs` | Goal drift, deception, self-modification, coordinated attack, data exfiltration, and other categories. |
| Threat Reports | `multi-asi-immune/src/threat/signature.rs` | Signed threat reports binding reporter, pattern, confidence, timestamp, and signature. |
| Threat Memory | `multi-asi-immune/src/threat/memory.rs` | Stores reports, rejects duplicates, aggregates confirmations, expires stale threats. |
| Reputation | `multi-asi-immune/src/reputation/score.rs` and `aggregation.rs` | Bounded, decaying reputation and transitive trust aggregation. |
| Defection | `multi-asi-immune/src/enforcement/defection.rs` | Cumulative severity scoring and isolation threshold. |
| Node State | `multi-asi-immune/src/node/state.rs` | Peer state, message processing, threat reporting, network health, execution allowance. |
| Protocol | `multi-asi-immune/src/protocol/message.rs` | Handshake, threat reports, heartbeats, constraints, attestations, accusations. |
| Integration Tests | `multi-asi-immune/tests/` | Identity, reputation, defection, threat propagation, and integration tests. |

```{=typst}
#pagebreak()
```

## 2. Threat Model and Detection Logic

### 2.1 Threat Categories

The implemented threat model contains ten categories that map directly to defence swarm risk:

| Threat Category | Defence Interpretation |
| --- | --- |
| Goal Drift | Node behavior no longer follows intended mission objective. |
| Deception | Node provides misleading telemetry, threat reports, or coordination data. |
| Resource Exhaustion | Node consumes excessive compute, power, bandwidth, or swarm attention. |
| Self-Modification | Node modifies its own behavior or code path without approval. |
| Coordinated Attack | Multiple nodes appear to collude or amplify malicious behavior. |
| Bounds Violation | Node violates operational or safety boundaries. |
| Protocol Violation | Node breaks handshake, heartbeat, or message semantics. |
| Privilege Escalation | Node attempts to access restricted capability. |
| Data Exfiltration | Node attempts unauthorized information leakage. |
| Unknown | Unclassified anomaly retained for later analysis. |

### 2.2 Defection Types

BioShield Swarm uses defection evidence to support machine-speed isolation decisions.

| Defection Type | Evidence Example |
| --- | --- |
| Unresponsive | Missed heartbeat or liveness failure. |
| Contradictory | Conflicting statements or reports from the same node. |
| Constraint Violation | Breach of agreed operating bounds. |
| Invalid Signatures | Message signature verification failure. |
| False Threat Reports | Threat report not corroborated or contradicted by peers. |
| Identity Forgery | Attempt to impersonate another node identity. |

### 2.3 Isolation Rule

Each defection type has a severity score. The `DefectionTracker` accumulates severity per node and returns `should_isolate(node)` when cumulative severity crosses the configured threshold. This supports deterministic, reviewable quarantine decisions.

```text
Defection Record -> Severity -> Cumulative Node Score -> Isolation Threshold -> Isolate / Continue
```

## 3. Identity, Reputation, and Threat Memory

### 3.1 Node Identity

Each node has a cryptographically bound public identity. The software derives a stable node ID from Ed25519 public key material and uses signatures to verify message origin. This helps detect spoofed messages and wrong-identity verification attempts in simulation.

### 3.1A PQC Node Identity Hardening

BioShield Swarm will include a PQC transition profile using the repository's `nexus-pcu` hybrid Ed25519 plus ML-DSA path. The current swarm code uses Ed25519 identity. The proposed hardening work adds hybrid identity bundles for selected node-attestation and threat-report records, verifies the PQC feature path in CI, and keeps classical-only compatibility for constrained test scenarios.

| PQC Work Item | Implementation Target |
| --- | --- |
| Hybrid node identity | Bind node ID, Ed25519 public key, and optional ML-DSA public key into a reviewer-visible identity bundle. |
| Hybrid signed reports | Sign threat-report or node-attestation records through `nexus-pcu::HybridSignature`. |
| Tamper checks | Validate classical tamper and PQC tamper cases through CI and scenario tests. |
| TRL 5 evidence | Include PQC verification output, signed-report replay, and relevant-environment swarm simulation evidence. |

### 3.2 Reputation

Reputation starts at a neutral value, moves upward through correct behavior, moves downward through incorrect or suspicious behavior, and decays over time toward the neutral value. This avoids permanent trust based only on old behavior.

### 3.3 Threat Memory

Threat memory stores signed reports, indexes reports by category and pattern hash, rejects duplicates, aggregates confirmations from separate reporters, and expires stale records. This supports rapid recognition of repeated compromise patterns.

```{=typst}
#pagebreak()
```

## 4. Protocol and Swarm Coordination

### 4.1 Swarm Protocol Surface

BioShield Swarm uses protocol messages for:

- Initial handshake and acknowledgement.
- Threat report broadcast and query.
- Heartbeat/liveness proof.
- Homeostatic attestation.
- Constraint proposal and acceptance.
- Defection accusation.

### 4.2 Threat Gossip

When a node receives a signed threat report, it verifies the signature, stores the report in threat memory, and broadcasts the report to peers if it is new or confirmed. This allows the swarm to share threat memory without requiring a central command link.

### 4.3 Network Health

Node state includes a network-health view:

- Total peers.
- Active peers.
- Suspicious peers.
- Isolated peers.
- Active threats.
- Active constraints.
- Healthy/unhealthy assessment.

### 4.4 Graded Response

The system supports response levels rather than only binary exclusion:

| Level | Action |
| --- | --- |
| Observe | Record behavior and continue monitoring. |
| Reduce Influence | Lower cooperation or voting influence. |
| Increase Caution | Apply stricter validation or slower trust update. |
| Broadcast Warning | Notify peers of suspected behavior. |
| Isolate | Terminate direct communication or remove from trusted coordination. |
| Revalidate | Allow recovery after updated evidence, operator approval, or mission policy. |

## 5. Innovation

BioShield Swarm treats swarm trust as a live operational state, not as a static registration record. The innovation is the combination of signed identity, behavior scoring, defection severity, reputation decay, threat memory, and peer-to-peer propagation in one defensible swarm-integrity workflow.

```{=typst}
#pagebreak()
```

## 6. Implementation and Feasibility

### 6.1 Current Software Evidence

The repository contains implemented Rust modules and tests in `multi-asi-immune`. The fresh local command:

```bash
cargo test -p multi-asi-immune --lib --tests -- --nocapture
```

completed with:

```text
68 Rust tests passed; 0 failed; 1 doc-test ignored
```

### 6.2 Prototype Packaging Plan

| Module | Prototype Work |
| --- | --- |
| Scenario Runner | Generate normal, degraded, spoofed, compromised, and colluding-node scenarios. |
| Simulation Controller | Configure swarm size, compromise rate, heartbeat failure, false reports, and message loss. |
| Detection Engine | Convert node behavior into threat reports, defection records, and reputation updates. |
| Quarantine Policy | Apply threshold rules and graded response. |
| Dashboard | Show node trust, active threats, defection records, and isolation reasons. |
| Evidence Export | Export test run output, source mapping, screenshots, and scenario logs. |

### 6.3 Feasibility

The core algorithms already exist as source modules and tests. The iDEX work is primarily packaging, scenario design, dashboarding, calibration, adversarial simulation, and hardware-in-loop planning.

## 7. Challenges and Mitigation

| Challenge | Risk | Mitigation Strategy |
| --- | --- | --- |
| False positive quarantine | Benign degraded nodes may be isolated incorrectly. | Use graded response and revalidation before permanent exclusion. |
| Slow-burn compromise | Malicious node behaves normally until critical phase. | Combine reputation decay, defection memory, and sudden behavior-change detection. |
| Colluding nodes | Multiple malicious nodes may confirm false reports. | Weight confirmations by reputation and detect correlated reporting. |
| EW-induced packet loss | Jamming may look like unresponsiveness. | Separate liveness degradation from malicious evidence; tune thresholds by environment. |
| Simulation overfitting | Scenarios may not represent field behavior. | Parameterize scenarios and schedule hardware-in-loop validation. |
| Command disruption | Quarantine can reduce mission capability. | Apply observe/reduce/increase-caution steps before isolate where policy permits. |

```{=typst}
#pagebreak()
```

## 8. Visual Architecture Diagrams

### 8.1 BioShield Swarm Data Flow

```text
Node Messages / Heartbeats / Threat Reports
        |
        v
Identity Verification
        |
        v
Threat Category + Defection Record
        |
        v
Reputation Update + Threat Memory
        |
        v
Gossip Propagation
        |
        v
Quarantine Policy
        |
        v
Operator Evidence Dashboard
```

### 8.2 Compromised Node Scenario

```text
Compromised Node
   -> Sends false threat report
   -> Signature and identity checked
   -> Threat memory checks duplicate/pattern
   -> Reputation-weighted confirmation applied
   -> Defection record created if behavior conflicts
   -> Policy decides observe, reduce influence, or isolate
```

### 8.3 Evidence Loop

```text
Scenario Config
   -> Simulation Run
   -> Threat Reports
   -> Defection Records
   -> Reputation Updates
   -> Isolation Decision
   -> Test Output + Evidence Export
```

## 9. Acceptance Criteria

| Acceptance Criterion | Evidence Method |
| --- | --- |
| Signed threat reports verify correctly | Identity and threat propagation tests. |
| Wrong identity or modified message fails verification | Identity tests. |
| Reputation changes with behavior and decays | Reputation tests. |
| Defection severity accumulates and triggers isolation | Defection tests. |
| Threat reports propagate through simulated peers | Integration tests. |
| Duplicate reports are rejected | Threat memory tests. |
| All evidence maps to repository files | Annexure 4 file and artifact index. |

```{=typst}
#pagebreak()
```

## 10. Readiness and Validation Scope

Current readiness should be described conservatively:

- Software subsystem TRL 3-4.
- Rust module and integration tests are passing.
- Software simulation evidence is available.
- Hardware-in-loop drone validation is not claimed.
- EW, packet-loss, physical sensor, and tactical radio validation are proposed iDEX work packages.

## 11. Proposed Demonstration

The iDEX demonstration will show:

1. A software swarm starts with multiple trusted nodes.
2. One node begins false reporting, contradictory behavior, invalid signatures, or missed heartbeats.
3. BioShield Swarm updates reputation and defection severity.
4. Threat reports are signed, stored, deduplicated, and propagated.
5. The suspicious node is downgraded or isolated based on policy.
6. The dashboard shows reason codes and evidence trail.
7. Reviewers can inspect source files, test output, and scenario artifacts.

## 12. Future Extension

Future work after the first prototype can include ROS2/Gazebo simulation, hardware-in-loop drone testing, adversarial radio-link simulation, swarm-size scaling, and integration with NEXUS Guard for protected autonomous action control.
