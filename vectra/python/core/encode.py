"""
VECTRA Encode Pipeline

Top-level encoding function that orchestrates:
1. Decomposition (structural vs variable)
2. FEE encoding (structure → generator + mappings)
3. EBTA validation (entropy gate)
4. Artifact construction (if validation passes)

Fail-open behavior: returns original payload if encoding cannot be proven safe.

Invented by Katta Naga Sri Ganesh
SYNTRIASS Labs Private Limited

Copyright © 2025 SYNTRIASS Labs Private Limited.
All rights reserved.
"""

from typing import Union

from core.decompose import decompose, DecompositionResult
from core.fee import encode_structure, FEEResult
from core.ebta import validate, ValidationResult
from core.artifact import build, Artifact


def encode(payload: bytes) -> Union[Artifact, bytes]:
    """
    VECTRA top-level encode function.
    
    Implements the system law from spec §1:
        E : D → A ∪ D
    
    Returns either:
        - Artifact: if encoding succeeds and is provably correct
        - bytes: original payload if encoding cannot be proven safe (fail-open)
    
    Invariants enforced:
        1. Determinism: same input + same version → same output
        2. Fail-open: uncertainty → return original unchanged
    
    Args:
        payload: Raw input bytes to encode
    
    Returns:
        Artifact or original payload
    
    Determinism guarantee:
        encode(D) == encode(D) for all D
    """
    # Validate input
    if not isinstance(payload, bytes):
        raise TypeError(f"payload must be bytes, got {type(payload)}")
    
    # Empty payload: pass through
    if not payload:
        return payload
    
    # Step 1: Decompose into structural and variable components
    decomposition: DecompositionResult = decompose(payload)
    
    # Validate decomposition integrity
    if not decomposition.validate():
        # Decomposition failed validation → fail-open
        return payload
    
    # Step 2: FEE encode structural components
    fee_result: FEEResult = encode_structure(decomposition.structural_segments)
    
    # Step 3: EBTA validate variable components (entropy gate)
    validation: ValidationResult = validate(decomposition.variable_segments)
    
    if not validation.is_valid:
        # Entropy constraint violated → fail-open
        return payload
    
    # Step 4: Build artifact
    artifact = build(
        generator=fee_result.generator,
        structure_mappings=fee_result.mappings,
        structure_hash=fee_result.integrity_hash,
        variable_segments=decomposition.variable_segments,
        total_segments=decomposition.total_segments,
        delimiter=decomposition.delimiter,
        original_payload=payload
    )
    
    return artifact


def encode_with_diagnostics(payload: bytes) -> dict:
    """
    Encode with detailed diagnostics.
    
    Useful for debugging and understanding encoding decisions.
    
    Returns dict with:
        - result: Artifact or original payload
        - decomposition: DecompositionResult details
        - fee: FEEResult details
        - validation: ValidationResult details
        - encoded: True if artifact was produced
    """
    if not isinstance(payload, bytes):
        raise TypeError(f"payload must be bytes, got {type(payload)}")
    
    diagnostics = {
        "input_size": len(payload),
        "encoded": False,
        "fail_reason": None
    }
    
    if not payload:
        diagnostics["result"] = payload
        diagnostics["fail_reason"] = "empty_payload"
        return diagnostics
    
    # Decompose
    decomposition = decompose(payload)
    diagnostics["decomposition"] = {
        "structural_count": len(decomposition.structural_segments),
        "variable_count": len(decomposition.variable_segments),
        "total_segments": decomposition.total_segments,
        "valid": decomposition.validate()
    }
    
    if not decomposition.validate():
        diagnostics["result"] = payload
        diagnostics["fail_reason"] = "decomposition_invalid"
        return diagnostics
    
    # FEE
    fee_result = encode_structure(decomposition.structural_segments)
    diagnostics["fee"] = {
        "generator_size": len(fee_result.generator),
        "mappings_count": len(fee_result.mappings),
        "integrity_hash": fee_result.integrity_hash[:16] + "..."
    }
    
    # EBTA
    validation = validate(decomposition.variable_segments)
    diagnostics["validation"] = {
        "entropy": round(validation.entropy, 4),
        "threshold": validation.threshold,
        "residual_size": validation.residual_size,
        "is_valid": validation.is_valid
    }
    
    if not validation.is_valid:
        diagnostics["result"] = payload
        diagnostics["fail_reason"] = "entropy_exceeded"
        return diagnostics
    
    # Build artifact
    artifact = build(
        generator=fee_result.generator,
        structure_mappings=fee_result.mappings,
        structure_hash=fee_result.integrity_hash,
        variable_segments=decomposition.variable_segments,
        total_segments=decomposition.total_segments,
        delimiter=decomposition.delimiter,
        original_payload=payload
    )
    
    diagnostics["result"] = artifact
    diagnostics["encoded"] = True
    return diagnostics
