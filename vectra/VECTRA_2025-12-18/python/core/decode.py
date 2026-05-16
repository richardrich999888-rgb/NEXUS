"""
VECTRA Decode Pipeline

Reconstructs original payload from artifact.

Implements spec §1: D : A → D
Guarantees exact reconstruction (losslessness) and integrity verification.

Invented by Katta Naga Sri Ganesh
SYNTRIASS Labs Private Limited

Copyright © 2025 SYNTRIASS Labs Private Limited.
All rights reserved.
"""

from typing import Dict, Any
import hashlib

from core.artifact import Artifact, verify_artifact
from core.fee import decode_structure, FEEResult
from core.decompose import recompose, DecompositionResult


def decode(artifact: Artifact) -> bytes:
    """
    VECTRA top-level decode function.
    
    Reconstructs original payload D from artifact A.
    
    Args:
        artifact: VECTRA Artifact
    
    Returns:
        Original payload bytes
    
    Raises:
        ValueError: If artifact integrity check fails or reconstruction mismatch
    
    Invariant:
        decode(encode(D)) == D
    """
    # 1. Verify integrity (tamper detection)
    if not verify_artifact(artifact):
        raise ValueError("Artifact integrity check failed: artifact has been tampered with")

    # 2. Reconstruct structure (FEE)
    # Convert hex components back to bytes
    generator_bytes = bytes.fromhex(artifact.generator)
    mappings = tuple(
        (pos, bytes.fromhex(suffix)) 
        for pos, suffix in artifact.structure_mappings
    )
    
    fee_result = FEEResult(
        generator=generator_bytes,
        mappings=mappings,
        integrity_hash=artifact.structure_hash
    )
    
    structural_segments = decode_structure(fee_result)

    # 3. Reconstruct variable segments
    # In MVP, this is just hex decoding. 
    # Full implementation would use NSGE/Residual application here.
    variable_segments = tuple(
        (pos, bytes.fromhex(content))
        for pos, content in artifact.variable_segments
    )

    # 4. Recompose
    delimiter_bytes = bytes.fromhex(artifact.delimiter)
    
    decomposition = DecompositionResult(
        structural_segments=structural_segments,
        variable_segments=variable_segments,
        total_segments=artifact.total_segments,
        delimiter=delimiter_bytes
    )

    payload = recompose(decomposition)

    # 5. Verify payload integrity
    payload_hash = hashlib.sha256(payload).hexdigest()
    if payload_hash != artifact.original_hash:
         raise ValueError(
             f"Reconstructed payload hash mismatch: "
             f"expected {artifact.original_hash}, got {payload_hash}"
         )

    return payload


def decode_with_diagnostics(artifact: Artifact) -> Dict[str, Any]:
    """
    Decode with detailed diagnostics.
    
    Returns dict with reconstruction details and verification status.
    """
    diagnostics = {
        "integrity_verified": False,
        "reconstruction_success": False,
        "payload_hash_match": False,
        "result": None,
        "error": None
    }
    
    try:
        if not verify_artifact(artifact):
            diagnostics["error"] = "Artifact integrity check failed"
            return diagnostics
        
        diagnostics["integrity_verified"] = True
        
        # Decode logic (duplicated for diagnostics visibility)
        generator_bytes = bytes.fromhex(artifact.generator)
        mappings = tuple(
            (pos, bytes.fromhex(suffix)) 
            for pos, suffix in artifact.structure_mappings
        )
        
        fee_result = FEEResult(
            generator=generator_bytes,
            mappings=mappings,
            integrity_hash=artifact.structure_hash
        )
        
        structural_segments = decode_structure(fee_result)
        
        variable_segments = tuple(
            (pos, bytes.fromhex(content))
            for pos, content in artifact.variable_segments
        )
        
        delimiter_bytes = bytes.fromhex(artifact.delimiter)
        
        decomposition = DecompositionResult(
            structural_segments=structural_segments,
            variable_segments=variable_segments,
            total_segments=artifact.total_segments,
            delimiter=delimiter_bytes
        )
        
        payload = recompose(decomposition)
        diagnostics["reconstruction_success"] = True
        diagnostics["result"] = payload
        
        payload_hash = hashlib.sha256(payload).hexdigest()
        if payload_hash == artifact.original_hash:
            diagnostics["payload_hash_match"] = True
        else:
            diagnostics["error"] = f"Hash mismatch: exp {artifact.original_hash}, got {payload_hash}"
            
    except Exception as e:
        diagnostics["error"] = str(e)
        
    return diagnostics
