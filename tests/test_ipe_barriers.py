import hashlib
from src.core.ria import ResonantInvariantAlgebra
from src.asi.ipe import IsogenyPotentialWells

def test_ipe_barrier_success():
    ria = ResonantInvariantAlgebra()
    # Set low security for testing to find a proof quickly
    ipe = IsogenyPotentialWells(ria, lambda_bits=8) # Only 1 byte (2 hex chars) of zero needed
    
    current_E = 1
    target_E = 123456789
    
    # Brute force a small proof for 1 byte of zero
    proof = b""
    found = False
    for i in range(1000):
        p = i.to_bytes(4, 'big')
        transition_id = f"{current_E}:{target_E}:{p.hex()}"
        h = hashlib.sha3_256(transition_id.encode()).hexdigest()
        if h.startswith("00"):
            proof = p
            found = True
            break
            
    assert found is True
    assert ipe.verify_state_transition(current_E, target_E, proof) is True

def test_ipe_barrier_failure():
    ria = ResonantInvariantAlgebra()
    ipe = IsogenyPotentialWells(ria, lambda_bits=16) # 2 bytes (4 hex chars)
    
    current_E = 1
    target_E = 999
    invalid_proof = b"invalid"
    
    # This should fail because 'invalid' doesn't meet the difficulty
    assert ipe.verify_state_transition(current_E, target_E, invalid_proof) is False

def test_ipe_report():
    ria = ResonantInvariantAlgebra()
    ipe = IsogenyPotentialWells(ria)
    report = ipe.generate_potential_report()
    
    assert "barrier_height" in report
    assert report["lambda_security"] == 128
