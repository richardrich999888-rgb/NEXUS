"""
FYNTRAX KPI Calculator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Key Performance Indicator calculation and reporting.
Computes energy, network, and QoS metrics.
"""

from typing import Dict, List, Any
from dataclasses import dataclass


@dataclass
class EnergyKPIs:
    """Energy-related KPIs."""
    total_energy_kwh: float
    avg_power_kw: float
    energy_per_bit_nj: float
    energy_savings_percent: float
    idle_time_percent: float


@dataclass
class NetworkKPIs:
    """Network performance KPIs."""
    avg_latency_ms: float
    p99_latency_ms: float
    throughput_mbps: float
    handover_success_rate: float
    signaling_efficiency: float


def calculate_energy_kpis(
    total_energy_joules: float,
    duration_seconds: float,
    total_bits: int,
    legacy_energy_joules: float,
    idle_seconds: float,
) -> EnergyKPIs:
    """
    Calculate energy KPIs.
    
    Args:
        total_energy_joules: Total energy consumed
        duration_seconds: Simulation duration
        total_bits: Total bits transferred
        legacy_energy_joules: Legacy mode energy for comparison
        idle_seconds: Time spent in idle/sleep states
        
    Returns:
        Energy KPIs
    """
    return EnergyKPIs(
        total_energy_kwh=total_energy_joules / 3.6e6,
        avg_power_kw=total_energy_joules / duration_seconds / 1000 if duration_seconds > 0 else 0,
        energy_per_bit_nj=total_energy_joules * 1e9 / total_bits if total_bits > 0 else 0,
        energy_savings_percent=(1 - total_energy_joules / legacy_energy_joules) * 100 if legacy_energy_joules > 0 else 0,
        idle_time_percent=idle_seconds / duration_seconds * 100 if duration_seconds > 0 else 0,
    )


def calculate_network_kpis(
    latencies_ms: List[float],
    bytes_transferred: int,
    duration_seconds: float,
    successful_handovers: int,
    total_handovers: int,
    useful_signaling_bits: int,
    total_signaling_bits: int,
) -> NetworkKPIs:
    """
    Calculate network KPIs.
    
    Returns:
        Network KPIs
    """
    import numpy as np
    
    return NetworkKPIs(
        avg_latency_ms=float(np.mean(latencies_ms)) if latencies_ms else 0,
        p99_latency_ms=float(np.percentile(latencies_ms, 99)) if latencies_ms else 0,
        throughput_mbps=bytes_transferred * 8 / duration_seconds / 1e6 if duration_seconds > 0 else 0,
        handover_success_rate=successful_handovers / total_handovers if total_handovers > 0 else 1.0,
        signaling_efficiency=useful_signaling_bits / total_signaling_bits if total_signaling_bits > 0 else 1.0,
    )


def generate_kpi_report(
    energy_kpis: EnergyKPIs,
    network_kpis: NetworkKPIs,
) -> str:
    """
    Generate human-readable KPI report.
    
    Returns:
        Formatted report string
    """
    lines = [
        "=" * 60,
        "FYNTRAX KPI Report",
        "=" * 60,
        "",
        "Energy KPIs:",
        f"  Total Energy:      {energy_kpis.total_energy_kwh:.3f} kWh",
        f"  Avg Power:         {energy_kpis.avg_power_kw:.3f} kW",
        f"  Energy/Bit:        {energy_kpis.energy_per_bit_nj:.2f} nJ/bit",
        f"  Energy Savings:    {energy_kpis.energy_savings_percent:.1f}%",
        f"  Idle Time:         {energy_kpis.idle_time_percent:.1f}%",
        "",
        "Network KPIs:",
        f"  Avg Latency:       {network_kpis.avg_latency_ms:.2f} ms",
        f"  P99 Latency:       {network_kpis.p99_latency_ms:.2f} ms",
        f"  Throughput:        {network_kpis.throughput_mbps:.2f} Mbps",
        f"  Handover Success:  {network_kpis.handover_success_rate:.1%}",
        f"  Signaling Eff:     {network_kpis.signaling_efficiency:.1%}",
        "",
        "=" * 60,
    ]
    
    return "\n".join(lines)
