# Syntriass Sovereign Trust Fabric

## Purpose

This is a proposed iDEX Open Challenge wedge that combines the existing NEXUS defence stack with a post-quantum migration entry point.

The product should not be pitched as "quantum immune." The safer and stronger claim is:

> A sovereign, edge-first trust fabric for discovering quantum-vulnerable cryptography, protecting long-life defence data, verifying tactical messages, and producing offline audit evidence.

## Verified 2026 Context

- iDEX official challenge page showed Open Challenge last date: 30 Jun 2026, 11:59 PM.
- The same iDEX page showed ADITI 4, DISC 14, and DRISHTI Challenges closed with last date: 04 May 2026, 5:00 PM.
- PIB confirmed that the Chief of Defence Staff released the Military Quantum Mission Policy Framework on 22 Jan 2026.
- DST's February 2026 PQ migration task force report explicitly references cryptographic bill of materials, hybrid PQC, supplier CBOM requirements, crypto-agility, and "no new classical-only deployments" as migration principles.

Primary references:

- https://idex.gov.in/challenges
- https://www.pib.gov.in/PressReleaseIframePage.aspx?PRID=2217374
- https://dst.gov.in/sites/default/files/Report_TaskForce_PQMigration_4Feb26%20%28v1%29.pdf

## Product Frame

| Component | Tactical Name | Defence Problem |
|---|---|---|
| Cryptographic inventory scanner | CBOM Scanner | Defence teams cannot migrate to PQC if they do not know where RSA, ECC, Ed25519, ECDHE, OpenSSL, JWT, SSH, database, and firmware signing dependencies exist. |
| Database encryption proxy | Vault Proxy | Long-life mission data, logs, personnel records, and backups are exposed to harvest-now-decrypt-later risk if protected only by classical public-key wrapping. |
| Tactical message verifier | Comm Sentinel | Drone, robot, sensor, and C2 messages can be spoofed, replayed, or accepted after credential revocation unless every packet is locally verifiable. |
| Offline provenance layer | AURA Notary | Contested environments require tamper-evident mission packet provenance even when cloud, command links, or PKI infrastructure are unavailable. |
| Governance and denial evidence | AEGIS/NEXUS Guard | High-consequence actions need a deny-first execution gate and an audit packet showing why an unauthorized action was blocked. |

## Exhibit A: NEXUS CBOM Demonstration

Scanner source:

- `tools/cbom_scanner/cbom_scan.py`
- `tools/cbom_scanner/README.md`
- `tests/test_cbom_scanner.py`

Demo command:

```bash
python3 tools/cbom_scanner/cbom_scan.py . \
  --exclude-glob 'docs/**' \
  --exclude-glob 'tools/cbom_scanner/**' \
  --exclude-glob 'tests/test_cbom_scanner.py' \
  --exclude-glob 'gstack-main/**' \
  --exclude-glob '*.md' \
  --exclude-glob '**/*.md' \
  --exclude-glob '*.txt' \
  --exclude-glob '**/*.txt' \
  --exclude-glob '*.html' \
  --exclude-glob '**/*.html' \
  --json /private/tmp/nexus-cbom-source.json \
  --markdown /private/tmp/nexus-cbom-source.md
```

Observed result on 04 Jun 2026:

| Metric | Result |
|---|---:|
| Files scanned | 833 |
| Files skipped | 45 |
| Total findings | 124 |
| Critical findings | 0 |
| High findings | 0 |
| Medium findings | 98 |
| Info findings | 26 |

Finding categories:

| Category | Count |
|---|---:|
| Dependency indicators | 121 |
| TLS/ECDHE indicators | 2 |
| Data-at-rest artifact indicators | 1 |

Top evidence paths:

| Path | What it Shows |
|---|---|
| `Cargo.lock` | Classical crypto dependencies such as `ring`, `ed25519-dalek`, and OpenSSL-related crates. |
| `core/quantum_ria.py` | Legacy ECDSA/P-521 and `cryptography` library paths that require migration review. |
| `nexus-pcu/src/pqc.rs` | Existing hybrid PQC implementation path and testable PQC feature gate. |
| `causalux/Cargo.lock` | Classical signing and OpenSSL dependency inventory in sync/provenance components. |
| `nexus-pcu/Cargo.toml` | PQC feature and dependency declarations for FIPS 203/204 path. |
| `aura_mvp.db` | Database artifact requiring encryption-at-rest and backup-retention review. |

Interpretation:

- The scanner did not find real private-key material in the source/config scan after fixture paths were excluded.
- The scanner did find a concrete PQC migration backlog: Ed25519 identity, OpenSSL/ring dependencies, ECDSA legacy paths, and database artifacts.
- This is exactly the first step demanded by a PQC migration program: discover, classify, prioritize, then migrate.

## Why This Wedge Is Sharper Than a Generic PQC Pitch

Most PQC pitches start with algorithms. This starts with the operational blocker: nobody can migrate what they have not inventoried.

The first demo is therefore not "trust us, we are quantum-safe." It is:

1. Run CBOM Scanner on a real codebase.
2. Show classical crypto dependencies and long-life data exposure.
3. Convert findings into a migration backlog.
4. Demonstrate Vault Proxy, Comm Sentinel, and AURA Notary as remediation modules.
5. Produce signed audit evidence for every denial or migration decision.

## Conservative Readiness Statement

Current readiness should be stated as:

- CBOM Scanner: software MVP, unit-tested, suitable for controlled repository demonstration.
- PQC Identity: prototype hybrid signature path in `nexus-pcu`; not certified cryptography.
- Vault Proxy: proposed next PoC; must prove envelope encryption, key lifecycle, and database compatibility.
- Comm Sentinel: proposed next PoC; must prove packet signing, replay rejection, revocation cache, and failure-deny behavior.
- AURA Notary: prototype-aligned concept; must prove offline packet verification and tamper/replay rejection.

Do not claim TRL 5 for the full trust fabric until it has customer-environment validation, HIL/field testing where applicable, and independent security review.

## Build Plan For iDEX Demo

| Sprint | Deliverable | Evidence |
|---|---|---|
| 1 | CBOM Scanner repository scan and CI gate | Unit tests, sample NEXUS report, before/after false-positive reduction. |
| 2 | Vault Proxy PoC | AES-GCM envelope encryption with ML-KEM-ready key wrapping interface, local policy token, database write/read demo. |
| 3 | Comm Sentinel PoC | Signed tactical message envelope, replay cache, revoked passport denial, local audit log. |
| 4 | AURA Notary PoC | Offline mission packet verification, provenance hash chain, tamper/replay rejection. |
| 5 | Integrated demo | One scenario: scan crypto debt, protect a mission record, reject spoofed tactical message, produce audit evidence. |

## Recommended Submission Wording

Use:

- "Post-quantum migration readiness"
- "Hybrid/PQC-capable migration path"
- "Crypto-agility"
- "Cryptographic bill of materials"
- "Tamper-evident audit evidence"
- "Offline tactical verification"

Avoid:

- "Quantum immune"
- "Unbreakable"
- "Unforgeable"
- "Certified PQC"
- "TRL 5 full system" before external validation

