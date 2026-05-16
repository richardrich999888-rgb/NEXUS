"""
FYNTRAX Idle Mode Orchestrator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Entropy-based idle mode orchestration.
Dynamically selects power state (DEEP_SLEEP, LIGHT_SLEEP, ACTIVE) based on
traffic entropy and load prediction.

States:
- DEEP_SLEEP: Only WuRx active (~1 μW)
- LIGHT_SLEEP: Quick wake capability (~10 mW)
- ACTIVE: Full radio chain (~1000 W)
"""

from enum import Enum
from typing import List, Optional
from dataclasses import dataclass
import time


class PowerState(Enum):
    """Base station power states."""
    DEEP_SLEEP = "DEEP_SLEEP"
    LIGHT_SLEEP = "LIGHT_SLEEP"
    ACTIVE = "ACTIVE"


@dataclass
class StateTransition:
    """Record of a state transition."""
    from_state: PowerState
    to_state: PowerState
    timestamp: float
    reason: str


class IdleModeOrchestrator:
    """
    Decides deep sleep vs light sleep vs active states
    based on predicted entropy demand.
    
    Decision rule:
    - entropy < threshold → DEEP_SLEEP
    - threshold ≤ entropy < 2×threshold → LIGHT_SLEEP
    - entropy ≥ 2×threshold → ACTIVE
    """
    
    # Power consumption by state (Watts)
    POWER_DEEP_SLEEP = 1e-6
    POWER_LIGHT_SLEEP = 0.01
    POWER_ACTIVE = 1000.0

    def __init__(
        self,
        threshold: float = 0.5,
        hysteresis: float = 0.1,
    ):
        """
        Initialize idle mode orchestrator.
        
        Args:
            threshold: Entropy threshold for state decisions
            hysteresis: Hysteresis to prevent oscillation
        """
        self.threshold = threshold
        self.hysteresis = hysteresis
        self.state = PowerState.DEEP_SLEEP
        self.transitions: List[StateTransition] = []
        self._last_entropy = 0.0

    def decide(self, predicted_entropy: float) -> PowerState:
        """
        Decide power state based on predicted entropy.
        
        Args:
            predicted_entropy: Predicted information demand
            
        Returns:
            Recommended power state
        """
        old_state = self.state
        
        # Apply hysteresis to prevent oscillation
        if self.state == PowerState.DEEP_SLEEP:
            if predicted_entropy >= self.threshold + self.hysteresis:
                if predicted_entropy >= 2 * self.threshold:
                    self.state = PowerState.ACTIVE
                else:
                    self.state = PowerState.LIGHT_SLEEP
        
        elif self.state == PowerState.LIGHT_SLEEP:
            if predicted_entropy < self.threshold - self.hysteresis:
                self.state = PowerState.DEEP_SLEEP
            elif predicted_entropy >= 2 * self.threshold + self.hysteresis:
                self.state = PowerState.ACTIVE
        
        else:  # ACTIVE
            if predicted_entropy < 2 * self.threshold - self.hysteresis:
                if predicted_entropy < self.threshold:
                    self.state = PowerState.DEEP_SLEEP
                else:
                    self.state = PowerState.LIGHT_SLEEP
        
        # Record transition
        if self.state != old_state:
            self.transitions.append(StateTransition(
                from_state=old_state,
                to_state=self.state,
                timestamp=time.time(),
                reason=f"entropy={predicted_entropy:.3f}",
            ))
        
        self._last_entropy = predicted_entropy
        return self.state

    def power_draw(self) -> float:
        """Get current power consumption in Watts."""
        if self.state == PowerState.DEEP_SLEEP:
            return self.POWER_DEEP_SLEEP
        elif self.state == PowerState.LIGHT_SLEEP:
            return self.POWER_LIGHT_SLEEP
        return self.POWER_ACTIVE

    def force_active(self) -> None:
        """Force transition to active state (for wake-up)."""
        if self.state != PowerState.ACTIVE:
            self.transitions.append(StateTransition(
                from_state=self.state,
                to_state=PowerState.ACTIVE,
                timestamp=time.time(),
                reason="forced_wake",
            ))
            self.state = PowerState.ACTIVE

    def force_sleep(self) -> None:
        """Force transition to deep sleep."""
        if self.state != PowerState.DEEP_SLEEP:
            self.transitions.append(StateTransition(
                from_state=self.state,
                to_state=PowerState.DEEP_SLEEP,
                timestamp=time.time(),
                reason="forced_sleep",
            ))
            self.state = PowerState.DEEP_SLEEP

    def transition_count(self) -> int:
        """Get total number of state transitions."""
        return len(self.transitions)

    def statistics(self) -> dict:
        """Get orchestrator statistics."""
        return {
            "current_state": self.state.value,
            "power_watts": self.power_draw(),
            "transition_count": len(self.transitions),
            "last_entropy": self._last_entropy,
            "threshold": self.threshold,
        }
