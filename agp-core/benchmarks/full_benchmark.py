#!/usr/bin/env python3
"""
AGP-OS Comprehensive Benchmark Suite
Measures performance across all system layers.
"""

import sys
import time
import asyncio
import statistics
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGP-OS COMPREHENSIVE BENCHMARK SUITE")
print("=" * 70)

results = {}

def benchmark(name: str, func, iterations: int = 1000):
    """Run a benchmark and collect statistics"""
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        func()
        elapsed = (time.perf_counter() - start) * 1000  # ms
        times.append(elapsed)
    
    results[name] = {
        "iterations": iterations,
        "mean_ms": statistics.mean(times),
        "median_ms": statistics.median(times),
        "stdev_ms": statistics.stdev(times) if len(times) > 1 else 0,
        "min_ms": min(times),
        "max_ms": max(times),
        "p99_ms": sorted(times)[int(iterations * 0.99)]
    }
    return results[name]

async def async_benchmark(name: str, func, iterations: int = 1000):
    """Run an async benchmark"""
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        await func()
        elapsed = (time.perf_counter() - start) * 1000
        times.append(elapsed)
    
    results[name] = {
        "iterations": iterations,
        "mean_ms": statistics.mean(times),
        "median_ms": statistics.median(times),
        "stdev_ms": statistics.stdev(times) if len(times) > 1 else 0,
        "min_ms": min(times),
        "max_ms": max(times),
        "p99_ms": sorted(times)[int(iterations * 0.99)]
    }
    return results[name]

def print_result(name: str, r: dict):
    print(f"   {name}:")
    print(f"      Mean: {r['mean_ms']:.3f}ms | P99: {r['p99_ms']:.3f}ms | Min: {r['min_ms']:.3f}ms | Max: {r['max_ms']:.3f}ms")

# ========== GOVERNANCE BENCHMARKS ==========
print("\n[1] GOVERNANCE BENCHMARKS")
print("-" * 40)

from src.governance.behavioral_rag import behavioral_rag, BehaviorRecord, ActionType, Outcome
from src.governance.rules import GovernanceRulesEngine
from src.governance import protocol_enforcer

# Behavior Storage
def bench_behavior_store():
    record = BehaviorRecord(
        agent_id="bench-agent",
        agent_name="BenchBot",
        action_type=ActionType.EXECUTE,
        input_summary="benchmark test action",
        outcome=Outcome.SUCCESS
    )
    behavioral_rag.store_behavior(record)

r = benchmark("Behavior Store", bench_behavior_store, 100)
print_result("Behavior Store (100 ops)", r)

# Alignment Calculation - use global verifier
from src.governance import alignment_verifier
def bench_alignment():
    alignment_verifier.compute_alignment("bench-agent")

r = benchmark("Alignment Calculation", bench_alignment, 50)
print_result("Alignment Calculation (50 ops)", r)

# Rule Evaluation
rules_engine = GovernanceRulesEngine()
def bench_rules():
    rules_engine.evaluate({
        "agent_id": "bench-agent",
        "action": "execute",
        "alignment": 0.8,
        "frequency": 50
    })

r = benchmark("Rule Evaluation", bench_rules, 500)
print_result("Rule Evaluation (500 ops)", r)

# ========== HAL BENCHMARKS ==========
print("\n[2] HAL BENCHMARKS")
print("-" * 40)

from src.os.hal.hal import hal

# Register test devices
hal.register_sensor("bench_sensor", "Bench Sensor", lambda: {"value": 42})
hal.register_actuator("bench_actuator", "Bench Actuator", lambda cmd: "ok")

def bench_sensor_read():
    hal.read_sensor("bench_sensor")

r = benchmark("Sensor Read", bench_sensor_read, 1000)
print_result("Sensor Read (1000 ops)", r)

def bench_actuator_move():
    hal.move_actuator("bench_actuator", {"speed": 0.5}, agent_alignment=0.9)

r = benchmark("Actuator Move", bench_actuator_move, 1000)
print_result("Actuator Move (1000 ops)", r)

def bench_safety_interlock():
    hal.move_actuator("bench_actuator", {"speed": 0.5}, agent_alignment=0.2)

r = benchmark("Safety Interlock (Block)", bench_safety_interlock, 1000)
print_result("Safety Interlock Block (1000 ops)", r)

# ========== MESH BENCHMARKS ==========
print("\n[3] MESH COORDINATION BENCHMARKS")
print("-" * 40)

from src.os.mesh.mesh import MeshCoordinator

mesh = MeshCoordinator()
mesh.register_agent("bench-a")
mesh.register_agent("bench-b")

def bench_message_send():
    mesh.send_message("bench-a", "bench-b", {"data": "test"})

r = benchmark("Message Send", bench_message_send, 1000)
print_result("Message Send (1000 ops)", r)

def bench_broadcast():
    mesh.broadcast("bench-a", {"alert": "test"})

r = benchmark("Broadcast", bench_broadcast, 500)
print_result("Broadcast (500 ops)", r)

# ========== RTOS BENCHMARKS ==========
print("\n[4] RTOS BENCHMARKS")
print("-" * 40)

from src.os.rtos.scheduler import RTScheduler, TaskPriority

scheduler = RTScheduler()

def bench_task_submit():
    scheduler.submit("bench-task", lambda: None, priority=TaskPriority.NORMAL)

r = benchmark("Task Submit", bench_task_submit, 1000)
print_result("Task Submit (1000 ops)", r)

async def bench_task_execute():
    scheduler.submit("exec-task", lambda: 42, priority=TaskPriority.HIGH)
    await scheduler.run_once()

async def run_rtos_bench():
    return await async_benchmark("Task Execute", bench_task_execute, 500)

r = asyncio.run(run_rtos_bench())
print_result("Task Execute (500 ops)", r)

# ========== ROS2 BRIDGE BENCHMARKS ==========
print("\n[5] ROS2 BRIDGE BENCHMARKS")
print("-" * 40)

from src.os.ros2.bridge import ROS2Bridge

ros_bridge = ROS2Bridge(simulation_mode=True)
ros_bridge.spawn_robot("bench_robot", "BenchBot")

def bench_ros_publish():
    ros_bridge.publish("bench_robot/cmd_vel", {"linear": {"x": 0.5}, "angular": {"z": 0.1}})

r = benchmark("ROS2 Publish", bench_ros_publish, 1000)
print_result("ROS2 Publish (1000 ops)", r)

def bench_robot_state():
    ros_bridge.get_robot_state("bench_robot")

r = benchmark("Robot State Query", bench_robot_state, 1000)
print_result("Robot State Query (1000 ops)", r)

# ========== RESOURCE CONTROLLER BENCHMARKS ==========
print("\n[6] RESOURCE CONTROLLER BENCHMARKS")
print("-" * 40)

from src.os.resources.controller import ResourceController, ResourceQuota, ResourceType

rc = ResourceController()
rc.register_agent("bench-rc", ResourceQuota())

def bench_resource_request():
    rc.request_resource("bench-rc", ResourceType.CPU_CYCLES, 100)

r = benchmark("Resource Request", bench_resource_request, 1000)
print_result("Resource Request (1000 ops)", r)

def bench_resource_check():
    rc.get_usage("bench-rc")

r = benchmark("Resource Usage Check", bench_resource_check, 1000)
print_result("Resource Usage Check (1000 ops)", r)

# ========== SUMMARY ==========
print("\n" + "=" * 70)
print("BENCHMARK SUMMARY")
print("=" * 70)

print("\n{:<35} {:>10} {:>10} {:>10}".format("Operation", "Mean (ms)", "P99 (ms)", "Ops/sec"))
print("-" * 70)

for name, r in results.items():
    ops_per_sec = 1000 / r["mean_ms"] if r["mean_ms"] > 0 else float('inf')
    print("{:<35} {:>10.3f} {:>10.3f} {:>10.0f}".format(
        name[:35], r["mean_ms"], r["p99_ms"], ops_per_sec
    ))

print("\n✅ BENCHMARK SUITE COMPLETE")
