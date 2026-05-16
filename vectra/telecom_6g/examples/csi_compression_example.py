"""
Example: VECTRA CSI Compression for 6G Massive MIMO

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Example: VECTRA CSI Compression for 6G Massive MIMO

Demonstrates how to use VECTRA to compress CSI feedback
for 6G massive MIMO systems.
"""

import numpy as np
import sys
import os

# Add integration path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../vectra_integration'))

from csi_compression import VectraCSICompressor, compress_csi_feedback


def example_csi_compression():
    """Example of CSI compression using VECTRA."""
    
    print("=" * 60)
    print("VECTRA CSI Compression Example")
    print("=" * 60)
    print()
    
    # Generate example CSI matrix (64 antennas, 12 subcarriers)
    num_antennas = 64
    num_subcarriers = 12
    
    # Create structured CSI with correlation (realistic scenario)
    csi = np.random.randn(num_antennas, num_subcarriers, 2).astype(np.float32)
    
    # Add structure: spatial correlation
    for i in range(1, num_antennas):
        csi[i, :, :] = 0.7 * csi[i-1, :, :] + 0.3 * csi[i, :, :]
    
    print(f"Original CSI shape: {csi.shape}")
    print(f"Original CSI size: {csi.nbytes} bytes")
    print()
    
    # Compress using VECTRA
    compressor = VectraCSICompressor(compression_ratio=0.1)
    compressed_data, ratio, metadata = compressor.compress_csi(csi)
    
    print(f"Compression Status: {metadata['status']}")
    if metadata['status'] == 'compressed':
        print(f"Compressed size: {metadata['compressed_size']} bytes")
        print(f"Compression ratio: {ratio:.2f}x")
        print(f"Size reduction: {(1 - 1/ratio) * 100:.1f}%")
    else:
        print(f"Reason: {metadata.get('reason', 'unknown')}")
    print()
    
    # Decompress
    if metadata['status'] == 'compressed':
        decompressed_csi = compressor.decompress_csi(compressed_data, metadata)
        
        # Verify losslessness
        if np.allclose(csi, decompressed_csi, atol=1e-5):
            print("✓ Losslessness verified: decompressed CSI matches original")
        else:
            print("✗ Losslessness check failed")
            print(f"  Max difference: {np.max(np.abs(csi - decompressed_csi))}")
    print()
    
    # Compare with baseline (no compression)
    baseline_size = csi.nbytes
    if metadata['status'] == 'compressed':
        vectra_size = metadata['compressed_size']
        improvement = (baseline_size - vectra_size) / baseline_size * 100
        print(f"Baseline (no compression): {baseline_size} bytes")
        print(f"VECTRA compressed: {vectra_size} bytes")
        print(f"Bandwidth reduction: {improvement:.1f}%")
    print()


def example_csi_feedback_compression():
    """Example of CSI feedback compression for uplink."""
    
    print("=" * 60)
    print("CSI Feedback Compression for 6G Uplink")
    print("=" * 60)
    print()
    
    # Simulate CSI feedback (64×8×12 = 6,144 complex values)
    # This is the bottleneck mentioned in INNOVATION_ROADMAP.md
    num_antennas = 64
    num_users = 8
    num_subcarriers = 12
    
    # Create CSI feedback matrix
    csi_feedback = np.random.randn(num_antennas, num_users, num_subcarriers, 2).astype(np.float32)
    
    original_size = csi_feedback.nbytes
    print(f"CSI Feedback shape: {csi_feedback.shape}")
    print(f"Original size: {original_size} bytes ({original_size/1024:.1f} KB)")
    print()
    
    # Compress each user's CSI
    total_compressed = 0
    total_original = 0
    
    for user_idx in range(num_users):
        user_csi = csi_feedback[:, user_idx, :, :]
        compressed, ratio, metadata = compress_csi_feedback(user_csi, target_ratio=0.1)
        
        total_original += user_csi.nbytes
        if metadata['status'] == 'compressed':
            total_compressed += len(compressed)
        else:
            total_compressed += user_csi.nbytes  # Fail-open
    
    overall_ratio = total_original / total_compressed if total_compressed > 0 else 1.0
    bandwidth_reduction = (1 - total_compressed / total_original) * 100
    
    print(f"Total original size: {total_original} bytes ({total_original/1024:.1f} KB)")
    print(f"Total compressed size: {total_compressed} bytes ({total_compressed/1024:.1f} KB)")
    print(f"Overall compression ratio: {overall_ratio:.2f}x")
    print(f"Bandwidth reduction: {bandwidth_reduction:.1f}%")
    print()
    
    # Compare with 10:1 baseline (current state)
    baseline_compressed = total_original / 10
    print(f"Current 10:1 compression: {baseline_compressed:.0f} bytes ({baseline_compressed/1024:.1f} KB)")
    print(f"VECTRA compression: {total_compressed:.0f} bytes ({total_compressed/1024:.1f} KB)")
    
    if total_compressed < baseline_compressed:
        additional_reduction = (baseline_compressed - total_compressed) / baseline_compressed * 100
        print(f"Additional reduction vs. 10:1: {additional_reduction:.1f}%")
    print()


if __name__ == "__main__":
    example_csi_compression()
    print()
    example_csi_feedback_compression()










