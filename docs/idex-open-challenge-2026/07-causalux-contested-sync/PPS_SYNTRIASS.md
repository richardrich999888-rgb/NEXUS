# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

CAUSALUX Contested Sync: Low-Bandwidth Disconnected State Synchronization With Provenance

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## 1. Brief Summary of the proposed Solution (upto 250 words)

Defence teams often operate in disconnected, degraded, intermittent, and low-bandwidth environments. Autonomous nodes, command posts, sensors, and edge devices may continue updating local state while separated. When connectivity returns, state must merge deterministically, preserve provenance, avoid replay or tamper acceptance, and minimize bandwidth consumption.

CAUSALUX Contested Sync proposes a disconnected synchronization layer for mission state and autonomous-agent coordination. It combines CAUSALUX causal state concepts, USO-style ordered evidence, VECTRA-style vector/state tracking, `nexus-sync` transfer logic, and compression tests. The objective is to allow nodes to diverge during disconnection, reconnect, exchange compact updates, merge deterministically, and retain a tamper-evident provenance trail.

The demonstration will run multiple disconnected software nodes, allow each to update state, reconnect them under constrained transfer settings, merge state deterministically, and show provenance for accepted updates. This package is recommended as reserve because it requires more scenario-specific evaluator alignment than the first five applications.

## 2. Key Technology(s) Used (5-6 keywords)

CAUSALUX, USO, VECTRA, nexus-sync, CRDT-style merge, compression

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | Disconnected-node simulator | Runs local updates while nodes are separated |
| 2 | Deterministic merge engine | Reconciles state with conflict policy and provenance |
| 3 | Compact sync transfer | Exchanges compressed state deltas |
| 4 | Replay/tamper rejection tests | Demonstrates rejection of stale or invalid updates |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Contested-sync threat model and state model |
| 2 | 3-5 | Disconnected node simulation and reconnect workflow |
| 3 | 6-8 | Deterministic merge, conflict trace, and provenance |
| 4 | 9-10 | Compression benchmarks and replay/tamper checks |
| 5 | 11-12 | Final tests, documentation, and network-in-loop plan |
