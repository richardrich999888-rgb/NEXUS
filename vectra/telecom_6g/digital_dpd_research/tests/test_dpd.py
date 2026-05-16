"""
Test Dpd

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import pytest
import numpy as np
import sys
import os
sys.path.append('.')

from models.neural_dpd import NeuralDPD, BeamAwareDPD
from models.pa_behavioral import RappModel, SalehModel, PAArrayModel
from utils.metrics import DPDEvaluator

def test_neural_dpd_forward():
    """Test NeuralDPD forward pass"""
    model = NeuralDPD(memory_depth=5, hidden_dims=[32, 32])
    
    # Test with real-valued input
    batch_size = 100
    x_real = torch.randn(batch_size, 2)
    output = model(x_real)
    
    assert output.shape == (batch_size, 2)
    assert not torch.isnan(output).any()
    
    # Test with complex input
    x_complex = torch.randn(batch_size, dtype=torch.cfloat)
    output = model(x_complex)
    
    assert output.shape == (batch_size, 2)
    print("✓ NeuralDPD forward pass test passed")

def test_beam_aware_dpd():
    """Test BeamAwareDPD with beam conditioning"""
    num_antennas = 8
    model = BeamAwareDPD(num_clusters=4, memory_depth=3, hidden_dims=[16, 16], num_antennas=num_antennas)
    
    batch_size = 50
    x = torch.randn(batch_size, num_antennas, 2)
    beam_weights = torch.randn(num_antennas)
    
    output = model(x, beam_weights=beam_weights)
    
    assert output.shape == (batch_size, num_antennas, 2)
    print("✓ BeamAwareDPD forward pass test passed")

def test_pa_models():
    """Test PA behavioral models"""
    # Test Rapp model
    rapp = RappModel(smoothness_factor=2.5, saturation_amplitude=1.0)
    x = torch.randn(100, dtype=torch.cfloat)
    y = rapp(x)
    
    assert y.shape == x.shape
    assert torch.abs(y).max() <= 1.0  # Should saturate
    
    # Test Saleh model
    saleh = SalehModel(alpha_a=2.0, beta_a=1.0, alpha_phi=2.0, beta_phi=1.0)
    y_saleh = saleh(x)
    
    assert y_saleh.shape == x.shape
    print("✓ PA model tests passed")

def test_pa_array_model():
    """Test PA array with variations"""
    pa_array = PAArrayModel(num_antennas=4, model_type='rapp')
    
    batch_size = 10
    x = torch.randn(batch_size, 4, 2)
    y = pa_array(x)
    
    assert y.shape == (batch_size, 4, 2)
    print("✓ PA array model test passed")

def test_metrics():
    """Test DPD evaluation metrics"""
    evaluator = DPDEvaluator()
    
    # Generate test signals
    reference = torch.randn(1000, dtype=torch.cfloat)
    measured = reference + 0.1 * torch.randn_like(reference)
    
    # Test EVM
    evm = evaluator.calculate_evm(reference, measured)
    assert 0 <= evm <= 100
    print(f"  EVM: {evm:.2f}%")
    
    # Test NMSE
    nmse = evaluator.calculate_nmse(reference, measured)
    assert nmse < 0  # Should be negative in dB
    print(f"  NMSE: {nmse:.2f} dB")
    
    print("✓ Metrics tests passed")

def test_quantization():
    """Test model quantization"""
    from utils.quantization import QuantizationUtils
    
    model = NeuralDPD(memory_depth=3, hidden_dims=[16, 16])
    
    # Get original model size
    size_before = QuantizationUtils.get_model_size(model, quantized=False)
    
    # Apply quantization
    QuantizationUtils.quantize_model(model, num_bits=8, symmetric=True)
    
    # Get quantized model size
    size_after = QuantizationUtils.get_model_size(model, quantized=True)
    
    assert size_after < size_before / 3  # Should be at least 3x smaller
    print(f"  Model size reduction: {size_before/size_after:.1f}x")
    print("✓ Quantization test passed")

def run_all_tests():
    """Run all tests"""
    print("Running DPD system tests...")
    print("=" * 50)
    
    test_neural_dpd_forward()
    test_beam_aware_dpd()
    test_pa_models()
    test_pa_array_model()
    test_metrics()
    test_quantization()
    
    print("=" * 50)
    print("✅ All tests passed!")

if __name__ == "__main__":
    run_all_tests()

