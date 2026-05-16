"""
Demo Joint Optimization

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Demo script for Joint Beamforming + DPD optimization
Shows the complete pipeline
"""

import torch
import numpy as np
import matplotlib.pyplot as plt
import yaml
import sys
sys.path.append('.')

from models.neural_dpd import JointBeamformingDPD, BeamAwareDPD
from models.pa_behavioral import PAArrayModel
from beamformers.tt_beamformer import TTBeamformer
from utils.signal_generation import SignalGenerator
from utils.metrics import DPDEvaluator

def load_config():
    """Load demo configuration"""
    config = {
        'system': {
            'num_antennas': 64,
            'num_clusters': 8,
            'sample_rate': 122.88e6,
            'bandwidth': 100e6
        },
        'pa_model': {
            'type': 'rapp',
            'smoothness_factor': 3.0
        }
    }
    return config

def run_demo():
    """Run complete demo"""
    print("=" * 60)
    print("Joint Beamforming + DPD Optimization Demo")
    print("=" * 60)
    
    config = load_config()
    
    # Initialize models
    print("\n1. Initializing models...")
    pa_model = PAArrayModel(
        num_antennas=config['system']['num_antennas'],
        model_type=config['pa_model']['type']
    )
    
    dpd_model = BeamAwareDPD(
        num_clusters=config['system']['num_clusters'],
        memory_depth=5
    )
    
    beamformer = TTBeamformer()
    
    joint_model = JointBeamformingDPD(
        tt_beamformer=beamformer,
        neural_dpd=dpd_model,
        num_antennas=config['system']['num_antennas']
    )
    
    print(f"   • PA model: {config['pa_model']['type']}")
    print(f"   • DPD clusters: {config['system']['num_clusters']}")
    print(f"   • Beamformer: Tensor-Train (compression: {beamformer.get_compression_ratio():.3f})")
    
    # Generate test signal
    print("\n2. Generating test signals...")
    signal_gen = SignalGenerator()
    
    # Generate OFDM signal
    test_signal = signal_gen.generate_ofdm_signal(
        num_symbols=100,
        modulation='64qam'
    )
    
    # Generate random channel
    channel = torch.randn(64, dtype=torch.cfloat) * (1/np.sqrt(2))
    
    print(f"   • Signal length: {len(test_signal)} samples")
    print(f"   • Modulation: 64QAM")
    print(f"   • Channel: Rayleigh fading")
    
    # Run inference
    print("\n3. Running joint optimization...")
    with torch.no_grad():
        # Without DPD
        beam_weights = beamformer.compute_beamweights(channel.unsqueeze(0).unsqueeze(1))
        # Broadcast to (1, 64, 371200)
        # beam_weights: (1, 64) -> (1, 64, 1)
        # test_signal: (371200) -> (1, 1, 371200)
        beamformed = test_signal.reshape(1, 1, -1) * beam_weights.reshape(1, -1, 1).conj()
        
        # Convert to I/Q for PA
        beamformed_iq = torch.stack([beamformed.real, beamformed.imag], dim=-1)
        pa_output_no_dpd = pa_model(beamformed_iq)
        pa_output_no_dpd_complex = torch.complex(
            pa_output_no_dpd[..., 0],
            pa_output_no_dpd[..., 1]
        )
        
        # With DPD
        outputs = joint_model(
            channel.unsqueeze(0),  # Add batch dimension
            test_signal.unsqueeze(0).unsqueeze(1)  # [1, 1, N]
        )
        
        pa_output_with_dpd = pa_model(
            torch.stack([outputs['predistorted'].real, 
                        outputs['predistorted'].imag], dim=-1)
        )
        pa_output_with_dpd_complex = torch.complex(
            pa_output_with_dpd[..., 0],
            pa_output_with_dpd[..., 1]
        )
    
    # Calculate metrics
    print("\n4. Calculating performance metrics...")
    evaluator = DPDEvaluator()
    
    # EVM
    evm_no_dpd = evaluator.calculate_evm(
        beamformed[0, :, 0],
        pa_output_no_dpd_complex[0, :, 0]
    )
    
    evm_with_dpd = evaluator.calculate_evm(
        outputs['predistorted'][0, :, 0],
        pa_output_with_dpd_complex[0, :, 0]
    )
    
    # ACLR
    aclr_no_dpd = evaluator.calculate_aclr(
        pa_output_no_dpd_complex[0, :, 0].cpu().numpy(),
        config['system']['sample_rate'],
        config['system']['bandwidth']
    )
    
    aclr_with_dpd = evaluator.calculate_aclr(
        pa_output_with_dpd_complex[0, :, 0].cpu().numpy(),
        config['system']['sample_rate'],
        config['system']['bandwidth']
    )
    
    print("\n" + "=" * 60)
    print("DEMO RESULTS")
    print("=" * 60)
    
    print(f"\nError Vector Magnitude (EVM):")
    print(f"  • Without DPD: {evm_no_dpd:.2f}%")
    print(f"  • With DPD:    {evm_with_dpd:.2f}%")
    print(f"  • Improvement: {evm_no_dpd - evm_with_dpd:.2f}%")
    
    print(f"\nAdjacent Channel Leakage Ratio (ACLR):")
    print(f"  • Without DPD: {aclr_no_dpd:.2f} dBc")
    print(f"  • With DPD:    {aclr_with_dpd:.2f} dBc")
    print(f"  • Improvement: {aclr_no_dpd - aclr_with_dpd:.2f} dB")
    
    print(f"\nModel Characteristics:")
    print(f"  • DPD model size: {dpd_model.get_model_size():.1f} KB")
    print(f"  • Beamformer compression: {beamformer.get_compression_ratio():.3f}")
    
    # Plot constellation diagrams
    print("\n5. Generating plots...")
    fig, axes = plt.subplots(1, 2, figsize=(12, 5))
    
    # Without DPD
    axes[0].scatter(pa_output_no_dpd_complex[0, :1000, 0].real.cpu().numpy(),
                   pa_output_no_dpd_complex[0, :1000, 0].imag.cpu().numpy(),
                   alpha=0.5, s=10)
    axes[0].set_title(f'Without DPD (EVM: {evm_no_dpd:.2f}%)')
    axes[0].set_xlabel('I')
    axes[0].set_ylabel('Q')
    axes[0].grid(True, alpha=0.3)
    axes[0].axis('equal')
    
    # With DPD
    axes[1].scatter(pa_output_with_dpd_complex[0, :1000, 0].real.cpu().numpy(),
                   pa_output_with_dpd_complex[0, :1000, 0].imag.cpu().numpy(),
                   alpha=0.5, s=10, color='green')
    axes[1].set_title(f'With DPD (EVM: {evm_with_dpd:.2f}%)')
    axes[1].set_xlabel('I')
    axes[1].set_ylabel('Q')
    axes[1].grid(True, alpha=0.3)
    axes[1].axis('equal')
    
    plt.tight_layout()
    plt.savefig('demo_constellations.png', dpi=150, bbox_inches='tight')
    
    print("\n" + "=" * 60)
    print("Demo completed successfully!")
    print("Constellation plots saved to 'demo_constellations.png'")
    print("=" * 60)
    
    return {
        'evm': {'without_dpd': evm_no_dpd, 'with_dpd': evm_with_dpd},
        'aclr': {'without_dpd': aclr_no_dpd, 'with_dpd': aclr_with_dpd}
    }

if __name__ == "__main__":
    results = run_demo()

