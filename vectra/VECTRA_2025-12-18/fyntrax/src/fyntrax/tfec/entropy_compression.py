"""
FYNTRAX Entropy Compression

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Entropy-based compression for RAN signaling.
Reduces signaling overhead using entropy analysis.
"""

import math
from typing import List, Dict


def calculate_compression_potential(data: bytes) -> float:
    """
    Estimate compression potential based on entropy.
    
    High entropy → low compression potential
    Low entropy → high compression potential
    
    Args:
        data: Input data
        
    Returns:
        Compression potential [0, 1]
    """
    if not data:
        return 0.0
    
    # Calculate byte entropy
    counts: Dict[int, int] = {}
    for byte in data:
        counts[byte] = counts.get(byte, 0) + 1
    
    total = len(data)
    entropy = -sum(
        (c / total) * math.log2(c / total)
        for c in counts.values()
    )
    
    # Max entropy is 8 bits/byte
    # Potential = 1 - (entropy / 8)
    return max(0.0, 1.0 - entropy / 8.0)


def protocol_entropy_estimate(message_type: str) -> float:
    """
    Estimate entropy of protocol message types.
    
    Used for deciding when compression is worthwhile.
    
    Returns:
        Estimated entropy in bits/byte
    """
    # Entropy estimates for common protocol messages
    estimates = {
        "rrc_setup": 2.5,      # Highly structured
        "rrc_reconfiguration": 3.0,
        "measurement_report": 4.5,
        "paging": 1.5,         # Very predictable
        "random_access": 5.5,   # High entropy
        "data": 7.5,           # User data, high entropy
    }
    
    return estimates.get(message_type, 5.0)


def should_compress(message_type: str, size_bytes: int) -> bool:
    """
    Decide if compression is beneficial.
    
    Args:
        message_type: Type of protocol message
        size_bytes: Message size
        
    Returns:
        True if compression is recommended
    """
    # Don't compress small messages (overhead > benefit)
    if size_bytes < 64:
        return False
    
    # Check entropy estimate
    entropy = protocol_entropy_estimate(message_type)
    
    # Compress if entropy < 5 bits/byte (reasonable potential)
    return entropy < 5.0
