"""Core type definitions for VECTRA.

These types map directly to the formal specification:
- Payload: D ∈ 𝒟
- Artifact: A ∈ 𝒜
- Structure: S (stable structural components)
- VariablePart: V (time-evolving components)

All types enforce determinism through immutability where practical.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Optional

# System version identifier. All artifacts are version-locked.
VERSION_ID: int = 0x0001_0000_0000_0001

# Maximum allowed Shannon entropy for residuals (H_MAX from spec §6).
H_MAX: float = 4.0


class SemanticType(Enum):
    """Semantic type hints for variable data."""
    COUNTER = auto()
    TIMESTAMP = auto()
    METRIC = auto()
    IDENTIFIER = auto()
    OPAQUE = auto()


@dataclass(frozen=True)
class ByteRange:
    """Byte range in original payload."""
    start: int
    end: int

    def __post_init__(self) -> None:
        if self.start < 0 or self.end < self.start:
            raise ValueError(f"Invalid byte range: [{self.start}, {self.end})")

    @property
    def length(self) -> int:
        return self.end - self.start


@dataclass(frozen=True)
class SchemaId:
    """Schema identifier for typed payload interpretation."""
    namespace: str
    name: str
    version: tuple[int, int, int]


@dataclass
class Payload:
    """Raw payload bytes. Represents D ∈ 𝒟."""
    data: bytes
    schema_id: Optional[SchemaId] = None

    def __len__(self) -> int:
        return len(self.data)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Payload):
            return NotImplemented
        return self.data == other.data and self.schema_id == other.schema_id

    def __hash__(self) -> int:
        return hash((self.data, self.schema_id))


@dataclass
class StructureLevel:
    """A single level in the structural hierarchy."""
    pattern_id: int
    children: list[int] = field(default_factory=list)
    literals: bytes = b""


@dataclass
class Structure:
    """Structural component extracted from payload (S from spec §3)."""
    levels: list[StructureLevel] = field(default_factory=list)
    byte_ranges: list[ByteRange] = field(default_factory=list)


@dataclass
class VariableSegment:
    """A segment of variable data."""
    range: ByteRange
    data: bytes
    semantic_type: SemanticType = SemanticType.OPAQUE


@dataclass
class VariablePart:
    """Variable component extracted from payload (V from spec §3)."""
    segments: list[VariableSegment] = field(default_factory=list)


@dataclass(frozen=True)
class RepetitionSpec:
    """Specification for how a pattern repeats."""
    count: int
    stride: int


@dataclass
class Generator:
    """Structural generator produced by FEE (G from spec §4)."""
    base: bytes
    repetition: RepetitionSpec


class MappingTransform(Enum):
    """Transformation applied by a mapping."""
    IDENTITY = auto()
    OFFSET = auto()
    CONCAT = auto()


@dataclass
class Mapping:
    """Recursive mapping function (φ from spec §4)."""
    from_level: int
    to_level: int
    transform: MappingTransform
    transform_param: int | list[int] = 0  # offset value or concat indices


@dataclass
class MappingSet:
    """Set of mappings Φ = {φ₀, φ₁, ..., φₖ} from spec §4."""
    mappings: list[Mapping] = field(default_factory=list)


@dataclass
class PredictorParameters:
    """Predictor model parameters."""
    counter_state: list[int] = field(default_factory=list)
    timestamp_base: int = 0
    timestamp_delta: int = 0
    metric_mean: int = 0  # Fixed-point, scale factor 1000
    metric_variance: int = 0


@dataclass
class PredictorState:
    """Predictor state (Θ from spec §5)."""
    version: int = VERSION_ID
    parameters: PredictorParameters = field(default_factory=PredictorParameters)


@dataclass
class ResidualSegment:
    """Residual for a single variable segment."""
    range: ByteRange
    delta: bytes


@dataclass
class Residual:
    """Residual Δ = V - V̂ from spec §5."""
    segments: list[ResidualSegment] = field(default_factory=list)


@dataclass
class IntegrityMeta:
    """Integrity metadata (I from spec §7)."""
    payload_hash: bytes  # 32 bytes (SHA-256)
    artifact_hash: bytes  # 32 bytes (SHA-256)
    version: int
    encoded_at: int  # Unix timestamp


@dataclass
class ReconstructionConstraints:
    """Reconstruction constraints (C from spec §7)."""
    output_length: int
    output_hash: bytes  # 32 bytes (SHA-256)


@dataclass
class Artifact:
    """Complete VECTRA artifact (A from spec §7)."""
    generator: Generator
    mappings: MappingSet
    predictor_state: PredictorState
    residual: Residual
    constraints: ReconstructionConstraints
    integrity: IntegrityMeta

    def to_bytes(self) -> bytes:
        """Serialize artifact to bytes (deterministic JSON)."""
        return json.dumps(self._to_dict(), sort_keys=True, separators=(",", ":")).encode()

    @classmethod
    def from_bytes(cls, data: bytes) -> "Artifact":
        """Deserialize artifact from bytes."""
        d = json.loads(data.decode())
        return cls._from_dict(d)

    def _to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "generator": {
                "base": self.generator.base.hex(),
                "repetition": {
                    "count": self.generator.repetition.count,
                    "stride": self.generator.repetition.stride,
                },
            },
            "mappings": [
                {
                    "from_level": m.from_level,
                    "to_level": m.to_level,
                    "transform": m.transform.name,
                    "transform_param": m.transform_param,
                }
                for m in self.mappings.mappings
            ],
            "predictor_state": {
                "version": self.predictor_state.version,
                "parameters": {
                    "counter_state": self.predictor_state.parameters.counter_state,
                    "timestamp_base": self.predictor_state.parameters.timestamp_base,
                    "timestamp_delta": self.predictor_state.parameters.timestamp_delta,
                    "metric_mean": self.predictor_state.parameters.metric_mean,
                },
            },
            "residual": [
                {
                    "range": {"start": s.range.start, "end": s.range.end},
                    "delta": s.delta.hex(),
                }
                for s in self.residual.segments
            ],
            "constraints": {
                "output_length": self.constraints.output_length,
                "output_hash": self.constraints.output_hash.hex(),
            },
            "integrity": {
                "payload_hash": self.integrity.payload_hash.hex(),
                "artifact_hash": self.integrity.artifact_hash.hex(),
                "version": self.integrity.version,
                "encoded_at": self.integrity.encoded_at,
            },
        }

    @classmethod
    def _from_dict(cls, d: dict) -> "Artifact":
        """Construct from dictionary."""
        return cls(
            generator=Generator(
                base=bytes.fromhex(d["generator"]["base"]),
                repetition=RepetitionSpec(
                    count=d["generator"]["repetition"]["count"],
                    stride=d["generator"]["repetition"]["stride"],
                ),
            ),
            mappings=MappingSet(
                mappings=[
                    Mapping(
                        from_level=m["from_level"],
                        to_level=m["to_level"],
                        transform=MappingTransform[m["transform"]],
                        transform_param=m["transform_param"],
                    )
                    for m in d["mappings"]
                ]
            ),
            predictor_state=PredictorState(
                version=d["predictor_state"]["version"],
                parameters=PredictorParameters(
                    counter_state=d["predictor_state"]["parameters"]["counter_state"],
                    timestamp_base=d["predictor_state"]["parameters"]["timestamp_base"],
                    timestamp_delta=d["predictor_state"]["parameters"]["timestamp_delta"],
                    metric_mean=d["predictor_state"]["parameters"]["metric_mean"],
                ),
            ),
            residual=Residual(
                segments=[
                    ResidualSegment(
                        range=ByteRange(start=s["range"]["start"], end=s["range"]["end"]),
                        delta=bytes.fromhex(s["delta"]),
                    )
                    for s in d["residual"]
                ]
            ),
            constraints=ReconstructionConstraints(
                output_length=d["constraints"]["output_length"],
                output_hash=bytes.fromhex(d["constraints"]["output_hash"]),
            ),
            integrity=IntegrityMeta(
                payload_hash=bytes.fromhex(d["integrity"]["payload_hash"]),
                artifact_hash=bytes.fromhex(d["integrity"]["artifact_hash"]),
                version=d["integrity"]["version"],
                encoded_at=d["integrity"]["encoded_at"],
            ),
        )


@dataclass
class EncodeResult:
    """Result of encoding: either an artifact or the original payload."""
    artifact: Optional[Artifact] = None
    pass_through: Optional[Payload] = None

    @property
    def is_encoded(self) -> bool:
        return self.artifact is not None

    @property
    def is_pass_through(self) -> bool:
        return self.pass_through is not None


class VectraError(Exception):
    """Base exception for VECTRA operations."""
    pass


class DecompositionError(VectraError):
    """Decomposition of payload failed."""
    pass


class FeeError(VectraError):
    """FEE encoding failed."""
    pass


class NsgeError(VectraError):
    """NSGE prediction failed."""
    pass


class EbtaError(VectraError):
    """EBTA validation failed."""
    def __init__(self, entropy: float, max_entropy: float):
        self.entropy = entropy
        self.max_entropy = max_entropy
        super().__init__(f"Entropy {entropy:.4f} exceeds maximum {max_entropy:.4f}")


class IntegrityError(VectraError):
    """Integrity verification failed."""
    pass


class DecodeError(VectraError):
    """Decoding failed."""
    pass
