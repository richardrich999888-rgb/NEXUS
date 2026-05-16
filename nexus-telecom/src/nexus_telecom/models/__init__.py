"""
NEXUS Telecom - Physics Models
Copyright (c) 2025 SYNTRIASS Labs Private Limited
Inventor: Katta Naga Sri Ganesh

Energy and entropy models for telecom optimization.
"""

import numpy as np
from typing import Dict, Optional
from dataclasses import dataclass


@dataclass
class SiteConfig:
    """Base station site configuration."""
    tx_power_watts: float = 40.0  # Typical macro cell
    pa_efficiency: float = 0.35  # Power amplifier efficiency
    static_power_watts: float = 100.0  # Baseband, cooling, etc.
    antenna_elements: int = 64  # MIMO elements
    bandwidth_mhz: float = 100.0  # Channel bandwidth


class EnergyModel:
    """
    Energy consumption model for RAN optimization.
    
    Computes total energy as function of traffic load and configuration.
    """
    
    def __init__(self, config: Optional[SiteConfig] = None):
        self.config = config or SiteConfig()

    def compute_power(self, load_fraction: float) -> float:
        """
        Compute instantaneous power consumption.
        
        Args:
            load_fraction: Traffic load [0, 1]
            
        Returns:
            Power in watts
        """
        # Dynamic power scales with load
        tx_power = self.config.tx_power_watts * load_fraction
        rf_power = tx_power / self.config.pa_efficiency
        
        # Total power = static + dynamic
        return self.config.static_power_watts + rf_power

    def compute_energy(self, load_fraction: float, duration_seconds: float) -> float:
        """Compute total energy consumption in joules."""
        power = self.compute_power(load_fraction)
        return power * duration_seconds

    def efficiency(self, load_fraction: float, throughput_bits: float) -> float:
        """
        Compute energy efficiency in bits per joule.
        
        Higher is better.
        """
        power = self.compute_power(load_fraction)
        if power <= 0:
            return 0.0
        return throughput_bits / power

    def optimal_load(self) -> float:
        """
        Compute load that maximizes efficiency.
        
        With linear power model, efficiency peaks at full load.
        With realistic models, there's an optimal point.
        """
        # For linear model, max efficiency at high load
        return 0.8


class EntropyCalculator:
    """
    Information theory utilities for telecom optimization.
    
    Computes entropy and information metrics for network decisions.
    """
    
    @staticmethod
    def shannon_entropy(probabilities: np.ndarray) -> float:
        """
        Compute Shannon entropy H(X) = -Σ p(x) log₂ p(x)
        
        Args:
            probabilities: Probability distribution
            
        Returns:
            Entropy in bits
        """
        p = np.asarray(probabilities)
        p = p[p > 0]  # Filter zeros
        return -np.sum(p * np.log2(p))

    @staticmethod
    def byte_entropy(data: bytes) -> float:
        """Compute entropy of byte sequence."""
        if len(data) == 0:
            return 0.0
        
        counts = np.zeros(256)
        for b in data:
            counts[b] += 1
        
        probs = counts / len(data)
        return EntropyCalculator.shannon_entropy(probs)

    @staticmethod
    def channel_capacity(snr_linear: float, bandwidth_hz: float) -> float:
        """
        Compute Shannon channel capacity.
        
        C = B × log₂(1 + SNR)
        
        Args:
            snr_linear: Signal-to-noise ratio (linear, not dB)
            bandwidth_hz: Channel bandwidth in Hz
            
        Returns:
            Capacity in bits per second
        """
        return bandwidth_hz * np.log2(1 + snr_linear)

    @staticmethod
    def spectral_efficiency(snr_db: float) -> float:
        """
        Compute spectral efficiency in bits/s/Hz.
        
        Uses Shannon formula with SNR in dB.
        """
        snr_linear = 10 ** (snr_db / 10)
        return np.log2(1 + snr_linear)


__all__ = ["SiteConfig", "EnergyModel", "EntropyCalculator"]
