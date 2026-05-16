"""
Multi User Example

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Multi-User MIMO Example
Demonstrates multi-user beamforming with interference cancellation
"""

import torch
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent.parent))

from utils.multi_user_mimo import MultiUserBeamformer, MultiUserBeamformingPipeline
from utils.threegpp_channel_simulator import ThreeGPPChannelSimulator

def main():
    print("=" * 60)
    print("Multi-User MIMO Beamforming Example")
    print("=" * 60)
    
    # Configuration
    num_antennas = 64
    num_users = 8
    batch_size = 4
    
    # Generate multi-user channels
    print("\n1. Generating multi-user channels...")
    simulator = ThreeGPPChannelSimulator(
        num_antennas=num_antennas,
        num_users=num_users,
        scenario="CDL-A"
    )
    
    H = simulator.generate_cdl_channel(batch_size=batch_size)  # (B, K, Nt)
    print(f"   Channel shape: {H.shape}")
    
    # Test different beamforming methods
    print("\n2. Testing beamforming methods...")
    
    methods = ['zero_forcing', 'mmse', 'dirty_paper']
    results = {}
    
    for method in methods:
        print(f"\n   Method: {method}")
        beamformer = MultiUserBeamformer(num_antennas, num_users, method=method)
        
        # Compute beamforming matrix
        W = beamformer(H)  # (B, Nt, K)
        print(f"   Beamforming matrix shape: {W.shape}")
        
        # Compute signal-to-interference ratio
        desired = torch.abs(torch.einsum('bki,bik->bk', H, W)) ** 2
        interference = torch.sum(
            torch.abs(torch.einsum('bki,bij->bkj', H, W)) ** 2,
            dim=-1
        ) - desired
        
        sir = desired / (interference + 1e-8)
        avg_sir = torch.mean(sir).item()
        
        results[method] = {
            'avg_sir_db': 10 * torch.log10(avg_sir),
            'beamforming_matrix': W
        }
        
        print(f"   Average SIR: {10 * torch.log10(avg_sir):.2f} dB")
    
    # Test complete pipeline
    print("\n3. Testing complete pipeline...")
    pipeline = MultiUserBeamformingPipeline(num_antennas, num_users)
    
    output = pipeline.forward(H, method='mmse')
    
    print(f"   Compressed CSI shape: {output['compressed_csi'].shape}")
    print(f"   Power allocation: {output['power_allocation'][0].tolist()}")
    print(f"   Average SIR: {10 * torch.log10(torch.mean(output['sir'])):.2f} dB")
    
    print("\n" + "=" * 60)
    print("Example completed successfully!")
    print("=" * 60)

if __name__ == "__main__":
    main()



