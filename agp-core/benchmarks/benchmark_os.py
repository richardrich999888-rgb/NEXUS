#!/usr/bin/env python3
"""
AGP-OS Performance Benchmarks
Measures actual performance of all OS components.
"""

import sys
import asyncio
import time
import statistics
from datetime import datetime

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGP-OS PERFORMANCE BENCHMARKS")
print("=" * 70)
print(f"Date: {datetime.now().isoformat()}\n")

def benchmark(name, iterations=1000):
    """Decorator to benchmark a function"""
    def decorator(func):
        async def wrapper(*args, **kwargs):
            times = []
            
            # Warmup
            for _ in range(10):
                if asyncio.iscoroutinefunction(func):
                    await func(*args, **kwargs)
                else:
                    func(*args, **kwargs)
            
            # Actual benchmark
            for _ in range(iterations):
                start = time.perf_counter_ns()
                if asyncio.iscoroutinefunction(func):
                    await func(*args, **kwargs)
                else:
                    func(*args, **kwargs)
                end = time.perf_counter_ns()
                times.append((end - start) / 1000)  # Convert to microseconds
            
            avg = statistics.mean(times)
            p50 = statistics.median(times)
            p99 = sorted(times)[int(len(times) * 0.99)]
            ops_per_sec = 1_000_000 / avg if avg > 0 else 0
            
            print(f"\n📊 {name}")
            print(f"   Iterations: {iterations}")
            print(f"   Avg:  {avg:.2f} µs")
            print(f"   P50:  {p50:.2f} µs")
            print(f"   P99:  {p99:.2f} µs")
            print(f"   Ops/sec: {ops_per_sec:,.0f}")
            
            return {
                "name": name,
                "avg_us": avg,
                "p50_us": p50,
                "p99_us": p99,
                "ops_per_sec": ops_per_sec
            }
        return wrapper
    return decorator

async def run_benchmarks():
    results = []
    
    # Setup
    from src.os import kernel
    from src.os.ipc import mq_manager, signal_handler, shm_manager, Signal, MessagePriority
    from src.os.logging import syslog, metrics, LogLevel
    from src.os.recovery import checkpoint_manager
    from src.os.scheduler import advanced_scheduler
    from src.os.fs import filesystem
    from src.agents import AgentFactory
    
    kernel.boot()
    agent = AgentFactory.create_engineer_agent("Bench_Agent")
    pid = kernel.spawn_process(agent)
    
    # Create shared memory for benchmarks
    shm_manager.create("bench_shm", 4096, pid)
    
    print("\n" + "=" * 70)
    print("COMPONENT BENCHMARKS")
    print("=" * 70)
    
    # 1. MESSAGE QUEUE BENCHMARKS
    @benchmark("Message Queue Send", iterations=10000)
    def bench_mq_send():
        mq_manager.send_message(1, 2, {"data": "x" * 100}, MessagePriority.NORMAL)
    
    @benchmark("Message Queue Receive (non-blocking)", iterations=10000)
    async def bench_mq_receive():
        await mq_manager.receive_message(2, block=False)
    
    results.append(await bench_mq_send())
    results.append(await bench_mq_receive())
    
    # 2. SIGNAL BENCHMARKS
    @benchmark("Signal Send (SIGUSR1)", iterations=10000)
    def bench_signal_send():
        signal_handler.send_signal(1, pid, Signal.SIGUSR1)
    
    results.append(await bench_signal_send())
    
    # 3. SHARED MEMORY BENCHMARKS
    test_data = b"x" * 1024  # 1KB
    
    @benchmark("Shared Memory Write (1KB)", iterations=10000)
    def bench_shm_write():
        shm_manager.write("bench_shm", 0, test_data, pid)
    
    @benchmark("Shared Memory Read (1KB)", iterations=10000)
    def bench_shm_read():
        shm_manager.read("bench_shm", 0, 1024, pid)
    
    results.append(await bench_shm_write())
    results.append(await bench_shm_read())
    
    # 4. SYSLOG BENCHMARKS
    @benchmark("Syslog Write", iterations=10000)
    def bench_syslog():
        syslog.info("bench", "Benchmark log message", iteration=1)
    
    @benchmark("Metrics Increment", iterations=10000)
    def bench_metrics():
        metrics.increment("bench_counter")
    
    results.append(await bench_syslog())
    results.append(await bench_metrics())
    
    # 5. SCHEDULER BENCHMARKS
    @benchmark("Scheduler Select Process", iterations=1000)
    async def bench_scheduler():
        await advanced_scheduler.schedule(kernel.process_table)
    
    results.append(await bench_scheduler())
    
    # 6. FILESYSTEM BENCHMARKS
    @benchmark("Filesystem /proc Read", iterations=1000)
    def bench_fs_proc():
        filesystem.read(f"/proc/{pid}/status")
    
    @benchmark("Filesystem /home Write", iterations=1000)
    def bench_fs_home():
        filesystem.write(f"/home/test/file.txt", b"benchmark data")
    
    @benchmark("Filesystem /home Read", iterations=1000)
    def bench_fs_read():
        filesystem.read("/home/test/file.txt")
    
    results.append(await bench_fs_proc())
    results.append(await bench_fs_home())
    results.append(await bench_fs_read())
    
    # 7. CHECKPOINT BENCHMARK
    @benchmark("Checkpoint Create", iterations=100)
    def bench_checkpoint():
        checkpoint_manager.create_checkpoint()
    
    results.append(await bench_checkpoint())
    
    # SUMMARY
    print("\n" + "=" * 70)
    print("BENCHMARK SUMMARY")
    print("=" * 70)
    
    print(f"\n{'Component':<40} {'Avg (µs)':<12} {'Ops/sec':<15}")
    print("-" * 70)
    
    for r in results:
        print(f"{r['name']:<40} {r['avg_us']:<12.2f} {r['ops_per_sec']:<15,.0f}")
    
    # Calculate overall stats
    total_ops = sum(r['ops_per_sec'] for r in results)
    avg_latency = statistics.mean(r['avg_us'] for r in results)
    
    print("\n" + "=" * 70)
    print("OVERALL METRICS")
    print("=" * 70)
    print(f"\n   Total Components Benchmarked: {len(results)}")
    print(f"   Average Latency: {avg_latency:.2f} µs")
    print(f"   Fastest Component: {min(results, key=lambda x: x['avg_us'])['name']}")
    print(f"   Slowest Component: {max(results, key=lambda x: x['avg_us'])['name']}")
    
    # Production readiness assessment
    print("\n" + "=" * 70)
    print("PRODUCTION READINESS ASSESSMENT")
    print("=" * 70)
    
    issues = []
    
    # Check latencies
    slow_components = [r for r in results if r['avg_us'] > 1000]
    if slow_components:
        issues.append(f"⚠️  {len(slow_components)} components have >1ms latency")
    
    # Check if networking is stubbed
    issues.append("⚠️  Networking is stubbed (no real connections)")
    issues.append("⚠️  Persistence is in-memory only")
    issues.append("⚠️  No authentication/authorization")
    issues.append("⚠️  No distributed consensus")
    issues.append("⚠️  No rate limiting")
    
    print("\n   Issues Found:")
    for issue in issues:
        print(f"   {issue}")
    
    print(f"\n   STATUS: PROTOTYPE (Not Production-Ready)")
    print(f"   Recommended: Add networking, persistence, security before production")

if __name__ == "__main__":
    asyncio.run(run_benchmarks())
