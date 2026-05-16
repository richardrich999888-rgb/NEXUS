"""
AGP-OS Token Budget Enforcer
Kernel-level resource control for LLM token usage
"""

import time
import structlog
from typing import Dict, Optional
from dataclasses import dataclass, field

from src.os.process import ProcessControlBlock, ProcessState
from src.models import Hormone

logger = structlog.get_logger()

@dataclass
class TokenReservation:
    """A token reservation for a process"""
    pcb: ProcessControlBlock
    reserved: int
    timestamp: float = field(default_factory=time.time)

class BudgetEnforcer:
    """
    Kernel-level token budget enforcement.
    Prevents processes from exceeding their allocated token quotas.
    """
    
    def __init__(self):
        self.reservations: Dict[int, TokenReservation] = {}  # PID -> Reservation
        self.global_quota = 1000000  # 1M tokens per day (system-wide)
        self.global_used = 0
        
    def check_quota(self, pcb: ProcessControlBlock, estimated_tokens: int) -> bool:
        """
        Check if process has enough quota for estimated tokens.
        Returns True if allowed, False if quota exceeded.
        """
        # Check process quota
        if pcb.usage.tokens_used + estimated_tokens > pcb.quota_tokens:
            logger.warning(
                "quota_exceeded",
                pid=pcb.pid,
                used=pcb.usage.tokens_used,
                quota=pcb.quota_tokens,
                requested=estimated_tokens
            )
            return False
        
        # Check global quota
        if self.global_used + estimated_tokens > self.global_quota:
            logger.warning(
                "global_quota_exceeded",
                global_used=self.global_used,
                global_quota=self.global_quota,
                requested=estimated_tokens
            )
            return False
        
        return True
    
    def reserve_tokens(self, pcb: ProcessControlBlock, tokens: int):
        """
        Reserve tokens for a process before execution.
        This is a soft reservation - actual usage may differ.
        """
        self.reservations[pcb.pid] = TokenReservation(
            pcb=pcb,
            reserved=tokens
        )
        logger.info(
            "tokens_reserved",
            pid=pcb.pid,
            tokens=tokens,
            quota_remaining=pcb.quota_tokens - pcb.usage.tokens_used
        )
    
    def release_tokens(self, pcb: ProcessControlBlock, actual_tokens: int):
        """
        Release reservation and account for actual token usage.
        """
        reservation = self.reservations.pop(pcb.pid, None)
        
        if reservation:
            # Log if actual usage differs significantly from estimate
            if abs(actual_tokens - reservation.reserved) > reservation.reserved * 0.2:
                logger.warning(
                    "token_estimate_mismatch",
                    pid=pcb.pid,
                    estimated=reservation.reserved,
                    actual=actual_tokens,
                    diff_pct=abs(actual_tokens - reservation.reserved) / reservation.reserved * 100
                )
        
        # Update global usage
        self.global_used += actual_tokens
        
        logger.info(
            "tokens_released",
            pid=pcb.pid,
            tokens=actual_tokens,
            global_used=self.global_used
        )
    
    def handle_quota_violation(self, pcb: ProcessControlBlock, agent):
        """
        Handle quota violation by injecting stress hormones.
        This triggers the endocrine scheduler to throttle the process.
        """
        logger.error(
            "quota_violation",
            pid=pcb.pid,
            name=pcb.name,
            tokens_used=pcb.usage.tokens_used,
            quota=pcb.quota_tokens
        )
        
        # Inject Cortisol (Stress) to trigger throttling
        if agent:
            agent.endocrine_state.levels[Hormone.CORTISOL] = 0.95
            pcb.calculate_priority(agent.endocrine_state)
            
        # Mark process as sleeping (throttled)
        pcb.state = ProcessState.SLEEPING
    
    def stream_monitor(self, pcb: ProcessControlBlock) -> 'TokenCounter':
        """
        Create a streaming token counter for real-time monitoring.
        """
        return TokenCounter(pcb, self)
    
    def reset_global_quota(self):
        """Reset global quota (called daily)"""
        self.global_used = 0
        logger.info("global_quota_reset", quota=self.global_quota)
    
    def get_stats(self) -> Dict:
        """Get budget statistics"""
        return {
            "global_quota": self.global_quota,
            "global_used": self.global_used,
            "global_remaining": self.global_quota - self.global_used,
            "utilization_pct": (self.global_used / self.global_quota) * 100,
            "active_reservations": len(self.reservations)
        }


class TokenCounter:
    """
    Real-time token counter for streaming LLM responses.
    Monitors token usage and can kill process if quota exceeded mid-stream.
    """
    
    def __init__(self, pcb: ProcessControlBlock, enforcer: BudgetEnforcer):
        self.pcb = pcb
        self.enforcer = enforcer
        self.count = 0
        self.should_stop = False
    
    def add(self, tokens: int):
        """Add tokens to counter and check if quota exceeded"""
        self.count += tokens
        
        # Check if we've exceeded quota
        if self.pcb.usage.tokens_used + self.count > self.pcb.quota_tokens:
            logger.error(
                "quota_exceeded_mid_stream",
                pid=self.pcb.pid,
                tokens_so_far=self.count,
                total_used=self.pcb.usage.tokens_used + self.count,
                quota=self.pcb.quota_tokens
            )
            self.should_stop = True
    
    def finalize(self) -> int:
        """Finalize count and return total"""
        return self.count


# Global enforcer instance
budget_enforcer = BudgetEnforcer()
