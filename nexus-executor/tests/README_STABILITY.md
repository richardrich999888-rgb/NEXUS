# Long-Running Stability Tests

These tests verify NEXUS remains stable under extended load. They are designed to run for configurable durations (default: 1 hour for CI, 24+ hours for production validation).

## Test Suite

### `test_memory_stability_extended`
- **Purpose**: Verify memory usage remains bounded over extended period
- **Duration**: Configurable via `STABILITY_TEST_DURATION_SECS` (default: 3600s = 1 hour)
- **Checks**: 
  - Memory usage doesn't grow unbounded
  - Executor remains functional throughout
  - No crashes or panics

### `test_throughput_stability`
- **Purpose**: Verify throughput (PCUs/second) remains stable over time
- **Duration**: Configurable (default: 1 hour)
- **Checks**:
  - Throughput coefficient of variation < 0.5
  - No degradation over time
  - Consistent performance windows

### `test_concurrent_stability`
- **Purpose**: Verify concurrent execution remains stable
- **Duration**: Configurable (default: 1 hour)
- **Concurrency**: 10 parallel executions
- **Checks**:
  - Failure rate < 1%
  - No deadlocks or hangs
  - Consistent performance under load

### `test_resource_usage_stability`
- **Purpose**: Verify CPU and memory usage remain bounded
- **Duration**: Configurable (default: 1 hour)
- **Checks**:
  - Execution time doesn't grow significantly
  - Resource usage remains stable
  - No resource leaks

## Running Stability Tests

### Quick Run (CI - 1 hour)
```bash
cargo test --test stability -- --ignored
```

### Extended Run (Production - 24 hours)
```bash
STABILITY_TEST_DURATION_SECS=86400 cargo test --test stability -- --ignored
```

### Custom Duration
```bash
STABILITY_TEST_DURATION_SECS=7200 cargo test --test stability -- --ignored  # 2 hours
```

### Run Specific Test
```bash
cargo test --test stability test_memory_stability_extended -- --ignored
```

## Notes

- Tests are marked `#[ignore]` by default to avoid slowing down regular test runs
- Use `--ignored` flag to run them explicitly
- Duration is configurable via environment variable
- Tests print progress information via `eprintln!`
- For production validation, run for 24+ hours

## Memory Leak Detection

For detailed memory leak detection, use external tools:
- `valgrind` (Linux)
- `dhat` (Rust memory profiler)
- `heaptrack` (Linux)

Example:
```bash
valgrind --leak-check=full --show-leak-kinds=all \
  cargo test --test stability test_memory_stability_extended -- --ignored
```

