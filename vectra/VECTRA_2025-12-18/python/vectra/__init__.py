"""VECTRA — Deterministic Lossless Data Volume Reduction

Core Invariants:
1. Determinism: Same input + same version → identical output
2. Losslessness: decode(encode(D)) == D always
3. Fail-open: Uncertainty → return original payload unchanged
4. Self-describing: Artifacts contain all reconstruction information
"""

from __future__ import annotations

import hashlib
import time
from typing import Optional

from .types import (
    VERSION_ID,
    H_MAX,
    Artifact,
    ByteRange,
    DecodeError,
    DecompositionError,
    EbtaError,
    EncodeResult,
    FeeError,
    Generator,
    IntegrityError,
    IntegrityMeta,
    Mapping,
    MappingSet,
    MappingTransform,
    NsgeError,
    Payload,
    PredictorParameters,
    PredictorState,
    ReconstructionConstraints,
    RepetitionSpec,
    Residual,
    ResidualSegment,
    SchemaId,
    SemanticType,
    Structure,
    StructureLevel,
    VariablePart,
    VariableSegment,
    VectraError,
)
from .ebta import (
    compute_byte_entropy,
    compute_residual,
    compute_residual_entropy,
    ebta_validate,
    apply_residual,
    EbtaResult,
)

__all__ = [
    "vectra_encode",
    "vectra_decode",
    "Artifact",
    "Payload",
    "EncodeResult",
    "VERSION_ID",
    "H_MAX",
    "VectraError",
    "DecodeError",
    "IntegrityError",
    "compute_byte_entropy",
    "sha256",
]


def sha256(data: bytes) -> bytes:
    """Compute SHA-256 hash."""
    return hashlib.sha256(data).digest()


# =============================================================================
# Decomposition
# =============================================================================

def decompose(payload: Payload) -> tuple[Structure, VariablePart]:
    """Decompose payload into structural and variable components."""
    if len(payload) == 0:
        return Structure(), VariablePart()

    data = payload.data
    patterns = _find_structural_patterns(data)

    if not patterns:
        return (
            Structure(levels=[StructureLevel(pattern_id=0)], byte_ranges=[]),
            VariablePart(segments=[
                VariableSegment(
                    range=ByteRange(start=0, end=len(data)),
                    data=data,
                    semantic_type=_infer_semantic_type(data),
                )
            ]),
        )

    structure, covered_ranges = _build_structure_from_patterns(data, patterns)
    variable = _extract_variable_regions(data, covered_ranges)
    return structure, variable


def _find_structural_patterns(data: bytes) -> list[tuple[bytes, list[int]]]:
    MIN_LEN, MIN_OCC = 4, 2
    if len(data) < MIN_LEN * MIN_OCC:
        return []

    patterns: list[tuple[bytes, list[int]]] = []
    for start in range(len(data) - MIN_LEN):
        for plen in range(MIN_LEN, min(65, len(data) - start)):
            pattern = data[start:start + plen]
            positions = _find_pattern_positions(data, pattern, start)
            if len(positions) >= MIN_OCC:
                patterns.append((pattern, positions))

    patterns.sort(key=lambda x: x[1][0] if x[1] else 0)
    return patterns[:10]  # Limit patterns


def _find_pattern_positions(data: bytes, pattern: bytes, start_from: int) -> list[int]:
    positions = [start_from]
    plen = len(pattern)
    pos = start_from + plen
    while pos + plen <= len(data):
        if data[pos:pos + plen] == pattern:
            positions.append(pos)
            pos += plen
        else:
            pos += 1
    return positions


def _build_structure_from_patterns(data: bytes, patterns: list[tuple[bytes, list[int]]]) -> tuple[Structure, list[ByteRange]]:
    levels = []
    covered: list[ByteRange] = []
    for idx, (pattern, positions) in enumerate(patterns):
        levels.append(StructureLevel(pattern_id=idx, literals=pattern))
        for pos in positions:
            covered.append(ByteRange(start=pos, end=pos + len(pattern)))
    covered.sort(key=lambda r: r.start)
    merged = _merge_ranges(covered)
    return Structure(levels=levels, byte_ranges=merged), merged


def _merge_ranges(ranges: list[ByteRange]) -> list[ByteRange]:
    if not ranges:
        return []
    merged = [ranges[0]]
    for r in ranges[1:]:
        last = merged[-1]
        if r.start <= last.end:
            merged[-1] = ByteRange(start=last.start, end=max(last.end, r.end))
        else:
            merged.append(r)
    return merged


def _extract_variable_regions(data: bytes, structural_ranges: list[ByteRange]) -> VariablePart:
    segments = []
    pos = 0
    for r in structural_ranges:
        if pos < r.start:
            vdata = data[pos:r.start]
            segments.append(VariableSegment(range=ByteRange(start=pos, end=r.start), data=vdata, semantic_type=_infer_semantic_type(vdata)))
        pos = r.end
    if pos < len(data):
        vdata = data[pos:]
        segments.append(VariableSegment(range=ByteRange(start=pos, end=len(data)), data=vdata, semantic_type=_infer_semantic_type(vdata)))
    return VariablePart(segments=segments)


def _infer_semantic_type(data: bytes) -> SemanticType:
    if not data:
        return SemanticType.OPAQUE
    if all(0x30 <= b <= 0x39 for b in data):
        return SemanticType.TIMESTAMP if 10 <= len(data) <= 13 else SemanticType.COUNTER
    if len(data) <= 8:
        return SemanticType.METRIC
    return SemanticType.OPAQUE


# =============================================================================
# FEE
# =============================================================================

def fee_encode(structure: Structure) -> tuple[Generator, MappingSet]:
    if not structure.levels:
        return Generator(base=b"", repetition=RepetitionSpec(count=0, stride=0)), MappingSet()
    base = structure.levels[0]
    stride = (structure.byte_ranges[1].start - structure.byte_ranges[0].start) if len(structure.byte_ranges) >= 2 else len(base.literals)
    return Generator(base=base.literals, repetition=RepetitionSpec(count=len(structure.byte_ranges), stride=stride)), MappingSet()


def regenerate_structure(generator: Generator, mappings: MappingSet) -> Structure:
    levels = [StructureLevel(pattern_id=0, literals=generator.base)]
    ranges = [ByteRange(start=i * generator.repetition.stride, end=i * generator.repetition.stride + len(generator.base)) for i in range(generator.repetition.count)]
    return Structure(levels=levels, byte_ranges=ranges)


# =============================================================================
# NSGE
# =============================================================================

def nsge_predict(variable: VariablePart) -> tuple[VariablePart, PredictorState]:
    state = PredictorState()
    predicted = [VariableSegment(range=s.range, data=bytes(len(s.data)), semantic_type=s.semantic_type) for s in variable.segments]
    return VariablePart(segments=predicted), state


def reconstruct_variable(predicted: VariablePart, residual_data: list[bytes]) -> VariablePart:
    segments = [VariableSegment(range=p.range, data=bytes(a ^ b for a, b in zip(p.data, r)), semantic_type=p.semantic_type) for p, r in zip(predicted.segments, residual_data)]
    return VariablePart(segments=segments)


# =============================================================================
# Artifact
# =============================================================================

def build_artifact(payload: Payload, generator: Generator, mappings: MappingSet, predictor_state: PredictorState, residual: Residual) -> Artifact:
    constraints = ReconstructionConstraints(output_length=len(payload), output_hash=sha256(payload.data))
    integrity = IntegrityMeta(
        payload_hash=sha256(payload.data),
        artifact_hash=_hash_components(generator, mappings, predictor_state, residual),
        version=VERSION_ID,
        encoded_at=int(time.time()),
    )
    return Artifact(generator=generator, mappings=mappings, predictor_state=predictor_state, residual=residual, constraints=constraints, integrity=integrity)


def _hash_components(g: Generator, m: MappingSet, p: PredictorState, r: Residual) -> bytes:
    h = hashlib.sha256()
    h.update(g.base)
    h.update(g.repetition.count.to_bytes(4, 'little'))
    h.update(g.repetition.stride.to_bytes(4, 'little'))
    h.update(len(m.mappings).to_bytes(8, 'little'))
    h.update(p.version.to_bytes(8, 'little'))
    for seg in r.segments:
        h.update(seg.range.start.to_bytes(8, 'little'))
        h.update(seg.range.end.to_bytes(8, 'little'))
        h.update(seg.delta)
    return h.digest()


# =============================================================================
# Recomposition
# =============================================================================

def recompose(structure: Structure, variable: VariablePart) -> Payload:
    struct_max = max((r.end for r in structure.byte_ranges), default=0)
    var_max = max((s.range.end for s in variable.segments), default=0)
    total = max(struct_max, var_max)
    if total == 0:
        return Payload(data=b"")
    output = bytearray(total)
    for br, lv in zip(structure.byte_ranges, structure.levels):
        output[br.start:br.end] = lv.literals[:br.end - br.start]
    for seg in variable.segments:
        output[seg.range.start:seg.range.end] = seg.data[:seg.range.end - seg.range.start]
    return Payload(data=bytes(output))


# =============================================================================
# Top-Level API
# =============================================================================

def vectra_encode(payload: Payload) -> EncodeResult:
    """Encode a payload. Returns artifact on success, pass_through on fail-open."""
    try:
        structure, variable = decompose(payload)
        generator, mappings = fee_encode(structure)
        predicted, predictor_state = nsge_predict(variable)
        
        if len(variable.segments) != len(predicted.segments):
            raise NsgeError("Segment mismatch")
        
        residual = Residual(segments=[compute_residual(a.data, p.data, a.range) for a, p in zip(variable.segments, predicted.segments)])
        
        ebta_result = ebta_validate(residual)
        if not ebta_result.valid:
            raise EbtaError(ebta_result.entropy, ebta_result.max_entropy)
        
        artifact = build_artifact(payload, generator, mappings, predictor_state, residual)
        return EncodeResult(artifact=artifact)
    except VectraError:
        return EncodeResult(pass_through=payload)


def vectra_decode(artifact: Artifact) -> Payload:
    """Decode an artifact back to original payload."""
    if artifact.integrity.version != VERSION_ID:
        raise IntegrityError(f"Version mismatch: {artifact.integrity.version:#x} != {VERSION_ID:#x}")
    
    computed = _hash_components(artifact.generator, artifact.mappings, artifact.predictor_state, artifact.residual)
    if computed != artifact.integrity.artifact_hash:
        raise IntegrityError("Artifact hash mismatch")
    
    structure = regenerate_structure(artifact.generator, artifact.mappings)
    predicted = VariablePart(segments=[VariableSegment(range=s.range, data=bytes(len(s.delta)), semantic_type=SemanticType.OPAQUE) for s in artifact.residual.segments])
    variable = reconstruct_variable(predicted, [s.delta for s in artifact.residual.segments])
    reconstructed = recompose(structure, variable)
    
    if len(reconstructed) != artifact.constraints.output_length:
        raise IntegrityError("Output length mismatch")
    if sha256(reconstructed.data) != artifact.constraints.output_hash:
        raise IntegrityError("Output hash mismatch")
    
    return reconstructed
