# Evidence - AURA Trust

## Relevant Code Modules

- AURA provenance and verification concepts
- ETK-compatible audit record path
- `nexus-pcu` proof-carrying unit primitives
- PQC feature path for hybrid signing and verification

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

Relevant existing command:

```bash
cargo test -p nexus-pcu --features pqc pqc -- --nocapture
```

Recorded pre-submission result: 7/7 PQC feature tests passed.

Additional AURA Trust tests should be added or rerun before portal submission:

```bash
cargo test -p aura-trust --test offline_verification -- --nocapture
cargo test -p aura-trust --test provenance_replay -- --nocapture
```

If the package name differs in the repository, the evidence file must be updated with the actual test target before submission.

## Demo Script

1. Generate a signed mission data packet with source, timestamp, nonce, provenance metadata, and payload hash.
2. Verify the packet offline and show acceptance.
3. Modify the payload and show tamper rejection.
4. Replay the original packet outside the accepted sequence/freshness window and show rejection.
5. Export an ETK-compatible audit record for each decision.
6. Show the PCU/PQC migration path for post-quantum signing readiness.

## TRL Caveats

- AURA Trust requires defence hardening before secure information platform claims.
- Offline key lifecycle, revocation, and rotation design are part of the iDEX work.
- PQC evidence is currently unit-level and must be integrated into the mission packet verifier.
- No classified-network or field deployment accreditation is claimed.
