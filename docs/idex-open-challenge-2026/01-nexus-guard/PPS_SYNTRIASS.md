# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

NEXUS Guard: Governed Execution Layer for Autonomous Defence Systems

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## Official Problem Alignment and PS24 Boundary

NEXUS Guard aligns with official autonomy demand signals in ADITI 4 and DISC 14 where autonomous platforms, UAS modules, robot teams, and mission software need stronger execution assurance. It is not a duplicate of the already submitted ADITI PS24 space-domain application: this Open Challenge package is scoped to pre-execution authorization, denial evidence, and audit records for protected autonomous actions.

## 1. Brief Summary of the proposed Solution (upto 250 words)

Autonomous defence systems can move from sensing to action faster than conventional command review, logging, or after-the-fact audit can control. The critical gap is whether any high-consequence action can execute without authorization, policy approval, and accountable evidence.

NEXUS Guard proposes a governed execution layer for autonomous software agents, robotic control stacks, and mission decision services. The product places a mandatory ExecutionGuard in front of protected actions. A command that does not satisfy authorization, policy, context, and consequence constraints is denied before execution. Denied commands do not generate success proofs or execution cache artifacts. Approved commands generate ETK-compatible audit evidence and TELOS consequence accounting so reviewers can reconstruct why execution was allowed.

The first prototype will demonstrate a red-team scenario where an unauthorized autonomous action is blocked, no proof/cache is generated for the denied path, and an authorized action produces a verifiable execution record. Current readiness is software subsystem TRL 3-4. The iDEX exit target is TRL 5 after PQC-enabled identity/audit integration, relevant-environment validation, latency characterization, and evaluator-witnessed mission-software-in-loop or hardware-in-loop testing.

## 2. Key Technology(s) Used (5-6 keywords)

ExecutionGuard, TELOS, ETK audit, hybrid Ed25519/ML-DSA, Rust, policy engine

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | Guarded execution runtime | Mandatory pre-execution policy gate for protected autonomous actions |
| 2 | Denial-path test harness | Red-team demo showing denied execution with no success proof/cache |
| 3 | ETK audit exporter | Verifiable audit record for allowed actions |
| 4 | PQC identity/audit profile | Hybrid Ed25519 plus ML-DSA verification path for selected audit records |
| 5 | TELOS consequence trace | Consequence-budget accounting for high-impact commands |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Threat model, protected-action schema, and policy profile |
| 2 | 3-5 | ExecutionGuard runtime and integration adapters |
| 3 | 6-8 | ETK audit and TELOS consequence integration |
| 4 | 9-10 | Red-team denial demo and latency characterization |
| 5 | 11-12 | PQC verification, TRL 5 exit evidence, final acceptance tests, documentation, and evaluation build |
