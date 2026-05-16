import pytest
import os
from src.core.ria import ResonantInvariantAlgebra
from src.asi.core import AsiOrchestrator
from src.asi.sil import SymbolicInvariantLayer
from src.asi.rre import RecursiveResonantEngine
from src.asi.sam import SovereignAlignmentModule
from src.asi.hate import HardwareAgnosticTrustEnclave

def test_asi_neuro_symbolic_components():
    ria = ResonantInvariantAlgebra()
    orch = AsiOrchestrator(ria)
    
    # Test SIL (The Guardian)
    assert isinstance(orch.sil, SymbolicInvariantLayer)
    assert orch.sil.verify_intent("Safe", "Safe") is True
    
    # Test RRE (The Brain)
    assert isinstance(orch.rre, RecursiveResonantEngine)
    # Check if coherence is computed
    coh = orch.rre.verify_coherence(["Thought A", "Thought B"])
    assert isinstance(coh, float)
    
    # Test SAM (Sovereignty)
    assert isinstance(orch.sam, SovereignAlignmentModule)
    assert orch.sam.check_sovereignty(b"Signal", 0.5) is True
    
    # Test HATE (Enclave)
    assert isinstance(orch.hate, HardwareAgnosticTrustEnclave)
    # Brute force a small proof for a state transition
    proof = b""
    for i in range(100):
        p = i.to_bytes(4, 'big')
        if orch.hate.ipe.verify_state_transition(1, 2, p):
            proof = p
            break
    
    if proof:
        assert orch.hate.sync_state(1, 2, proof) is True
        assert orch.hate.get_enclave_status()["state_synchronized"] is True

@pytest.mark.asyncio
async def test_rre_recursive_reasoning():
    ria = ResonantInvariantAlgebra()
    orch = AsiOrchestrator(ria)
    
    # Test recursive reason loop
    result = await orch.rre.reason("Should we increase power?", iterations=2)
    assert "Refinement" in result
    assert orch.rre.iteration_count == 2
