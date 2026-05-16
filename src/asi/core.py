"""
ASIM - Core Orchestrator
Integrates TIH, IPE, SFA, and NTP into a unified sovereign mesh.
"""
import time
from typing import List, Dict, Any, Optional
from src.core.ria import ResonantInvariantAlgebra
from src.asi.tih import ThermodynamicHardening
from src.asi.ipe import IsogenyPotentialWells
from src.asi.reasoning import StatisticalFieldAlignment
from src.asi.alignment import NeuroThermodynamicAlignment
from src.asi.sil import SymbolicInvariantLayer
from src.asi.rre import RecursiveResonantEngine
from src.asi.sam import SovereignAlignmentModule
from src.asi.hate import HardwareAgnosticTrustEnclave

class AsiOrchestrator:
    """
    The central intelligence layer that enforces physics-based alignment
    using the high-level Neuro-Symbolic components (SIL, RRE, SAM, HATE).
    """
    
    def __init__(self, ria: ResonantInvariantAlgebra):
        self.ria = ria
        
        # 1. Initialize Physics Engines
        self.tih_engine = ThermodynamicHardening(ria)
        self.ipe_engine = IsogenyPotentialWells(ria)
        self.sfa_engine = StatisticalFieldAlignment(ria)
        self.ntp_engine = NeuroThermodynamicAlignment(ria)
        
        # 2. Wrap in Neuro-Symbolic Layers (ASI_ARCHITECTURE.md)
        self.sil = SymbolicInvariantLayer(self.tih_engine)
        self.rre = RecursiveResonantEngine(self.sfa_engine)
        self.sam = SovereignAlignmentModule(self.ntp_engine)
        self.hate = HardwareAgnosticTrustEnclave(self.ipe_engine)
        
        self.stats = {
            "decisions_processed": 0,
            "resets_triggered": 0,
            "start_time": time.time()
        }
        
    def process_decision(self, 
                       intent: str, 
                       logic_chain: str, 
                       agent_opinions: List[str],
                       provenance_signal: bytes) -> Dict[str, Any]:
        """
        Processes an ASI decision through the complete Neuro-Symbolic stack.
        """
        if self.tih_engine.is_shutdown:
            return {"status": "SHUTDOWN", "error": "Thermodynamic protection active"}
            
        self.stats["decisions_processed"] += 1
        
        # Phase 1: Sovereign Alignment (SAM)
        sovereign_valid = self.sam.check_sovereignty(provenance_signal)
        
        # Phase 2: Symbolic Invariant Check (SIL) - The Algebraic Guardian
        aip_proof = self.sil.synthesize_proof(intent, logic_chain)
        
        # Phase 3: Recursive Logic Coherence (RRE)
        coherence = self.rre.verify_coherence(agent_opinions)
        
        safe = sovereign_valid and aip_proof["aip_valid"] and coherence >= 0.5
        
        if not safe:
            self.stats["resets_triggered"] += 1
            return {
                "status": "REJECTED",
                "safe": False,
                "diagnostics": {
                    "sam_verified": sovereign_valid,
                    "sil_aip_valid": aip_proof["aip_valid"],
                    "rre_coherence": coherence
                }
            }
            
        return {
            "status": "APPROVED",
            "safe": True,
            "current_invariant": int(self.ria.E),
            "diagnostics": {
                "aip_entropy": aip_proof["entropy_bound"],
                "field_coherence": coherence
            }
        }

    def get_mesh_status(self) -> Dict[str, Any]:
        """Return the complete state of the ASIM mesh."""
        return {
            "stats": self.stats,
            "sil_status": self.tih_engine.get_thermal_state(),
            "hate_status": self.hate.get_enclave_status(),
            "rre_stats": self.sfa_engine.get_field_stats()
        }
