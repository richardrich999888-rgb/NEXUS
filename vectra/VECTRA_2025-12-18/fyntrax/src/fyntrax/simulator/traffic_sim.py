"""
FYNTRAX Traffic Simulator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Traffic pattern simulation and generation.
Models realistic user traffic for RAN testing.
"""

from typing import List, Optional
import numpy as np

from ..models.traffic import DiurnalTraffic, PoissonTraffic


class TrafficSimulator:
    """
    Traffic simulator for FYNTRAX testing.
    """
    
    def __init__(self, seed: Optional[int] = None):
        """
        Initialize traffic simulator.
        
        Args:
            seed: Random seed for reproducibility
        """
        self.seed = seed
        self.rng = np.random.default_rng(seed)

    def generate_diurnal(
        self,
        hours: float = 24.0,
        dt_seconds: float = 60.0,
        base_rate: float = 10.0,
        peak_multiplier: float = 5.0,
    ) -> List[float]:
        """
        Generate diurnal (24-hour cyclic) load profile.
        
        Args:
            hours: Duration in hours
            dt_seconds: Time step
            base_rate: Base traffic rate
            peak_multiplier: Peak/base ratio
            
        Returns:
            Load profile [0, 1]
        """
        traffic = DiurnalTraffic(
            base_rate=base_rate,
            peak_multiplier=peak_multiplier,
            seed=self.seed,
        )
        return traffic.generate(hours, dt_seconds)

    def generate_bursty(
        self,
        duration_seconds: float = 3600.0,
        dt: float = 1.0,
        background_load: float = 0.1,
        burst_probability: float = 0.01,
        burst_load: float = 0.8,
        burst_duration: int = 10,
    ) -> List[float]:
        """
        Generate bursty traffic pattern.
        
        Args:
            duration_seconds: Total duration
            dt: Time step
            background_load: Background load level
            burst_probability: Probability of burst start
            burst_load: Load during burst
            burst_duration: Duration of burst in steps
            
        Returns:
            Load profile
        """
        num_steps = int(duration_seconds / dt)
        loads = []
        
        burst_remaining = 0
        for _ in range(num_steps):
            if burst_remaining > 0:
                loads.append(burst_load + self.rng.normal(0, 0.1))
                burst_remaining -= 1
            elif self.rng.random() < burst_probability:
                burst_remaining = burst_duration
                loads.append(burst_load)
            else:
                loads.append(background_load + self.rng.normal(0, 0.02))
        
        # Clip to valid range
        return [max(0.0, min(1.0, l)) for l in loads]

    def generate_iot(
        self,
        duration_seconds: float = 3600.0,
        dt: float = 1.0,
        devices: int = 1000,
        report_interval: float = 60.0,  # seconds
    ) -> List[float]:
        """
        Generate IoT traffic pattern.
        
        IoT devices typically have very low, periodic traffic.
        
        Returns:
            Load profile
        """
        num_steps = int(duration_seconds / dt)
        loads = []
        
        # Devices report at random offsets within interval
        device_offsets = self.rng.uniform(0, report_interval, devices)
        
        for step in range(num_steps):
            t = step * dt
            
            # Count devices reporting at this time
            active = sum(1 for offset in device_offsets 
                        if abs((t % report_interval) - offset) < dt)
            
            # Normalize to load
            max_active = devices * dt / report_interval
            load = min(1.0, active / max_active) if max_active > 0 else 0.0
            loads.append(load * 0.1)  # IoT is low bandwidth
        
        return loads


def generate_test_profile(scenario: str = "diurnal") -> List[float]:
    """
    Generate test traffic profile.
    
    Args:
        scenario: "diurnal", "bursty", "iot", or "constant"
        
    Returns:
        Load profile
    """
    sim = TrafficSimulator(seed=42)
    
    if scenario == "diurnal":
        return sim.generate_diurnal(hours=24, dt_seconds=60)
    elif scenario == "bursty":
        return sim.generate_bursty(duration_seconds=3600)
    elif scenario == "iot":
        return sim.generate_iot(duration_seconds=3600)
    else:
        return [0.5] * 3600  # Constant 50% load
