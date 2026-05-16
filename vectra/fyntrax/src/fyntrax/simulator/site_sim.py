"""
FYNTRAX Site Simulator

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Cellular site simulation with energy modeling.
Compares baseline vs FYNTRAX energy consumption.
"""

from typing import List, Optional, Dict
from dataclasses import dataclass
import numpy as np

from ..models.energy import RANEnergyModel, FyntraxEnergyModel
from ..ran.idle_orchestrator import IdleModeOrchestrator, PowerState
from ..ran.wur import WakeUpReceiver


@dataclass
class SimulationResult:
    """Result of site simulation."""
    total_energy_joules: float
    avg_power_watts: float
    time_in_deep_sleep: float
    time_in_light_sleep: float
    time_in_active: float
    wakeup_events: int
    energy_savings_percent: float


class SiteSimulator:
    """
    Cellular site simulator with FYNTRAX.
    
    Simulates energy consumption with:
    - Legacy mode (always-on SSB)
    - FYNTRAX mode (receiver-initiated)
    """
    
    def __init__(
        self,
        energy_model: Optional[RANEnergyModel] = None,
        fyntrax_model: Optional[FyntraxEnergyModel] = None,
    ):
        """
        Initialize site simulator.
        
        Args:
            energy_model: Legacy energy model
            fyntrax_model: FYNTRAX energy model
        """
        self.energy_model = energy_model or RANEnergyModel()
        self.fyntrax_model = fyntrax_model or FyntraxEnergyModel()
        self.orchestrator = IdleModeOrchestrator()
        self.wur = WakeUpReceiver()

    def simulate_legacy(
        self,
        load_profile: List[float],
        dt: float = 1.0,
    ) -> float:
        """
        Simulate legacy (always-on) operation.
        
        Args:
            load_profile: Load values over time [0, 1]
            dt: Time step in seconds
            
        Returns:
            Total energy in Joules
        """
        return self.energy_model.energy(load_profile, dt)

    def simulate_fyntrax(
        self,
        load_profile: List[float],
        dt: float = 1.0,
    ) -> SimulationResult:
        """
        Simulate FYNTRAX operation.
        
        Args:
            load_profile: Load values over time [0, 1]
            dt: Time step in seconds
            
        Returns:
            Detailed simulation result
        """
        total_energy = 0.0
        time_deep = 0.0
        time_light = 0.0
        time_active = 0.0
        wakeup_events = 0
        
        for load in load_profile:
            # Orchestrator decides state based on "entropy" (using load as proxy)
            state = self.orchestrator.decide(load)
            
            # Track time in each state
            if state == PowerState.DEEP_SLEEP:
                power = IdleModeOrchestrator.POWER_DEEP_SLEEP
                time_deep += dt
            elif state == PowerState.LIGHT_SLEEP:
                power = IdleModeOrchestrator.POWER_LIGHT_SLEEP
                time_light += dt
            else:
                power = self.fyntrax_model.power(load)
                time_active += dt
            
            # Check for wakeup
            if state == PowerState.ACTIVE and len(self.orchestrator.transitions) > 0:
                last_transition = self.orchestrator.transitions[-1]
                if last_transition.to_state == PowerState.ACTIVE:
                    wakeup_events += 1
            
            total_energy += power * dt
        
        # Calculate legacy energy for comparison
        legacy_energy = self.simulate_legacy(load_profile, dt)
        savings = (legacy_energy - total_energy) / legacy_energy * 100 if legacy_energy > 0 else 0
        
        total_time = len(load_profile) * dt
        
        return SimulationResult(
            total_energy_joules=total_energy,
            avg_power_watts=total_energy / total_time if total_time > 0 else 0,
            time_in_deep_sleep=time_deep,
            time_in_light_sleep=time_light,
            time_in_active=time_active,
            wakeup_events=wakeup_events,
            energy_savings_percent=savings,
        )

    def simulate(
        self,
        load_profile: List[float],
        dt: float = 1.0,
        mode: str = "fyntrax",
    ) -> Dict:
        """
        Run simulation.
        
        Args:
            load_profile: Load profile
            dt: Time step
            mode: "fyntrax" or "legacy"
            
        Returns:
            Simulation results
        """
        if mode == "legacy":
            energy = self.simulate_legacy(load_profile, dt)
            return {
                "mode": "legacy",
                "energy_joules": energy,
                "avg_power_watts": energy / (len(load_profile) * dt),
            }
        
        result = self.simulate_fyntrax(load_profile, dt)
        return {
            "mode": "fyntrax",
            "energy_joules": result.total_energy_joules,
            "avg_power_watts": result.avg_power_watts,
            "time_deep_sleep": result.time_in_deep_sleep,
            "time_light_sleep": result.time_in_light_sleep,
            "time_active": result.time_in_active,
            "wakeup_events": result.wakeup_events,
            "energy_savings_percent": result.energy_savings_percent,
        }


def main():
    """Main entry point for simulation."""
    # Simple demo
    model = RANEnergyModel(p_static=800, alpha=200)
    sim = SiteSimulator(energy_model=model)
    
    load = [0.1] * 100  # Low traffic for 100 seconds
    result = sim.simulate(load, mode="fyntrax")
    
    print("FYNTRAX Simulation Result:")
    for key, value in result.items():
        print(f"  {key}: {value}")


if __name__ == "__main__":
    main()
