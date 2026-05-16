"""
AGP-CORE: Real-Time Scheduler (RTOS Integration)
Priority-based scheduling with deadline awareness for motor control.
"""

import time
import asyncio
import heapq
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from enum import IntEnum
import structlog

logger = structlog.get_logger()

class TaskPriority(IntEnum):
    """Task priority levels (lower value = higher priority)"""
    CRITICAL = 0    # Motor safety, emergency stops
    HIGH = 1        # Sensor polling, actuator commands
    NORMAL = 2      # Standard agent operations
    LOW = 3         # Background processing, analytics
    IDLE = 4        # Cleanup, logging, non-essential

@dataclass(order=True)
class RTTask:
    """Real-time task with deadline"""
    priority: TaskPriority
    deadline: float  # Unix timestamp when task must complete
    task_id: str = field(compare=False)
    func: Callable = field(compare=False)
    args: tuple = field(default=(), compare=False)
    kwargs: dict = field(default_factory=dict, compare=False)
    created_at: float = field(default_factory=time.time, compare=False)
    
    @property
    def time_to_deadline(self) -> float:
        return self.deadline - time.time()
    
    @property
    def is_overdue(self) -> bool:
        return time.time() > self.deadline

class RTScheduler:
    """
    Real-Time Scheduler with priority queues and deadline awareness.
    
    Design:
    - Critical tasks (motor safety) always run first
    - Deadline-aware: tasks approaching deadline get boosted
    - Preemption: long-running tasks can be interrupted
    """
    
    def __init__(self):
        # Priority queue (min-heap by priority, then deadline)
        self.task_queue: List[RTTask] = []
        
        # Running tasks
        self.running: Dict[str, RTTask] = {}
        
        # Statistics
        self.completed_count = 0
        self.missed_deadlines = 0
        
        # Configuration
        self.max_concurrent = 4
        self.deadline_boost_threshold = 0.1  # Boost if < 100ms to deadline
        
        self._running = False
        
        logger.info("rt_scheduler_initialized", max_concurrent=self.max_concurrent)
    
    def submit(self, task_id: str, func: Callable, 
               priority: TaskPriority = TaskPriority.NORMAL,
               deadline_ms: int = 1000,
               args: tuple = (), kwargs: dict = None) -> str:
        """Submit a task to the scheduler"""
        deadline = time.time() + (deadline_ms / 1000.0)
        
        task = RTTask(
            priority=priority,
            deadline=deadline,
            task_id=task_id,
            func=func,
            args=args,
            kwargs=kwargs or {}
        )
        
        heapq.heappush(self.task_queue, task)
        logger.debug("task_submitted", id=task_id, priority=priority.name)
        
        return task_id
    
    def submit_critical(self, task_id: str, func: Callable, 
                       deadline_ms: int = 50, **kwargs) -> str:
        """Submit a critical motor control task (highest priority, tight deadline)"""
        return self.submit(
            task_id, func, 
            priority=TaskPriority.CRITICAL,
            deadline_ms=deadline_ms,
            **kwargs
        )
    
    def submit_background(self, task_id: str, func: Callable, **kwargs) -> str:
        """Submit a background governance task (low priority, relaxed deadline)"""
        return self.submit(
            task_id, func,
            priority=TaskPriority.LOW,
            deadline_ms=5000,  # 5 second deadline
            **kwargs
        )
    
    async def run_once(self) -> Optional[Dict]:
        """Execute the highest priority task once"""
        if not self.task_queue:
            return None
        
        # Apply deadline boosting before selection
        self._apply_deadline_boost()
        
        task = heapq.heappop(self.task_queue)
        
        # Check for deadline miss
        if task.is_overdue:
            self.missed_deadlines += 1
            logger.warning("deadline_missed", id=task.task_id, 
                          overdue_ms=(time.time() - task.deadline) * 1000)
        
        # Execute
        self.running[task.task_id] = task
        start = time.perf_counter()
        
        try:
            if asyncio.iscoroutinefunction(task.func):
                result = await task.func(*task.args, **task.kwargs)
            else:
                result = task.func(*task.args, **task.kwargs)
            
            elapsed = (time.perf_counter() - start) * 1000
            self.completed_count += 1
            
            return {
                "task_id": task.task_id,
                "priority": task.priority.name,
                "result": result,
                "elapsed_ms": elapsed,
                "deadline_met": not task.is_overdue
            }
        except Exception as e:
            logger.error("task_failed", id=task.task_id, error=str(e))
            return {"task_id": task.task_id, "error": str(e)}
        finally:
            del self.running[task.task_id]
    
    async def run_loop(self, duration_seconds: float = None):
        """Run the scheduler loop"""
        self._running = True
        start = time.time()
        
        while self._running:
            if duration_seconds and (time.time() - start) > duration_seconds:
                break
            
            if self.task_queue:
                await self.run_once()
            else:
                await asyncio.sleep(0.001)  # 1ms idle loop
    
    def stop(self):
        """Stop the scheduler loop"""
        self._running = False
    
    def _apply_deadline_boost(self):
        """Boost priority of tasks approaching deadline"""
        boosted = []
        for task in self.task_queue:
            if task.time_to_deadline < self.deadline_boost_threshold:
                if task.priority > TaskPriority.CRITICAL:
                    task.priority = TaskPriority(task.priority - 1)
                    boosted.append(task.task_id)
        
        if boosted:
            heapq.heapify(self.task_queue)
            logger.debug("deadline_boost_applied", tasks=boosted)
    
    def get_stats(self) -> Dict:
        """Get scheduler statistics"""
        return {
            "queued": len(self.task_queue),
            "running": len(self.running),
            "completed": self.completed_count,
            "missed_deadlines": self.missed_deadlines,
            "queue_breakdown": self._get_queue_breakdown()
        }
    
    def _get_queue_breakdown(self) -> Dict[str, int]:
        """Get count of tasks by priority"""
        breakdown = {p.name: 0 for p in TaskPriority}
        for task in self.task_queue:
            breakdown[task.priority.name] += 1
        return breakdown

# Global instance
rt_scheduler = RTScheduler()
