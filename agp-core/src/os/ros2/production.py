"""
AGP-CORE: Production ROS2 Adapter
Real hardware integration with safety watchdog.

This module provides:
1. Real ROS2 connection (when rclpy is available)
2. Hardware safety watchdog
3. Graceful degradation to simulation mode
"""

import asyncio
import time
from typing import Dict, Optional, Callable, Any
from dataclasses import dataclass
import structlog

logger = structlog.get_logger()

# Try to import rclpy for real ROS2 support
try:
    import rclpy
    from rclpy.node import Node
    from geometry_msgs.msg import Twist
    from sensor_msgs.msg import LaserScan
    from nav_msgs.msg import Odometry
    ROS2_AVAILABLE = True
except ImportError:
    ROS2_AVAILABLE = False
    logger.warning("rclpy_not_available", msg="Running in simulation-only mode")

@dataclass
class WatchdogConfig:
    """Safety watchdog configuration"""
    heartbeat_timeout_ms: int = 500   # Max time between heartbeats
    emergency_stop_on_timeout: bool = True
    max_velocity: float = 1.0         # m/s
    max_angular: float = 1.0          # rad/s

class SafetyWatchdog:
    """
    Hardware safety watchdog.
    Monitors robot health and triggers emergency stop on failures.
    """
    
    def __init__(self, config: WatchdogConfig = None):
        self.config = config or WatchdogConfig()
        self.last_heartbeat = time.time()
        self.is_armed = False
        self.emergency_stop_triggered = False
        self.on_emergency_stop: Optional[Callable] = None
        
        logger.info("safety_watchdog_initialized", 
                   timeout_ms=self.config.heartbeat_timeout_ms)
    
    def arm(self):
        """Arm the watchdog"""
        self.is_armed = True
        self.last_heartbeat = time.time()
        logger.info("watchdog_armed")
    
    def disarm(self):
        """Disarm the watchdog"""
        self.is_armed = False
        logger.info("watchdog_disarmed")
    
    def heartbeat(self):
        """Send heartbeat to keep watchdog happy"""
        self.last_heartbeat = time.time()
    
    def check(self) -> bool:
        """Check watchdog status. Returns False if emergency stop needed."""
        if not self.is_armed:
            return True
        
        elapsed_ms = (time.time() - self.last_heartbeat) * 1000
        
        if elapsed_ms > self.config.heartbeat_timeout_ms:
            if not self.emergency_stop_triggered:
                self._trigger_emergency_stop("Heartbeat timeout")
            return False
        
        return True
    
    def validate_velocity(self, linear: float, angular: float) -> Dict:
        """Validate and cap velocity commands"""
        capped = False
        
        if abs(linear) > self.config.max_velocity:
            linear = self.config.max_velocity if linear > 0 else -self.config.max_velocity
            capped = True
        
        if abs(angular) > self.config.max_angular:
            angular = self.config.max_angular if angular > 0 else -self.config.max_angular
            capped = True
        
        if capped:
            logger.warning("velocity_capped", linear=linear, angular=angular)
        
        return {"linear": linear, "angular": angular, "capped": capped}
    
    def _trigger_emergency_stop(self, reason: str):
        """Trigger emergency stop"""
        self.emergency_stop_triggered = True
        logger.error("emergency_stop_triggered", reason=reason)
        
        if self.on_emergency_stop:
            self.on_emergency_stop(reason)
    
    def reset(self):
        """Reset emergency stop state"""
        self.emergency_stop_triggered = False
        self.last_heartbeat = time.time()
        logger.info("watchdog_reset")

class ProductionROS2Adapter:
    """
    Production-ready ROS2 adapter with safety features.
    Falls back to simulation mode if rclpy unavailable.
    """
    
    def __init__(self, node_name: str = "agp_os_robot"):
        self.node_name = node_name
        self.simulation_mode = not ROS2_AVAILABLE
        self.watchdog = SafetyWatchdog()
        self.connected = False
        
        # ROS2 node (if available)
        self.node = None
        self.publishers: Dict[str, Any] = {}
        self.subscriptions: Dict[str, Any] = {}
        
        # Simulation fallback
        from src.os.ros2.bridge import ros2_bridge
        self.sim_bridge = ros2_bridge
        
        logger.info("production_adapter_initialized", 
                   simulation=self.simulation_mode)
    
    def connect(self) -> Dict:
        """Initialize ROS2 connection"""
        if self.simulation_mode:
            self.connected = True
            return {"status": "connected", "mode": "simulation"}
        
        try:
            if not rclpy.ok():
                rclpy.init()
            
            self.node = rclpy.create_node(self.node_name)
            self.connected = True
            self.watchdog.arm()
            
            logger.info("ros2_connected", node=self.node_name)
            return {"status": "connected", "mode": "hardware"}
        
        except Exception as e:
            logger.error("ros2_connection_failed", error=str(e))
            self.simulation_mode = True
            self.connected = True
            return {"status": "connected", "mode": "simulation", "fallback": True}
    
    def disconnect(self):
        """Shutdown ROS2 connection"""
        self.watchdog.disarm()
        
        if self.node:
            self.node.destroy_node()
            self.node = None
        
        self.connected = False
        logger.info("ros2_disconnected")
    
    def create_velocity_publisher(self, topic: str) -> str:
        """Create a velocity command publisher"""
        if self.simulation_mode:
            from src.os.ros2.bridge import ROSMessageType
            self.sim_bridge.create_publisher(topic, ROSMessageType.CMD_VEL)
            return topic
        
        pub = self.node.create_publisher(Twist, topic, 10)
        self.publishers[topic] = pub
        logger.info("velocity_publisher_created", topic=topic)
        return topic
    
    def publish_velocity(self, topic: str, linear_x: float, angular_z: float) -> Dict:
        """Publish velocity command with safety validation"""
        # Watchdog check
        if not self.watchdog.check():
            return {"status": "blocked", "reason": "Emergency stop active"}
        
        # Validate velocity
        validated = self.watchdog.validate_velocity(linear_x, angular_z)
        
        if self.simulation_mode:
            cmd = {
                "linear": {"x": validated["linear"], "y": 0.0, "z": 0.0},
                "angular": {"x": 0.0, "y": 0.0, "z": validated["angular"]}
            }
            result = self.sim_bridge.publish(topic, cmd)
            self.watchdog.heartbeat()
            return {**result, "capped": validated["capped"]}
        
        # Real ROS2 publish
        if topic not in self.publishers:
            return {"status": "error", "reason": "Publisher not found"}
        
        msg = Twist()
        msg.linear.x = validated["linear"]
        msg.angular.z = validated["angular"]
        
        self.publishers[topic].publish(msg)
        self.watchdog.heartbeat()
        
        return {"status": "published", "capped": validated["capped"]}
    
    def emergency_stop(self, topic: str) -> Dict:
        """Send emergency stop (zero velocity)"""
        logger.warning("emergency_stop_commanded", topic=topic)
        return self.publish_velocity(topic, 0.0, 0.0)
    
    def get_status(self) -> Dict:
        """Get adapter status"""
        return {
            "connected": self.connected,
            "mode": "simulation" if self.simulation_mode else "hardware",
            "ros2_available": ROS2_AVAILABLE,
            "watchdog_armed": self.watchdog.is_armed,
            "emergency_stop": self.watchdog.emergency_stop_triggered,
            "publishers": len(self.publishers),
            "subscriptions": len(self.subscriptions)
        }

# Global instance
production_adapter = ProductionROS2Adapter()
