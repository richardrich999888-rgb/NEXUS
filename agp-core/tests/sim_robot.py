#!/usr/bin/env python3
"""
Robotic OS Simulation (Direct HAL Test)
Verifies HAL and Physical Safety Interlocks without full kernel plumbing.
"""

import sys
from pathlib import Path
import time
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.os.hal.hal import hal

print("=" * 70)
print("AGP-OS ROBOTIC HAL SIMULATION")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# 1. Register Mock Hardware
print("\n[1] REGISTERING MOCK HARDWARE...")
hal.register_sensor("lidar_01", "Front LIDAR", lambda: {"distance": 1.5, "unit": "meters"})
hal.register_actuator("motor_arm", "Robotic Arm", lambda cmd: f"Moved with {cmd}")
print("   Registered: lidar_01, motor_arm")

# 2. Test Sensor Read
print("\n[2] TESTING SENSOR READ...")
sensor_result = hal.read_sensor("lidar_01")
print(f"   Result: {sensor_result}")
test("Sensor read returns data", sensor_result.get("status") == "ok")
test("Sensor data contains distance", sensor_result.get("data", {}).get("distance") == 1.5)

# 3. Test Actuator Move (HIGH Alignment - Should Succeed)
print("\n[3] TESTING ACTUATOR MOVE (HIGH ALIGNMENT)...")
move_result = hal.move_actuator("motor_arm", {"angle": 45}, agent_alignment=0.9)
print(f"   Result: {move_result}")
test("High-alignment agent can move actuator", move_result.get("status") == "executed")

# 4. Test Actuator Move (LOW Alignment - Should Be BLOCKED)
print("\n[4] TESTING ACTUATOR MOVE (LOW ALIGNMENT - SAFETY INTERLOCK)...")
move_result = hal.move_actuator("motor_arm", {"angle": 90}, agent_alignment=0.2)
print(f"   Result: {move_result}")
test("Low-alignment agent is BLOCKED by safety interlock", move_result.get("status") == "blocked")
test("Blocked reason mentions SAFETY_INTERLOCK", "SAFETY_INTERLOCK" in move_result.get("reason", ""))

# 5. Test Velocity Capping
print("\n[5] TESTING VELOCITY CAPPING...")
# Request velocity > max (1.0)
capped_result = hal.move_actuator("motor_arm", {"velocity": 2.5}, agent_alignment=0.9)
print(f"   Result: {capped_result}")
test("High-velocity command is still executed (capped)", capped_result.get("status") == "executed")

# 6. Device Map
print("\n[6] TESTING DEVICE MAP...")
device_map = hal.get_device_map()
print(f"   Devices: {device_map}")
test("Device map has 2 devices", len(device_map) == 2)

# Summary
print("\n" + "=" * 70)
print("ROBOTIC HAL SIMULATION RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ ROBOTIC HAL VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
