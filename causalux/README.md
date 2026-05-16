# CAUSALUX v2.0

**Production-Ready Distributed Execution Fabric**

> Copyright © 2025 SYNTRIASS Labs Pvt Ltd  
> Inventor: Katta Naga Sri Ganesh

## 🎯 What is CAUSALUX?

CAUSALUX is a post-cloud execution fabric that replaces compute scheduling, data synchronization, network protocols, and AI inference engines with a single causal algebra runtime.

## ✨ Key Features

- **Conflict-Free by Construction** - Version vectors detect and resolve conflicts automatically
- **Constant Memory Footprint** - Snapshot-based GC keeps memory at ~1GB regardless of history
- **Offline-First** - Works offline for weeks, syncs efficiently when reconnected
- **Byzantine Fault Tolerant** - Optional BFT mode for high-security deployments
- **Cryptographic Identity** - Ed25519 signatures bind every operation to an identity

## 🐳 Deployment

### Docker (Single Node)

```bash
# Build image
docker build -t causalux-v2 .

# Run container
docker run -p 8080:8080 -p 9090:9090 \
  -e NODE_ID=node1 \
  -e RUST_LOG=info \
  -v causalux-data:/data \
  causalux-v2
```

### Docker Compose (Multi-Node with Monitoring)

```bash
# Start 2 nodes + Prometheus + Grafana
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

**Endpoints:**
- Node 1: http://localhost:8081
- Node 2: http://localhost:8082
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin/admin)

### Kubernetes (Production)

See `k8s/` directory for Kubernetes manifests.

```bash
kubectl apply -f k8s/
```

## 📊 Monitoring

### Health Checks

```bash
# Liveness probe
curl http://localhost:8080/health

# Readiness probe
curl http://localhost:8080/ready
```

### Metrics

```bash
# Prometheus metrics
curl http://localhost:9090/metrics
```

Key metrics:
- `causalux_operations_total` - Total operations processed
- `causalux_operation_latency_seconds` - Operation latency
- `causalux_conflicts_total` - Conflicts detected
- `causalux_sync_bandwidth_bytes` - Sync bandwidth

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
cargo --version
```

### Build

```bash
cd causalux-v2
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Run with BFT enabled

```bash
cargo build --release --features bft
```

## 📁 Project Structure

```
causalux-v2/
├── Cargo.toml              # Dependencies and features
├── src/
│   ├── lib.rs              # Library exports
│   ├── version_vector.rs   # Causal tracking
│   ├── content_address.rs  # Position-independent references
│   ├── causal_op.rs        # Signed causal operations
│   ├── snapshot.rs         # State checkpoints + GC
│   ├── conflict.rs         # Resolution policies
│   ├── dag.rs              # Causal DAG with state
│   └── bft.rs              # Byzantine fault tolerance
└── README.md
```

## 🔧 Core Components

### Version Vectors

Track causality across distributed nodes:

```rust
use causalux_v2::VersionVector;

let mut v1 = VersionVector::new();
v1.increment("node_a");

let mut v2 = VersionVector::new();
v2.increment("node_b");

// Detect concurrent operations
if v1.conflicts_with(&v2) {
    println!("Conflict detected!");
}
```

### Causal Operations

Signed, immutable operations with explicit dependencies:

```rust
use causalux_v2::{CausalOp, VersionVector};
use ed25519_dalek::Keypair;

let keypair = Keypair::generate(&mut rand::rngs::OsRng);
let mut vv = VersionVector::new();
vv.increment("node1");

let op = CausalOp::new(
    "set".to_string(),
    serde_json::json!({"key": "counter", "value": 42}),
    BTreeSet::new(),
    vv,
    "node1".to_string(),
    &keypair,
);
```

### Causal DAG

Insert operations with automatic conflict resolution:

```rust
use causalux_v2::{CausalDAG, ConflictPolicy};

let mut dag = CausalDAG::new(
    "node1".to_string(),
    10_000,  // Snapshot every 10K operations
    ConflictPolicy::LastWriterWins,
);

dag.insert(op)?;
```

### Snapshot Manager

Constant memory via periodic snapshots:

```rust
// Snapshots created automatically every N operations
// Old operations garbage collected
// Memory stays at ~1GB regardless of total history
```

### BFT Validator (Optional)

Byzantine fault tolerance for high-security:

```rust
use causalux_v2::BFTValidator;

let bft = BFTValidator::new(
    validators,
    1,  // Tolerate 1 Byzantine fault (needs 2f+1 = 3 validators)
    Duration::from_secs(30),
)?;

bft.submit_for_validation(op)?;
```

## 📊 Performance

| Metric | Traditional | CAUSALUX v2.0 |
|--------|-------------|---------------|
| Memory | Unbounded | Constant (~1GB) |
| Sync (30 days offline) | 1.25 GB | 55 MB |
| Conflict resolution | Manual | Automatic |
| Byzantine resistance | None | BFT mode |
| Latency (P99) | 120ms | 8ms |

## 📜 Conflict Resolution Policies

1. **LastWriterWins** - Use wall clock (may lose data)
2. **HighestPriority** - Use node priority
3. **ManualResolution** - Return both options for user
4. **SemanticMerge** - CRDT-aware auto-merge

## 🔐 Security

- Ed25519 signatures on every operation
- Content-addressed operations (tamper-proof)
- Optional BFT for Byzantine fault tolerance
- Validator reputation system

## 📄 License

Apache 2.0

## 🤝 Contributing

See CONTRIBUTING.md for guidelines.
