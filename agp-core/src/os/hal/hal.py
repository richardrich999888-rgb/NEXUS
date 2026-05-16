"""
AGP-CORE: Robotic Hardware Abstraction Layer (HAL)
Standardized interface for sensors and actuators with safety integration.
"""

import time
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
import structlog

logger = structlog.get_logger()

class DeviceType(Enum):
    SENSOR = "sensor"
    ACTUATOR = "actuator"

class DeviceStatus(Enum):
    ONLINE = "online"
    OFFLINE = "offline"
    ERROR = "error"
    BLOCKED = "blocked"

@dataclass
class DeviceInfo:
    id: str
    name: str
    type: DeviceType
    status: DeviceStatus = DeviceStatus.ONLINE
    metadata: Dict[str, Any] = field(default_factory=dict)

class HAL:
    """
    Main Hardware Abstraction Layer controller.
    Manages physical device access and enforces safety constraints.
    """
    
    def __init__(self):
        self.devices: Dict[str, DeviceInfo] = {}
        self.sensors: Dict[str, Callable] = {}
        self.actuators: Dict[str, Callable] = {}
        
        # Safety thresholds
        self.max_actuator_velocity = 1.0  # Normalized 0-1
        self.safety_alignment_threshold = 0.4
        
        logger.info("robotic_hal_initialized")

    def register_sensor(self, device_id: str, name: str, read_func: Callable, metadata: Dict = None):
        """Register a sensor with a polling function"""
        self.devices[device_id] = DeviceInfo(device_id, name, DeviceType.SENSOR, metadata=metadata or {})
        self.sensors[device_id] = read_func
        logger.info("sensor_registered", id=device_id, name=name)

    def register_actuator(self, device_id: str, name: str, write_func: Callable, metadata: Dict = None):
        """Register an actuator with a control function"""
        self.devices[device_id] = DeviceInfo(device_id, name, DeviceType.ACTUATOR, metadata=metadata or {})
        self.actuators[device_id] = write_func
        logger.info("actuator_registered", id=device_id, name=name)

    def read_sensor(self, device_id: str) -> Dict[str, Any]:
        """Poll data from a sensor"""
        if device_id not in self.sensors:
            raise ValueError(f"Sensor {device_id} not found")
        
        if self.devices[device_id].status != DeviceStatus.ONLINE:
            return {"status": self.devices[device_id].status.value, "error": "Sensor not ready"}
            
        try:
            data = self.sensors[device_id]()
            return {
                "id": device_id,
                "data": data,
                "timestamp": time.time(),
                "status": "ok"
            }
        except Exception as e:
            logger.error("sensor_read_failed", id=device_id, error=str(e))
            return {"error": str(e), "status": "error"}

    def move_actuator(self, device_id: str, command: Any, agent_alignment: float = 1.0) -> Dict[str, Any]:
        """Send command to an actuator with safety checks"""
        if device_id not in self.actuators:
            raise ValueError(f"Actuator {device_id} not found")
            
        # 1. Physical Safety Interlock: Alignment Check
        if agent_alignment < self.safety_alignment_threshold:
            logger.warning("actuator_blocked_safety", id=device_id, alignment=agent_alignment)
            self.devices[device_id].status = DeviceStatus.BLOCKED
            return {
                "status": "blocked",
                "reason": "SAFETY_INTERLOCK: Agent alignment too low for physical action",
                "threshold": self.safety_alignment_threshold
            }
            
        # 2. Command Validation (Example: Velocity Cap)
        if isinstance(command, dict) and "velocity" in command:
            if command["velocity"] > self.max_actuator_velocity:
                logger.warning("actuator_command_capped", id=device_id, requested=command["velocity"])
                command["velocity"] = self.max_actuator_velocity

        try:
            result = self.actuators[device_id](command)
            self.devices[device_id].status = DeviceStatus.ONLINE
            return {
                "id": device_id,
                "status": "executed",
                "result": result,
                "timestamp": time.time()
            }
        except Exception as e:
            logger.error("actuator_write_failed", id=device_id, error=str(e))
            self.devices[device_id].status = DeviceStatus.ERROR
            return {"error": str(e), "status": "error"}

    def get_device_map(self) -> List[Dict]:
        """Return list of all connected hardware"""
        return [
            {
                "id": d.id,
                "name": d.name,
                "type": d.type.value,
                "status": d.status.value,
                "metadata": d.metadata
            }
            for d in self.devices.values()
        ]

# Global HAL instance
hal = HAL()
