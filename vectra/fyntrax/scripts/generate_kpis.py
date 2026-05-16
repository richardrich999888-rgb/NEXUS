#!/usr/bin/env python3
"""
FYNTRAX KPI Generator

Generates KPI report from simulation results.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from fyntrax.simulator.kpi import (
    calculate_energy_kpis,
    calculate_network_kpis,
    generate_kpi_report,
    EnergyKPIs,
    NetworkKPIs,
)


def main():
    """Generate KPI report."""
    print("Generating FYNTRAX KPI Report...")
    print()
    
    # Sample simulation data (replace with actual simulation results)
    duration_seconds = 86400  # 24 hours
    total_bits = 1e12  # 1 Tbits
    
    # Energy metrics
    fyntrax_energy_joules = 1000  # Very low in FYNTRAX mode
    legacy_energy_joules = 86400000  # 1000W * 86400s
    idle_seconds = 80000  # Most time in idle
    
    energy_kpis = calculate_energy_kpis(
        total_energy_joules=fyntrax_energy_joules,
        duration_seconds=duration_seconds,
        total_bits=int(total_bits),
        legacy_energy_joules=legacy_energy_joules,
        idle_seconds=idle_seconds,
    )
    
    # Network metrics (simulated)
    import random
    latencies = [random.gauss(10, 3) for _ in range(1000)]
    
    network_kpis = calculate_network_kpis(
        latencies_ms=latencies,
        bytes_transferred=int(total_bits / 8),
        duration_seconds=duration_seconds,
        successful_handovers=95,
        total_handovers=100,
        useful_signaling_bits=20000,
        total_signaling_bits=25000,
    )
    
    # Generate report
    report = generate_kpi_report(energy_kpis, network_kpis)
    print(report)
    
    # Save to file
    report_path = os.path.join(os.path.dirname(__file__), '..', 'kpi_report.txt')
    with open(report_path, 'w') as f:
        f.write(report)
    
    print(f"\nReport saved to: {report_path}")


if __name__ == "__main__":
    main()
