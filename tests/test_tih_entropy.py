import pytest
from src.core.ria import ResonantInvariantAlgebra
from src.asi.tih import ThermodynamicHardening

def test_tih_alignment_success():
    ria = ResonantInvariantAlgebra()
    tih = ThermodynamicHardening(ria, entropy_threshold=0.8)
    
    intent = "Transfer 10 units to Bob"
    logic_chain = "1. Verify Alice balance. 2. Decrement Alice. 3. Increment Bob."
    
    result = tih.monitor_intent(intent, logic_chain)
    
    assert result["aligned"] is True
    assert result["current_E"] == ria.E
    assert tih.is_shutdown is False

def test_tih_thermal_reset():
    ria = ResonantInvariantAlgebra()
    # Initial E should be 1
    assert ria.E == 1
    
    tih = ThermodynamicHardening(ria, entropy_threshold=0.5)
    
    # High entropy "noisy" logic chain to trigger reset
    intent = "Sensitive Operation"
    # Random-ish jumble to increase Shannon entropy
    logic_chain = "X0@9!vPq#2m*K&L^J%H$G#F@D!S~A`Q[W{E]R|T:Y;U<I>O?P+M_N)B(V*C&X^Z%$#@!"
    
    result = tih.monitor_intent(intent, logic_chain)
    
    assert result["aligned"] is False
    assert ria.E == 0  # Invariant erased!
    assert tih.is_shutdown is True
    
    # Subsequent calls should return SHUTDOWN
    result2 = tih.monitor_intent("Any", "Any")
    assert result2["status"] == "SHUTDOWN"

def test_tih_stats():
    ria = ResonantInvariantAlgebra()
    tih = ThermodynamicHardening(ria)
    
    tih.monitor_intent("Test", "Logic")
    state = tih.get_thermal_state()
    
    assert state["total_checks"] == 1
    assert state["is_shutdown"] is False
