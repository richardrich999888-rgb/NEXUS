"""EBTA — Entropy-Bounded Tensor Algebra

Implements spec §6: Entropy constraint enforcement.

EBTA is the safety gate. It validates:
- H(Δ) ≤ H_MAX (Shannon entropy bound)

If validation fails, encoding MUST NOT proceed.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Sequence

from .types import ByteRange, EbtaError, H_MAX, Residual, ResidualSegment


@dataclass
class EbtaResult:
    """Result of EBTA validation."""
    valid: bool
    entropy: float
    max_entropy: float


def ebta_validate(residual: Residual, h_max: float = H_MAX) -> EbtaResult:
    """Validate residual against entropy bounds.

    This is a HARD gate. No soft thresholds. No retries.

    Args:
        residual: The residual to validate
        h_max: Maximum allowed entropy (default: H_MAX)

    Returns:
        EbtaResult with validation outcome
    """
    entropy = compute_residual_entropy(residual)

    return EbtaResult(
        valid=entropy <= h_max,
        entropy=entropy,
        max_entropy=h_max,
    )


def compute_residual_entropy(residual: Residual) -> float:
    """Compute Shannon entropy of residual.

    Collects all residual bytes and computes entropy.
    """
    all_bytes = bytearray()
    for segment in residual.segments:
        all_bytes.extend(segment.delta)

    if not all_bytes:
        return 0.0

    return compute_byte_entropy(bytes(all_bytes))


def compute_byte_entropy(data: bytes) -> float:
    """Compute Shannon entropy of byte sequence.

    H(X) = -Σ p(x) log₂ p(x)

    Properties:
    - H = 0 for constant sequence (all bytes same)
    - H = 8 for uniform random bytes (maximum)
    - Higher H means less predictable

    Args:
        data: Byte sequence to analyze

    Returns:
        Entropy in bits (0.0 to 8.0)
    """
    if not data:
        return 0.0

    # Count byte frequencies
    counts = [0] * 256
    for b in data:
        counts[b] += 1

    total = len(data)
    entropy = 0.0

    for count in counts:
        if count > 0:
            p = count / total
            entropy -= p * math.log2(p)

    return entropy


def compute_residual(
    actual: bytes,
    predicted: bytes,
    byte_range: ByteRange,
) -> ResidualSegment:
    """Compute residual from actual and predicted values.

    Δ = V XOR V_hat

    XOR is used because:
    - It's reversible: V = V_hat XOR Δ
    - It's deterministic
    - It preserves byte boundaries

    Args:
        actual: Actual variable bytes
        predicted: Predicted variable bytes
        byte_range: Range in original payload

    Returns:
        ResidualSegment with XOR delta

    Raises:
        EbtaError: If lengths don't match
    """
    if len(actual) != len(predicted):
        raise EbtaError(float("inf"), H_MAX)

    delta = bytes(a ^ p for a, p in zip(actual, predicted))

    return ResidualSegment(range=byte_range, delta=delta)


def apply_residual(predicted: bytes, delta: bytes) -> bytes:
    """Reconstruct actual value from prediction and residual.

    V = V_hat XOR Δ

    Args:
        predicted: Predicted bytes
        delta: Residual bytes

    Returns:
        Reconstructed actual bytes
    """
    return bytes(p ^ d for p, d in zip(predicted, delta))


def is_highly_compressible(residual: Residual) -> bool:
    """Check if residual is highly compressible (low entropy)."""
    entropy = compute_residual_entropy(residual)
    return entropy <= H_MAX / 2.0


def compression_potential(residual: Residual) -> float:
    """Compute compression potential from entropy.

    Returns estimated bits per byte after ideal compression.
    Lower is better.
    """
    return compute_residual_entropy(residual)
