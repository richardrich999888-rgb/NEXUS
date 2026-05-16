#!/usr/bin/env python3
"""
Production ROS2 Adapter Test
Verifies safety watchdog and hardware deployment features.
"""

import sys
from pathlib import Path
import time
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.os.ros2.production import (
    production_adapter, SafetyWatchdog, WatchdogConfig
)

print("=" * 70)
print("PRODUCTION ROS2 ADAPTER TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# 1. Test Safety Watchdog
print("\n[1] TESTING SAFETY WATCHDOG...")
watchdog = SafetyWatchdog(WatchdogConfig(
    heartbeat_timeout_ms=100,
    max_velocity=1.0,
    max_angular=0.5
))

test("Watchdog initialized", watchdog is not None)
test("Watchdog not armed initially", not watchdog.is_armed)

watchdog.arm()
test("Watchdog armed", watchdog.is_armed)
test("Watchdog check passes", watchdog.check())

# 2. Test velocity validation
print("\n[2] TESTING VELOCITY VALIDATION...")
validated = watchdog.validate_velocity(0.5, 0.3)
test("Normal velocity not capped", not validated["capped"])

validated = watchdog.validate_velocity(2.0, 1.5)
test("Excessive velocity capped", validated["capped"])
test("Linear capped to max", validated["linear"] == 1.0)
test("Angular capped to max", validated["angular"] == 0.5)

# 3. Test heartbeat timeout
print("\n[3] TESTING HEARTBEAT TIMEOUT...")
watchdog.heartbeat()
time.sleep(0.05)  # 50ms - under timeout
test("Watchdog OK under timeout", watchdog.check())

time.sleep(0.1)  # Now over 100ms total
test("Watchdog triggers on timeout", not watchdog.check())
test("Emergency stop triggered", watchdog.emergency_stop_triggered)

watchdog.reset()
test("Watchdog reset", not watchdog.emergency_stop_triggered)

# 4. Test Production Adapter
print("\n[4] TESTING PRODUCTION ADAPTER...")
status = production_adapter.get_status()
test("Adapter initialized", status is not None)
test("Simulation mode (no rclpy)", status["mode"] == "simulation" or status["ros2_available"])

# 5. Test connection
print("\n[5] TESTING CONNECTION...")
result = production_adapter.connect()
test("Connection successful", result.get("status") == "connected")
test("Mode determined", "mode" in result)

status = production_adapter.get_status()
test("Adapter connected", status["connected"])

# 6. Test velocity publishing
print("\n[6] TESTING VELOCITY PUBLISHING...")
from src.os.ros2.bridge import ros2_bridge

# Spawn a test robot first
ros2_bridge.spawn_robot("test_robot", "TestBot")
production_adapter.create_velocity_publisher("test_robot/cmd_vel")

result = production_adapter.publish_velocity("test_robot/cmd_vel", 0.5, 0.2)
test("Velocity published", result.get("status") == "published")

# 7. Test emergency stop
print("\n[7] TESTING EMERGENCY STOP...")
result = production_adapter.emergency_stop("test_robot/cmd_vel")
test("Emergency stop sent", result.get("status") == "published")

# 8. Verify deployment files exist
print("\n[8] VERIFYING DEPLOYMENT FILES...")
deploy_dir = ROOT / "deploy"
test("Dockerfile.ros2 exists", (deploy_dir / "Dockerfile.ros2").exists())
test("entrypoint.sh exists", (deploy_dir / "entrypoint.sh").exists())
test("systemd service exists", (deploy_dir / "agp-os-robot.service").exists())

# Cleanup
production_adapter.disconnect()

# Summary
print("\n" + "=" * 70)
print("PRODUCTION ROS2 ADAPTER TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ PRODUCTION ADAPTER VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
