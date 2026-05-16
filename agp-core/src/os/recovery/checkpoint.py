"""
AGP-OS: Crash Recovery System
Checkpoint/restore and kernel panic handling.
"""

import json
import pickle
import structlog
from typing import Dict, Optional, Any
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path

logger = structlog.get_logger()

@dataclass
class KernelCheckpoint:
    """A snapshot of kernel state"""
    timestamp: datetime
    pid_counter: int
    process_states: Dict
    daemons: Dict
    running: bool
    
    def to_dict(self) -> Dict:
        return {
            "timestamp": self.timestamp.isoformat(),
            "pid_counter": self.pid_counter,
            "process_states": self.process_states,
            "daemons": self.daemons,
            "running": self.running
        }

class CheckpointManager:
    """
    Manages kernel state checkpoints for crash recovery.
    """
    
    def __init__(self, checkpoint_dir: str = "/tmp/agp-os/checkpoints"):
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(parents=True, exist_ok=True)
        self.last_checkpoint: Optional[KernelCheckpoint] = None
        self.auto_checkpoint_interval = 60  # seconds
    
    def create_checkpoint(self) -> KernelCheckpoint:
        """Create a checkpoint of current kernel state"""
        from src.os.kernel import kernel
        from src.os.process import ProcessState
        
        # Serialize process states
        process_states = {}
        for pid, pcb in kernel.process_table.items():
            process_states[pid] = {
                "pid": pcb.pid,
                "name": pcb.name,
                "agent_id": pcb.agent_id,
                "state": pcb.state.value,
                "priority": pcb.priority,
                "nice": pcb.nice,
                "quota_tokens": pcb.quota_tokens,
                "created_at": pcb.created_at,
                "total_runtime": pcb.total_runtime,
                "usage": {
                    "cpu_cycles": pcb.usage.cpu_cycles,
                    "tokens_used": pcb.usage.tokens_used,
                    "memory_pages": pcb.usage.memory_pages,
                    "disk_bytes": pcb.usage.disk_bytes
                }
            }
        
        checkpoint = KernelCheckpoint(
            timestamp=datetime.utcnow(),
            pid_counter=kernel.pid_counter,
            process_states=process_states,
            daemons=dict(kernel.daemons),
            running=kernel.running
        )
        
        self.last_checkpoint = checkpoint
        
        # Save to disk
        self._save_checkpoint(checkpoint)
        
        logger.info("checkpoint_created", 
                   process_count=len(process_states),
                   timestamp=checkpoint.timestamp.isoformat())
        
        return checkpoint
    
    def _save_checkpoint(self, checkpoint: KernelCheckpoint):
        """Save checkpoint to disk"""
        filename = f"checkpoint_{checkpoint.timestamp.strftime('%Y%m%d_%H%M%S')}.json"
        filepath = self.checkpoint_dir / filename
        
        with open(filepath, 'w') as f:
            json.dump(checkpoint.to_dict(), f, indent=2)
        
        # Keep only last 10 checkpoints
        checkpoints = sorted(self.checkpoint_dir.glob("checkpoint_*.json"))
        for old in checkpoints[:-10]:
            old.unlink()
    
    def load_latest_checkpoint(self) -> Optional[KernelCheckpoint]:
        """Load the most recent checkpoint"""
        checkpoints = sorted(self.checkpoint_dir.glob("checkpoint_*.json"))
        if not checkpoints:
            return None
        
        latest = checkpoints[-1]
        
        with open(latest, 'r') as f:
            data = json.load(f)
        
        checkpoint = KernelCheckpoint(
            timestamp=datetime.fromisoformat(data["timestamp"]),
            pid_counter=data["pid_counter"],
            process_states=data["process_states"],
            daemons=data["daemons"],
            running=data["running"]
        )
        
        logger.info("checkpoint_loaded", filepath=str(latest))
        return checkpoint
    
    def restore_from_checkpoint(self, checkpoint: KernelCheckpoint) -> bool:
        """Restore kernel state from checkpoint"""
        from src.os.kernel import kernel
        from src.os.process import ProcessControlBlock, ProcessState, ResourceUsage
        
        try:
            # Restore PID counter
            kernel.pid_counter = checkpoint.pid_counter
            kernel.daemons = dict(checkpoint.daemons)
            
            # Restore processes (without agents - they need to be re-created)
            for pid, state in checkpoint.process_states.items():
                pcb = ProcessControlBlock(
                    pid=int(pid),
                    agent_id=state["agent_id"],
                    name=state["name"]
                )
                pcb.state = ProcessState(state["state"])
                pcb.priority = state["priority"]
                pcb.nice = state["nice"]
                pcb.quota_tokens = state["quota_tokens"]
                pcb.created_at = state["created_at"]
                pcb.total_runtime = state["total_runtime"]
                pcb.usage = ResourceUsage(
                    cpu_cycles=state["usage"]["cpu_cycles"],
                    tokens_used=state["usage"]["tokens_used"],
                    memory_pages=state["usage"]["memory_pages"],
                    disk_bytes=state["usage"]["disk_bytes"]
                )
                
                kernel.process_table[int(pid)] = pcb
            
            logger.info("checkpoint_restored", process_count=len(kernel.process_table))
            return True
            
        except Exception as e:
            logger.error("checkpoint_restore_failed", error=str(e))
            return False

class PanicHandler:
    """
    Handles kernel panic situations.
    Dumps diagnostic info and attempts recovery.
    """
    
    def __init__(self):
        self.panic_log: list = []
        self.checkpoint_manager = CheckpointManager()
    
    def panic(self, reason: str, error: Optional[Exception] = None):
        """
        Handle a kernel panic.
        Logs diagnostic info and attempts recovery.
        """
        from src.os.kernel import kernel
        from src.os.logging.syslog import syslog, LogLevel
        
        panic_info = {
            "timestamp": datetime.utcnow().isoformat(),
            "reason": reason,
            "error": str(error) if error else None,
            "process_count": len(kernel.process_table),
            "running": kernel.running
        }
        
        self.panic_log.append(panic_info)
        
        # Log to syslog
        syslog.critical("kernel", f"KERNEL PANIC: {reason}", **panic_info)
        logger.critical("KERNEL_PANIC", reason=reason, error=str(error))
        
        # Create emergency checkpoint
        try:
            self.checkpoint_manager.create_checkpoint()
        except:
            pass
        
        # Attempt recovery
        return self._attempt_recovery()
    
    def _attempt_recovery(self) -> bool:
        """Attempt to recover from panic"""
        logger.info("recovery_attempting")
        
        # Kill all non-essential processes
        from src.os.kernel import kernel
        from src.os.process import ProcessState
        
        recovered = 0
        for pid, pcb in list(kernel.process_table.items()):
            if pid not in kernel.daemons.values():
                pcb.state = ProcessState.TERMINATED
                recovered += 1
        
        logger.info("recovery_complete", terminated_processes=recovered)
        return True
    
    def get_panic_log(self) -> list:
        """Get all panic events"""
        return self.panic_log

# Global instances
checkpoint_manager = CheckpointManager()
panic_handler = PanicHandler()
