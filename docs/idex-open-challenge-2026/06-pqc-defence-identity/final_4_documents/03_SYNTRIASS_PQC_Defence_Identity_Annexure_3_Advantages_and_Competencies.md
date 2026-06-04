# IDEX OPEN CHALLENGE SUBMISSION

# Annexure-3

Advantages, competencies, and benefits

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

# Advantages and Competencies

## 1. Defence Benefits

| Benefit | Defence Value |
| --- | --- |
| Transition-safe identity | Allows classical and PQC verification paths to coexist during migration. |
| Compatibility preservation | Ed25519 path remains usable for current systems. |
| PQC readiness path | ML-DSA-65 path is available under the explicit `pqc` feature. |
| Policy-visible fallback | Verifier can record whether classical, PQC, or both paths succeeded. |
| Proof identity binding | PCU identity and NodeAttestation provide integration points for proof packets. |
| Tamper evidence | Feature-gated test exercises behavior when classical and PQC components are modified. |
| Conservative deployment path | Prototype does not claim certified cryptographic accreditation. |

## 2. Technical Advantages

PQC Defence Identity is strongest as a migration and verification layer, not as a claim of immediate operational cryptographic certification. It focuses on the defence question: how can long-life systems start carrying post-quantum identity evidence without breaking current verification and audit workflows.

| Technical Advantage | Evidence |
| --- | --- |
| Hybrid signature type exists | `HybridSignature` in `nexus-pcu/src/pqc.rs`. |
| Hybrid keypair path exists | `HybridKeyPair::generate()` and `sign()`. |
| Public-key bundle exists | `PublicKeyBundle` supports classical and optional PQC key material. |
| PQC feature tests pass | `cargo test -p nexus-pcu --features pqc pqc -- --nocapture` reports 7/7. |
| Reviewer traceability | Annexure 4 includes screenshots, source paths, output logs, and artifact maps. |

```{=typst}
#pagebreak()
```

## 3. Product and Commercial Potential

| Market Segment | Potential Productization Path |
| --- | --- |
| Defence device identity | Hybrid key bundles for agents, sensors, gateways, and robotic nodes. |
| Secure proof packets | Hybrid-signed execution proofs and mission data verification records. |
| C2 modernization | Migration envelope for command systems that need a phased cryptographic transition. |
| Critical infrastructure | Dual-use hybrid identity for grid, maritime, disaster response, and industrial telemetry. |
| Cyber ranges and testbeds | PQC migration and downgrade-scenario testing environment. |

## 4. Team Competencies

| Competency | Repository Evidence |
| --- | --- |
| Hybrid cryptographic engineering | `nexus-pcu/src/pqc.rs` and feature-gated tests. |
| Capability-based identity | `nexus-pcu/src/identity.rs`. |
| Proof-carrying computation | `nexus-pcu/src/pcu.rs` and `nexus-pcu/src/proof.rs`. |
| Classical crypto baseline | `nexus-pcu/src/crypto.rs`. |
| Evidence packaging | Annexure 4 includes source screenshots, test output, repo links, and artifact maps. |

## 5. Why iDEX Support Is Required

The remaining work is transition hardening: evaluator-approved cryptographic profile, key lifecycle, revocation bundles, downgrade policy, packet/proof integration, HSM or secure element adapter, audit export, sample device identities, and red-team migration scenarios.

## 6. Readiness Caveat

The current package should be evaluated as a software-subsystem prototype. It does not claim certified cryptographic module status, operational key authority approval, HSM-backed deployment, secure element provisioning, or network-wide PQC enforcement. Those are proposed milestones under the 12-month iDEX work plan.
