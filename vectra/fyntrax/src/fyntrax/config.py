"""
FYNTRAX Configuration

Central configuration for all FYNTRAX parameters.
"""

from dataclasses import dataclass, field
from typing import Dict, Any


@dataclass
class FyntraxConfig:
    """Global FYNTRAX configuration."""
    
    # Energy Model Parameters
    p_static_watts: float = 800.0  # Static power consumption
    p_dynamic_watts: float = 200.0  # Dynamic power coefficient
    
    # Wake-Up Receiver Parameters
    wur_sensitivity_dbm: float = -110.0
    wur_power_watts: float = 1e-6  # 1 microwatt
    
    # Idle Mode Orchestration
    entropy_threshold_deep_sleep: float = 0.5
    entropy_threshold_light_sleep: float = 1.0
    
    # Lyapunov Controller
    lyapunov_alpha: float = 0.1  # Minimum decay rate
    lyapunov_max_v: float = 1000.0  # Maximum Lyapunov value
    
    # Simulation Parameters
    simulation_dt: float = 1.0  # Time step in seconds
    simulation_duration: float = 3600.0  # 1 hour default
    
    # TFEC Integration
    tfec_enabled: bool = True
    tfec_compression_target: float = 0.5  # 50% compression
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert config to dictionary."""
        return {
            "p_static_watts": self.p_static_watts,
            "p_dynamic_watts": self.p_dynamic_watts,
            "wur_sensitivity_dbm": self.wur_sensitivity_dbm,
            "wur_power_watts": self.wur_power_watts,
            "entropy_threshold_deep_sleep": self.entropy_threshold_deep_sleep,
            "lyapunov_alpha": self.lyapunov_alpha,
            "simulation_dt": self.simulation_dt,
            "tfec_enabled": self.tfec_enabled,
        }


# Default configuration instance
default_config = FyntraxConfig()
