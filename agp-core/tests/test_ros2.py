#!/usr/bin/env python3
"""
ROS2/Gazebo Bridge Test
Verifies robot simulation and AGP-OS governance integration.
"""

import sys
import asyncio
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.os.ros2.bridge import ros2_bridge, ROSMessageType
from src.os.hal.hal import hal
from src.governance import protocol_enforcer

print("=" * 70)
print("ROS2/GAZEBO BRIDGE TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# 1. Spawn robots
print("\n[1] SPAWNING ROBOTS...")
result = ros2_bridge.spawn_robot("turtlebot_1", "TurtleBot3", 
                                 position={"x": 0.0, "y": 0.0, "z": 0.0},
                                 agent_id="agent-turtle")
test("Robot spawned", result.get("status") == "spawned")
test("3 topics created", result.get("topics") == 3)

# 2. Verify topic creation
print("\n[2] VERIFYING TOPICS...")
stats = ros2_bridge.get_stats()
test("Topics registered", stats["topics"] >= 3)
test("Robot registered", stats["robots"] == 1)
test("Has publishers", stats["publishers"] >= 1)
test("Has subscribers", stats["subscribers"] >= 2)

# 3. Publish velocity command
print("\n[3] PUBLISHING VELOCITY COMMAND...")
cmd = {"linear": {"x": 0.5, "y": 0.0, "z": 0.0}, "angular": {"z": 0.1}}
result = ros2_bridge.publish("turtlebot_1/cmd_vel", cmd)
test("Command published", result.get("status") == "published")

# Check robot moved
state = ros2_bridge.get_robot_state("turtlebot_1")
test("Robot position updated", state["position"]["x"] > 0.0)
print(f"   Position: x={state['position']['x']:.4f}, y={state['position']['y']:.4f}")

# 4. Simulate sensor data
print("\n[4] SIMULATING SENSOR DATA...")
lidar_data = {"ranges": [1.5, 1.2, 0.8, 1.0, 1.4], "angle_min": -1.57, "angle_max": 1.57}
result = ros2_bridge.simulate_sensor("turtlebot_1", "lidar", lidar_data)
test("Sensor data simulated", result.get("status") == "simulated")

state = ros2_bridge.get_robot_state("turtlebot_1")
test("Sensor data stored", "lidar" in state["sensors"])

# 5. Link agent to robot
print("\n[5] LINKING AGENT TO ROBOT...")
result = ros2_bridge.link_agent("turtlebot_1", "agent-governed-turtle")
test("Agent linked", result.get("status") == "linked")

state = ros2_bridge.get_robot_state("turtlebot_1")
test("Agent ID updated", state["agent_id"] == "agent-governed-turtle")

# 6. Spawn second robot
print("\n[6] SPAWNING SECOND ROBOT...")
result = ros2_bridge.spawn_robot("drone_1", "QuadCopter", 
                                 position={"x": 5.0, "y": 5.0, "z": 2.0})
test("Second robot spawned", result.get("status") == "spawned")

stats = ros2_bridge.get_stats()
test("2 robots total", stats["robots"] == 2)
test("6 topics total", stats["topics"] == 6)

# 7. Integration with HAL
print("\n[7] TESTING HAL INTEGRATION...")
# Register ROS2 sensor as HAL device
def ros_lidar_read():
    state = ros2_bridge.get_robot_state("turtlebot_1")
    return state["sensors"].get("lidar", {})

hal.register_sensor("ros_lidar_turtlebot", "ROS2 LIDAR (TurtleBot)", ros_lidar_read)
sensor_result = hal.read_sensor("ros_lidar_turtlebot")
test("ROS sensor accessible via HAL", sensor_result.get("status") == "ok")

# Summary
print("\n" + "=" * 70)
print("ROS2/GAZEBO BRIDGE TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ ROS2/GAZEBO BRIDGE VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
