#!/usr/bin/env python3
"""
Resource Controller Test
Verifies quota enforcement and resource tracking.
"""

import sys
from pathlib import Path
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.os.resources.controller import resource_controller, ResourceQuota, ResourceType

print("=" * 70)
print("RESOURCE CONTROLLER TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# 1. Register agents with quotas
print("\n[1] REGISTERING AGENTS WITH QUOTAS...")
resource_controller.register_agent("agent-big", ResourceQuota(
    memory_mb_max=1024.0,
    tokens_max=50000
))
resource_controller.register_agent("agent-small", ResourceQuota(
    memory_mb_max=256.0,
    tokens_max=10000
))
test("Two agents registered", len(resource_controller.quotas) == 2)

# 2. Request memory (within quota)
print("\n[2] TESTING MEMORY ALLOCATION (WITHIN QUOTA)...")
result = resource_controller.request_resource("agent-big", ResourceType.MEMORY_MB, 500.0)
test("Memory granted within quota", result.get("status") == "granted")

usage = resource_controller.get_usage("agent-big")
test("Usage tracking correct", usage["memory"]["used"] == 500.0)

# 3. Request memory (exceed quota)
print("\n[3] TESTING MEMORY ALLOCATION (EXCEED QUOTA)...")
result = resource_controller.request_resource("agent-big", ResourceType.MEMORY_MB, 600.0)
test("Memory denied when exceeding quota", result.get("status") == "denied")
test("Denial reason is correct", "quota exceeded" in result.get("reason", "").lower())

# 4. Request tokens
print("\n[4] TESTING TOKEN ALLOCATION...")
result = resource_controller.request_resource("agent-small", ResourceType.TOKENS, 5000)
test("Tokens granted", result.get("status") == "granted")

result = resource_controller.request_resource("agent-small", ResourceType.TOKENS, 6000)
test("Tokens denied when exceeding quota", result.get("status") == "denied")

# 5. Priority update (governance integration)
print("\n[5] TESTING PRIORITY UPDATE (GOVERNANCE)...")
resource_controller.set_priority("agent-big", 0.9)
usage = resource_controller.get_usage("agent-big")
test("Priority updated to 0.9", usage["priority"] == 0.9)

# 6. System-wide status
print("\n[6] TESTING SYSTEM STATUS...")
status = resource_controller.get_system_status()
test("System memory tracked", status["memory"]["used"] == 500.0)
test("Both agents registered", status["agents_registered"] == 2)

# 7. Release resource
print("\n[7] TESTING RESOURCE RELEASE...")
resource_controller.release_resource("agent-big", ResourceType.MEMORY_MB, 200.0)
usage = resource_controller.get_usage("agent-big")
test("Memory released correctly", usage["memory"]["used"] == 300.0)

status = resource_controller.get_system_status()
test("System memory updated after release", status["memory"]["used"] == 300.0)

# Summary
print("\n" + "=" * 70)
print("RESOURCE CONTROLLER TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ RESOURCE CONTROLLER VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
