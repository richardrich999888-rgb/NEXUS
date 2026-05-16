"""
FYNTRAX Channel Models

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Wireless channel models for RAN simulation.
Includes path loss, fading, and SINR calculations.
"""

import numpy as np
from typing import Tuple


class RayleighChannel:
    """
    Rayleigh fading channel model.
    
    Models multipath fading in urban environments.
    """
    
    def __init__(self, seed: int = None):
        self.rng = np.random.default_rng(seed)

    def fade(self, num_samples: int = 1) -> np.ndarray:
        """
        Generate Rayleigh fading coefficients.
        
        Args:
            num_samples: Number of samples
            
        Returns:
            Complex fading coefficients
        """
        real = self.rng.standard_normal(num_samples)
        imag = self.rng.standard_normal(num_samples)
        return (real + 1j * imag) / np.sqrt(2)

    def path_loss(self, power_dbm: float) -> float:
        """Apply Rayleigh fading to signal power."""
        h = self.fade(1)[0]
        return power_dbm + 10 * np.log10(np.abs(h) ** 2)


class PathLossModel:
    """
    Distance-based path loss model.
    
    Uses 3GPP Urban Macro model.
    """
    
    def __init__(self, frequency_ghz: float = 3.5):
        self.frequency_ghz = frequency_ghz

    def loss_db(self, distance_m: float) -> float:
        """
        Calculate path loss in dB.
        
        3GPP 38.901 UMa NLOS model (simplified).
        
        Args:
            distance_m: Distance in meters
            
        Returns:
            Path loss in dB
        """
        if distance_m < 10:
            distance_m = 10
        
        # Simplified 3GPP model
        pl = (
            13.54 +
            39.08 * np.log10(distance_m) +
            20.0 * np.log10(self.frequency_ghz) -
            0.6 * 1.5  # Assuming 1.5m UE height
        )
        return pl


def estimate_snr(
    tx_power_dbm: float,
    distance_m: float,
    noise_figure_db: float = 7.0,
    bandwidth_mhz: float = 100.0,
) -> float:
    """
    Estimate SNR at receiver.
    
    Args:
        tx_power_dbm: Transmit power in dBm
        distance_m: Distance in meters
        noise_figure_db: Receiver noise figure
        bandwidth_mhz: Channel bandwidth
        
    Returns:
        SNR in dB
    """
    # Path loss
    pl_model = PathLossModel()
    path_loss = pl_model.loss_db(distance_m)
    
    # Received power
    rx_power_dbm = tx_power_dbm - path_loss
    
    # Thermal noise
    thermal_noise_dbm = -174 + 10 * np.log10(bandwidth_mhz * 1e6)
    noise_power_dbm = thermal_noise_dbm + noise_figure_db
    
    # SNR
    snr_db = rx_power_dbm - noise_power_dbm
    return snr_db
