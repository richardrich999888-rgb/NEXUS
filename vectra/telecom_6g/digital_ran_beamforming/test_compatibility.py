"""
Test Compatibility

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Test script to verify all components work together
"""

import torch
import sys
import os
sys.path.append('.')

from models.neural_csi_encoder import NeuralCSIEncoder
from models.sparse_beam_mask_generator import SparseBeamMaskGenerator
from beamformers.tt_beamformer import TTBeamformer
from utils.quantization_utils import QuantizationUtils
from utils.threegpp_cdl import ThreeGPP_CDL
from utils.threegpp_channel_simulator import ThreeGPPChannelSimulator

def test_cdl_simulator():
    """Test the 3GPP CDL simulator"""
    print("Testing 3GPP CDL Simulator...")
    
    # Test CDL profiles
    for profile in ["A", "B", "C"]:
        cdl = ThreeGPP_CDL(
            profile=profile,
            carrier_freq=3.5e9,
            num_ant=64,
            num_users=8,
            num_subcarriers=12,
            speed=3.0,
            device="cpu"
        )
        
        # Generate batch of channels
        H = cdl.generate_csi_batch(batch_size=4)
        print(f"CDL-{profile} output shape: {H.shape}")
        
        # Verify properties
        assert H.shape == (4, 8, 64, 12), f"Wrong shape for CDL-{profile}"
        assert torch.is_complex(H), f"Output not complex for CDL-{profile}"
    
    print("✅ CDL simulator working correctly!")
    return True

def test_channel_simulator_wrapper():
    """Test the backward compatibility wrapper"""
    print("\nTesting Channel Simulator Wrapper...")
    
    simulator = ThreeGPPChannelSimulator(
        num_antennas=64,
        num_users=8,
        scenario="CDL-A",
        carrier_freq=3.5e9
    )
    
    # Test all generation methods
    H_cdl = simulator.generate_cdl_channel(batch_size=4)
    H_rayleigh = simulator.generate_rayleigh_channel(batch_size=4)
    H_3gpp = simulator.generate_3gpp_channel(batch_size=4)
    
    print(f"CDL channel shape: {H_cdl.shape}")
    print(f"Rayleigh channel shape: {H_rayleigh.shape}")
    print(f"3GPP channel shape: {H_3gpp.shape}")
    
    assert H_cdl.shape == (4, 8, 64)
    assert H_rayleigh.shape == (4, 8, 64)
    assert H_3gpp.shape == (4, 8, 64)
    
    print("✅ Channel simulator wrapper working correctly!")
    return True

def test_full_pipeline():
    """Test the complete pipeline integration"""
    print("\nTesting Full Pipeline...")
    
    # Configuration
    batch_size = 4
    num_antennas = 64
    num_users = 8
    num_subcarriers = 12
    
    # Generate channels using wrapper
    simulator = ThreeGPPChannelSimulator(
        num_antennas=num_antennas,
        num_users=num_users,
        scenario="CDL-A"
    )
    
    H = simulator.generate_cdl_channel(batch_size=batch_size)
    print(f"Channel shape: {H.shape}")
    
    # 1. Neural CSI Encoder
    encoder = NeuralCSIEncoder(
        latent_dim=128,
        num_antennas=num_antennas,
        num_subcarriers=num_subcarriers
    )
    
    compressed = encoder.compress(H)
    print(f"Encoder output shape: {compressed.shape}")
    
    # 2. Sparse Beam Mask Generator
    mask_generator = SparseBeamMaskGenerator(
        latent_dim=128,
        num_antennas=num_antennas,
        hidden=256,
        topk=19
    )
    
    beam_mask, beam_probs = mask_generator(compressed, hard=True)
    print(f"Beam mask shape: {beam_mask.shape}")
    print(f"Sparsity: {1.0 - beam_mask.float().mean():.3f}")
    
    # 3. TT Beamformer
    beamformer = TTBeamformer(num_ant=num_antennas, num_users=num_users)
    weights = beamformer.compute_beamweights(H, beam_mask)
    print(f"Beamformer weights shape: {weights.shape}")
    
    # 4. Quantization
    quantized_latent, scale_latent = QuantizationUtils.quantize_tensor(compressed, num_bits=8)
    reconstructed_latent = QuantizationUtils.dequantize_tensor(quantized_latent, scale_latent)
    quantization_error = torch.mean(torch.abs(compressed - reconstructed_latent))
    print(f"Quantization error: {quantization_error:.6f}")
    
    print("\n✅ All components integrated successfully!")
    return True

if __name__ == "__main__":
    print("=" * 60)
    print("COMPREHENSIVE COMPATIBILITY TEST")
    print("=" * 60)
    
    success = True
    success &= test_cdl_simulator()
    success &= test_channel_simulator_wrapper()
    success &= test_full_pipeline()
    
    if success:
        print("\n" + "=" * 60)
        print("🎉 ALL TESTS PASSED! Repository is fully functional.")
        print("=" * 60)
        print("\nNext steps:")
        print("1. Run: python run_benchmark.py --all")
        print("2. Check generated benchmark_results.json")
        print("3. Review logs for performance metrics")
    else:
        print("\n❌ Some tests failed. Please check the implementation.")
        sys.exit(1)
