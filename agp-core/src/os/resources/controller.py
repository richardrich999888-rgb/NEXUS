"""
AGP-CORE: Resource Controller
Enforces hard limits on CPU, Memory, Tokens, and I/O per agent.
"""

import time
from typing import Dict, Optional
from dataclasses import dataclass, field
from enum import Enum
import structlog

logger = structlog.get_logger()

class ResourceType(Enum):
    CPU_CYCLES = "cpu"
    MEMORY_MB = "memory"
    TOKENS = "tokens"
    IO_OPS = "io"

@dataclass
class ResourceQuota:
    """Resource limits for an agent"""
    cpu_cycles_max: int = 1000000      # Max CPU cycles per period
    memory_mb_max: float = 512.0       # Max memory in MB
    tokens_max: int = 100000           # Max LLM tokens per day
    io_ops_max: int = 1000             # Max I/O operations per minute
    
    # Priority (higher = more access when contended)
    priority: float = 0.5              # 0.0 to 1.0

@dataclass
class ResourceUsage:
    """Current resource consumption for an agent"""
    cpu_cycles: int = 0
    memory_mb: float = 0.0
    tokens_used: int = 0
    io_ops: int = 0
    
    # Tracking
    last_reset: float = field(default_factory=time.time)
    period_seconds: int = 86400  # 24 hours for tokens

class ResourceController:
    """
    Enforces resource quotas per agent.
    Integrates with governance for priority-based allocation.
    """
    
    def __init__(self):
        # agent_id -> ResourceQuota
        self.quotas: Dict[str, ResourceQuota] = {}
        # agent_id -> ResourceUsage
        self.usage: Dict[str, ResourceUsage] = {}
        
        # Global system limits
        self.system_memory_mb = 16384.0  # 16 GB total
        self.system_memory_used = 0.0
        
        logger.info("resource_controller_initialized")
    
    def register_agent(self, agent_id: str, quota: Optional[ResourceQuota] = None):
        """Register an agent with resource quotas"""
        self.quotas[agent_id] = quota or ResourceQuota()
        self.usage[agent_id] = ResourceUsage()
        logger.info("agent_quota_set", agent_id=agent_id, 
                   memory=self.quotas[agent_id].memory_mb_max,
                   tokens=self.quotas[agent_id].tokens_max)
    
    def set_priority(self, agent_id: str, priority: float):
        """Update priority based on governance alignment"""
        if agent_id in self.quotas:
            self.quotas[agent_id].priority = max(0.0, min(1.0, priority))
            logger.info("priority_updated", agent_id=agent_id, priority=priority)
    
    def request_resource(self, agent_id: str, resource: ResourceType, amount: float) -> Dict:
        """Request resource allocation, returns approval or denial"""
        if agent_id not in self.quotas:
            return {"status": "error", "reason": "Agent not registered"}
        
        quota = self.quotas[agent_id]
        usage = self.usage[agent_id]
        
        # Check for period reset (tokens reset daily)
        self._check_period_reset(usage)
        
        # Evaluate based on resource type
        if resource == ResourceType.CPU_CYCLES:
            if usage.cpu_cycles + amount > quota.cpu_cycles_max:
                return self._deny("CPU quota exceeded", usage.cpu_cycles, quota.cpu_cycles_max)
            usage.cpu_cycles += int(amount)
            
        elif resource == ResourceType.MEMORY_MB:
            # Check both agent quota and global system limit
            if usage.memory_mb + amount > quota.memory_mb_max:
                return self._deny("Memory quota exceeded", usage.memory_mb, quota.memory_mb_max)
            if self.system_memory_used + amount > self.system_memory_mb:
                return self._deny("System memory exhausted", self.system_memory_used, self.system_memory_mb)
            usage.memory_mb += amount
            self.system_memory_used += amount
            
        elif resource == ResourceType.TOKENS:
            if usage.tokens_used + amount > quota.tokens_max:
                return self._deny("Token quota exceeded", usage.tokens_used, quota.tokens_max)
            usage.tokens_used += int(amount)
            
        elif resource == ResourceType.IO_OPS:
            if usage.io_ops + amount > quota.io_ops_max:
                return self._deny("I/O quota exceeded", usage.io_ops, quota.io_ops_max)
            usage.io_ops += int(amount)
        
        return {"status": "granted", "resource": resource.value, "amount": amount}
    
    def release_resource(self, agent_id: str, resource: ResourceType, amount: float):
        """Release previously allocated resources"""
        if agent_id not in self.usage:
            return
        
        usage = self.usage[agent_id]
        
        if resource == ResourceType.MEMORY_MB:
            usage.memory_mb = max(0, usage.memory_mb - amount)
            self.system_memory_used = max(0, self.system_memory_used - amount)
    
    def get_usage(self, agent_id: str) -> Dict:
        """Get current resource usage for an agent"""
        if agent_id not in self.usage:
            return {"error": "Agent not found"}
        
        usage = self.usage[agent_id]
        quota = self.quotas[agent_id]
        
        return {
            "cpu": {"used": usage.cpu_cycles, "max": quota.cpu_cycles_max, 
                   "pct": usage.cpu_cycles / quota.cpu_cycles_max * 100},
            "memory": {"used": usage.memory_mb, "max": quota.memory_mb_max,
                      "pct": usage.memory_mb / quota.memory_mb_max * 100},
            "tokens": {"used": usage.tokens_used, "max": quota.tokens_max,
                      "pct": usage.tokens_used / quota.tokens_max * 100},
            "io": {"used": usage.io_ops, "max": quota.io_ops_max,
                  "pct": usage.io_ops / quota.io_ops_max * 100},
            "priority": quota.priority
        }
    
    def get_system_status(self) -> Dict:
        """Get global system resource status"""
        return {
            "memory": {
                "total": self.system_memory_mb,
                "used": self.system_memory_used,
                "available": self.system_memory_mb - self.system_memory_used,
                "pct_used": self.system_memory_used / self.system_memory_mb * 100
            },
            "agents_registered": len(self.quotas)
        }
    
    def _check_period_reset(self, usage: ResourceUsage):
        """Reset usage counters if period has elapsed"""
        now = time.time()
        if now - usage.last_reset > usage.period_seconds:
            usage.tokens_used = 0
            usage.cpu_cycles = 0
            usage.io_ops = 0
            usage.last_reset = now
            logger.info("usage_period_reset")
    
    def _deny(self, reason: str, current: float, max_val: float) -> Dict:
        """Build denial response"""
        logger.warning("resource_denied", reason=reason, current=current, max=max_val)
        return {
            "status": "denied",
            "reason": reason,
            "current": current,
            "max": max_val
        }

# Global instance
resource_controller = ResourceController()
