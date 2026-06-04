# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

BioShield Swarm: Immune-System Inspired Rogue Agent Detection For Defence Swarms

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## Official Problem Alignment and PS24 Boundary

BioShield Swarm aligns with ADITI 4 counter-UAS/autonomy problem areas and DISC 14 drone-management, C-UAS, and multi-agent UAS problem areas. It is not a duplicate of the already submitted ADITI PS24 space-domain application: this Open Challenge package is scoped to compromised-node detection, swarm trust scoring, quarantine/revalidation, and reviewer-visible evidence.

## 1. Brief Summary of the proposed Solution (upto 250 words)

Drone swarms and distributed autonomous teams create a defence challenge that conventional perimeter security does not solve. A node may be compromised, spoofed, degraded by electronic warfare, or behaviorally unreliable while still appearing to participate in the swarm. One rogue node can poison shared state, mislead coordination, or trigger unsafe collective behavior.

BioShield Swarm proposes an immune-system inspired monitoring and response layer for multi-agent defence swarms. The system evaluates behavioral signals, reputation decay, defection scoring, anomaly memory, identity failures, and quarantine rules. The objective is to identify compromised or unreliable swarm participants early, reduce their influence, isolate them when required, and retain an audit trail for post-event review.

The first prototype will use software simulation. A swarm scenario will inject compromised-node behavior, spoofed coordination signals, or defection patterns. BioShield will score behavior, decay reputation, record threat memory, and quarantine the suspected node. Current readiness is software subsystem TRL 3-4. The iDEX exit target is TRL 5 after PQC-enabled node identity/signature integration, relevant-environment swarm simulation, contested-link testing, and evaluator-witnessed drone-simulator or hardware-in-loop validation.

## 2. Key Technology(s) Used (5-6 keywords)

Multi-agent immunity, Ed25519/ML-DSA identity, reputation scoring, defection detection, threat memory, quarantine

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | Swarm compromise simulator | Software swarm environment with compromised-node injection |
| 2 | Immune threat detector | Converts abnormal behavior into threat signals |
| 3 | Reputation and defection scorer | Scores unreliable or compromised agents |
| 4 | PQC node identity profile | Hybrid Ed25519 plus ML-DSA path for selected node identity and signed-report records |
| 5 | Quarantine dashboard | Shows isolation action, reason codes, and audit trace |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Swarm threat model and scenario library |
| 2 | 3-5 | Multi-agent simulation and compromised-node injection |
| 3 | 6-8 | Reputation, defection, and threat-memory integration |
| 4 | 9-10 | Quarantine workflow and adversarial tests |
| 5 | 11-12 | PQC verification, TRL 5 exit evidence, final demo, acceptance tests, and documentation |
