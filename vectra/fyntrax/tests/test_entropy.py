"""Tests for FYNTRAX entropy models."""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from fyntrax.models.entropy import (
    shannon_entropy,
    byte_entropy,
    demand_entropy,
    protocol_entropy_overhead,
    information_efficiency,
)


def test_shannon_entropy_uniform():
    """Uniform distribution has maximum entropy."""
    probs = [0.25, 0.25, 0.25, 0.25]
    
    h = shannon_entropy(probs)
    
    assert abs(h - 2.0) < 1e-10  # log2(4) = 2


def test_shannon_entropy_certain():
    """Certain outcome has zero entropy."""
    probs = [1.0]
    
    h = shannon_entropy(probs)
    
    assert h == 0.0


def test_shannon_entropy_binary():
    """Binary 50/50 has 1 bit entropy."""
    probs = [0.5, 0.5]
    
    h = shannon_entropy(probs)
    
    assert abs(h - 1.0) < 1e-10


def test_byte_entropy_constant():
    """Constant data has zero entropy."""
    data = bytes([0x00] * 100)
    
    h = byte_entropy(data)
    
    assert h == 0.0


def test_byte_entropy_random():
    """Random data has high entropy."""
    import random
    data = bytes(random.randint(0, 255) for _ in range(1000))
    
    h = byte_entropy(data)
    
    assert h > 7.0  # Close to 8 bits


def test_protocol_overhead():
    """Protocol overhead calculation."""
    useful = 20  # bits
    total = 650  # bits (legacy handover)
    
    overhead = protocol_entropy_overhead(useful, total)
    
    assert overhead == 630


def test_information_efficiency():
    """Information efficiency calculation."""
    useful = 20
    total = 650
    
    eff = information_efficiency(useful, total)
    
    assert abs(eff - 20/650) < 1e-10
    assert eff < 0.05  # Very inefficient


def test_fyntrax_efficiency():
    """FYNTRAX should have high efficiency."""
    useful = 20
    total = 20  # No overhead in FYNTRAX
    
    eff = information_efficiency(useful, total)
    
    assert eff == 1.0  # 100% efficient


if __name__ == "__main__":
    test_shannon_entropy_uniform()
    test_shannon_entropy_certain()
    test_shannon_entropy_binary()
    test_byte_entropy_constant()
    test_byte_entropy_random()
    test_protocol_overhead()
    test_information_efficiency()
    test_fyntrax_efficiency()
    print("All entropy tests passed!")
