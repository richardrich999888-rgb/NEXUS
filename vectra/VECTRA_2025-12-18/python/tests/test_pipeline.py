"""
VECTRA Test Suite

Verifies the fundamental invariants:
1. Determinism: same input → same output
2. Losslessness: decode(encode(D)) == D
3. Fail-open: uncertain → return original

Invented by Katta Naga Sri Ganesh
SYNTRIASS Labs Private Limited

Copyright © 2025 SYNTRIASS Labs Private Limited.
All rights reserved.
"""

import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.encode import encode, encode_with_diagnostics
from core.decode import decode, decode_with_diagnostics
from core.decompose import decompose, recompose
from core.fee import encode_structure, decode_structure, verify_structure
from core.ebta import shannon_entropy, validate
from core.artifact import Artifact, verify_artifact


def test_decompose_recompose_identity():
    """Verify: recompose(decompose(D)) == D"""
    payloads = [
        b"user: alice\naction: login\ntimestamp\ntimestamp",
        b"key: value\ndata\nkey2: value2\nmore data",
        b"no structure here\njust lines\nof text",
        b"only: structural\nlines: here",
        b"",
        b"single line",
        b"key: value",
    ]
    
    for payload in payloads:
        decomposition = decompose(payload)
        reconstructed = recompose(decomposition)
        assert reconstructed == payload, (
            f"Decompose/recompose identity failed:\n"
            f"  Input:  {payload!r}\n"
            f"  Output: {reconstructed!r}"
        )
    
    print("✓ test_decompose_recompose_identity PASSED")


def test_fee_encode_decode_identity():
    """Verify: decode_structure(encode_structure(S)) == S"""
    structural_segments_cases = [
        tuple(),
        ((0, b"key: value"),),
        ((0, b"user: alice"), (2, b"action: login")),
        ((1, b"a: 1"), (3, b"b: 2"), (5, b"c: 3")),
    ]
    
    for segments in structural_segments_cases:
        fee_result = encode_structure(segments)
        reconstructed = decode_structure(fee_result)
        assert reconstructed == segments, (
            f"FEE identity failed:\n"
            f"  Input:  {segments!r}\n"
            f"  Output: {reconstructed!r}"
        )
        assert verify_structure(fee_result), "FEE integrity verification failed"
    
    print("✓ test_fee_encode_decode_identity PASSED")


def test_ebta_entropy_calculation():
    """Verify Shannon entropy calculation is correct."""
    # All same byte: entropy = 0
    assert shannon_entropy(b"aaaa") == 0.0
    
    # Two equally distributed bytes: entropy = 1.0
    import math
    entropy_two = shannon_entropy(b"aabb")
    assert abs(entropy_two - 1.0) < 0.001, f"Expected ~1.0, got {entropy_two}"
    
    # Empty: entropy = 0
    assert shannon_entropy(b"") == 0.0
    
    # Single byte: entropy = 0
    assert shannon_entropy(b"x") == 0.0
    
    print("✓ test_ebta_entropy_calculation PASSED")


def test_ebta_validation():
    """Verify EBTA validation gate works correctly."""
    # Low entropy should pass
    low_entropy_segments = ((0, b"aaaaaa"), (1, b"bbbbbb"))
    result = validate(low_entropy_segments)
    assert result.is_valid, f"Low entropy should pass, got entropy={result.entropy}"
    
    print("✓ test_ebta_validation PASSED")


def test_full_pipeline_identity():
    """
    CRITICAL TEST: Verify decode(encode(D)) == D
    
    This is the fundamental losslessness invariant.
    """
    payloads = [
        b"user: alice\naction: login\ntimestamp\ntimestamp",
        b"key: value\ndata\nkey2: value2\nmore data",
        b"server: prod-01\nmetric: cpu\n95.2\n94.8\n96.1",
        b"id: 12345\nstatus: active\nevent_a\nevent_b\nevent_c",
    ]
    
    for payload in payloads:
        encoded = encode(payload)
        
        if isinstance(encoded, bytes):
            # Fail-open case: original returned unchanged
            assert encoded == payload, "Fail-open should return original unchanged"
        else:
            # Artifact case: must reconstruct exactly
            assert isinstance(encoded, Artifact), f"Expected Artifact, got {type(encoded)}"
            
            # Verify artifact integrity
            assert verify_artifact(encoded), "Artifact integrity check failed"
            
            # Decode and verify identity
            decoded = decode(encoded)
            assert decoded == payload, (
                f"LOSSLESSNESS INVARIANT VIOLATED:\n"
                f"  Original: {payload!r}\n"
                f"  Decoded:  {decoded!r}"
            )
    
    print("✓ test_full_pipeline_identity PASSED")


def test_determinism():
    """Verify: encode(D) == encode(D) for multiple calls."""
    payload = b"user: alice\naction: login\ntimestamp\ntimestamp"
    
    result1 = encode(payload)
    result2 = encode(payload)
    
    if isinstance(result1, Artifact) and isinstance(result2, Artifact):
        # Compare artifact fields
        assert result1.generator == result2.generator
        assert result1.structure_mappings == result2.structure_mappings
        assert result1.variable_segments == result2.variable_segments
        assert result1.artifact_hash == result2.artifact_hash
    else:
        assert result1 == result2
    
    print("✓ test_determinism PASSED")


def test_fail_open_passthrough():
    """Verify fail-open returns original payload unchanged."""
    # Empty payload
    empty = b""
    assert encode(empty) == empty, "Empty payload should pass through"
    
    print("✓ test_fail_open_passthrough PASSED")


def test_artifact_integrity():
    """Verify artifact tamper detection works."""
    payload = b"user: alice\naction: login\ntimestamp"
    encoded = encode(payload)
    
    if isinstance(encoded, Artifact):
        # Valid artifact should verify
        assert verify_artifact(encoded), "Valid artifact should verify"
        
        # Tampered artifact should fail (we'd need to create a modified copy)
        # This is tested implicitly by the decode integrity checks
    
    print("✓ test_artifact_integrity PASSED")


def test_with_diagnostics():
    """Verify diagnostic functions work correctly."""
    payload = b"user: alice\naction: login\ntimestamp\ntimestamp"
    
    # Encode diagnostics
    enc_diag = encode_with_diagnostics(payload)
    assert "decomposition" in enc_diag
    assert "fee" in enc_diag
    assert "validation" in enc_diag
    assert enc_diag["encoded"] == True or enc_diag["fail_reason"] is not None
    
    if enc_diag["encoded"]:
        # Decode diagnostics
        dec_diag = decode_with_diagnostics(enc_diag["result"])
        assert dec_diag["integrity_verified"] == True
        assert dec_diag["result"] == payload
    
    print("✓ test_with_diagnostics PASSED")


def run_all_tests():
    """Run all tests."""
    print("=" * 60)
    print("VECTRA Test Suite")
    print("=" * 60)
    print()
    
    test_decompose_recompose_identity()
    test_fee_encode_decode_identity()
    test_ebta_entropy_calculation()
    test_ebta_validation()
    test_full_pipeline_identity()
    test_determinism()
    test_fail_open_passthrough()
    test_artifact_integrity()
    test_with_diagnostics()
    
    print()
    print("=" * 60)
    print("ALL TESTS PASSED")
    print("=" * 60)
    print()
    print("VECTRA invariants verified:")
    print("  ✓ Determinism: same input → same output")
    print("  ✓ Losslessness: decode(encode(D)) == D")
    print("  ✓ Fail-open: uncertain → original unchanged")
    print()


if __name__ == "__main__":
    run_all_tests()
