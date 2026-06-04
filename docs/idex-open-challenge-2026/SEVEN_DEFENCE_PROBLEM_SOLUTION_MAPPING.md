# Seven Defence Problems And NEXUS-Derived Solutions

Date: May 16, 2026  
Applicant: SYNTRIASS Labs Private Limited  
Scope: iDEX Open Challenge 2026 proposal framing  

This section maps the seven proposed applications to distinct defence problems. Each solution is framed as a separate product, while acknowledging the shared NEXUS governed-autonomy backbone. Current readiness should be described as software subsystem TRL 3-4 unless package-specific evidence supports a narrower claim. Hardware-in-loop, field, EW, and service-specific validation remain proposed work.

## Summary Matrix

| # | Defence problem | Proposed product | Core solution | Evidence basis | Demo promise |
| --- | --- | --- | --- | --- | --- |
| 1 | Unauthorized high-consequence autonomous execution | NEXUS Guard | ExecutionGuard chain with first-deny-wins policy, TELOS consequence budget, and ETK audit evidence | `nexus-executor`, red-team execution tests, TELOS/ETK modules | Unauthorized action is denied before execution; allowed action creates audit evidence |
| 2 | Rogue or compromised swarm units | BioShield Swarm | Immune-style threat detection, defection scoring, reputation decay, threat memory, and quarantine | `multi-asi-immune`, identity, reputation, threat, enforcement tests | Compromised node is detected, scored, reputation-reduced, and quarantined in simulation |
| 3 | Unsafe robot command flow and resource exhaustion | AGP-OS Robotics Safety Layer | ROS2 bridge, AGP policy admission, RTOS priority scheduling, resource denial, and HAL safety interlocks | `agp-core`, RTOS tests, ROS2 simulation tests, resource and production tests | Governed robot command flow denies unsafe or over-budget commands in ROS2/Gazebo-style simulation |
| 4 | Mission data, telemetry, orders, or intelligence cannot be trusted offline | AURA Trust | Offline packet provenance verification, tamper/replay rejection, ETK audit record, and PCU/PQC migration path | AURA provenance concepts, ETK audit path, `nexus-pcu` PQC tests | Signed mission packet verifies offline; tampered or replayed packet is rejected with audit record |
| 5 | Cyber teams cannot respond fast enough without unsafe automation | Cyber Immune SOAR | Cyber events converted into immune threat signals, governed response actions, quarantine, reputation updates, and audit trail | AGP immune bridge, immune system, unified immune, multi-agent governance tests | Simulated cyber events trigger observe/throttle/quarantine/escalate actions under policy |
| 6 | Agent and device identities are vulnerable to long-term quantum migration risk | PQC Defence Identity | Hybrid classical plus ML-DSA-compatible signing envelope with policy-controlled verification behavior | `nexus-pcu` hybrid signature/PQC feature tests | Hybrid proof packet verifies; tampering is rejected; fallback behavior is policy-visible |
| 7 | Disconnected units cannot safely merge state after contested communication loss | CAUSALUX Contested Sync | Causal state sync, deterministic merge, provenance, ordered evidence, compact deltas, and replay rejection | CAUSALUX, USO, VECTRA, `nexus-sync`, compression test targets | Disconnected nodes update locally, reconnect, merge deterministically, and preserve provenance |

## 1. NEXUS Guard

### Problem

How do you prevent an autonomous system from executing a high-consequence action without authorization, policy approval, and audit evidence?

### Solution

NEXUS Guard places a mandatory ExecutionGuard in front of protected actions. The guard evaluates authorization, context, mission policy, and consequence budget before execution. The core semantics are first-deny-wins: if any guard denies the request, the action does not execute and the denied path does not produce a success proof or execution cache artifact. Allowed actions produce ETK-compatible audit evidence and TELOS consequence-budget traces.

### Defence Value

This shifts assurance from after-the-fact logging to pre-execution control. It is directly relevant to autonomous software agents, robotic controllers, simulation systems, mission decision services, and any high-impact command path where denial must happen before action.

### Evidence And Caveat

Evidence comes from `nexus-executor`, ExecutionGuard interfaces, red-team execution tests, ETK audit components, and TELOS consequence accounting. Current evidence is software-subsystem level. Physical integration and latency characterization remain required before higher TRL claims.

## 2. BioShield Swarm

### Problem

How do you detect and isolate a drone, robot, sensor, or software agent that has been compromised, is spoofing messages, or is behaving against swarm objectives?

### Solution

BioShield Swarm applies immune-system concepts to multi-agent coordination. It scores threat patterns, defection behavior, identity failures, contradictory messages, false reports, and missed liveness signals. Reputation decays over time, threat memory accelerates repeated-pattern detection, and quarantine rules reduce or isolate the influence of suspicious agents.

### Defence Value

The product addresses compromised swarm units, insider behavior, spoofed coordination, firmware drift, false threat reports, and distributed corruption. It supports machine-speed response while preserving an audit trail for reviewer analysis.

### Evidence And Caveat

Evidence comes from `multi-asi-immune`, Ed25519 identity paths, reputation modules, defection scoring, threat pattern tests, and multi-agent protocol tests. Current validation is software and simulation oriented. Embedded ARM porting, drone telemetry integration, and EW degradation scenarios remain proposed work.

## 3. AGP-OS Robotics Safety Layer

### Problem

How do you run governed autonomy on ROS2 robots while enforcing real-time priorities, resource limits, and safety interlocks?

### Solution

AGP-OS Robotics Safety Layer inserts governance into robot command flow. A ROS2 bridge converts topics, services, or actions into governed requests. AGP policy admits or denies commands. RTOS scheduling prioritizes safety-critical tasks. Resource controllers deny commands that exceed CPU, memory, token, or mission budgets. HAL safety interlocks prevent unsafe actuator access in simulated control paths.

### Defence Value

The product creates a safer bridge between AI autonomy and physical robotic control. It is relevant to unmanned ground, aerial, maritime, sensor, and logistics platforms where ROS2-style systems need command governance and resource enforcement.

### Evidence And Caveat

Evidence comes from `agp-core`, RTOS tests, ROS2 simulation tests, resource tests, production-mode checks, and HAL safety logic. Current evidence is simulation-first. Hard real-time guarantees and physical board/robot validation remain planned under the iDEX effort.

## 4. AURA Trust

### Problem

How do you verify mission data, telemetry, orders, or intelligence provenance when systems are offline, disconnected, or communicating through untrusted relays?

### Solution

AURA Trust defines an offline verification path for mission information packets. Packets carry source, timestamp, nonce, payload hash, provenance metadata, and signatures. The verifier accepts valid packets, rejects tampered payloads, rejects stale or replayed packets, and emits ETK-compatible audit records. The PCU/PQC path gives the package a migration route toward approved post-quantum signing profiles.

### Defence Value

The product addresses tampered mission data, spoofed orders, stale telemetry, replayed intelligence packets, and provenance loss in disconnected operations. It is positioned as an offline verification and provenance prototype, not as an already accredited secure information platform.

### Evidence And Caveat

Evidence comes from AURA provenance concepts, ETK audit records, proof-carrying unit primitives, and `nexus-pcu` PQC feature tests. AURA Trust still needs defence hardening, key lifecycle design, packet format finalization, and cryptographic profile alignment.

## 5. Cyber Immune SOAR

### Problem

How do cyber defenders respond to fast-moving attacks without giving an unconstrained autonomous system permission to disrupt critical services?

### Solution

Cyber Immune SOAR converts cyber events into immune threat signals. A governance bridge maps signal severity and confidence into bounded actions: observe, throttle, quarantine, or escalate. Reputation updates track repeatedly suspicious services, agents, or identities. Every response action emits an audit trail for replay and review.

### Defence Value

The product targets cyber alert overload, delayed containment, compromised software agents, repeated policy violations, and unsafe automation risk. It provides autonomy under explicit response policy rather than unconstrained self-defence.

### Evidence And Caveat

Evidence comes from AGP immune bridge tests, immune system tests, unified immune tests, and multi-agent governance tests. Initial demos should use simulated cyber events. Live SOC integration, realistic telemetry feeds, and threshold calibration remain proposed work.

## 6. PQC Defence Identity

### Problem

How do defence agents, devices, and mission packets prepare for post-quantum identity risk without breaking current classical verification flows?

### Solution

PQC Defence Identity provides a hybrid signing envelope. It supports classical verification for near-term interoperability and an ML-DSA-compatible post-quantum path for migration readiness. Policy can define whether classical-only, hybrid, or PQC-required verification is accepted for a given mission class. Tamper rejection and fallback behavior are visible in audit evidence.

### Defence Value

The product supports long-life defence systems, autonomous device identity, mission packet signing, and staged migration toward quantum-resistant verification. It is strategically important as a reserve or follow-on application because it is narrower than the first five operational prototypes.

### Evidence And Caveat

Evidence comes from `nexus-pcu` hybrid signature types and PQC feature tests. Current PQC evidence is unit-level. Full production enforcement, key lifecycle management, revocation, provisioning, and approved cryptographic profile alignment remain required.

## 7. CAUSALUX Contested Sync

### Problem

How do disconnected defence nodes continue operating locally and later merge state safely when communication is restored?

### Solution

CAUSALUX Contested Sync lets nodes diverge during disconnection and reconcile deterministically during reconnect. Causal metadata tracks dependencies. USO-style ordered evidence anchors accepted updates. VECTRA-style state tracking supports synchronization context. `nexus-sync` exchanges compact deltas, while compression reduces transfer payloads. Replay, stale, or invalid updates are rejected with provenance evidence.

### Defence Value

The product addresses disconnected, degraded, intermittent, and low-bandwidth operations. It is relevant to command posts, autonomous agents, edge sensors, mission-state sharing, and teams operating with contested communications.

### Evidence And Caveat

Evidence comes from CAUSALUX, USO, VECTRA, `nexus-sync`, and compression test targets. Package names and test commands should be verified before portal upload. Field network behavior, packet loss, jamming, and mission-specific merge policies remain proposed validation work.

## Recommended Submission Order

Submit these five first:

1. NEXUS Guard
2. BioShield Swarm
3. AGP-OS Robotics Safety Layer
4. AURA Trust
5. Cyber Immune SOAR

Hold these as reserve or follow-on applications:

6. PQC Defence Identity
7. CAUSALUX Contested Sync

This order prioritizes applications with concrete operational demos and broader reviewer relevance while keeping cryptographic migration and contested synchronization ready as specialized follow-ons.
