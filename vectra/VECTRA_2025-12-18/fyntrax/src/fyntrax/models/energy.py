"""
FYNTRAX Energy Models

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Energy consumption models for RAN components.
Includes baseline and FYNTRAX energy models with state-dependent power consumption.
"""

import numpy as np
from typing import List, Tuple


class RANEnergyModel:
    """
    Physics-informed RAN energy model.
    
    P_total(L) = P_static + alpha * L
    
    where P_static dominates in 5G (typically 50-70% of peak power).
    """
    
    def __init__(self, p_static: float = 800.0, alpha: float = 200.0):
        """
        Initialize energy model.
        
        Args:
            p_static: Static power consumption in Watts
            alpha: Dynamic power coefficient (Watts per unit load)
        """
        self.p_static = p_static  # Watts
        self.alpha = alpha        # Watts per unit load

    def power(self, load: float) -> float:
        """
        Calculate power consumption at given load.
        
        Args:
            load: Traffic load [0, 1]
            
        Returns:
            Power consumption in Watts
        """
        load = max(0.0, min(1.0, load))
        return self.p_static + self.alpha * load

    def energy(self, load_profile: List[float], dt: float = 1.0) -> float:
        """
        Compute total energy over time.
        
        Args:
            load_profile: List of load values over time
            dt: Time step in seconds
            
        Returns:
            Total energy in Joules
        """
        return sum(self.power(L) * dt for L in load_profile)

    def energy_per_bit(self, load: float, capacity_bps: float) -> float:
        """
        Calculate energy per bit at given load.
        
        E_b = P / (L * C)
        
        Problem: As L → 0, E_b → ∞
        
        Args:
            load: Traffic load [0, 1]
            capacity_bps: Channel capacity in bits per second
            
        Returns:
            Energy per bit in Joules/bit
        """
        if load <= 0:
            return float('inf')
        
        power = self.power(load)
        throughput = load * capacity_bps
        return power / throughput


class FyntraxEnergyModel:
    """
    FYNTRAX receiver-initiated energy model.
    
    P_total(L) = P_wur + P_active * L
    
    Key innovation: lim_{L→0} P → P_wur ≈ 0
    """
    
    def __init__(self, p_wur: float = 1e-6, p_active: float = 1000.0):
        """
        Initialize FYNTRAX energy model.
        
        Args:
            p_wur: Wake-up receiver power in Watts (default 1 μW)
            p_active: Active transmission power in Watts
        """
        self.p_wur = p_wur
        self.p_active = p_active

    def power(self, load: float) -> float:
        """Calculate power consumption at given load."""
        load = max(0.0, min(1.0, load))
        return self.p_wur + self.p_active * load

    def energy(self, load_profile: List[float], dt: float = 1.0) -> float:
        """Compute total energy over time."""
        return sum(self.power(L) * dt for L in load_profile)

    def energy_per_bit(self, load: float, capacity_bps: float) -> float:
        """
        Calculate energy per bit.
        
        Unlike legacy, E_b remains finite as L → 0.
        """
        if load <= 0:
            return self.p_wur  # Finite, not infinite
        
        power = self.power(load)
        throughput = load * capacity_bps
        return power / throughput


def compare_models(load_profile: List[float], dt: float = 1.0) -> Tuple[float, float, float]:
    """
    Compare legacy vs FYNTRAX energy consumption.
    
    Returns:
        Tuple of (legacy_energy, fyntrax_energy, savings_percent)
    """
    legacy = RANEnergyModel()
    fyntrax = FyntraxEnergyModel()
    
    legacy_energy = legacy.energy(load_profile, dt)
    fyntrax_energy = fyntrax.energy(load_profile, dt)
    
    savings = (legacy_energy - fyntrax_energy) / legacy_energy * 100
    
    return legacy_energy, fyntrax_energy, savings
