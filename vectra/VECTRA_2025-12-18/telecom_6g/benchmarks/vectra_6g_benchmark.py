"""
VECTRA 6G Performance Benchmarks

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

VECTRA 6G Performance Benchmarks

Benchmarks VECTRA compression for 6G use cases:
- CSI feedback compression
- Signaling message compression
- Beamforming weight compression
- DPD coefficient compression
"""

import numpy as np
import time
import sys
import os

# Add integration path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../vectra_integration'))

from csi_compression import VectraCSICompressor
from signaling_compression import VectraSignalingCompressor
from beamforming_compression import VectraBeamformingCompressor
from dpd_compression import VectraDPDCompressor


def benchmark_csi_compression():
    """Benchmark CSI compression performance."""
    
    print("=" * 60)
    print("CSI Compression Benchmark")
    print("=" * 60)
    print()
    
    # Test configurations
    configs = [
        (64, 12),   # 64 antennas, 12 subcarriers
        (128, 24),  # 128 antennas, 24 subcarriers
        (256, 48),  # 256 antennas, 48 subcarriers
    ]
    
    compressor = VectraCSICompressor()
    
    results = []
    
    for num_ant, num_sub in configs:
        # Generate CSI with strong structure (spatial correlation)
        csi = np.random.randn(num_ant, num_sub, 2).astype(np.float32)
        
        # Add strong structure (spatial correlation) - makes it compressible
        base_pattern = csi[0, :, :].copy()
        for i in range(1, num_ant):
            csi[i, :, :] = 0.8 * base_pattern + 0.2 * csi[i, :, :]
        
        # Add frequency correlation
        for j in range(1, num_sub):
            csi[:, j, :] = 0.7 * csi[:, j-1, :] + 0.3 * csi[:, j, :]
        
        # Benchmark compression
        start = time.time()
        compressed, ratio, metadata = compressor.compress_csi(csi)
        compress_time = (time.time() - start) * 1000  # ms
        
        # Benchmark decompression
        if metadata['status'] == 'compressed':
            start = time.time()
            decompressed = compressor.decompress_csi(compressed, metadata)
            decompress_time = (time.time() - start) * 1000  # ms
        else:
            decompress_time = 0
        
        original_size = csi.nbytes
        compressed_size = len(compressed) if metadata['status'] == 'compressed' else original_size
        
        results.append({
            'config': f"{num_ant}×{num_sub}",
            'original_size': original_size,
            'compressed_size': compressed_size,
            'ratio': ratio,
            'compress_time_ms': compress_time,
            'decompress_time_ms': decompress_time,
            'status': metadata['status']
        })
    
    # Print results
    print(f"{'Config':<12} {'Original':<12} {'Compressed':<12} {'Ratio':<8} {'Compress':<10} {'Decompress':<12} {'Status':<12}")
    print("-" * 80)
    
    for r in results:
        print(f"{r['config']:<12} "
              f"{r['original_size']/1024:>6.1f} KB  "
              f"{r['compressed_size']/1024:>6.1f} KB  "
              f"{r['ratio']:>6.2f}x  "
              f"{r['compress_time_ms']:>6.2f} ms  "
              f"{r['decompress_time_ms']:>8.2f} ms  "
              f"{r['status']:<12}")
    
    print()


def benchmark_signaling_compression():
    """Benchmark signaling message compression."""
    
    print("=" * 60)
    print("Signaling Message Compression Benchmark")
    print("=" * 60)
    print()
    
    # Test message types - use longer, more structured messages for better compression
    messages = {
        'NAS_ATTACH': b"NAS-5GS:type:ATTACH_REQUEST:ue_id:12345:amf_id:67890:security:enabled:plmn:001-01:guti:12345678",
        'NAS_DETACH': b"NAS-5GS:type:DETACH_REQUEST:ue_id:12345:amf_id:67890:cause:normal:plmn:001-01",
        'RRC_SETUP': b"RRC:type:CONNECTION_SETUP:ue_id:12345:cell_id:001:bearer:QCI9:security:enabled:rrc_version:15",
        'RRC_RELEASE': b"RRC:type:CONNECTION_RELEASE:ue_id:12345:cause:normal:redirect:true",
        'NGAP_INIT': b"NGAP:type:INITIAL_UE_MESSAGE:ue_id:12345:amf_id:67890:ran_ue_id:98765:tai:001-01-12345",
    }
    
    compressor = VectraSignalingCompressor()
    
    results = []
    
    for msg_type, message in messages.items():
        # Benchmark compression
        start = time.time()
        compressed, ratio, metadata = compressor.compress_message(message, msg_type.split('_')[0])
        compress_time = (time.time() - start) * 1000  # ms
        
        # Benchmark decompression
        if metadata['status'] == 'compressed':
            start = time.time()
            decompressed = compressor.decompress_message(compressed, metadata)
            decompress_time = (time.time() - start) * 1000  # ms
        else:
            decompress_time = 0
        
        original_size = len(message)
        compressed_size = len(compressed) if metadata['status'] == 'compressed' else original_size
        
        results.append({
            'type': msg_type,
            'original_size': original_size,
            'compressed_size': compressed_size,
            'ratio': ratio,
            'compress_time_ms': compress_time,
            'decompress_time_ms': decompress_time,
            'status': metadata['status']
        })
    
    # Print results
    print(f"{'Message Type':<15} {'Original':<10} {'Compressed':<12} {'Ratio':<8} {'Compress':<10} {'Decompress':<12} {'Status':<12}")
    print("-" * 85)
    
    for r in results:
        print(f"{r['type']:<15} "
              f"{r['original_size']:>6} B   "
              f"{r['compressed_size']:>8} B   "
              f"{r['ratio']:>6.2f}x  "
              f"{r['compress_time_ms']:>6.2f} ms  "
              f"{r['decompress_time_ms']:>8.2f} ms  "
              f"{r['status']:<12}")
    
    print()


def benchmark_throughput():
    """Benchmark throughput (messages per second)."""
    
    print("=" * 60)
    print("Throughput Benchmark")
    print("=" * 60)
    print()
    
    # CSI compression throughput
    csi = np.random.randn(64, 12, 2).astype(np.float32)
    csi_compressor = VectraCSICompressor()
    
    num_iterations = 100
    start = time.time()
    for _ in range(num_iterations):
        csi_compressor.compress_csi(csi)
    csi_time = time.time() - start
    csi_throughput = num_iterations / csi_time
    
    # Signaling compression throughput
    nas_msg = b"NAS-5GS:type:ATTACH_REQUEST:ue_id:12345:amf_id:67890"
    sig_compressor = VectraSignalingCompressor()
    
    start = time.time()
    for _ in range(num_iterations):
        sig_compressor.compress_message(nas_msg, "NAS")
    sig_time = time.time() - start
    sig_throughput = num_iterations / sig_time
    
    print(f"CSI Compression:")
    print(f"  Throughput: {csi_throughput:.0f} compressions/second")
    print(f"  Latency: {(csi_time/num_iterations)*1000:.2f} ms")
    print()
    
    print(f"Signaling Compression:")
    print(f"  Throughput: {sig_throughput:.0f} compressions/second")
    print(f"  Latency: {(sig_time/num_iterations)*1000:.2f} ms")
    print()
    
    # Compare with requirements
    print("Telecom Requirements:")
    print(f"  Signaling: 1,000-10,000 msg/s required")
    print(f"  VECTRA: {sig_throughput:.0f} msg/s")
    if sig_throughput >= 1000:
        print(f"  ✓ Meets requirement")
    else:
        print(f"  ✗ Below requirement")
    print()


if __name__ == "__main__":
    benchmark_csi_compression()
    benchmark_signaling_compression()
    benchmark_throughput()








