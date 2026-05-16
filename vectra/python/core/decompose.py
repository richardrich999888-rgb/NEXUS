"""
VECTRA Decomposition Module

Decomposes payload into structural and variable components while preserving
position information required for exact reconstruction.

Invented by Katta Naga Sri Ganesh
SYNTRIASS Labs Private Limited

Copyright © 2025 SYNTRIASS Labs Private Limited.
All rights reserved.
"""

from dataclasses import dataclass
from typing import List, Tuple


@dataclass(frozen=True)
class DecompositionResult:
    """
    Immutable result of payload decomposition.
    
    Contains all information required for exact reconstruction:
    - structural_segments: list of (position, content) tuples for stable components
    - variable_segments: list of (position, content) tuples for time-varying components
    - total_segments: total number of segments (for validation)
    
    Invariant: structural_segments ∪ variable_segments covers all positions [0, total_segments)
    """
    structural_segments: Tuple[Tuple[int, bytes], ...]
    variable_segments: Tuple[Tuple[int, bytes], ...]
    total_segments: int
    delimiter: bytes

    def validate(self) -> bool:
        """Verify decomposition covers all positions exactly once."""
        all_positions = set(p for p, _ in self.structural_segments)
        all_positions.update(p for p, _ in self.variable_segments)
        expected = set(range(self.total_segments))
        return all_positions == expected


def decompose(payload: bytes, delimiter: bytes = b"\n") -> DecompositionResult:
    """
    Decompose payload into structural and variable segments.
    
    Classification rule (MVP):
    - Structural: segments containing ':' (key-value pairs, stable schema)
    - Variable: segments without ':' (timestamps, counters, dynamic data)
    
    This rule is deterministic: same input always produces same classification.
    
    Args:
        payload: Raw input bytes
        delimiter: Segment delimiter (default: newline)
    
    Returns:
        DecompositionResult with position-tagged segments
    
    Determinism guarantee:
        decompose(D) == decompose(D) for all D
    """
    segments = payload.split(delimiter)
    
    structural: List[Tuple[int, bytes]] = []
    variable: List[Tuple[int, bytes]] = []
    
    for position, segment in enumerate(segments):
        if b":" in segment:
            structural.append((position, segment))
        else:
            variable.append((position, segment))
    
    return DecompositionResult(
        structural_segments=tuple(structural),
        variable_segments=tuple(variable),
        total_segments=len(segments),
        delimiter=delimiter
    )


def recompose(result: DecompositionResult) -> bytes:
    """
    Reconstruct original payload from decomposition result.
    
    This is the inverse of decompose().
    
    Invariant:
        recompose(decompose(D)) == D for all D
    
    Args:
        result: DecompositionResult from decompose()
    
    Returns:
        Original payload bytes
    """
    # Merge all segments with their positions
    all_segments: List[Tuple[int, bytes]] = []
    all_segments.extend(result.structural_segments)
    all_segments.extend(result.variable_segments)
    
    # Sort by position to restore original order
    all_segments.sort(key=lambda x: x[0])
    
    # Validate completeness
    if not result.validate():
        raise ValueError("Decomposition result is incomplete or corrupted")
    
    # Extract content in order and join
    ordered_content = [segment for _, segment in all_segments]
    return result.delimiter.join(ordered_content)
