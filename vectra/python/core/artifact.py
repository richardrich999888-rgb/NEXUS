"""
VECTRA Artifact (TDF — Transformation Data Format)

Constructs self-describing, self-verifiable artifacts that contain
everything required for exact reconstruction without external context.

Artifact structure (from spec §7):
    A = { G, Φ, Θ, Δ, C, I }

MVP Implementation:
    - G (generator): from FEE
    - Φ (mappings): from FEE  
    - Θ (predictor_state): placeholder (NSGE not implemented)
    - Δ (residual): variable segments
    - C (constraints): reconstruction metadata
    - I (integrity): verification hashes

Invented by Katta Naga Sri Ganesh
SYNTRIASS Labs Private Limited

Copyright © 2025 SYNTRIASS Labs Private Limited.
All rights reserved.
"""

import hashlib
import json
from dataclasses import dataclass, asdict
from typing import Tuple, Optional, Dict, Any

from core import __version__


@dataclass(frozen=True)
class Artifact:
    """
    VECTRA encoded artifact.
    
    Self-describing: contains all information for reconstruction
    Self-verifiable: contains integrity hashes for validation
    Version-locked: includes encoder version for determinism
    
    This is the output of VECTRA encoding when encoding succeeds.
    """
    # Attribution (required)
    inventor: str
    organization: str
    version: str
    
    # Structural encoding (FEE)
    generator: str  # hex-encoded bytes
    structure_mappings: Tuple[Tuple[int, str], ...]  # (position, hex-encoded suffix)
    structure_hash: str
    
    # Variable encoding (residual)
    variable_segments: Tuple[Tuple[int, str], ...]  # (position, hex-encoded content)
    
    # Reconstruction constraints
    total_segments: int
    delimiter: str  # hex-encoded delimiter
    
    # Integrity metadata
    original_hash: str  # SHA-256 of original payload
    artifact_hash: str  # SHA-256 of artifact content (excluding this field)


def _compute_artifact_hash(data: Dict[str, Any]) -> str:
    """Compute deterministic hash of artifact content."""
    # Remove artifact_hash field if present (it's what we're computing)
    data_copy = {k: v for k, v in data.items() if k != "artifact_hash"}
    # Deterministic JSON serialization
    serialized = json.dumps(data_copy, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(serialized.encode()).hexdigest()


def build(
    generator: bytes,
    structure_mappings: Tuple[Tuple[int, bytes], ...],
    structure_hash: str,
    variable_segments: Tuple[Tuple[int, bytes], ...],
    total_segments: int,
    delimiter: bytes,
    original_payload: bytes
) -> Artifact:
    """
    Build a VECTRA artifact from encoding components.
    
    Args:
        generator: FEE generator (common prefix)
        structure_mappings: FEE mappings (position, suffix) pairs
        structure_hash: Integrity hash from FEE
        variable_segments: Position-tagged variable content
        total_segments: Total number of segments in original
        delimiter: Segment delimiter
        original_payload: Original input (for integrity hash)
    
    Returns:
        Complete, self-verifiable Artifact
    """
    # Convert bytes to hex for JSON-safe storage
    hex_generator = generator.hex()
    hex_structure_mappings = tuple(
        (pos, suffix.hex()) for pos, suffix in structure_mappings
    )
    hex_variable_segments = tuple(
        (pos, content.hex()) for pos, content in variable_segments
    )
    hex_delimiter = delimiter.hex()
    
    # Compute original payload hash
    original_hash = hashlib.sha256(original_payload).hexdigest()
    
    # Build artifact data (without artifact_hash yet)
    artifact_data = {
        "inventor": "Katta Naga Sri Ganesh",
        "organization": "SYNTRIASS Labs Private Limited",
        "version": __version__,
        "generator": hex_generator,
        "structure_mappings": hex_structure_mappings,
        "structure_hash": structure_hash,
        "variable_segments": hex_variable_segments,
        "total_segments": total_segments,
        "delimiter": hex_delimiter,
        "original_hash": original_hash,
    }
    
    # Compute artifact hash
    artifact_hash = _compute_artifact_hash(artifact_data)
    
    return Artifact(
        inventor=artifact_data["inventor"],
        organization=artifact_data["organization"],
        version=artifact_data["version"],
        generator=artifact_data["generator"],
        structure_mappings=hex_structure_mappings,
        structure_hash=artifact_data["structure_hash"],
        variable_segments=hex_variable_segments,
        total_segments=artifact_data["total_segments"],
        delimiter=artifact_data["delimiter"],
        original_hash=artifact_data["original_hash"],
        artifact_hash=artifact_hash
    )


def verify_artifact(artifact: Artifact) -> bool:
    """
    Verify artifact integrity.
    
    Checks that artifact has not been tampered with.
    
    Returns True if artifact is valid.
    """
    artifact_data = asdict(artifact)
    computed_hash = _compute_artifact_hash(artifact_data)
    return computed_hash == artifact.artifact_hash


def to_dict(artifact: Artifact) -> Dict[str, Any]:
    """Convert artifact to dictionary for serialization."""
    return asdict(artifact)


def from_dict(data: Dict[str, Any]) -> Artifact:
    """Reconstruct artifact from dictionary."""
    return Artifact(
        inventor=data["inventor"],
        organization=data["organization"],
        version=data["version"],
        generator=data["generator"],
        structure_mappings=tuple(tuple(m) for m in data["structure_mappings"]),
        structure_hash=data["structure_hash"],
        variable_segments=tuple(tuple(v) for v in data["variable_segments"]),
        total_segments=data["total_segments"],
        delimiter=data["delimiter"],
        original_hash=data["original_hash"],
        artifact_hash=data["artifact_hash"]
    )
