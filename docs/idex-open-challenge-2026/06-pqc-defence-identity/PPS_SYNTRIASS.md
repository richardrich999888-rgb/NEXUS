# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

PQC Defence Identity: Hybrid Classical And Post-Quantum Signing For Defence Agents And Devices

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## 1. Brief Summary of the proposed Solution (upto 250 words)

Defence systems are entering a transition period where classical digital signatures remain necessary for interoperability, while long-life systems must prepare for quantum-era cryptographic risk. Autonomous agents, sensors, robotic devices, and secure information packets need identity and signing mechanisms that can support migration without breaking current verification flows.

PQC Defence Identity proposes a hybrid identity and signing prototype for defence software agents and edge devices. It combines classical Ed25519-style verification with a post-quantum ML-DSA-compatible verification path through the NEXUS proof-carrying unit layer. The objective is not to claim full cryptographic accreditation in phase one. The objective is to demonstrate a practical hybrid signing envelope, tamper rejection, fallback behavior, key metadata handling, and audit evidence that can support a future approved cryptographic profile.

The demonstration will create hybrid-signed proof packets, verify the classical and post-quantum paths, reject tampered packets, and show policy behavior when only one path verifies. This package is recommended as a reserve or follow-on application because it is strategically important but narrower than the first five operational packages.

## 2. Key Technology(s) Used (5-6 keywords)

Ed25519, ML-DSA, hybrid signatures, proof-carrying units, key lifecycle, audit evidence

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | Hybrid signing envelope | Packet format carrying classical and PQC verification metadata |
| 2 | Verification CLI/SDK | Verifies classical, PQC, and policy result paths |
| 3 | Tamper/fallback tests | Demonstrates rejection and policy-visible fallback behavior |
| 4 | Key lifecycle plan | Defines provisioning, revocation, rotation, and compromise assumptions |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Cryptographic profile and threat model |
| 2 | 3-5 | Hybrid envelope and verifier prototype |
| 3 | 6-8 | Policy behavior and key lifecycle design |
| 4 | 9-10 | Agent/device/packet signing examples |
| 5 | 11-12 | Final tests, documentation, and evaluation build |
