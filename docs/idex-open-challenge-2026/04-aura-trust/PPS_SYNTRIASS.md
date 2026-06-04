# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

AURA Trust: Offline Provenance Verification For Mission Data, Telemetry, Orders, And Intelligence

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## 1. Brief Summary of the proposed Solution (upto 250 words)

Defence operations often rely on mission data, telemetry, orders, sensor reports, and intelligence products that may pass through disconnected, low-bandwidth, or contested environments. The core problem is not only encryption during transit. Reviewers must know whether a packet was produced by an authorized source, whether it was modified, whether it is stale or replayed, and whether it can be verified offline when network trust is unavailable.

AURA Trust proposes an offline provenance and verification platform for defence information packets. It combines AURA provenance concepts, ETK-compatible audit records, proof-carrying unit patterns, and a migration path toward NIST post-quantum cryptography through the NEXUS PCU layer. The objective is to make mission information tamper-evident, source-verifiable, replay-resistant, and reviewable in disconnected settings.

The proposed demo will verify signed mission data packets offline. It will accept valid packets, reject tampered packets, reject replayed packets, and emit an ETK-compatible audit record showing provenance, verification result, and reason for denial. AURA Trust requires defence hardening before secure information platform claims.

## 2. Key Technology(s) Used (5-6 keywords)

AURA provenance, ETK audit, PCU proofs, PQC migration, offline verification, replay rejection

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | Mission packet schema | Defines source, timestamp, nonce, hash, provenance, and signature metadata |
| 2 | Offline verifier CLI | Verifies packets without continuous network access |
| 3 | Tamper/replay test suite | Demonstrates rejection of modified or stale packets |
| 4 | ETK audit exporter | Produces verification audit records |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Secure information threat model and packet schema |
| 2 | 3-5 | Offline verifier prototype and tamper rejection |
| 3 | 6-8 | Replay/freshness checks and provenance trace |
| 4 | 9-10 | PCU/PQC migration path and key lifecycle draft |
| 5 | 11-12 | Final demo, acceptance tests, and hardening roadmap |
