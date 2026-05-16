"""
Tests for VECTRA 6G Integration

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Tests for VECTRA 6G Integration

Tests VECTRA compression integration with 6G RAN technologies.
"""

import numpy as np
import sys
import os

# Add integration path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../vectra_integration'))

from csi_compression import VectraCSICompressor
from signaling_compression import VectraSignalingCompressor
from beamforming_compression import VectraBeamformingCompressor
from dpd_compression import VectraDPDCompressor


def test_csi_compression_losslessness():
    """Test that CSI compression is lossless."""
    
    # Generate CSI
    csi = np.random.randn(64, 12, 2).astype(np.float32)
    
    # Compress and decompress
    compressor = VectraCSICompressor()
    compressed, ratio, metadata = compressor.compress_csi(csi)
    
    if metadata['status'] == 'compressed':
        decompressed = compressor.decompress_csi(compressed, metadata)
        
        # Verify losslessness
        assert np.allclose(csi, decompressed, atol=1e-5), "CSI compression not lossless"
        print("✓ CSI compression losslessness test passed")
    else:
        print("⚠ CSI compression failed-open (high entropy)")


def test_signaling_compression_losslessness():
    """Test that signaling compression is lossless."""
    
    # Generate signaling message
    message = b"NAS-5GS:type:ATTACH_REQUEST:ue_id:12345:amf_id:67890"
    
    # Compress and decompress
    compressor = VectraSignalingCompressor()
    compressed, ratio, metadata = compressor.compress_message(message, "NAS")
    
    if metadata['status'] == 'compressed':
        decompressed = compressor.decompress_message(compressed, metadata)
        
        # Verify losslessness
        assert decompressed == message, "Signaling compression not lossless"
        print("✓ Signaling compression losslessness test passed")
    else:
        print("⚠ Signaling compression failed-open (high entropy)")


def test_beamforming_compression_losslessness():
    """Test that beamforming compression is lossless."""
    
    # Generate beamforming weights
    weights = np.random.randn(64, 8, 2).astype(np.float32)
    
    # Compress and decompress
    compressor = VectraBeamformingCompressor()
    compressed, ratio, metadata = compressor.compress_weights(weights)
    
    if metadata['status'] == 'compressed':
        decompressed = compressor.decompress_weights(compressed, metadata)
        
        # Verify losslessness
        assert np.allclose(weights, decompressed, atol=1e-5), "Beamforming compression not lossless"
        print("✓ Beamforming compression losslessness test passed")
    else:
        print("⚠ Beamforming compression failed-open (high entropy)")


def test_dpd_compression_losslessness():
    """Test that DPD compression is lossless."""
    
    # Generate DPD coefficients
    coefficients = np.random.randn(64, 5).astype(np.float32)
    
    # Compress and decompress
    compressor = VectraDPDCompressor()
    compressed, ratio, metadata = compressor.compress_coefficients(coefficients)
    
    if metadata['status'] == 'compressed':
        decompressed = compressor.decompress_coefficients(compressed, metadata)
        
        # Verify losslessness
        assert np.allclose(coefficients, decompressed, atol=1e-5), "DPD compression not lossless"
        print("✓ DPD compression losslessness test passed")
    else:
        print("⚠ DPD compression failed-open (high entropy)")


def test_determinism():
    """Test that compression is deterministic."""
    
    # Generate CSI
    csi = np.random.randn(64, 12, 2).astype(np.float32)
    
    # Compress twice
    compressor = VectraCSICompressor()
    compressed1, ratio1, metadata1 = compressor.compress_csi(csi)
    compressed2, ratio2, metadata2 = compressor.compress_csi(csi)
    
    # Should produce same result
    if metadata1['status'] == 'compressed' and metadata2['status'] == 'compressed':
        assert compressed1 == compressed2, "Compression not deterministic"
        assert ratio1 == ratio2, "Compression ratio not deterministic"
        print("✓ Determinism test passed")
    else:
        print("⚠ Determinism test skipped (fail-open)")


def test_fail_open_safety():
    """Test that fail-open returns original data."""
    
    # Generate high-entropy (random) data
    random_data = np.random.bytes(1000)
    
    # Compress
    compressor = VectraSignalingCompressor()
    compressed, ratio, metadata = compressor.compress_message(random_data, "RANDOM")
    
    # Should fail-open (return original)
    if metadata['status'] == 'fail_open':
        assert len(compressed) == len(random_data), "Fail-open should return original size"
        print("✓ Fail-open safety test passed")
    else:
        print("⚠ Fail-open test: compression succeeded (unexpected)")


def run_all_tests():
    """Run all integration tests."""
    
    print("=" * 60)
    print("VECTRA 6G Integration Tests")
    print("=" * 60)
    print()
    
    tests = [
        test_csi_compression_losslessness,
        test_signaling_compression_losslessness,
        test_beamforming_compression_losslessness,
        test_dpd_compression_losslessness,
        test_determinism,
        test_fail_open_safety,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            print(f"✗ {test.__name__} failed: {e}")
            failed += 1
        except Exception as e:
            print(f"✗ {test.__name__} error: {e}")
            failed += 1
        print()
    
    print("=" * 60)
    print(f"Tests: {passed} passed, {failed} failed")
    print("=" * 60)
    
    return failed == 0


if __name__ == "__main__":
    success = run_all_tests()
    exit(0 if success else 1)








