# iDEX Open Challenge 2026 Proposal Packages

Applicant: SYNTRIASS Labs Private Limited

This folder prepares seven separate iDEX Open Challenge proposal packages for NEXUS-derived defence products. The submission strategy is to file the strongest five first and keep applications six and seven ready as reserve or follow-on submissions.

Readiness language is intentionally conservative: software subsystem TRL 3-4, prototype validation proposed, simulation-first demonstration, and hardware-in-loop validation planned. These packages should not claim field qualification, physical military hardware validation, or operational deployment.

## Submission Matrix

| Priority | Package | Submission wave | Defence problem | Primary evidence |
| --- | --- | --- | --- | --- |
| 1 | NEXUS Guard | Wave 1 | Prevent unauthorized autonomous execution | `nexus-executor`, ExecutionGuard, ETK, TELOS red-team tests |
| 2 | BioShield Swarm | Wave 1 | Detect rogue or compromised swarm agents | `multi-asi-immune` tests and swarm compromise simulation |
| 3 | AGP-OS Robotics Safety Layer | Wave 1 | Govern ROS2 robots with RTOS scheduling and resource limits | AGP, ROS2 simulation tests, RTOS and resource-control tests |
| 4 | AURA Trust | Wave 1 | Verify mission data, telemetry, orders, and intelligence provenance offline | AURA provenance path, ETK audit model, PCU/PQC proof path |
| 5 | Cyber Immune SOAR | Wave 1 | Autonomous cyber defence, quarantine, and audit actions | AGP immunity, governance bridge, anomaly and multi-agent governance tests |
| 6 | PQC Defence Identity | Reserve | Quantum-safe identity and signing for defence agents/devices | `nexus-pcu` hybrid Ed25519 plus ML-DSA tests |
| 7 | CAUSALUX Contested Sync | Reserve | Low-bandwidth disconnected tamper-evident state sync | CAUSALUX, USO, VECTRA, `nexus-sync`, compression tests |

## Shared Evidence References

- [Pre-submission test report](../IDEX_PRE_SUBMISSION_TEST_REPORT.md)
- [Defence capability audit](../DEFENCE_CAPABILITY_AUDIT.md)
- [iDEX open challenge pitch](../IDEX_OPEN_CHALLENGE_PITCH.md)

## Portal Checklist

- Insert final company registration identifiers, DPIIT recognition number, address, and authorized signatory details before upload.
- Export each `PPS`, `PTS`, `PSB`, `AD`, `evidence`, and `budget` document to PDF and keep each file under the portal size limit.
- Keep the five Wave 1 packages independent in title, problem statement, demo, deliverables, and evidence.
- Run package-specific tests again immediately before portal submission and paste fresh command output summaries into `evidence.md`.
- Conduct legal, export-control, and security review before any external defence submission.
