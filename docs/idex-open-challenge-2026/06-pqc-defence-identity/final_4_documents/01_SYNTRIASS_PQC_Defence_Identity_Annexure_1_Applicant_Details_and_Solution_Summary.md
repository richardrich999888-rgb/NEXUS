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

Purpose: technical architecture and implementation approach for PQC Defence Identity.

Contents:

- Hybrid Ed25519 plus ML-DSA signature envelope.
- Feature-gated PQC implementation status.
- Public-key bundle and verification policy.
- PCU identity, delegation, and proof integration path.
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
| Technology / Platform Name | PQC Defence Identity |
| Intended Defence End User | Indian Armed Forces technology teams, DRDO cryptographic-transition evaluators, secure device identity teams, autonomous-system identity teams, command-and-control assurance teams, and defence integrators planning post-quantum migration. |
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

PQC Defence Identity: Hybrid Classical and Post-Quantum Signing for Defence Agents, Devices, and Proof Packets

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Defence identity teams | Plan migration from classical-only signing to hybrid identity records. |
| DRDO cryptographic-transition evaluators | Inspect implementation evidence for Ed25519 plus ML-DSA feature path. |
| Autonomous-system assurance teams | Bind agent/device identity to computation and execution proof records. |
| Secure information teams | Prepare packet and proof verification for long-life systems. |
| C2 system integrators | Evaluate public-key bundle, fallback policy, and audit behavior before deployment. |
| Procurement and audit panels | Review test output, source paths, feature gates, and hardening caveats. |

```{=typst}
#pagebreak()
```

## 4. A. Brief Summary of Proposed Solution

Defence systems are entering a transition period where classical digital signatures remain necessary for interoperability, while long-life systems must prepare for post-quantum cryptographic risk. Autonomous agents, sensors, robotic devices, proof packets, and secure information flows need identity and signing mechanisms that can migrate without breaking existing verification paths.

PQC Defence Identity proposes a hybrid identity and signing prototype for defence software agents, edge devices, and proof-carrying computation records. It combines current Ed25519 verification with a feature-gated ML-DSA-65 post-quantum verification path inside the NEXUS Portable Computation Unit layer. The objective is not to claim cryptographic accreditation in phase one. The objective is to demonstrate a practical hybrid signing envelope, public-key bundle, tamper/fallback behavior, key metadata handling, and audit evidence that can support a future approved cryptographic profile.

The iDEX prototype will demonstrate hybrid key generation, hybrid signatures, classical verification, ML-DSA verification under the `pqc` feature, public-key bundle verification, tamper rejection when both signature components are invalid, and integration planning for PCU identity and execution proof records.

Current evidence is software-subsystem TRL 3-4. Network-wide PQC enforcement, HSM-backed key lifecycle, certified cryptographic profile approval, and operational deployment are proposed work packages.

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Classical-only identity risk | Long-life devices may outlive classical signature assumptions. | Hybrid Ed25519 plus ML-DSA signature envelope. |
| Migration disruption | Sudden cryptographic replacement can break existing systems. | Public-key bundle supports classical and PQC paths. |
| Tampered signatures | Adversary may alter one signature component. | Tests verify behavior when classical and PQC components are modified. |
| Unclear verification policy | Different mission systems may require different acceptance rules. | Explicit verifier policy: classical, PQC, hybrid, warning, or migration-required. |
| Weak proof identity binding | Execution proof must identify the node/device producing it. | PCU NodeAttestation and IdentityContext integration path. |
| Key lifecycle complexity | Provisioning, rotation, revocation, and compromise handling must be defined. | 12-month work package for lifecycle and evaluator-approved profile. |

```{=typst}
#pagebreak()
```

## 6. B. Key Technologies Used

- Ed25519 signatures
- ML-DSA-65 feature path
- Hybrid signature envelope
- Public key bundle
- PCU identity context
- Execution proof attestation

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| Hybrid Signing Envelope | Defines payload, classical signature, optional PQC signature, version, and verification metadata. |
| Public Key Bundle Format | Carries Ed25519 and optional ML-DSA public key material with versioning. |
| Verification CLI/SDK | Verifies classical, PQC, and hybrid paths under explicit policy classes. |
| Tamper/Fallback Test Suite | Demonstrates behavior when one or both signature components are modified. |
| PCU Identity Adapter | Maps hybrid identity into PCU IdentityContext and execution proof records. |
| Key Lifecycle Plan | Defines provisioning, revocation, rotation, expiry, compromise response, and offline bundle update. |
| Audit Evidence Exporter | Records verification path, key ID, signature mode, result, and reason code. |
| Evaluation Report | Provides test output, artifact locations, source paths, caveats, and deployment roadmap. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Cryptographic transition threat model | Threat model, acceptance modes, and evaluator-approved assumptions. |
| Phase 2 | Month 3 to Month 4 | Hybrid envelope specification | Packet/proof format, versioning, metadata, and public-key bundle schema. |
| Phase 3 | Month 5 to Month 6 | Verifier CLI/SDK | Classical, PQC, hybrid, warn, and reject policy paths. |
| Phase 4 | Month 7 to Month 8 | PCU identity integration | IdentityContext and ExecutionProof adapters. |
| Phase 5 | Month 9 | Key lifecycle design | Provisioning, expiry, rotation, revocation, and offline trust bundle plan. |
| Phase 6 | Month 10 | Adversarial tests | Signature tamper, key mismatch, stale bundle, downgrade, and missing-component scenarios. |
| Phase 7 | Month 11 | Evaluator packaging | Runbooks, source map, audit export, and sample hybrid proof packets. |
| Phase 8 | Month 12 | Final demonstration | iDEX demo package, test report, and cryptographic-profile hardening roadmap. |

## 9. E. Readiness Position

Fresh evidence includes `cargo test -p nexus-pcu --features pqc pqc -- --nocapture` reporting 7 passed and 0 failed.

Submission boundary: software-subsystem prototype only. Certified cryptographic accreditation, network-wide enforcement, HSM deployment, secure element provisioning, and defence key authority approval remain proposed iDEX work packages.
