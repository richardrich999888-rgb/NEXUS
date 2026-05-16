"""
FYNTRAX SSB Scheduler

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Adaptive SSB (Synchronization Signal Block) scheduling.
Reduces SSB transmission frequency during low-entropy periods
to save energy while maintaining synchronization.
"""

from typing import List, Optional, Tuple
from dataclasses import dataclass
import numpy as np


@dataclass
class SSBConfig:
    """SSB configuration."""
    periodicity_ms: float = 20.0  # SSB periodicity
    num_beams: int = 64  # Number of SSB beams
    power_dbm: float = 43.0  # SSB transmit power
    duration_symbols: int = 4  # SSB duration


@dataclass
class SSBBurst:
    """SSB burst record."""
    beam_id: int
    timestamp_ms: float
    target_direction: Optional[float] = None  # Degrees


class SSBScheduler:
    """
    SSB (Synchronization Signal Block) Scheduler.
    
    Legacy mode: Broadcast SSB periodically across all beams
    FYNTRAX mode: Targeted SSB only when/where needed
    """
    
    def __init__(
        self,
        config: Optional[SSBConfig] = None,
        targeted_mode: bool = True,
    ):
        """
        Initialize SSB scheduler.
        
        Args:
            config: SSB configuration
            targeted_mode: If True, use targeted SSB (FYNTRAX mode)
        """
        self.config = config or SSBConfig()
        self.targeted_mode = targeted_mode
        self.scheduled_bursts: List[SSBBurst] = []
        self.current_time_ms = 0.0

    def schedule_legacy(self) -> List[SSBBurst]:
        """
        Schedule legacy SSB burst (all beams).
        
        Energy cost: N_beams × E_SSB
        """
        bursts = []
        for beam_id in range(self.config.num_beams):
            burst = SSBBurst(
                beam_id=beam_id,
                timestamp_ms=self.current_time_ms,
            )
            bursts.append(burst)
        
        self.scheduled_bursts.extend(bursts)
        return bursts

    def schedule_targeted(self, direction_degrees: float) -> List[SSBBurst]:
        """
        Schedule targeted SSB burst (single beam).
        
        Energy cost: 1 × E_SSB (N_beams × reduction)
        
        Args:
            direction_degrees: Target direction in degrees
            
        Returns:
            List containing single SSB burst
        """
        # Calculate beam index from direction
        beam_id = self._direction_to_beam(direction_degrees)
        
        burst = SSBBurst(
            beam_id=beam_id,
            timestamp_ms=self.current_time_ms,
            target_direction=direction_degrees,
        )
        
        self.scheduled_bursts.append(burst)
        return [burst]

    def schedule(self, direction: Optional[float] = None) -> List[SSBBurst]:
        """
        Schedule SSB based on mode.
        
        Args:
            direction: Target direction (required for targeted mode)
            
        Returns:
            List of scheduled SSB bursts
        """
        if self.targeted_mode and direction is not None:
            return self.schedule_targeted(direction)
        return self.schedule_legacy()

    def advance_time(self, delta_ms: float) -> None:
        """Advance simulation time."""
        self.current_time_ms += delta_ms

    def energy_consumed_joules(self, power_per_beam_watts: float = 10.0) -> float:
        """
        Calculate total energy consumed by scheduled SSBs.
        
        Args:
            power_per_beam_watts: Power per beam during SSB
            
        Returns:
            Total energy in Joules
        """
        duration_seconds = (
            self.config.duration_symbols * 
            (1 / 14) *  # Symbols per slot
            1e-3  # Slot duration ~1ms
        )
        
        return len(self.scheduled_bursts) * power_per_beam_watts * duration_seconds

    def energy_savings_factor(self) -> float:
        """
        Calculate energy savings vs legacy mode.
        
        Returns:
            Savings factor (targeted beams / total beams)
        """
        if not self.scheduled_bursts:
            return 1.0
        
        # Count targeted vs broadcast
        targeted = sum(1 for b in self.scheduled_bursts if b.target_direction is not None)
        total = len(self.scheduled_bursts)
        
        if targeted == 0:
            return 1.0  # All broadcast, no savings
        
        # Each targeted replaces N beams
        legacy_equivalent = (total - targeted) + targeted * self.config.num_beams
        return legacy_equivalent / total if total > 0 else 1.0

    def _direction_to_beam(self, direction_degrees: float) -> int:
        """Convert direction to beam index."""
        # Normalize direction to [0, 360)
        direction = direction_degrees % 360
        
        # Map to beam index
        beam_width = 360 / self.config.num_beams
        return int(direction / beam_width) % self.config.num_beams

    def statistics(self) -> dict:
        """Get scheduler statistics."""
        return {
            "mode": "targeted" if self.targeted_mode else "legacy",
            "total_bursts": len(self.scheduled_bursts),
            "num_beams": self.config.num_beams,
            "energy_joules": self.energy_consumed_joules(),
            "savings_factor": self.energy_savings_factor(),
        }
