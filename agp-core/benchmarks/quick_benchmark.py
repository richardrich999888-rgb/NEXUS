#!/usr/bin/env python3
"""
AGP-OS Quick Benchmark Suite
Fast performance validation across all layers.
"""

import sys
import time
import asyncio
import statistics
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGP-OS QUICK BENCHMARK SUITE")
print("=" * 70)

results = {}

def benchmark(name: str, func, iterations: int = 100):
    """Run a benchmark"""
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        func()
        elapsed = (time.perf_counter() - start) * 1000
        times.append(elapsed)
    
    mean = statistics.mean(times)
    p99 = sorted(times)[int(iterations * 0.99)]
    ops_sec = 1000 / mean if mean > 0 else float('inf')
    results[name] = {"mean": mean, "p99": p99, "ops_sec": ops_sec}
    return mean, p99, ops_sec

# ========== BENCHMARKS ==========

print("\n[1] GOVERNANCE")
from src.governance import alignment_verifier
from src.governance.rules import GovernanceRulesEngine

rules = GovernanceRulesEngine()
m, p, o = benchmark("Rule Evaluation", lambda: rules.evaluate("test-agent", {"alignment": 0.8}), 500)
print(f"   Rule Evaluation:     {m:.3f}ms mean | {o:.0f} ops/sec")

m, p, o = benchmark("Alignment Calc", lambda: alignment_verifier.compute_alignment("test-agent"), 50)
print(f"   Alignment Calc:      {m:.3f}ms mean | {o:.0f} ops/sec")

print("\n[2] HAL")
from src.os.hal.hal import hal
hal.register_sensor("bench", "Bench", lambda: {"v": 1})
hal.register_actuator("bench_act", "Bench", lambda x: "ok")

m, p, o = benchmark("Sensor Read", lambda: hal.read_sensor("bench"), 1000)
print(f"   Sensor Read:         {m:.3f}ms mean | {o:.0f} ops/sec")

m, p, o = benchmark("Actuator Move", lambda: hal.move_actuator("bench_act", {}, 0.9), 1000)
print(f"   Actuator Move:       {m:.3f}ms mean | {o:.0f} ops/sec")

m, p, o = benchmark("Safety Block", lambda: hal.move_actuator("bench_act", {}, 0.2), 1000)
print(f"   Safety Interlock:    {m:.3f}ms mean | {o:.0f} ops/sec")

print("\n[3] MESH")
from src.os.mesh.mesh import MeshCoordinator
mesh = MeshCoordinator()
mesh.register_agent("a")
mesh.register_agent("b")

m, p, o = benchmark("Message Send", lambda: mesh.send_message("a", "b", {"x": 1}), 1000)
print(f"   Message Send:        {m:.3f}ms mean | {o:.0f} ops/sec")

m, p, o = benchmark("Broadcast", lambda: mesh.broadcast("a", {"x": 1}), 500)
print(f"   Broadcast:           {m:.3f}ms mean | {o:.0f} ops/sec")

print("\n[4] RTOS")
from src.os.rtos.scheduler import RTScheduler, TaskPriority
sched = RTScheduler()

m, p, o = benchmark("Task Submit", lambda: sched.submit("t", lambda: 1), 1000)
print(f"   Task Submit:         {m:.3f}ms mean | {o:.0f} ops/sec")

print("\n[5] ROS2 BRIDGE")
from src.os.ros2.bridge import ROS2Bridge
ros = ROS2Bridge(simulation_mode=True)
ros.spawn_robot("bench_bot", "Bot")

m, p, o = benchmark("ROS Publish", lambda: ros.publish("bench_bot/cmd_vel", {"linear": {"x": 0.5}}), 1000)
print(f"   ROS Publish:         {m:.3f}ms mean | {o:.0f} ops/sec")

m, p, o = benchmark("Robot State", lambda: ros.get_robot_state("bench_bot"), 1000)
print(f"   Robot State:         {m:.3f}ms mean | {o:.0f} ops/sec")

print("\n[6] RESOURCES")
from src.os.resources.controller import ResourceController, ResourceQuota, ResourceType
rc = ResourceController()
rc.register_agent("bench", ResourceQuota())

m, p, o = benchmark("Resource Req", lambda: rc.request_resource("bench", ResourceType.CPU_CYCLES, 10), 1000)
print(f"   Resource Request:    {m:.3f}ms mean | {o:.0f} ops/sec")

# ========== SUMMARY ==========
print("\n" + "=" * 70)
print("BENCHMARK SUMMARY")
print("=" * 70)
print("\n{:<25} {:>12} {:>12}".format("Operation", "Mean (ms)", "Ops/sec"))
print("-" * 50)
for name, r in results.items():
    print("{:<25} {:>12.3f} {:>12,.0f}".format(name, r["mean"], r["ops_sec"]))

# Performance targets
print("\n" + "-" * 50)
print("PERFORMANCE TARGETS")
print("-" * 50)
targets = {
    "Rule Evaluation": 1.0,
    "Sensor Read": 0.1,
    "Actuator Move": 0.1,
    "Message Send": 0.1,
    "ROS Publish": 0.5
}

all_pass = True
for name, target in targets.items():
    if name in results:
        actual = results[name]["mean"]
        status = "✓ PASS" if actual < target else "✗ FAIL"
        if actual >= target:
            all_pass = False
        print(f"   {name}: {actual:.3f}ms < {target}ms target → {status}")

print("\n" + "=" * 70)
if all_pass:
    print("✅ ALL PERFORMANCE TARGETS MET - PRODUCTION READY")
else:
    print("⚠️  SOME TARGETS MISSED - Review performance")
print("=" * 70)
