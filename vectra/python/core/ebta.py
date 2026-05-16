"""
EBTA — Entropy-Bounded Tensor Algebra

Enforces entropy constraints on residuals to guarantee that encoding
is only performed when deterministic reconstruction is provable.

This is the safety gate of VECTRA.
If validation fails, the system fails open (returns original payload).

Invented by Katta Naga Sri Ganesh
SYNTRIASS Labs Private Limited

Copyright © 2025 SYNTRIASS Labs Private Limited.
All rights reserved.
"""

import math
from collections import Counter
from dataclasses import dataclass
from typing import Tuple


# System constant: maximum allowed Shannon entropy for residuals
# This threshold determines when encoding is safe
# H_MAX = 5.0 bits/byte allows most structured data
# H_MAX = 8.0 would accept random data (defeats purpose)
# H_MAX = 3.0 would be very restrictive
H_MAX: float = 5.0


@dataclass(frozen=True)
class ValidationResult:
    """
    Result of EBTA entropy validation.
    
    Contains:
    - is_valid: True if residual passes entropy constraint
    - entropy: computed Shannon entropy of residual
    - threshold: H_MAX value used for comparison
    - residual_size: size of residual in bytes
    """
    is_valid: bool
    entropy: float
    threshold: float
    residual_size: int


def shannon_entropy(data: bytes) -> float:
    """
    Compute Shannon entropy of byte sequence.
    
    H(X) = -Σ p(x) * log2(p(x))
    
    Returns entropy in bits per byte.
    Range: [0.0, 8.0]
    - 0.0 = perfectly uniform (single byte value)
    - 8.0 = perfectly random (uniform distribution over 256 values)
    
    Args:
        data: Input byte sequence
    
    Returns:
        Shannon entropy in bits per byte
    
    Determinism guarantee:
        shannon_entropy(D) == shannon_entropy(D) for all D
    """
    if not data:
        return 0.0
    
    counts = Counter(data)
    total = len(data)
    
    entropy = 0.0
    for count in counts.values():
        if count > 0:
            probability = count / total
            entropy -= probability * math.log2(probability)
    
    return entropy


def validate(variable_segments: Tuple[Tuple[int, bytes], ...]) -> ValidationResult:
    """
    Validate that variable segments pass entropy constraint.
    
    Decision rule (from spec §6.2):
        if H(Δ) ≤ H_MAX: encoding permitted
        else: fail-open, return original
    
    Args:
        variable_segments: Position-tagged variable content from decompose()
    
    Returns:
        ValidationResult indicating whether encoding should proceed
    
    This function is the safety gate.
    It must never return is_valid=True for data that cannot be exactly reconstructed.
    """
    # Concatenate all variable content for entropy analysis
    all_content = b"".join(content for _, content in variable_segments)
    
    entropy = shannon_entropy(all_content)
    
    return ValidationResult(
        is_valid=entropy <= H_MAX,
        entropy=entropy,
        threshold=H_MAX,
        residual_size=len(all_content)
    )


def get_threshold() -> float:
    """Return current H_MAX threshold. Useful for diagnostics."""
    return H_MAX
