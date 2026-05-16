# NEXUS-Core v0.1

Deterministic execution log with algebraic merge.

## Build

```bash
cargo build --release
```

## Test

```bash
cargo test
```

## Run

```bash
# Initialize
./target/release/nexus-core init

# Execute operation
./target/release/nexus-core exec <wasm-file> <input-file>

# Replay log
./target/release/nexus-core replay

# Show status
./target/release/nexus-core status
```

## Invariants

1. Deterministic execution
2. Causal append-only log
3. Commutative merge
4. Replay-verifiable state
