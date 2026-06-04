# Evidence - Syntriass Sovereign Trust Fabric

## Relevant Code Modules

| Module | Repository Location | Evidence Role | Status |
| --- | --- | --- |
| CBOM Scanner | `tools/cbom_scanner/cbom_scan.py` | Cryptographic bill-of-materials discovery | Implemented MVP |
| CBOM tests | `tests/test_cbom_scanner.py` | Unit tests for detection, reporting, exclusions, and false-positive control | Implemented |
| CBOM documentation | `tools/cbom_scanner/README.md` | Usage, boundaries, and conservative claims | Implemented |
| PQC feature path | `nexus-pcu/src/pqc.rs` | Hybrid Ed25519 plus ML-DSA-compatible migration path | Implemented behind `pqc` feature |
| ML-KEM dependency | `nexus-pcu/Cargo.toml` | FIPS 203 dependency declaration | Dependency present; API not implemented |
| Encrypted envelope | `causalux/src/envelope.rs` | AES-256-GCM envelope for CRDT operations | Implemented for Causalux scope |
| Execution governance | `nexus-executor` | Deny-before-execute guard layer | Implemented foundation |
| Red-team execution tests | `nexus-executor` red-team tests | Unauthorized execution denial evidence | Implemented tests |
| AURA/ETK/PCU paths | AURA, ETK, and PCU-related modules | Provenance, proof, and audit concepts | Prototype/conceptual integration |

## Explicit Non-Claims

The following items were referenced in external brainstorm material but are not present in this NEXUS workspace and are not claimed as implemented in this package:

| Not Claimed As Implemented | Reason |
| --- | --- |
| `aip-service/src/passport.rs` Agent Passport service | Path is not present in NEXUS |
| `aip-standalone/airon-verify` verifier CLI | Path is not present in NEXUS |
| `aegis-quantum-ebpf` or XDP packet filter | Path is not present in NEXUS |
| Full ML-KEM encapsulation/decapsulation API | `fips203` dependency is present, but no KEM functions were found in NEXUS |
| Cascade revocation / BFS revocation | Not implemented in current NEXUS identity path |
| Full device passport schema with firmware hash and hardware fingerprint | Proposed extension beyond current `PrincipalId` and `IdentityContext` |
| Certified cryptographic accreditation | Not in current evidence scope |

## Existing Test Evidence

Shared report:

- `docs/IDEX_PRE_SUBMISSION_TEST_REPORT.md`

Relevant commands:

```bash
python3 -m unittest tests/test_cbom_scanner.py
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
cargo test -p nexus-pcu --features pqc pqc -- --nocapture
cargo test -p nexus-executor --test red_team_execution -- --nocapture
```

## Recorded CBOM Scanner Evidence

CBOM Scanner unit tests on 04 Jun 2026:

```text
Ran 6 tests
OK
```

Clean NEXUS source/config scan on 04 Jun 2026:

| Metric | Result |
| --- | ---: |
| Files scanned | 833 |
| Files skipped | 45 |
| Total findings | 124 |
| Critical findings | 0 |
| High findings | 0 |
| Medium findings | 98 |
| Info findings | 26 |

Finding categories:

| Category | Count |
| --- | ---: |
| Dependency indicators | 121 |
| TLS/ECDHE indicators | 2 |
| Data-at-rest artifact indicators | 1 |

Top evidence locations from the scan:

| Path | Meaning |
| --- | --- |
| `Cargo.lock` | Classical crypto dependencies such as `ring`, `ed25519-dalek`, OpenSSL-related crates |
| `core/quantum_ria.py` | Legacy ECDSA/P-521 and Python `cryptography` paths requiring migration review |
| `nexus-pcu/src/pqc.rs` | Hybrid PQC implementation path and testable feature gate |
| `causalux/Cargo.lock` | Classical signing and OpenSSL dependency inventory in sync/provenance components |
| `nexus-pcu/Cargo.toml` | PQC feature/dependency declarations |
| `aura_mvp.db` | Database artifact requiring encryption-at-rest and backup-retention review |

## Demo Script

1. Run CBOM Scanner against a controlled repository or NEXUS source/config profile.
2. Show generated report with classical crypto dependencies and migration priorities.
3. Select one mission-data artifact and protect it through Vault Proxy PoC.
4. Send a valid tactical message through Comm Sentinel and show acceptance.
5. Replay or tamper with the same message and show rejection.
6. Verify an AURA mission packet offline.
7. Modify the packet payload or nonce and show tamper/replay rejection.
8. Attempt an unauthorized high-consequence action through NEXUS Guard.
9. Show deny-first behavior and ETK-compatible audit evidence.
10. Export final evidence bundle for evaluator review.

## TRL Caveats

- Current readiness is software subsystem TRL 3-4.
- TRL 5 is an exit target, not a current full-system claim.
- CBOM Scanner is an MVP static scanner; findings require human review.
- Vault Proxy and Comm Sentinel are proposed PoCs to be built and integrated under iDEX.
- `nexus-pcu` PQC evidence is unit-level and not cryptographic accreditation.
- ML-DSA hybrid signatures exist behind the `pqc` feature; ML-KEM key exchange must still be implemented before key-exchange claims.
- Operational key management, revocation, provisioning, HSM/KMS integration, and certification remain future work.
- No claim is made of classified-network deployment, field qualification, or certified secure information platform status.

## Reviewer Verification Checklist

| Reviewer Question | Where To Check |
| --- | --- |
| Does the scanner run locally? | `tools/cbom_scanner/cbom_scan.py` |
| Are there tests for scanner behavior? | `tests/test_cbom_scanner.py` |
| Are PQC claims bounded? | `nexus-pcu/src/pqc.rs`, `evidence.md`, and TRL caveats |
| Does the proposal avoid "quantum immune" claims? | `PPS_SYNTRIASS.md`, `PTS_SYNTRIASS.md`, `PSB_SYNTRIASS.md` |
| Is the demo realistic for phase one? | Demo script and 12-month milestones |
