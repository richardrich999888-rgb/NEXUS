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

Purpose: technical architecture and implementation approach for AURA Trust.

Contents:

- Mission packet schema.
- Offline verification flow.
- Tamper, replay, freshness, and source checks.
- ETK-compatible audit record path.
- PCU/PQC migration path.
- Secure information platform hardening caveats.

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
| Technology / Platform Name | AURA Trust |
| Intended Defence End User | Indian Armed Forces, DRDO secure information evaluators, ISR data teams, command-and-control assurance teams, cyber/AI trust teams, and defence system integrators handling mission data provenance. |
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

AURA Trust: Offline Provenance Verification for Mission Data, Telemetry, Orders, and Intelligence

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Mission data and ISR teams | Verify whether a report is signed, current, source-bound, and unmodified. |
| Command-and-control assurance teams | Reject replayed, stale, tampered, or unknown-source packets during disconnected operations. |
| DRDO secure information evaluators | Test packet provenance, audit records, and PQC migration path. |
| Cyber defence teams | Identify modified payloads, replayed nonces, stale packets, and invalid source identities. |
| Defence system integrators | Add offline verification to existing mission data pipelines without requiring constant network trust. |
| Procurement and audit panels | Review test output, source paths, proof records, and hardening caveats. |

```{=typst}
#pagebreak()
```

## 4. A. Brief Summary of Proposed Solution

Defence operations depend on mission data, telemetry, orders, sensor reports, and intelligence packets that may move through disconnected or contested environments. The risk is not only interception. A packet may be modified, replayed, stale, unauthorised, or stripped of source context before it reaches an operator or downstream system.

AURA Trust proposes an offline provenance and verification layer for defence information packets. Each packet carries source identity, payload hash, timestamp, nonce, sequence number, provenance metadata, policy class, and signature material. The verifier checks source trust, freshness, replay status, sequence window, payload integrity, signature validity, and emits an ETK-compatible audit record for every accept or reject decision.

The 12-month iDEX prototype will demonstrate signed mission packet acceptance, tamper rejection, replay rejection, stale-packet rejection, unknown-source rejection, audit export, and a PCU/PQC migration path. Current evidence is software-subsystem TRL 3-4. It is not claimed as an accredited secure information platform, classified network product, or field-deployed defence system. Defence hardening, key lifecycle, revocation bundles, and deployment accreditation are proposed work packages.

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Tampered mission data | Modified telemetry or intelligence can mislead operators and systems. | Payload hash and signature verification. |
| Replay attack | Old orders or sensor packets can be resent in a different context. | Nonce memory and sequence-number checks. |
| Stale packet acceptance | Disconnected systems may accept outdated data. | Timestamp freshness profile. |
| Unknown source | Data may come from an untrusted or spoofed producer. | Offline public-key trust store. |
| Weak auditability | Operators need reason codes for accept/reject decisions. | ETK-compatible audit record with packet hash, policy ref, result, and reason. |
| Quantum transition risk | Long-lived defence data requires a migration path. | PCU hybrid Ed25519 plus ML-DSA feature path. |

```{=typst}
#pagebreak()
```

## 6. B. Key Technologies Used

- Offline packet verification
- Ed25519 source signatures
- Replay and freshness checking
- Provenance-root hashing
- ETK-compatible audit records
- PCU/PQC migration path

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| Mission Packet Schema | Defines source, payload hash, timestamp, nonce, sequence, provenance, policy, and signature fields. |
| Offline Verifier CLI | Verifies packets without continuous access to a central service. |
| Replay and Freshness Engine | Rejects stale packets, repeated nonces, and old sequence numbers. |
| Tamper Detection Tests | Demonstrates rejection when payload is modified after signing. |
| ETK Audit Exporter | Emits decision records containing packet hash, payload hash, provenance root, policy reference, result, and reason. |
| PCU/PQC Migration Path | Maps packet signing and verification toward NEXUS PCU hybrid-signature support. |
| Evidence Dashboard Prototype | Shows accepted/rejected packet counts, reason codes, and audit hashes. |
| Validation Report | Provides test output, artifact locations, hardening gaps, and deployment roadmap. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Secure information threat model | Packet schema, source model, replay model, and policy classes. |
| Phase 2 | Month 3 to Month 4 | Offline verifier prototype | CLI/API verifier for signature, payload hash, source, freshness, and replay checks. |
| Phase 3 | Month 5 to Month 6 | Audit and provenance export | ETK-compatible audit record, reason codes, provenance root, and test dataset. |
| Phase 4 | Month 7 to Month 8 | Key lifecycle design | Offline trust bundles, revocation bundle format, rotation policy, and field sync plan. |
| Phase 5 | Month 9 | PQC migration integration | PCU hybrid-signature adapter and ML-DSA feature-path validation. |
| Phase 6 | Month 10 | Adversarial packet tests | Tamper, replay, stale, unknown-source, and sequence rollback scenarios. |
| Phase 7 | Month 11 | Secure packaging | Evaluator runbook, demo scripts, audit export, and deployment-hardening checklist. |
| Phase 8 | Month 12 | Final demonstration | iDEX demo package, test report, and hardening roadmap. |

## 9. E. Readiness Position

Fresh evidence includes AURA Trust offline packet harness 8/8, `nexus-pcu` PQC feature tests 7/7, and `nexus-etk` tests 9/9.

Submission boundary: software-subsystem prototype only. Classified network integration, secure information platform accreditation, key lifecycle approval, HSM integration, and field deployment remain proposed iDEX work packages.
