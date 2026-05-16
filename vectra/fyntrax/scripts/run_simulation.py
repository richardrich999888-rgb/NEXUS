#!/usr/bin/env python3
"""
FYNTRAX Simulation Runner

Runs a complete FYNTRAX simulation with configurable parameters.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from fyntrax.models.energy import RANEnergyModel, FyntraxEnergyModel
from fyntrax.simulator.site_sim import SiteSimulator
from fyntrax.simulator.traffic_sim import TrafficSimulator


def main():
    """Run FYNTRAX simulation."""
    print("=" * 60)
    print("FYNTRAX Site Simulation")
    print("Physics-First Entropy-Optimized RAN Control")
    print("=" * 60)
    
    # Create energy models
    legacy_model = RANEnergyModel(p_static=800, alpha=200)
    fyntrax_model = FyntraxEnergyModel(p_wur=1e-6, p_active=1000)
    
    # Create simulator
    sim = SiteSimulator(
        energy_model=legacy_model,
        fyntrax_model=fyntrax_model,
    )
    
    # Generate traffic
    print("\n[1] Generating traffic profile...")
    traffic = TrafficSimulator(seed=42)
    load_profile = traffic.generate_diurnal(hours=24, dt_seconds=60)
    
    print(f"    Duration: 24 hours")
    print(f"    Samples: {len(load_profile)}")
    print(f"    Avg load: {sum(load_profile)/len(load_profile):.2%}")
    
    # Run legacy simulation
    print("\n[2] Simulating legacy mode...")
    legacy_result = sim.simulate(load_profile, dt=60.0, mode="legacy")
    
    print(f"    Energy: {legacy_result['energy_joules']/3.6e6:.3f} kWh")
    print(f"    Avg Power: {legacy_result['avg_power_watts']:.1f} W")
    
    # Run FYNTRAX simulation
    print("\n[3] Simulating FYNTRAX mode...")
    fyntrax_result = sim.simulate(load_profile, dt=60.0, mode="fyntrax")
    
    print(f"    Energy: {fyntrax_result['energy_joules']/3.6e6:.6f} kWh")
    print(f"    Avg Power: {fyntrax_result['avg_power_watts']:.6f} W")
    print(f"    Deep Sleep: {fyntrax_result['time_deep_sleep']/3600:.1f} hours")
    print(f"    Light Sleep: {fyntrax_result['time_light_sleep']/3600:.1f} hours")
    print(f"    Active: {fyntrax_result['time_active']/3600:.1f} hours")
    print(f"    Wake-ups: {fyntrax_result['wakeup_events']}")
    
    # Comparison
    print("\n[4] Comparison")
    print("-" * 40)
    savings = fyntrax_result['energy_savings_percent']
    print(f"    Energy Savings: {savings:.1f}%")
    print(f"    Power Reduction: {legacy_result['avg_power_watts'] / max(fyntrax_result['avg_power_watts'], 1e-10):.0f}x")
    
    # Annual projection
    print("\n[5] Annual Projection (100,000 cells)")
    print("-" * 40)
    annual_legacy_kwh = legacy_result['energy_joules'] / 3.6e6 * 365 * 100000
    annual_fyntrax_kwh = fyntrax_result['energy_joules'] / 3.6e6 * 365 * 100000
    savings_kwh = annual_legacy_kwh - annual_fyntrax_kwh
    
    print(f"    Legacy: {annual_legacy_kwh/1e9:.2f} TWh/year")
    print(f"    FYNTRAX: {annual_fyntrax_kwh/1e6:.2f} GWh/year")
    print(f"    Savings: {savings_kwh/1e9:.2f} TWh/year")
    print(f"    Cost Savings: ${savings_kwh * 0.10 / 1e9:.2f}B/year")
    print(f"    CO2 Savings: {savings_kwh * 0.4 / 1e9:.2f}M tonnes/year")
    
    print("\n" + "=" * 60)
    print("Simulation complete.")


if __name__ == "__main__":
    main()
