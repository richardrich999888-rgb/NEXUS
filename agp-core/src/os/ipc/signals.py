"""
AGP-OS: Signal System
Unix-like signals for process control and inter-process communication.
"""

import asyncio
import structlog
from typing import Dict, Callable, Optional, Any
from dataclasses import dataclass
from enum import Enum, auto

logger = structlog.get_logger()

class Signal(Enum):
    """Standard Unix-like signals"""
    SIGHUP = 1      # Hangup - Reload config
    SIGINT = 2      # Interrupt - Ctrl+C
    SIGQUIT = 3     # Quit - Core dump
    SIGKILL = 9     # Kill - Cannot be caught
    SIGUSR1 = 10    # User defined 1
    SIGUSR2 = 12    # User defined 2
    SIGTERM = 15    # Terminate - Graceful shutdown
    SIGSTOP = 19    # Stop - Pause process
    SIGCONT = 18    # Continue - Resume process
    SIGCHLD = 17    # Child terminated
    
    # AGP-OS specific signals
    SIGSTRESS = 50  # Hormone stress alert
    SIGQUOTA = 51   # Token quota warning
    SIGPANIC = 52   # Kernel panic

@dataclass
class SignalInfo:
    """Information about a sent signal"""
    signal: Signal
    sender_pid: int
    data: Optional[Any] = None

# Default signal handlers
def default_term_handler(pcb, info: SignalInfo):
    """Default handler for SIGTERM - graceful shutdown"""
    from src.os.process import ProcessState
    logger.info("signal_term", pid=pcb.pid, sender=info.sender_pid)
    pcb.state = ProcessState.TERMINATED

def default_stop_handler(pcb, info: SignalInfo):
    """Default handler for SIGSTOP - pause process"""
    from src.os.process import ProcessState
    logger.info("signal_stop", pid=pcb.pid)
    pcb.state = ProcessState.SLEEPING

def default_cont_handler(pcb, info: SignalInfo):
    """Default handler for SIGCONT - resume process"""
    from src.os.process import ProcessState
    logger.info("signal_cont", pid=pcb.pid)
    pcb.state = ProcessState.READY

def default_stress_handler(pcb, info: SignalInfo):
    """Handle stress signal by adjusting hormones"""
    from src.models import Hormone
    from src.agents import agent_registry
    import uuid
    
    try:
        agent = agent_registry.get_agent(uuid.UUID(pcb.agent_id))
        if agent:
            agent.endocrine_state.levels[Hormone.CORTISOL] = min(1.0, 
                agent.endocrine_state.levels.get(Hormone.CORTISOL, 0.5) + 0.2)
            pcb.calculate_priority(agent.endocrine_state)
            logger.info("signal_stress", pid=pcb.pid, cortisol=agent.endocrine_state.levels[Hormone.CORTISOL])
    except:
        pass

class SignalHandler:
    """
    Manages signal delivery and handling for processes.
    """
    
    def __init__(self):
        # PID -> Signal -> Handler function
        self.handlers: Dict[int, Dict[Signal, Callable]] = {}
        
        # Default handlers for all processes
        self.default_handlers: Dict[Signal, Callable] = {
            Signal.SIGTERM: default_term_handler,
            Signal.SIGSTOP: default_stop_handler,
            Signal.SIGCONT: default_cont_handler,
            Signal.SIGSTRESS: default_stress_handler,
        }
        
        # Pending signals per process
        self.pending: Dict[int, list] = {}
    
    def register_handler(self, pid: int, signal: Signal, handler: Callable):
        """
        Register a custom signal handler for a process.
        Handler signature: handler(pcb, signal_info)
        """
        if signal == Signal.SIGKILL:
            logger.warning("cannot_catch_sigkill", pid=pid)
            return False
        
        if pid not in self.handlers:
            self.handlers[pid] = {}
        
        self.handlers[pid][signal] = handler
        logger.info("handler_registered", pid=pid, signal=signal.name)
        return True
    
    def send_signal(self, sender_pid: int, target_pid: int, signal: Signal, data: Any = None):
        """
        Send a signal to a process.
        Returns True if signal was delivered, False otherwise.
        """
        from src.os.kernel import kernel
        
        # Get target process
        pcb = kernel.process_table.get(target_pid)
        if not pcb:
            logger.warning("signal_no_target", target=target_pid, signal=signal.name)
            return False
        
        info = SignalInfo(signal=signal, sender_pid=sender_pid, data=data)
        
        # SIGKILL is always handled immediately
        if signal == Signal.SIGKILL:
            from src.os.process import ProcessState
            pcb.state = ProcessState.TERMINATED
            logger.info("signal_kill", pid=target_pid, sender=sender_pid)
            return True
        
        # Get handler
        handler = None
        if target_pid in self.handlers and signal in self.handlers[target_pid]:
            handler = self.handlers[target_pid][signal]
        elif signal in self.default_handlers:
            handler = self.default_handlers[signal]
        
        if handler:
            try:
                handler(pcb, info)
                logger.info("signal_delivered", pid=target_pid, signal=signal.name, sender=sender_pid)
                return True
            except Exception as e:
                logger.error("signal_handler_error", pid=target_pid, signal=signal.name, error=str(e))
                return False
        else:
            # Queue signal for later delivery
            if target_pid not in self.pending:
                self.pending[target_pid] = []
            self.pending[target_pid].append(info)
            logger.info("signal_queued", pid=target_pid, signal=signal.name)
            return True
    
    def process_pending(self, pid: int):
        """Process any pending signals for a process"""
        if pid not in self.pending or not self.pending[pid]:
            return
        
        from src.os.kernel import kernel
        pcb = kernel.process_table.get(pid)
        if not pcb:
            return
        
        for info in self.pending[pid]:
            handler = None
            if pid in self.handlers and info.signal in self.handlers[pid]:
                handler = self.handlers[pid][info.signal]
            elif info.signal in self.default_handlers:
                handler = self.default_handlers[info.signal]
            
            if handler:
                try:
                    handler(pcb, info)
                except:
                    pass
        
        self.pending[pid].clear()
    
    def cleanup_process(self, pid: int):
        """Clean up handlers for terminated process"""
        if pid in self.handlers:
            del self.handlers[pid]
        if pid in self.pending:
            del self.pending[pid]

# Global signal handler
signal_handler = SignalHandler()
