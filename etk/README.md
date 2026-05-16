# ETK — Execution Truth Kernel (Production Foundation)

Regulator-grade, production-foundation kernel for verifiable execution attestation.

- **Deterministic binary encoding** — big-endian canonical; no JSON.
- **Crypto-safe hashing** — SHA-256; same input → same hash everywhere.
- **Zero trust** — verifier assumes nothing about runtime, cloud, or actor.
- **Offline verifier** — no API calls; proof + events + policy + pubkey only.
- **Strict schema lock** — fixed-size types; no optional fields; v1.0 immutable.

## Layout

```
etk/
├── Cargo.toml
├── rust-toolchain.toml     # Locked compiler for deterministic builds
├── Makefile                # build | repro | sign | verify | sbom
├── build/
│   ├── reproducible.sh    # Deterministic build, SOURCE_DATE_EPOCH, build_hash.txt
│   ├── sign_artifacts.sh   # Sign binary (openssl, offline key)
│   ├── verify_artifacts.sh # Verify signature (auditors)
│   └── sbom.sh             # SBOM / cargo-auditable
├── profiles/               # sovereign.toml, telecom.toml, defense.toml
├── supply-chain/           # provenance.json, SBOM.json
├── crates/
│   ├── etk-types/          # Schema lock v1.0 — types only
│   ├── etk-core/           # Crypto kernel + agility (CryptoSuite: Sha256, Blake3)
│   └── etk-cli/            # Regulator/auditor CLI
└── crates/etk-core/tests/
    └── integration.rs      # genesis -> append -> proof -> verify
```

## Build & test

```bash
cd etk && cargo build && cargo test
```

From NEXUS root:

```bash
cargo build -p etk-core -p etk-cli
cargo test -p etk-core
```

## CLI

```bash
cargo run -p etk-cli --bin etk -- verify <proof.bin> <events.bin> <policy.bin> <pubkey.bin> [--tolerance-ms N] [--verbose]
cargo run -p etk-cli --bin etk -- version
```

- **proof.bin**: 178 bytes (ExecutionProof canonical).
- **events.bin**: N × 276 bytes (ExecutionEvent canonical, concatenated).
- **policy.bin**: raw policy snapshot (verifier hashes and checks against proof.policy_ref).
- **pubkey.bin**: 32 bytes Ed25519 verifying key (producer of the proof).

Output: `VALID` (exit 0) or `INVALID` (exit 1).

## Library (etk-core)

```rust
use etk_core::{EventChain, create_genesis, hash256, verify, Verdict};
use etk_types::{Hash256, OutcomeCode, ResourceClass};
use ed25519_dalek::SigningKey;

let actor = hash256(b"actor");
let workload = hash256(b"workload");
let ctx = hash256(b"context");
let policy = hash256(b"policy-snapshot");
let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
chain.append(now_ms + 1, OutcomeCode::Unknown).unwrap();
chain.append(now_ms + 2, OutcomeCode::Success).unwrap();
let proof = chain.finalize(&signing_key).unwrap();
// Offline verify: proof, events, policy_resolver, verifier_pubkey, tolerance_ms
```

## Regulator-grade build stack

- **Deterministic builds**: `make repro` — SOURCE_DATE_EPOCH, no build-id; writes `build/build_hash.txt` and updates `supply-chain/provenance.json` (if `jq` present).
- **Signing**: `make sign` — signs `target/release/etk` with `private.pem`; output `build/etk.sig`. Keep private key offline.
- **Verification**: `make verify` — verifies binary with `public.pem` and `build/etk.sig` (what auditors run).
- **SBOM**: `make sbom` — auditable build; dependency list in `supply-chain/`.

First-time: `rust-toolchain.toml` pins 1.77.0; run `rustup show` to ensure the toolchain is installed.

## What is intentionally missing

- Identity graph, policy engine, networking, databases, APIs, cloud SDKs.
- Adding those too early destroys inevitability. This is the runtime primitive; everything else stacks on top.
