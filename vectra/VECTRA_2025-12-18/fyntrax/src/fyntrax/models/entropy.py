"""
FYNTRAX Entropy Models

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Entropy calculation and prediction for traffic patterns.
Used for state transition decisions in idle mode orchestration.
"""

import math
from typing import List, Dict


def shannon_entropy(probabilities: List[float]) -> float:
    """
    Compute Shannon entropy of a probability distribution.
    
    H(X) = -Σ p(x) log₂ p(x)
    
    Args:
        probabilities: List of probabilities (must sum to 1)
        
    Returns:
        Entropy in bits
    """
    return -sum(
        p * math.log2(p)
        for p in probabilities if p > 0
    )


def byte_entropy(data: bytes) -> float:
    """
    Compute empirical entropy of byte sequence.
    
    Args:
        data: Byte sequence
        
    Returns:
        Entropy in bits per byte
    """
    if not data:
        return 0.0
    
    counts: Dict[int, int] = {}
    for byte in data:
        counts[byte] = counts.get(byte, 0) + 1
    
    total = len(data)
    probabilities = [count / total for count in counts.values()]
    
    return shannon_entropy(probabilities)


def demand_entropy(request_rate: float, max_rate: float) -> float:
    """
    Compute information demand entropy.
    
    Maps request rate to entropy measure for idle mode decisions.
    
    Args:
        request_rate: Current request rate (requests/second)
        max_rate: Maximum expected rate
        
    Returns:
        Normalized entropy [0, 8]
    """
    if max_rate <= 0 or request_rate <= 0:
        return 0.0
    
    p = min(1.0, request_rate / max_rate)
    
    # Binary entropy scaled to byte range
    if p == 0 or p == 1:
        return 0.0
    
    h = -p * math.log2(p) - (1 - p) * math.log2(1 - p)
    return h * 8  # Scale to 0-8 bits


def protocol_entropy_overhead(useful_bits: int, total_bits: int) -> float:
    """
    Calculate protocol entropy overhead.
    
    ΔH_P = B_P - H_S
    
    Where:
        B_P = total bits exchanged
        H_S = useful information bits
    
    Args:
        useful_bits: Bits of actual information
        total_bits: Total bits exchanged
        
    Returns:
        Overhead in bits
    """
    return total_bits - useful_bits


def information_efficiency(useful_bits: int, total_bits: int) -> float:
    """
    Calculate information efficiency.
    
    η = H_S / B_P
    
    Perfect efficiency: η = 1
    Typical legacy: η ≈ 0.01 - 0.1
    FYNTRAX target: η > 0.5
    
    Returns:
        Efficiency [0, 1]
    """
    if total_bits <= 0:
        return 0.0
    return min(1.0, useful_bits / total_bits)
