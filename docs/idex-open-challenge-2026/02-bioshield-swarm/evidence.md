# Evidence - BioShield Swarm

## Relevant Code Modules

- `multi-asi-immune`
- Agent reputation and defection scoring components
- Threat memory and anomaly response modules
- Governance bridge for quarantine decisions
- `nexus-pcu` hybrid Ed25519 plus ML-DSA PQC feature path

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

## Official Problem Alignment Evidence

- Open Challenge package: `docs/idex-open-challenge-2026/02-bioshield-swarm/`
- ADITI 4 adjacency: counter-UAS and autonomy problem areas needing compromised-node detection.
- DISC 14 adjacency: drone-management, C-UAS, and multi-agent UAS problem areas needing swarm trust scoring.
- PS24 boundary: not a space-domain training, surveillance, or operations deliverable.

Relevant command:

```bash
cargo test -p multi-asi-immune --lib --tests -- --nocapture
cargo test -p nexus-pcu --features pqc pqc -- --nocapture
```

Recorded pre-submission result: 68/68 tests passed, with one ignored doctest noted in the shared report.

## Demo Script

1. Start a simulated swarm with baseline cooperative agents.
2. Inject one compromised node with spoofed coordination or defection behavior.
3. Show anomaly conversion into immune threat signals.
4. Show reputation decay and defection score increase.
5. Trigger graded quarantine under policy threshold.
6. Replay the event using threat memory and audit trace.

## TRL Caveats

- Current evidence is software simulation validation only.
- TRL 5 is a 12-month exit target, not a current claim.
- PQC must be integrated into selected node identity / signed-report records before the TRL 5 claim.
- Thresholds require calibration with service-specific telemetry.
- No physical swarm hardware test is claimed in the current evidence.
- Electronic-warfare degradation must be added as a controlled simulation scenario during the iDEX effort.
