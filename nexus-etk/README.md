# nexus-etk — Execution Truth Kernel (ETK) v1.0

Passive, append-only system that generates verifiable cryptographic proofs that a specific execution occurred under specific constraints. **Truth capture only** — not enforcement, scheduling, intelligence, or policy engines.

## Schema lock

- **ExecutionEvent v1.0**: `event_id`, `execution_id`, `sequence_number`, `timestamp_utc`, `actor_id`, `workload_id`, `execution_context`, `resource_class`, `jurisdiction_code`, `policy_ref`, `outcome_code`, `previous_event_hash`. Canonical big-endian serialization; no optional fields.
- **ExecutionProof v1.0**: `execution_id`, `event_chain_root`, `start_timestamp`, `end_timestamp`, `policy_ref`, `jurisdiction_code`, `verifier_signature`. One proof per execution; constant size.

## Build

```bash
cargo build -p nexus-etk
cargo test -p nexus-etk
```

## Library usage

```rust
use nexus_etk::{EventChain, create_genesis, Hash256, OutcomeCode, ResourceClass, verify, Verdict};
use ed25519_dalek::SigningKey;

// Start execution (Genesis)
let actor = Hash256::of(b"actor");
let workload = Hash256::of(b"workload");
let ctx = Hash256::of(b"context");
let policy_ref = Hash256::of(b"policy-snapshot");
let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy_ref);

// Append events (monotonic timestamps)
chain.append(now_ms + 1, OutcomeCode::Unknown)?;
chain.append(now_ms + 2, OutcomeCode::Success)?;

// Finalize and sign
let proof = chain.finalize(&signing_key)?;

// Offline verify (no API calls)
let policy_resolver = |_| Some(policy_snapshot_bytes.clone());
let verdict = verify(&proof, chain.events(), &policy_resolver, &verifier_pubkey, tolerance_ms)?;
```

## Offline verifier CLI

```bash
cargo run -p nexus-etk --bin etk_verifier -- <proof_file> <events_file> <policy_file> <verifier_pubkey_file> [--tolerance-ms N] [--verbose]
```

- **proof_file**: Raw canonical bytes of `ExecutionProofV1` (178 bytes).
- **events_file**: Concatenated canonical `ExecutionEventV1` (each 276 bytes).
- **policy_file**: Raw policy snapshot; verifier hashes and checks against `proof.policy_ref`.
- **verifier_pubkey_file**: Raw 32-byte Ed25519 verifying key (producer of the proof).

Output: `VALID` (exit 0) or `INVALID` (exit 1).

## 30-day checkpoint

> "Here is a binary. Run any job. Kill the machine. I can still prove what ran and where."

The library emits `ExecutionEvent`, maintains the hash chain, and outputs `ExecutionProof`. The verifier CLI validates offline with no trust in the runtime. No UI, no dashboards, no integrations.
