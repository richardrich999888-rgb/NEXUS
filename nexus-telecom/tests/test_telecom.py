"""
NEXUS Telecom Tests
Copyright (c) 2025 SYNTRIASS Labs Private Limited
Inventor: Katta Naga Sri Ganesh
"""

import numpy as np
import pytest


class TestWakeUpReceiver:
    """Tests for Wake-Up Receiver."""
    
    def test_detection_above_threshold(self):
        from nexus_telecom.ran import WakeUpReceiver, WuRConfig
        
        config = WuRConfig(sensitivity_dbm=-100)
        wur = WakeUpReceiver(config)
        
        # Signal above threshold should wake up
        assert wur.detect(-90) is True
        assert wur.awake is True
        
    def test_detection_below_threshold(self):
        from nexus_telecom.ran import WakeUpReceiver, WuRConfig
        
        config = WuRConfig(sensitivity_dbm=-100)
        wur = WakeUpReceiver(config)
        
        # Signal below threshold should not wake up
        assert wur.detect(-110) is False
        assert wur.awake is False
        
    def test_power_consumption(self):
        from nexus_telecom.ran import WakeUpReceiver
        
        wur = WakeUpReceiver()
        # Should be 1 microwatt by default
        assert wur.power_watts == 1e-6


class TestLyapunovController:
    """Tests for Lyapunov Stability Controller."""
    
    def test_safe_transition(self):
        from nexus_telecom.control import LyapunovController
        
        # Identity P matrix, 4 dimensions
        controller = LyapunovController.create_identity(dim=4, alpha=0.1)
        
        # Transition toward origin is safe
        x = np.array([1.0, 1.0, 1.0, 1.0])
        x_next = np.array([0.5, 0.5, 0.5, 0.5])
        
        assert controller.is_safe(x, x_next) is True
        
    def test_unsafe_transition(self):
        from nexus_telecom.control import LyapunovController
        
        controller = LyapunovController.create_identity(dim=4, alpha=0.1)
        
        # Transition away from origin is unsafe
        x = np.array([1.0, 1.0, 1.0, 1.0])
        x_next = np.array([2.0, 2.0, 2.0, 2.0])
        
        assert controller.is_safe(x, x_next) is False
        
    def test_filter_action(self):
        from nexus_telecom.control import LyapunovController
        
        controller = LyapunovController.create_identity(dim=2, alpha=0.1)
        
        x = np.array([1.0, 1.0])
        unsafe_next = np.array([5.0, 5.0])
        
        # Should return current state as fallback
        filtered = controller.filter_action(x, unsafe_next)
        np.testing.assert_array_equal(filtered, x)


class TestEnergyModel:
    """Tests for Energy Model."""
    
    def test_power_computation(self):
        from nexus_telecom.models import EnergyModel, SiteConfig
        
        config = SiteConfig(
            tx_power_watts=40.0,
            static_power_watts=100.0,
            pa_efficiency=0.4,
        )
        model = EnergyModel(config)
        
        # At 50% load
        power = model.compute_power(0.5)
        
        # RF power = 40 * 0.5 / 0.4 = 50W
        # Total = 100 + 50 = 150W
        assert abs(power - 150.0) < 0.1
        
    def test_efficiency(self):
        from nexus_telecom.models import EnergyModel
        
        model = EnergyModel()
        
        # Higher throughput = better efficiency
        eff_low = model.efficiency(0.5, 1e6)
        eff_high = model.efficiency(0.5, 2e6)
        
        assert eff_high > eff_low


class TestEntropyCalculator:
    """Tests for Entropy Calculator."""
    
    def test_uniform_entropy(self):
        from nexus_telecom.models import EntropyCalculator
        
        # Uniform distribution has maximum entropy
        probs = np.array([0.25, 0.25, 0.25, 0.25])
        entropy = EntropyCalculator.shannon_entropy(probs)
        
        # log2(4) = 2 bits
        assert abs(entropy - 2.0) < 0.01
        
    def test_channel_capacity(self):
        from nexus_telecom.models import EntropyCalculator
        
        # 10 dB SNR, 100 MHz bandwidth
        snr_linear = 10 ** (10 / 10)  # 10
        capacity = EntropyCalculator.channel_capacity(snr_linear, 100e6)
        
        # C = 100e6 * log2(11) ≈ 345 Mbps
        assert capacity > 300e6
        assert capacity < 400e6


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
