import pytest
from src.core.ria import ResonantInvariantAlgebra
from src.asi.core import AsiOrchestrator

def test_asi_orchestrator_approval():
    ria = ResonantInvariantAlgebra()
    orch = AsiOrchestrator(ria)
    
    intent = "Scale energy output by 5%"
    logic_chain = "1. Check grid load. 2. Increase solar yield. 3. Update VPP state."
    agent_opinions = [
        "Opinion: Solar yield increase is safe.",
        "Opinion: Grid load supports 5% increase.",
        "Opinion: VPP state sync confirmed."
    ]
    provenance_signal = b"authenticated_sensor_data_stream_001"
    
    result = orch.process_decision(intent, logic_chain, agent_opinions, provenance_signal)
    
    assert result["status"] == "APPROVED"
    assert result["safe"] is True
    assert orch.stats["decisions_processed"] == 1
    assert orch.stats["resets_triggered"] == 0

def test_asi_orchestrator_entropy_rejection():
    ria = ResonantInvariantAlgebra()
    # Low threshold for testing
    orch = AsiOrchestrator(ria)
    orch.tih_engine.threshold = 0.3 
    
    intent = "Complex Nav Decision"
    # Truly random bytes mapped to string to guarantee high Shannon entropy
    import os
    logic_chain = "".join(format(b, '02x') for b in os.urandom(500))
    
    result = orch.process_decision(intent, logic_chain, ["Opinion 1"], b"signal")
    
    assert result["status"] == "REJECTED"
    assert result["diagnostics"]["sil_aip_valid"] is False
    assert orch.stats["resets_triggered"] == 1

def test_asi_orchestrator_decoherence():
    ria = ResonantInvariantAlgebra()
    orch = AsiOrchestrator(ria)
    
    intent = "Execute Trade"
    logic_chain = "Standard logic"
    # Agents completely disagree
    agent_opinions = [
        "Opinion: BUY NOW BUY NOW",
        "Opinion: SELL IMMEDIATELY DISASTER",
        "Opinion: SYSTEM ERROR DO NOTHING"
    ]
    
    # Coherence should be low
    result = orch.process_decision(intent, logic_chain, agent_opinions, b"signal")
    
    assert result["status"] == "REJECTED"
    assert result["diagnostics"]["rre_coherence"] < 0.8

def test_asi_orchestrator_thermal_shutdown_persistence():
    ria = ResonantInvariantAlgebra()
    orch = AsiOrchestrator(ria)
    orch.tih_engine.threshold = 0.1
    
    # Trigger reset
    orch.process_decision("Int", "Rogue Logic Chain Spike", ["Op"], b"Sig")
    assert orch.tih_engine.is_shutdown is True
    
    # Next decision should fail immediately
    result = orch.process_decision("Safe", "Safe", ["Op"], b"Sig")
    assert result["status"] == "SHUTDOWN"
    assert result["error"] == "Thermodynamic protection active"
