"""
AGP-OS BioKernel
The central nervous system of the Agent Operating System.
"""

import asyncio
import time
import structlog
from typing import Dict, List, Optional, Any

from src.models import Agent, Hormone, EndocrineState
from src.agents import AGPAgent, agent_registry, AgentFactory
from src.telos import telos_membrane, Decision, ConsequenceTier, ExecutionBlocked
from src.os.process import ProcessControlBlock, ProcessState, ResourceUsage
from src.os.context_manager import context_manager
# Note: filesystem is imported lazily to avoid circular imports
from src.os.persistence.database import db
from src.os.observability.prometheus import prom_metrics
from src.os.resilience.circuit_breaker import db_circuit, with_circuit_breaker
from src.core.reputation_engine import ReputationEngine

logger = structlog.get_logger()

class BioKernel:
    """
    The Micro-Kernel for AI Agents.
    Manages:
    1. Process Lifecycle (Spawn, Kill)
    2. Endocrine Scheduling (Priority)
    3. Resource Allocation (Tokens)
    """
    
    def __init__(self):
        self.scheduler_interval = 0.1 # 100ms tick
        self.reputation = ReputationEngine()
        self.running = False
        self.start_time = time.time()
        self.pid_counter = 1
        self.process_table: Dict[int, ProcessControlBlock] = {}
        self.daemons: Dict[str, int] = {}
        
        # Initialize filesystem mount points
        self._init_fs()
        
        logger.info("kernel_initialized", version="1.0.0-prod")

    def _init_fs(self):
        """Mount essential filesystems"""
        # (This is usually handled in src/os/fs/__init__.py but we ensure it here)
        pass

    def boot(self, recover: bool = True):
        """Boot the Bio-Kernel"""
        self.running = True
        self.start_time = time.time()
        
        # 1. Database connection check
        logger.info("kernel_boot", status="starting", persistence="sqlite")
        
        # 2. Recovery from database
        if recover:
            self._recover_state()
            
        # 3. Spawn System_Init if not present
        if not self.process_table:
            from src.agents import AgentFactory
            init_agent = AgentFactory.create_system_agent("System_Init")
            self.spawn_process(init_agent)
            
        logger.info("kernel_boot", status="ready", uptime=f"{time.time() - self.start_time:.2f}s")

    def _recover_state(self):
        """Recover process table from persistent database"""
        try:
            persisted_procs = db.load_all_processes()
            for proc_data in persisted_procs:
                pid = proc_data['pid']
                pcb = ProcessControlBlock(
                    pid=pid,
                    agent_id=proc_data['agent_id'],
                    name=proc_data['name']
                )
                pcb.state = ProcessState(proc_data['state'])
                pcb.priority = proc_data['priority']
                pcb.nice = proc_data['nice']
                pcb.quota_tokens = proc_data['quota_tokens']
                pcb.created_at = proc_data['created_at']
                pcb.total_runtime = proc_data['total_runtime']
                pcb.usage = ResourceUsage(
                    cpu_cycles=proc_data['cpu_cycles'],
                    tokens_used=proc_data['tokens_used'],
                    memory_pages=proc_data['memory_pages'],
                    disk_bytes=proc_data['disk_bytes']
                )
                self.process_table[pid] = pcb
                if pid >= self.pid_counter:
                    self.pid_counter = pid + 1
            
            logger.info("kernel_recovery", status="success", processes_recovered=len(self.process_table))
        except Exception as e:
            logger.error("kernel_recovery_failed", error=str(e))

    def spawn_process(self, agent: Agent) -> int:
        """Create a new process for an agent"""
        pid = self.pid_counter
        self.pid_counter += 1
        
        pcb = ProcessControlBlock(
            pid=pid,
            agent_id=str(agent.id),
            name=agent.name
        )
        
        # Initial priority calculation
        pcb.calculate_priority(agent.endocrine_state)
        
        self.process_table[pid] = pcb

        # Register with TELOS membrane so execution can cross (authority scope)
        try:
            telos_membrane.register_agent(str(agent.id), ["execute:*", "read:*", "write:*"])
        except Exception:
            pass  # Idempotent if already registered

        # Persist to DB
        db.save_process(pcb)
        
        # Create AGP Agent wrapper to sync with Registry
        agp_agent = AGPAgent(
            name=agent.name,
            initial_state=agent.endocrine_state
        )
        agp_agent.id = agent.id # Synchronize ID with PCB/Model
        agent_registry.register_agent(agp_agent)
        
        logger.info("process_spawn", pid=pid, name=agent.name, priority=f"{pcb.priority:.2f}")
        prom_metrics.update_process_priority(pid, agent.name, pcb.priority)
        return pid

    def spawn_system_process(self, name: str, agent: Agent) -> int:
        """Spawn a high-priority system process"""
        pid = self.spawn_process(agent)
        pcb = self.process_table[pid]
        pcb.nice = -20 # Max priority
        pcb.priority = 1.0
        self.daemons[name] = pid
        return pid

    async def schedule(self):
        """
        The Scheduler Loop.
        Selects next process based on Endocrine Priority.
        """
        while self.running:
            runnable = [p for p in self.process_table.values() if p.is_runnable]
            if not runnable:
                await asyncio.sleep(0.1)
                continue
            
            # Update metrics
            state_counts = {}
            for state in ProcessState:
                state_counts[state.value] = len([p for p in runnable if p.state == state])
            prom_metrics.update_process_count(state_counts)
            prom_metrics.update_kernel_uptime(time.time() - self.start_time)

            # Sort by priority
            runnable.sort(key=lambda p: p.priority, reverse=True)
            
            next_process = runnable[0]
            await self.context_switch(next_process)
            
            # Update DB with runtime stats periodically
            if int(next_process.total_runtime) % 10 == 0:
                db.save_process(next_process)

            await asyncio.sleep(self.scheduler_interval)

    async def context_switch(self, pcb: ProcessControlBlock):
        """Switch execution context to the given process. No execution without TELOS crossing."""
        decision = Decision(
            decision_id=f"{pcb.pid}:{time.time()}",
            action="execute",
            agent_id=pcb.agent_id,
            tier=ConsequenceTier.MEDIUM,
            context={"process_id": pcb.pid, "name": pcb.name},
        )
        result = telos_membrane.request_crossing(decision, required_scope="execute:*")
        if not result.allowed:
            raise ExecutionBlocked(result.reason or "TELOS membrane rejected execution")

        if pcb.state != ProcessState.RUNNING:
            pcb.state = ProcessState.RUNNING
        pcb.last_scheduled_at = time.time()
        
        # Simulate Time Slice (Quantum)
        # Higher cortisol = shorter time slice (distracted)
        # Higher serotonin = longer time slice (flow state)
        # But we don't block here, we just mark it.

    def kill_process(self, pid: int, reason: str = "term"):
        """Terminate a process"""
        if pid in self.process_table:
            name = self.process_table[pid].name
            del self.process_table[pid]
            db.delete_process(pid)
            logger.info("process_kill", pid=pid, name=name, reason=reason)
            return True
        return False

    def ps(self) -> List[Dict]:
        """List processes (for 'ps' command)"""
        return [
            {
                "PID": p.pid,
                "Name": p.name,
                "State": p.state.value,
                "Pri": f"{p.priority:.2f}",
                "Time": f"{p.total_runtime:.2f}s",
                "Tokens": p.usage.tokens_used
            }
            for p in self.process_table.values()
        ]

# Global Kernel Instance
kernel = BioKernel()
