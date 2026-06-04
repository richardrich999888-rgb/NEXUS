# Evidence - PQC Defence Identity

## Relevant Code Modules

- `nexus-pcu`
- Hybrid proof-carrying unit path
- Ed25519-compatible classical verification path
- ML-DSA-compatible post-quantum feature path

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

Relevant command:

```bash
cargo test -p nexus-pcu --features pqc pqc -- --nocapture
```

Recorded pre-submission result: 7/7 PQC feature tests passed.

## Demo Script

1. Generate a proof packet signed with the classical path.
2. Generate a proof packet with the post-quantum path enabled.
3. Verify both paths and show policy result.
4. Modify packet contents and show tamper rejection.
5. Disable one verification path and show configured fallback behavior.
6. Export audit evidence with key metadata and verification status.

## TRL Caveats

- PQC evidence is currently unit-level.
- Formal cryptographic accreditation is not claimed.
- Key provisioning, revocation, and lifecycle management must be designed under the iDEX effort.
- Approved cryptographic profile alignment is required before operational defence deployment.
