"""Tests for FYNTRAX energy models."""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from fyntrax.models.energy import RANEnergyModel, FyntraxEnergyModel, compare_models


def test_legacy_power_at_zero_load():
    """Legacy power should be high even at zero load."""
    model = RANEnergyModel(p_static=500, alpha=200)
    
    power = model.power(0.0)
    
    assert power == 500.0
    assert power > 0  # The fundamental problem


def test_legacy_power_at_full_load():
    """Legacy power at full load."""
    model = RANEnergyModel(p_static=500, alpha=200)
    
    power = model.power(1.0)
    
    assert power == 700.0


def test_fyntrax_power_at_zero_load():
    """FYNTRAX power should be near-zero at zero load."""
    model = FyntraxEnergyModel(p_wur=1e-6, p_active=1000)
    
    power = model.power(0.0)
    
    assert power == 1e-6
    assert power < 1e-5  # Microwatt level


def test_fyntrax_power_at_full_load():
    """FYNTRAX power at full load."""
    model = FyntraxEnergyModel(p_wur=1e-6, p_active=1000)
    
    power = model.power(1.0)
    
    assert power == 1000.000001


def test_energy_calculation():
    """Test energy calculation over time."""
    model = RANEnergyModel(p_static=100, alpha=100)
    
    load_profile = [0.5] * 10  # 10 seconds at 50% load
    energy = model.energy(load_profile, dt=1.0)
    
    # Power = 100 + 100*0.5 = 150W, Energy = 150W * 10s = 1500J
    assert energy == 1500.0


def test_energy_per_bit_legacy():
    """Legacy energy per bit diverges at low load."""
    model = RANEnergyModel(p_static=500, alpha=200)
    
    # At 1% load
    e_b = model.energy_per_bit(0.01, capacity_bps=1e9)
    
    # At 0% load
    e_b_zero = model.energy_per_bit(0.0, capacity_bps=1e9)
    
    assert e_b > 0
    assert e_b_zero == float('inf')  # Diverges!


def test_model_comparison():
    """Compare legacy vs FYNTRAX."""
    load_profile = [0.0] * 100  # 100% idle
    
    legacy, fyntrax, savings = compare_models(load_profile)
    
    assert savings > 99.9  # >99.9% savings at idle


if __name__ == "__main__":
    test_legacy_power_at_zero_load()
    test_legacy_power_at_full_load()
    test_fyntrax_power_at_zero_load()
    test_fyntrax_power_at_full_load()
    test_energy_calculation()
    test_energy_per_bit_legacy()
    test_model_comparison()
    print("All energy tests passed!")
