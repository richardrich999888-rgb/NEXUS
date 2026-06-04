# Annexure - 3

Preferably on Company's letterhead (if available)

## 1. Technology Advantages (upto 150 words)

- Provides a migration path from classical signatures to post-quantum verification.
- Allows dual-path verification behavior to be tested before full deployment.
- Captures key metadata and verification policy in an inspectable envelope.
- Supports tamper-evident proof packets for agents and devices.

## 2. Product Advantages (upto 150 words)

- Can be delivered as a signing CLI, verification SDK, or identity envelope library.
- Supports integration with NEXUS Guard, AURA Trust, and CAUSALUX packets.
- Provides clear unit and integration test targets for cryptographic migration.
- Allows policy control over classical-only, hybrid, and PQC-required states.

## 3. Commercial Advantages (upto 150 words)

- Dual-use relevance to critical infrastructure, industrial IoT, supply-chain provenance, and regulated identity systems.
- Product options include signing SDK, embedded verification library, and migration assessment tool.
- Supports organizations preparing for post-quantum transition while maintaining current interoperability.

## 4. Capabilities & Competencies (upto 250 words)

- SYNTRIASS Labs is developing NEXUS proof-carrying unit and cryptographic verification paths.
- Relevant components include `nexus-pcu`, hybrid signatures, Ed25519-compatible verification, and ML-DSA-compatible PQC feature paths.
- Existing evidence includes `cargo test -p nexus-pcu --features pqc pqc -- --nocapture` from the shared pre-submission report.
- The iDEX effort will focus on envelope specification, verifier implementation, key lifecycle, revocation, and cryptographic profile alignment.
