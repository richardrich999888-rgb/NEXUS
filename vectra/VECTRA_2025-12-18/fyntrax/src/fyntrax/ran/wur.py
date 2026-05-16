"""
FYNTRAX Wake-Up Receiver (WuR)

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Receiver-Initiated Wake-Up Radio (RI-WuR) abstraction.
Models a microwatt-level always-on listener that triggers main radio activation.

Core innovation: Base station is OFF by default.
The WuR consumes ~1μW while the main radio consumes ~1000W.
"""

from dataclasses import dataclass
from typing import Tuple, Optional
import numpy as np


@dataclass
class WuRConfig:
    """Wake-up receiver configuration."""
    sensitivity_dbm: float = -110.0
    power_watts: float = 1e-6  # 1 microwatt
    detection_threshold: float = 0.5
    false_alarm_rate: float = 1e-4


class WakeUpReceiver:
    """
    Receiver-Initiated Wake-Up Radio (RI-WuR) abstraction.
    
    Models a microwatt-level always-on listener.
    """
    
    def __init__(self, config: Optional[WuRConfig] = None):
        """
        Initialize wake-up receiver.
        
        Args:
            config: WuR configuration (uses defaults if None)
        """
        self.config = config or WuRConfig()
        self.awake = False
        self.detection_count = 0
        self.false_alarm_count = 0

    @property
    def sensitivity_dbm(self) -> float:
        return self.config.sensitivity_dbm

    @property
    def power_watts(self) -> float:
        return self.config.power_watts

    def detect(self, signal_dbm: float) -> bool:
        """
        Perform detection based on received signal power.
        
        Args:
            signal_dbm: Received signal power in dBm
            
        Returns:
            True if wake-up signal detected
        """
        if signal_dbm >= self.sensitivity_dbm:
            self.awake = True
            self.detection_count += 1
        return self.awake

    def detect_with_noise(
        self, 
        signal_dbm: float, 
        noise_std: float = 3.0,
        rng: Optional[np.random.Generator] = None,
    ) -> Tuple[bool, float]:
        """
        Perform detection with additive noise.
        
        Models realistic detection with measurement uncertainty.
        
        Args:
            signal_dbm: Received signal power in dBm
            noise_std: Standard deviation of noise in dB
            rng: Random number generator
            
        Returns:
            Tuple of (detected, measured_power)
        """
        if rng is None:
            rng = np.random.default_rng()
        
        # Add measurement noise
        measured_dbm = signal_dbm + rng.normal(0, noise_std)
        detected = measured_dbm >= self.sensitivity_dbm
        
        if detected:
            self.awake = True
            self.detection_count += 1
        
        return detected, measured_dbm

    def reset(self) -> None:
        """Reset to sleep state."""
        self.awake = False

    def statistics(self) -> dict:
        """Get detection statistics."""
        return {
            "awake": self.awake,
            "detections": self.detection_count,
            "false_alarms": self.false_alarm_count,
            "power_watts": self.power_watts,
        }


class WakeUpSignal:
    """
    Wake-up signal generator.
    
    Generates the ultra-low-power signal transmitted by UE
    to request base station activation.
    """
    
    def __init__(self, length: int = 512, power_dbm: float = -60):
        """
        Initialize wake-up signal.
        
        Args:
            length: Sequence length in samples
            power_dbm: Transmit power in dBm
        """
        self.length = length
        self.power_dbm = power_dbm

    def generate(self, seed: Optional[int] = None) -> np.ndarray:
        """
        Generate wake-up signal sequence.
        
        Uses normalized bipolar (+1/-1) sequence for
        non-coherent energy detection at WuRx.
        
        Returns:
            Normalized signal vector
        """
        rng = np.random.default_rng(seed)
        seq = rng.choice([-1, 1], size=self.length)
        return seq / np.linalg.norm(seq)

    def energy(self, signal: np.ndarray) -> float:
        """Compute signal energy."""
        return float(np.sum(signal * signal))
