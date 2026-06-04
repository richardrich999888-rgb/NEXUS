# Evidence - CAUSALUX Contested Sync

## Relevant Code Modules

- CAUSALUX causal execution and state concepts
- USO ordered evidence path
- VECTRA state tracking
- `nexus-sync`
- Compression tests and transfer measurement utilities

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

Required pre-submission commands to confirm before portal upload:

```bash
cargo test -p nexus-sync -- --nocapture
cargo test -p nexus-pcu uso -- --nocapture
cargo test -p causalux -- --nocapture
cargo test -p vectra -- --nocapture
```

If any package names differ in the repository workspace, this evidence file must be updated with the actual test targets and results.

## Demo Script

1. Start three software nodes with shared initial mission state.
2. Disconnect nodes and apply independent local updates.
3. Reconnect nodes under constrained transfer settings.
4. Exchange compressed deltas through the sync layer.
5. Merge state deterministically and display conflict handling.
6. Reject stale or replayed updates.
7. Export provenance and ordered audit evidence for accepted updates.

## TRL Caveats

- Contested-network behavior is simulation-first.
- Package-level test names must be verified against the final repository workspace before submission.
- Field bandwidth, packet loss, and adversarial network behavior require hardware/network-in-loop validation.
- Mission-specific merge policies require evaluator review.
