# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

Syntriass Sovereign Trust Fabric: PQC-Ready Trust, Messaging, Data Protection, and Audit Layer for Defence Systems

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## Official Problem Alignment

This Open Challenge proposal is aligned to defence needs around quantum-readiness, secure information exchange, tactical edge trust, autonomous systems assurance, and mission data provenance. It is framed as a self-defined iDEX Open Challenge solution, not as a duplicate of an already submitted ADITI PS24 space-domain application.

Verified context used for framing:

- iDEX Open Challenge last date shown on official iDEX portal: 30 Jun 2026, 11:59 PM.
- ADITI 4, DISC 14, and DRISHTI challenge windows are closed on the same portal.
- PIB confirmed release of the Military Quantum Mission Policy Framework on 22 Jan 2026.
- DST's February 2026 PQ migration task force report references CBOM, hybrid PQC, crypto-agility, and "no new classical-only deployments" as migration principles.

## Repository Evidence Boundary

This proposal is based on the NEXUS repository evidence available in this workspace. It does not rely on external AIP/AEGIS repository paths such as `aip-service`, `aip-standalone`, `aegis-quantum-ebpf`, or `aegis-quantum-userspace`. Kernel eBPF/XDP filtering, a full Agent Passport service, cascade revocation, and ML-KEM key exchange are not claimed as currently implemented in NEXUS; they are treated as proposed iDEX development work where relevant.

## 1. Brief Summary of the proposed Solution (upto 250 words)

Defence systems face a combined trust failure: legacy RSA/ECC cryptography creates post-quantum migration risk; long-life mission databases are exposed to harvest-now-decrypt-later threats; tactical messages can be spoofed or replayed; and autonomous systems often lack inspectable denial evidence when unsafe actions are blocked.

Syntriass Sovereign Trust Fabric proposes a sovereign, edge-first trust layer for defence software, robotic systems, and mission data flows. The first module is a CBOM Scanner that inventories cryptographic dependencies and identifies quantum-vulnerable trust paths. The remediation stack then adds a Vault Proxy for protected data-at-rest workflows, a Comm Sentinel for signed and replay-resistant tactical messages, an AURA Notary for offline provenance verification, and NEXUS Guard for deny-first execution governance with audit evidence.

The first prototype will demonstrate a full mission-trust workflow: scan a codebase for cryptographic debt, protect a mission record, reject a spoofed or replayed command message, verify a mission packet offline, and produce tamper-evident audit records for accepted and denied actions. Current readiness is software subsystem TRL 3-4. The iDEX exit target is TRL 5 after controlled relevant-environment validation, PQC/hybrid identity hardening, evaluator-witnessed demos, latency characterization, and independent security review.

## 2. Key Technology(s) Used (5-6 keywords)

CBOM, ML-KEM/ML-DSA migration path, ExecutionGuard, AURA provenance, ETK audit, Rust/Python

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | CBOM Scanner | Local cryptographic bill of materials scanner for RSA/ECC/Ed25519/ECDHE/OpenSSL/JWT/SSH/database indicators |
| 2 | Vault Proxy PoC | Policy-controlled data protection proxy with AES-GCM envelope encryption and ML-KEM-ready key wrapping interface |
| 3 | Comm Sentinel PoC | Signed tactical message verifier with replay rejection, revocation cache, and fail-deny behavior |
| 4 | AURA Notary PoC | Offline mission packet provenance verifier with tamper and replay rejection |
| 5 | NEXUS Guard audit bridge | Deny-first execution governance and ETK-compatible audit evidence for high-consequence actions |
| 6 | Integrated demo and evaluation kit | End-to-end iDEX demo, test scripts, evidence package, and deployment documentation |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Defence threat model, CBOM baseline, crypto-risk taxonomy, and data/message packet schemas |
| 2 | 3-4 | CBOM Scanner hardening, reporting dashboard, and CI gate |
| 3 | 5-6 | Vault Proxy PoC with protected write/read workflow and audit record |
| 4 | 7-8 | Comm Sentinel PoC with signed message envelope, replay cache, and revocation denial |
| 5 | 9-10 | AURA Notary and NEXUS Guard integration with offline verification and denial evidence |
| 6 | 11-12 | Integrated demonstration, latency/security review, TRL 5 exit evidence, and final submission pack |
