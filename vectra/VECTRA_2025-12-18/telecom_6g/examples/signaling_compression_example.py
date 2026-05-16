"""
Example: VECTRA Signaling Message Compression

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Example: VECTRA Signaling Message Compression

Demonstrates how to use VECTRA to compress 5G/6G signaling messages
(NAS, RRC, NGAP) for bandwidth reduction.
"""

import sys
import os

# Add integration path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../vectra_integration'))

from signaling_compression import (
    VectraSignalingCompressor,
    compress_nas_message,
    compress_rrc_message,
    compress_ngap_message
)


def example_nas_compression():
    """Example of NAS message compression."""
    
    print("=" * 60)
    print("NAS Message Compression Example")
    print("=" * 60)
    print()
    
    # Example NAS ATTACH REQUEST message (structured format)
    nas_message = b"NAS-5GS:type:ATTACH_REQUEST:ue_id:12345:amf_id:67890:security:enabled:plmn:001-01"
    
    print(f"Original NAS message: {nas_message}")
    print(f"Original size: {len(nas_message)} bytes")
    print()
    
    # Compress
    compressor = VectraSignalingCompressor()
    compressed, ratio, metadata = compressor.compress_message(nas_message, "NAS")
    
    print(f"Compression Status: {metadata['status']}")
    if metadata['status'] == 'compressed':
        print(f"Compressed size: {metadata['compressed_size']} bytes")
        print(f"Compression ratio: {ratio:.2f}x")
        print(f"Size reduction: {(1 - 1/ratio) * 100:.1f}%")
    print()
    
    # Decompress and verify
    if metadata['status'] == 'compressed':
        decompressed = compressor.decompress_message(compressed, metadata)
        
        if decompressed == nas_message:
            print("✓ Losslessness verified: decompressed message matches original")
        else:
            print("✗ Losslessness check failed")
            print(f"  Original: {nas_message}")
            print(f"  Decompressed: {decompressed}")
    print()


def example_rrc_compression():
    """Example of RRC message compression."""
    
    print("=" * 60)
    print("RRC Message Compression Example")
    print("=" * 60)
    print()
    
    # Example RRC CONNECTION SETUP message
    rrc_message = b"RRC:type:CONNECTION_SETUP:ue_id:12345:cell_id:001:bearer:QCI9:security:enabled"
    
    print(f"Original RRC message size: {len(rrc_message)} bytes")
    
    # Compress
    compressed, ratio, metadata = compress_rrc_message(rrc_message)
    
    print(f"Compression Status: {metadata['status']}")
    if metadata['status'] == 'compressed':
        print(f"Compressed size: {metadata['compressed_size']} bytes")
        print(f"Compression ratio: {ratio:.2f}x")
    print()


def example_multiple_messages():
    """Example of compressing multiple signaling messages."""
    
    print("=" * 60)
    print("Multiple Signaling Messages Compression")
    print("=" * 60)
    print()
    
    # Simulate multiple signaling messages per second
    messages_per_second = 1000
    
    # Example messages
    nas_messages = [
        b"NAS-5GS:type:ATTACH_REQUEST:ue_id:{}:amf_id:67890".format(i).encode()
        for i in range(100)
    ]
    
    rrc_messages = [
        b"RRC:type:CONNECTION_SETUP:ue_id:{}:cell_id:001".format(i).encode()
        for i in range(100)
    ]
    
    # Compress all messages
    compressor = VectraSignalingCompressor()
    
    total_original = 0
    total_compressed = 0
    
    for msg in nas_messages + rrc_messages:
        compressed, ratio, metadata = compressor.compress_message(msg)
        total_original += len(msg)
        if metadata['status'] == 'compressed':
            total_compressed += metadata['compressed_size']
        else:
            total_compressed += len(msg)  # Fail-open
    
    overall_ratio = total_original / total_compressed if total_compressed > 0 else 1.0
    bandwidth_reduction = (1 - total_compressed / total_original) * 100
    
    print(f"Total messages: {len(nas_messages) + len(rrc_messages)}")
    print(f"Total original size: {total_original} bytes ({total_original/1024:.1f} KB)")
    print(f"Total compressed size: {total_compressed} bytes ({total_compressed/1024:.1f} KB)")
    print(f"Overall compression ratio: {overall_ratio:.2f}x")
    print(f"Bandwidth reduction: {bandwidth_reduction:.1f}%")
    print()
    
    # Estimate bandwidth savings
    messages_per_second = 1000
    bytes_per_second_original = (total_original / len(nas_messages + rrc_messages)) * messages_per_second
    bytes_per_second_compressed = (total_compressed / len(nas_messages + rrc_messages)) * messages_per_second
    
    print(f"Estimated bandwidth (1000 msg/s):")
    print(f"  Original: {bytes_per_second_original/1024:.1f} KB/s")
    print(f"  Compressed: {bytes_per_second_compressed/1024:.1f} KB/s")
    print(f"  Savings: {(bytes_per_second_original - bytes_per_second_compressed)/1024:.1f} KB/s")
    print()


if __name__ == "__main__":
    example_nas_compression()
    print()
    example_rrc_compression()
    print()
    example_multiple_messages()








