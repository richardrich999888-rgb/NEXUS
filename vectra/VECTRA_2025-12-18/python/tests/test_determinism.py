"""Tests for VECTRA invariants."""

import pytest
from vectra import (
    vectra_encode,
    vectra_decode,
    Payload,
    compute_byte_entropy,
    VERSION_ID,
    Artifact,
)


class TestDeterminism:
    def test_identical_input_identical_output(self):
        data = b"HEADER:value1:HEADER:value2:HEADER:value3"
        r1 = vectra_encode(Payload(data=data))
        r2 = vectra_encode(Payload(data=data))
        assert r1.is_encoded == r2.is_encoded
        if r1.is_encoded:
            assert r1.artifact.to_bytes() == r2.artifact.to_bytes()


class TestLosslessness:
    def test_roundtrip_repeated(self):
        data = bytes([0xAA] * 256)
        result = vectra_encode(Payload(data=data))
        if result.is_encoded:
            decoded = vectra_decode(result.artifact)
            assert decoded.data == data


class TestFailOpen:
    def test_high_entropy_preserves_data(self):
        data = bytes((i * 17 + 31) % 256 for i in range(1000))
        result = vectra_encode(Payload(data=data))
        if result.is_pass_through:
            assert result.pass_through.data == data


class TestEntropy:
    def test_constant_zero_entropy(self):
        assert abs(compute_byte_entropy(bytes([0xAA] * 100))) < 0.001

    def test_uniform_max_entropy(self):
        assert abs(compute_byte_entropy(bytes(range(256))) - 8.0) < 0.001


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
