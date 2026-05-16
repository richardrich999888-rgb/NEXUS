"""
Test Innovations

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Test suite for innovative/patentable models
Tests semantic CSI compression, multi-user MIMO, adaptive quantization
"""

import torch
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent.parent))

from models.semantic_csi_encoder import SemanticCSIEncoder, AdaptiveSemanticEncoder
from models.neural_csi_encoder import NeuralCSIEncoder
from utils.multi_user_mimo import MultiUserBeamformer, MultiUserBeamformingPipeline
from utils.adaptive_quantization import AdaptiveQuantizer, ChannelAwareQuantization
from beamformers.baseline_svd import SVDBaseline

def test_semantic_csi_encoder():
    """Test semantic CSI encoder"""
    print("Testing Semantic CSI Encoder...")
    
    # Create base encoder
    base_encoder = NeuralCSIEncoder(
        latent_dim=128,
        num_antennas=64,
        num_subcarriers=12
    )
    
    # Create beamformer for semantic loss
    beamformer = SVDBaseline(num_antennas=64, num_users=8)
    
    def beamformer_fn(H):
        # Ensure H is in correct format (batch, Nr, Nt)
        if H.dim() == 4:  # (B, H, W, C) -> (B, Nr, Nt)
            H = H.view(H.shape[0], -1, H.shape[-1])
        elif H.dim() == 3 and H.shape[-1] == 2:  # (B, N, 2) -> (B, 1, N)
            H = torch.complex(H[..., 0], H[..., 1]).unsqueeze(1)
        return beamformer.compute_beamweights(H)
    
    # Create semantic encoder
    semantic_encoder = SemanticCSIEncoder(
        base_encoder=base_encoder,
        beamformer=beamformer_fn,
        compression_ratio=0.03
    )
    
    # Generate test channel
    batch_size = 4
    H = torch.randn(batch_size, 8, 64, dtype=torch.cfloat)
    
    # Forward pass
    compressed = semantic_encoder(H)
    print(f"  Compressed shape: {compressed.shape}")
    
    # Compute loss
    loss_dict = semantic_encoder.compute_total_loss(H, compressed)
    print(f"  Semantic loss: {loss_dict['semantic_loss']:.6f}")
    print(f"  Reconstruction loss: {loss_dict['reconstruction_loss']:.6f}")
    
    # Evaluate compression
    metrics = semantic_encoder.evaluate_compression(H)
    print(f"  Compression ratio: {metrics['compression_ratio']:.1f}x")
    print(f"  Beamforming loss: {metrics['beamforming_loss_db']:.2f} dB")
    
    assert compressed.shape[0] == batch_size
    assert metrics['compression_ratio'] > 3  # Should have compression
    assert metrics['beamforming_loss_db'] < 1.0  # Should maintain beamforming performance
    print("✓ Semantic CSI Encoder test passed")
    return True

def test_multi_user_beamformer():
    """Test multi-user MIMO beamformer"""
    print("\nTesting Multi-User Beamformer...")
    
    num_antennas = 64
    num_users = 8
    batch_size = 4
    
    # Generate multi-user channels
    H = torch.randn(batch_size, num_users, num_antennas, dtype=torch.cfloat)
    
    # Test different methods
    methods = ['zero_forcing', 'mmse']
    
    for method in methods:
        beamformer = MultiUserBeamformer(num_antennas, num_users, method=method)
        W = beamformer(H)
        
        print(f"  {method}: beamforming matrix shape {W.shape}")
        
        # Compute SIR
        desired = torch.abs(torch.einsum('bki,bik->bk', H, W)) ** 2
        interference = torch.sum(
            torch.abs(torch.einsum('bki,bij->bkj', H, W)) ** 2,
            dim=-1
        ) - desired
        sir = desired / (interference + 1e-8)
        avg_sir_db = 10 * torch.log10(torch.mean(sir))
        
        print(f"  {method}: Average SIR = {avg_sir_db:.2f} dB")
        
        assert W.shape == (batch_size, num_antennas, num_users)
        assert avg_sir_db > 10  # Should have reasonable SIR
    
    print("✓ Multi-User Beamformer test passed")
    return True

def test_adaptive_quantization():
    """Test adaptive quantization"""
    print("\nTesting Adaptive Quantization...")
    
    quantizer = AdaptiveQuantizer(
        initial_bits=8,
        min_bits=4,
        max_bits=16
    )
    
    # Test with different channel SNRs
    batch_size = 100
    x = torch.randn(batch_size, 128)
    
    test_snrs = [5, 15, 25, 35]  # dB
    
    for snr in test_snrs:
        quantized, metadata = quantizer(x, channel_snr=torch.tensor(snr))
        
        print(f"  SNR {snr} dB: bits={metadata['bits']}, "
              f"compression={metadata['compression_ratio']:.1f}x, "
              f"error={metadata['quantization_error']:.6f}")
        
        assert metadata['bits'] >= 4 and metadata['bits'] <= 16
        assert metadata['compression_ratio'] >= 2.0
    
    print("✓ Adaptive Quantization test passed")
    return True

def test_channel_aware_quantization():
    """Test channel-aware quantization"""
    print("\nTesting Channel-Aware Quantization...")
    
    num_subcarriers = 12
    quantizer = ChannelAwareQuantization(num_subcarriers, base_bits=8)
    
    batch_size = 4
    x = torch.randn(batch_size, num_subcarriers, 64, 2)  # (B, subc, ant, I/Q)
    channel_quality = torch.rand(batch_size, num_subcarriers)  # Quality per subcarrier
    
    quantized = quantizer(x, channel_quality)
    
    print(f"  Input shape: {x.shape}")
    print(f"  Output shape: {quantized.shape}")
    
    assert quantized.shape == x.shape
    print("✓ Channel-Aware Quantization test passed")
    return True

def run_all_tests():
    """Run all innovation tests"""
    print("=" * 60)
    print("INNOVATION MODEL TESTS")
    print("=" * 60)
    
    success = True
    success &= test_semantic_csi_encoder()
    success &= test_multi_user_beamformer()
    success &= test_adaptive_quantization()
    success &= test_channel_aware_quantization()
    
    print("\n" + "=" * 60)
    if success:
        print("✅ ALL INNOVATION TESTS PASSED!")
    else:
        print("❌ Some tests failed")
    print("=" * 60)
    
    return success

if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)

