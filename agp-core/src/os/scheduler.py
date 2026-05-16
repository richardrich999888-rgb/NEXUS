"""
AGP-OS: Advanced Scheduler
Preemptive scheduling with deadlock detection.
"""

import asyncio
import time
import structlog
from typing import Dict, List, Set, Optional
from dataclasses import dataclass
from enum import Enum

from src.os.process import ProcessControlBlock, ProcessState

logger = structlog.get_logger()

class SchedulerPolicy(Enum):
    FIFO = "fifo"
    ROUND_ROBIN = "round_robin"
    PRIORITY = "priority"
    ENDOCRINE = "endocrine"  # Our bio-inspired default

@dataclass
class ResourceLock:
    """Represents a resource lock held by a process"""
    resource_id: str
    owner_pid: int
    waiting_pids: List[int]

class AdvancedScheduler:
    """
    Advanced scheduler with preemption and deadlock detection.
    """
    
    def __init__(self, policy: SchedulerPolicy = SchedulerPolicy.ENDOCRINE):
        self.policy = policy
        self.quantum_ms = 100  # Time slice in milliseconds
        self.current_process: Optional[int] = None
        self.ready_queue: List[int] = []
        self.resource_locks: Dict[str, ResourceLock] = {}
        self.wait_for_graph: Dict[int, Set[int]] = {}  # PID -> PIDs it's waiting for
        self.running = False
    
    async def schedule(self, process_table: Dict[int, ProcessControlBlock]) -> Optional[int]:
        """
        Select next process to run.
        Returns PID of selected process.
        """
        # Get runnable processes
        runnable = [
            pcb for pcb in process_table.values() 
            if pcb.state in (ProcessState.READY, ProcessState.RUNNING)
        ]
        
        if not runnable:
            return None
        
        # Select based on policy
        if self.policy == SchedulerPolicy.FIFO:
            selected = runnable[0]
        
        elif self.policy == SchedulerPolicy.ROUND_ROBIN:
            # Rotate through processes
            if self.current_process and self.ready_queue:
                # Move current to back
                if self.current_process in [p.pid for p in runnable]:
                    runnable = [p for p in runnable if p.pid != self.current_process]
                    current = process_table.get(self.current_process)
                    if current:
                        runnable.append(current)
            selected = runnable[0] if runnable else None
        
        elif self.policy == SchedulerPolicy.PRIORITY:
            selected = max(runnable, key=lambda p: p.priority)
        
        elif self.policy == SchedulerPolicy.ENDOCRINE:
            # Bio-inspired: Priority based on hormones + starvation prevention
            now = time.time()
            
            def calculate_effective_priority(pcb):
                # Base priority from hormones
                base = pcb.priority
                
                # Aging: Boost priority for processes waiting too long
                wait_time = now - pcb.last_scheduled_at if pcb.last_scheduled_at else 0
                aging_boost = min(0.3, wait_time / 60.0)  # Max 0.3 boost after 1 min
                
                return base + aging_boost
            
            selected = max(runnable, key=calculate_effective_priority)
        
        else:
            selected = runnable[0]
        
        if selected:
            self.current_process = selected.pid
            return selected.pid
        
        return None
    
    def preempt(self, process_table: Dict[int, ProcessControlBlock], 
                new_priority_pid: int) -> bool:
        """
        Preempt current process if new process has higher priority.
        """
        if not self.current_process:
            return False
        
        current = process_table.get(self.current_process)
        new_proc = process_table.get(new_priority_pid)
        
        if not current or not new_proc:
            return False
        
        if new_proc.priority > current.priority + 0.1:  # Threshold to avoid thrashing
            logger.info("preemption", 
                       old_pid=current.pid, old_priority=current.priority,
                       new_pid=new_proc.pid, new_priority=new_proc.priority)
            
            current.state = ProcessState.READY
            new_proc.state = ProcessState.RUNNING
            self.current_process = new_priority_pid
            return True
        
        return False
    
    def acquire_resource(self, pid: int, resource_id: str) -> bool:
        """
        Try to acquire a resource lock.
        """
        if resource_id in self.resource_locks:
            lock = self.resource_locks[resource_id]
            if lock.owner_pid != pid:
                # Resource is locked, add to waiters
                lock.waiting_pids.append(pid)
                
                # Update wait-for graph
                if pid not in self.wait_for_graph:
                    self.wait_for_graph[pid] = set()
                self.wait_for_graph[pid].add(lock.owner_pid)
                
                logger.info("resource_wait", pid=pid, resource=resource_id, owner=lock.owner_pid)
                return False
        
        # Acquire lock
        self.resource_locks[resource_id] = ResourceLock(
            resource_id=resource_id,
            owner_pid=pid,
            waiting_pids=[]
        )
        
        logger.info("resource_acquired", pid=pid, resource=resource_id)
        return True
    
    def release_resource(self, pid: int, resource_id: str) -> Optional[int]:
        """
        Release a resource lock.
        Returns PID of next process to get the lock, if any.
        """
        if resource_id not in self.resource_locks:
            return None
        
        lock = self.resource_locks[resource_id]
        if lock.owner_pid != pid:
            return None
        
        next_pid = None
        if lock.waiting_pids:
            # Give to first waiter
            next_pid = lock.waiting_pids.pop(0)
            lock.owner_pid = next_pid
            
            # Update wait-for graph
            if next_pid in self.wait_for_graph:
                self.wait_for_graph[next_pid].discard(pid)
                if not self.wait_for_graph[next_pid]:
                    del self.wait_for_graph[next_pid]
            
            logger.info("resource_transferred", resource=resource_id, from_pid=pid, to_pid=next_pid)
        else:
            del self.resource_locks[resource_id]
            logger.info("resource_released", pid=pid, resource=resource_id)
        
        return next_pid
    
    def detect_deadlock(self) -> List[int]:
        """
        Detect deadlock using cycle detection in wait-for graph.
        Returns list of PIDs involved in deadlock.
        """
        visited = set()
        rec_stack = set()
        deadlocked = []
        
        def dfs(pid):
            visited.add(pid)
            rec_stack.add(pid)
            
            for waiting_for in self.wait_for_graph.get(pid, set()):
                if waiting_for not in visited:
                    if dfs(waiting_for):
                        return True
                elif waiting_for in rec_stack:
                    # Cycle found
                    return True
            
            rec_stack.remove(pid)
            return False
        
        for pid in self.wait_for_graph:
            if pid not in visited:
                if dfs(pid):
                    deadlocked.extend(list(rec_stack))
        
        if deadlocked:
            logger.warning("deadlock_detected", pids=deadlocked)
        
        return list(set(deadlocked))
    
    def resolve_deadlock(self, deadlocked_pids: List[int], 
                         process_table: Dict[int, ProcessControlBlock]):
        """
        Resolve deadlock by terminating lowest-priority process.
        """
        if not deadlocked_pids:
            return
        
        # Find lowest priority process in deadlock
        victim = min(
            [process_table[pid] for pid in deadlocked_pids if pid in process_table],
            key=lambda p: p.priority
        )
        
        logger.warning("deadlock_victim", pid=victim.pid, name=victim.name)
        
        # Terminate victim
        victim.state = ProcessState.TERMINATED
        
        # Release all resources held by victim
        for resource_id, lock in list(self.resource_locks.items()):
            if lock.owner_pid == victim.pid:
                self.release_resource(victim.pid, resource_id)

# Global scheduler instance
advanced_scheduler = AdvancedScheduler()
