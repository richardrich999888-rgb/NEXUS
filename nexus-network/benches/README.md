# NEXUS Network Benchmarks

This directory contains network layer benchmarks for NEXUS.

## Benchmark Suites

### `network_gossip.rs`
Basic gossip protocol benchmarks:
- Broadcast performance (1KB messages)

### `multi_node.rs`
Multi-node network behavior benchmarks:
- **State sync convergence**: Time to sync state across 3, 5, 10 nodes
- **Message overhead**: Bytes per sync operation (1KB, 10KB, 100KB)
- **Network partition recovery**: Split-brain recovery scenarios
- **Concurrent updates**: Simultaneous broadcasts from all nodes

## Running Benchmarks

```bash
# Run all network benchmarks
cargo bench -p nexus-network

# Run specific benchmark suite
cargo bench -p nexus-network --bench multi_node
cargo bench -p nexus-network --bench network_gossip

# Run with filter
cargo bench -p nexus-network --bench multi_node -- state-sync
```

## Benchmark Results

Results are saved to `target/criterion/` with HTML reports for visualization.

## Notes

- Multi-node benchmarks use a test harness that creates N nodes in a mesh topology
- Network partition scenarios simulate split-brain conditions
- Message overhead includes serialization and transport overhead
- All benchmarks use development TLS certificates (self-signed)

