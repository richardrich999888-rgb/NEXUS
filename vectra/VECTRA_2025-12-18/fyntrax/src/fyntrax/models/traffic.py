"""
FYNTRAX Traffic Models

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Traffic pattern models and generators.
Supports various traffic distributions for simulation.
"""

import numpy as np
from typing import List, Tuple
from dataclasses import dataclass


@dataclass
class TrafficPattern:
    """Traffic pattern parameters."""
    mean_rate: float  # Mean request rate (requests/second)
    peak_rate: float  # Peak rate
    period_hours: float  # Period for cyclic patterns
    burst_probability: float  # Probability of traffic burst


class PoissonTraffic:
    """Poisson traffic generator."""
    
    def __init__(self, rate: float, seed: int = None):
        """
        Initialize Poisson traffic generator.
        
        Args:
            rate: Mean arrival rate (events/second)
            seed: Random seed
        """
        self.rate = rate
        self.rng = np.random.default_rng(seed)

    def generate(self, duration: float, dt: float = 1.0) -> List[int]:
        """
        Generate traffic arrivals.
        
        Args:
            duration: Duration in seconds
            dt: Time step
            
        Returns:
            List of arrival counts per time step
        """
        num_steps = int(duration / dt)
        arrivals = self.rng.poisson(self.rate * dt, num_steps)
        return arrivals.tolist()


class DiurnalTraffic:
    """
    Diurnal (24-hour cyclic) traffic model.
    
    Models typical daily traffic patterns in cellular networks.
    """
    
    def __init__(
        self,
        base_rate: float = 10.0,
        peak_multiplier: float = 5.0,
        peak_hour: int = 18,  # 6 PM
        seed: int = None,
    ):
        self.base_rate = base_rate
        self.peak_multiplier = peak_multiplier
        self.peak_hour = peak_hour
        self.rng = np.random.default_rng(seed)

    def rate_at_hour(self, hour: float) -> float:
        """
        Get traffic rate at given hour.
        
        Uses sinusoidal model with peak at peak_hour.
        """
        # Shift so peak is at peak_hour
        phase = 2 * np.pi * (hour - self.peak_hour + 6) / 24
        
        # Sinusoidal variation
        multiplier = 1 + (self.peak_multiplier - 1) * (1 + np.sin(phase)) / 2
        
        return self.base_rate * multiplier

    def generate(self, hours: float, dt_seconds: float = 60.0) -> List[float]:
        """
        Generate load profile over time.
        
        Args:
            hours: Duration in hours
            dt_seconds: Time step in seconds
            
        Returns:
            List of normalized load values [0, 1]
        """
        num_steps = int(hours * 3600 / dt_seconds)
        loads = []
        
        for i in range(num_steps):
            hour = (i * dt_seconds / 3600) % 24
            rate = self.rate_at_hour(hour)
            
            # Add noise
            noise = self.rng.normal(0, rate * 0.1)
            rate = max(0, rate + noise)
            
            # Normalize to [0, 1] based on peak rate
            peak_rate = self.base_rate * self.peak_multiplier
            load = min(1.0, rate / peak_rate)
            loads.append(load)
        
        return loads


def predict_load(history: List[float], horizon: int = 10) -> List[float]:
    """
    Simple load prediction using exponential smoothing.
    
    Args:
        history: Historical load values
        horizon: Prediction horizon (number of steps)
        
    Returns:
        Predicted load values
    """
    if not history:
        return [0.0] * horizon
    
    # Exponential smoothing
    alpha = 0.3
    smoothed = history[-1]
    
    predictions = []
    for _ in range(horizon):
        predictions.append(smoothed)
        # Assume gradual decay to mean
        smoothed = alpha * np.mean(history[-10:]) + (1 - alpha) * smoothed
    
    return predictions
