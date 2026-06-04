# Evidence - Cyber Immune SOAR

## Relevant Code Modules

- AGP immune bridge
- Immune system and unified immune modules
- Multi-agent governance bridge
- Reputation and anomaly scoring modules
- `nexus-pcu` hybrid Ed25519 plus ML-DSA PQC feature path

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

## Official Problem Alignment Evidence

- Open Challenge package: `docs/idex-open-challenge-2026/05-cyber-immune-soar/`
- DISC 14 adjacency: network monitoring, secure information exchange, and cyber deception problem areas needing bounded response automation.
- ADITI 4 adjacency: EW/OSINT-style high-volume event analysis needing anomaly detection and policy-limited response.
- PS24 boundary: not a space-domain training, surveillance, or operations deliverable.

Relevant test files:

```bash
pytest agp-core/tests/test_immune_bridge.py
pytest agp-core/tests/test_immune_system.py
pytest agp-core/tests/test_unified_immune.py
pytest agp-core/tests/test_multi_agent_governance.py
cargo test -p nexus-pcu --features pqc pqc -- --nocapture
```

The shared report records AGP immune, governance, and anomaly-related tests as part of the software validation scope. Before portal submission, rerun these commands and paste fresh command summaries here.

## Demo Script

1. Load simulated cyber event feed.
2. Inject suspicious process, anomalous service, compromised agent, and repeated policy violation events.
3. Convert events into immune threat signals with severity and confidence.
4. Apply governance policy to select observe, throttle, quarantine, or escalate.
5. Update reputation for the affected entity.
6. Export audit trail showing event, signal, decision, action, and result.

## TRL Caveats

- Current evidence is software simulation validation only.
- TRL 5 is a 12-month exit target, not a current claim.
- PQC must be integrated into selected event/audit records before the TRL 5 claim.
- Cyber events are simulated in phase one.
- Operational telemetry adapters must be built and approved before live SOC integration.
- Autonomous response policies require evaluator review.
- False-positive and false-negative thresholds require calibration with realistic datasets.
