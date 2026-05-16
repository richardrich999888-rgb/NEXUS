"""
AGP-OS Process Management
Defines the Process Control Block (PCB) for AI Agents.
"""

import uuid
import time
from enum import Enum
from dataclasses import dataclass, field
from typing import Dict, Optional, Any

from src.models import (
    Agent, AgentType, EndocrineState, Hormone, 
    PrivilegeLevel, HealthStatus
)

class ProcessState(str, Enum):
    CREATED = "created"
    READY = "ready"
    RUNNING = "running"
    WAITING = "waiting" # IO wait (Task execution)
    SLEEPING = "sleeping" # Throttled/Cool-down
    TERMINATED = "terminated"
    ZOMBIE = "zombie" # Finished but not reaped

@dataclass
class ResourceUsage:
    """Resource accounting for a process"""
    cpu_cycles: int = 0      # Logical cycles (steps)
    tokens_used: int = 0     # LLM tokens
    memory_pages: int = 0    # Context window utilization
    disk_bytes: int = 0      # RAG/Storage usage

@dataclass
class ProcessControlBlock:
    """
    PCB: The Kernel's view of an Agent.
    """
    pid: int
    agent_id: str
    name: str
    
    # Process State
    state: ProcessState = ProcessState.CREATED
    priority: float = 0.0 # 0.0 to 1.0 (Higher = Scheduler preference)
    nice: int = 0         # User-defined adjustment (-20 to 19)
    
    # Scheduling info
    created_at: float = field(default_factory=time.time)
    last_scheduled_at: float = 0.0
    total_runtime: float = 0.0
    
    # Resources
    usage: ResourceUsage = field(default_factory=ResourceUsage)
    quota_tokens: int = 100000 # Max tokens per day
    
    # Context (Memory Management)
    page_table_id: str = "" # Reference to Chroma/FAISS collection ID
    
    # Signals
    pending_signals: list = field(default_factory=list)

    @property
    def is_runnable(self) -> bool:
        return self.state in [ProcessState.READY, ProcessState.RUNNING]

    def update_usage(self, tokens: int, runtime: float):
        self.usage.tokens_used += tokens
        self.usage.cpu_cycles += 1
        self.total_runtime += runtime

    def calculate_priority(self, endocrine_state: EndocrineState) -> float:
        """
        Calculate dynamic priority based on biological state.
        
        Formula:
        Base Priority (Privilege)
        + Dopamine (Focus/Drive)
        + Norepinephrine (Urgency)
        - Cortisol * 0.5 (Stress penalty, unless 'Fight or Flight' mode)
        + Nice value adjustment
        """
        base_scores = {
            "minimal": 0.1,
            "basic": 0.3,
            "standard": 0.5,
            "elevated": 0.8,
            "maximum": 1.0
        }
        # Assuming we can access privilege somehow, or passed in. 
        # For now, base is 0.5
        base = 0.5 
        
        levels = endocrine_state.levels
        dopamine = levels.get(Hormone.DOPAMINE, 0.5)
        norepi = levels.get(Hormone.NOREPINEPHRINE, 0.5)
        cortisol = levels.get(Hormone.CORTISOL, 0.5)
        
        # Bio-Boost
        bio_priority = base + (dopamine * 0.3) + (norepi * 0.2)
        
        # Stress Regulation: High stress de-prioritizes standard tasks to prevent burnout
        if cortisol > 0.9:
             bio_priority = 0.1 # Panic Mode: Immediate throttling
        elif cortisol > 0.7:
             bio_priority -= 0.5
             
        # Normalize
        self.priority = max(0.0, min(1.0, bio_priority))
        return self.priority
