"""
Test Innovations

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Test suite for innovative/patentable DPD models
Tests coupled array DPD and predictive DPD
"""

import torch
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent.parent))

from models.coupled_array_dpd import CoupledArrayDPD, AdaptiveCoupledDPD
from models.predictive_dpd import PredictiveDPD, PAStateEncoder, TemporalPredictor
from models.neural_dpd import NeuralDPD

def test_coupled_array_dpd():
    """Test coupled array DPD"""
    print("Testing Coupled Array DPD...")
    
    num_antennas = 8
    # Create antenna positions (linear array)
    positions = torch.stack([
        torch.arange(num_antennas, dtype=torch.float32) * 0.5,  # x
        torch.zeros(num_antennas),  # y
        torch.zeros(num_antennas)   # z
    ], dim=1)
    
    # Create coupled DPD
    coupled_dpd = CoupledArrayDPD(
        num_antennas=num_antennas,
        antenna_positions=positions,
        coupling_radius=1.5,
        memory_depth=3,
        hidden_dims=[32, 32]
    )
    
    # Test forward pass
    batch_size = 4
    length = 100
    x = torch.randn(batch_size, num_antennas, length, 2)
    
    output = coupled_dpd(x)
    
    print(f"  Input shape: {x.shape}")
    print(f"  Output shape: {output.shape}")
    print(f"  Adjacency matrix shape: {coupled_dpd.adjacency.shape}")
    print(f"  Coupling connections: {(coupled_dpd.adjacency > 0).sum().item()}")
    
    assert output.shape == x.shape
    assert coupled_dpd.adjacency.shape == (num_antennas, num_antennas)
    
    # Test coupling loss
    pa_output = torch.randn_like(x)
    coupling_loss = coupled_dpd.compute_coupling_loss(x, pa_output)
    print(f"  Coupling loss: {coupling_loss.item():.6f}")
    
    print("✓ Coupled Array DPD test passed")
    return True

def test_predictive_dpd():
    """Test predictive DPD"""
    print("\nTesting Predictive DPD...")
    
    # Create base DPD
    base_dpd = NeuralDPD(memory_depth=5, hidden_dims=[32, 32])
    
    # Create predictive DPD
    predictive_dpd = PredictiveDPD(
        base_dpd=base_dpd,
        state_dim=64,
        prediction_horizon=10,
        history_length=50
    )
    
    # Test forward pass
    batch_size = 1
    length = 100
    x = torch.randn(batch_size, length, 2)
    
    # Create PA history
    pa_history = torch.randn(batch_size, 20, 2)
    
    output = predictive_dpd(x, pa_history=pa_history)
    
    print(f"  Input shape: {x.shape}")
    print(f"  Output shape: {output.shape}")
    print(f"  Prediction horizon: {predictive_dpd.prediction_horizon} samples")
    
    assert output.shape == x.shape
    
    # Test prediction accuracy (with dummy future state)
    predicted_state = torch.randn(batch_size, 64)
    actual_future_state = torch.randn(batch_size, 64)
    
    accuracy = predictive_dpd.get_prediction_accuracy(predicted_state, actual_future_state)
    print(f"  Prediction MSE: {accuracy['mse']:.6f}")
    print(f"  Prediction MAE: {accuracy['mae']:.6f}")
    
    print("✓ Predictive DPD test passed")
    return True

def test_pa_state_encoder():
    """Test PA state encoder"""
    print("\nTesting PA State Encoder...")
    
    encoder = PAStateEncoder(state_dim=64)
    
    batch_size = 4
    history_length = 20
    pa_history = torch.randn(batch_size, history_length, 2)
    
    state = encoder(pa_history)
    
    print(f"  History shape: {pa_history.shape}")
    print(f"  State shape: {state.shape}")
    
    assert state.shape == (batch_size, 64)
    print("✓ PA State Encoder test passed")
    return True

def test_temporal_predictor():
    """Test temporal predictor"""
    print("\nTesting Temporal Predictor...")
    
    predictor = TemporalPredictor(
        state_dim=64,
        hidden_dim=128,
        num_layers=2,
        prediction_horizon=10
    )
    
    batch_size = 4
    history_length = 50
    state_history = torch.randn(batch_size, history_length, 64)
    
    predicted_state = predictor(state_history)
    
    print(f"  History shape: {state_history.shape}")
    print(f"  Predicted state shape: {predicted_state.shape}")
    
    assert predicted_state.shape == (batch_size, 64)
    print("✓ Temporal Predictor test passed")
    return True

def run_all_tests():
    """Run all innovation tests"""
    print("=" * 60)
    print("DPD INNOVATION MODEL TESTS")
    print("=" * 60)
    
    success = True
    success &= test_coupled_array_dpd()
    success &= test_predictive_dpd()
    success &= test_pa_state_encoder()
    success &= test_temporal_predictor()
    
    print("\n" + "=" * 60)
    if success:
        print("✅ ALL DPD INNOVATION TESTS PASSED!")
    else:
        print("❌ Some tests failed")
    print("=" * 60)
    
    return success

if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)



