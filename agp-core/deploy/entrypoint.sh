#!/bin/bash
# AGP-OS Robot Entrypoint
# Sources ROS2 and starts the AGP-OS robot controller

set -e

# Source ROS2
source /opt/ros/humble/setup.bash

# Wait for ROS2 master if needed
echo "[AGP-OS] Waiting for ROS2 network..."
sleep 2

# Check for robot connection
echo "[AGP-OS] Checking robot connection..."
ros2 topic list || echo "Warning: No ROS2 topics found"

# Start AGP-OS
echo "[AGP-OS] Starting AGP-OS Robot Controller..."
exec "$@"
