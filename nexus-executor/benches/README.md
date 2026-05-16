# NEXUS Executor Benchmarks

This directory contains comprehensive benchmarks for the NEXUS executor, organized by measurement category.

## Benchmark Suites

### `execution_bench.rs`
Basic execution benchmarks:
- Cache hit vs miss performance
- Baseline execution timing

### `overhead_breakdown.rs`
Detailed overhead analysis:
- **Serialization overhead**: PCU to/from bytes (100B to 100KB)
- **Cache lookup overhead**: Hit vs miss timing
- **Proof generation overhead**: Create, verify, signing bytes computation
- **Module compilation overhead**: Wasmtime compilation time

### `resource_usage.rs`
Resource consumption profiling:
- **Memory usage**: Peak memory across PCU sizes (1KB to 1MB)
- **CPU usage**: Execution CPU time
- **Storage I/O**: Serialization/deserialization as proxy for storage operations

### `throughput.rs`
System capacity benchmarks:
- **Sequential throughput**: PCUs/second for batch sizes (1, 10, 100)
- **Concurrent throughput**: Parallel execution (1, 4, 8, 16 concurrent)
- **Mixed workload**: Realistic cache hit rates (50%, 90%)

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench execution_bench
cargo bench --bench overhead_breakdown
cargo bench --bench resource_usage
cargo bench --bench throughput

# Run with specific filter
cargo bench --bench overhead_breakdown -- cache_lookup
```

## Benchmark Results

Results are saved to `target/criterion/` with HTML reports for visualization.

## Notes

- All benchmarks use minimal valid WASM modules to focus on NEXUS overhead, not WASM execution time
- Cache benchmarks pre-populate cache before measuring hits
- Throughput benchmarks measure system capacity, not comparative performance
- No fake numbers — all results are reproducible and auditable

