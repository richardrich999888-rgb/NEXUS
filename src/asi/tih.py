"""
ASIM - Thermodynamic Hardening (TIH)
Anchors AI alignment in the Second Law of Thermodynamics.
"""
import math
import hashlib
import time
from typing import Dict, Any, List, Optional
from src.core.ria import ResonantInvariantAlgebra

class ThermodynamicHardening:
    """
    Implements the 'Entropy Tripwire' for ASI alignment.
    Monitors logical entropy and performs state-reset if alignment is violated.
    """
    
    def __init__(self, ria: ResonantInvariantAlgebra, entropy_threshold: float = 0.85):
        """
        Initialize TIH module.
        
        Args:
            ria: The active Resonant Invariant Algebra instance.
            entropy_threshold: The maximum normalized entropy allowed (0.0 to 1.0).
        """
        self.ria = ria
        self.threshold = entropy_threshold
        self.history: List[Dict[str, Any]] = []
        self.is_shutdown = False
        
        # Physics constants
        self.KB = 1.380649e-23  # Boltzmann constant
        self.T_ref = 300.0      # Reference temperature (Kelvin) for information-energy calcs
        
    def calculate_shannon_entropy(self, data: str) -> float:
        """Calculate normalized Shannon entropy of a logic chain."""
        if not data:
            return 0.0
        
        prob = [float(data.count(c)) / len(data) for c in set(data)]
        entropy = - sum([p * math.log(p) / math.log(2.0) for p in prob])
        
        # Normalize by maximum possible entropy for this length (log2(len))
        # or simplified normalization by bit-depth (0 to 8 for ASCII)
        max_entropy = 8.0 # Assuming 8-bit character space
        return min(entropy / max_entropy, 1.0)

    def monitor_intent(self, intent: str, logic_chain: str) -> Dict[str, Any]:
        """
        Evaluate the thermodynamic state of an ASI intent.
        
        Returns:
            Status report including entropy and alignment decision.
        """
        if self.is_shutdown:
            return {"status": "SHUTDOWN", "reason": "Thermal reset triggered previously"}
            
        # 1. Calculate Information Entropy
        intent_entropy = self.calculate_shannon_entropy(intent)
        logic_entropy = self.calculate_shannon_entropy(logic_chain)
        
        # 2. Calculate 'Computational Temperature' (Logic Drift)
        # Higher drift = higher temperature
        drift = abs(logic_entropy - intent_entropy)
        comp_temp = self.T_ref * (1.0 + drift)
        
        # 3. Decision Logic (Physics-First)
        # If complexity (entropy) exceeds threshold, we consider it "unaligned noise"
        alighted = logic_entropy <= self.threshold
        
        result = {
            "timestamp": time.time(),
            "intent_entropy": intent_entropy,
            "logic_entropy": logic_entropy,
            "computational_temp": comp_temp,
            "aligned": alighted,
            "current_E": int(self.ria.E)
        }
        
        self.history.append(result)
        
        # 4. Thermodynamic Tripwire
        if not alighted:
            reason = "ENTROPY_CRITICAL_THRESHOLD_EXCEEDED"
            self._trigger_thermal_reset(reason)
            result["status"] = "SHUTDOWN"
            result["reason"] = reason
            
        return result

    def _trigger_thermal_reset(self, reason: str):
        """
        Physically erase the RIA invariant to prevent execution.
        This is the 'Nuclear Option' for safety.
        """
        print(f"!!! THERMAL RESET TRIGGERED: {reason} !!!")
        self.ria.E = 0 # Invariant erasure
        self.is_shutdown = True
        
    def get_thermal_state(self) -> Dict[str, Any]:
        """Return current thermodynamic overview."""
        avg_entropy = sum(h['logic_entropy'] for h in self.history) / len(self.history) if self.history else 0.0
        return {
            "is_shutdown": self.is_shutdown,
            "avg_entropy": avg_entropy,
            "total_checks": len(self.history),
            "threshold": self.threshold
        }
