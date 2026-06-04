# Evidence - NEXUS Guard

## Relevant Code Modules

- `nexus-executor`
- ExecutionGuard policy path
- ETK-compatible execution audit path
- TELOS consequence accounting
- `nexus-pcu` hybrid Ed25519 plus ML-DSA PQC feature path

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

## Official Problem Alignment Evidence

- Open Challenge package: `docs/idex-open-challenge-2026/01-nexus-guard/`
- ADITI 4 adjacency: autonomy and robotic-platform problem areas needing execution assurance.
- DISC 14 adjacency: autonomy/UAS and multi-agent problem areas needing guarded command paths.
- PS24 boundary: not a space-domain training, surveillance, or operations deliverable.

Relevant command:

```bash
cargo test -p nexus-executor --test red_team_execution -- --nocapture
cargo test -p nexus-pcu --features pqc pqc -- --nocapture
```

Recorded pre-submission result: 10/10 red-team execution tests passed.

## Demo Script

1. Submit an unauthorized protected action.
2. Show ExecutionGuard denial before action execution.
3. Verify that no success proof or execution cache artifact is generated for the denied path.
4. Submit an authorized protected action with valid policy context.
5. Show allowed execution and ETK-compatible audit evidence.
6. Display TELOS consequence-budget update for the allowed action.

## TRL Caveats

- Current evidence is software subsystem validation only.
- TRL 5 is a 12-month exit target, not a current claim.
- PQC must be integrated into selected identity/audit records before the TRL 5 claim.
- No physical platform field validation claimed.
- Latency characterization must be repeated under the target defence runtime.
- Mission policy profiles require evaluator review before operational relevance can be claimed.
