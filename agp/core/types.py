"""
AGP Core Domain Types

All domain types are immutable value objects with validation.
No external dependencies allowed in this module.

Patent Note: These types support the novel algorithms in task_clustering,
reputation, governance, and verification modules.
"""

from __future__ import annotations

import hashlib
import struct
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Tuple, FrozenSet, Optional
import math


# =============================================================================
# CONSTANTS
# =============================================================================

REPUTATION_MIN: float = 0.0
REPUTATION_MAX: float = 1.0
REPUTATION_DEFAULT: float = 0.5
REPUTATION_FLOOR_VERIFIED_FORK: float = 0.1

AGENT_ID_BYTES: int = 32
TASK_TYPE_LEVELS: int = 3
EMBEDDING_DIMENSIONS: int = 64

# Decay half-lives in seconds (task-type specific defaults)
DEFAULT_DECAY_HALF_LIFE: int = 86400 * 30  # 30 days
HIGH_SENSITIVITY_DECAY_HALF_LIFE: int = 86400 * 7  # 7 days
LOW_SENSITIVITY_DECAY_HALF_LIFE: int = 86400 * 90  # 90 days


# =============================================================================
# ENUMS
# =============================================================================

class VerificationTier(Enum):
    """
    Verification tiers ordered by trust level (ascending).
    
    OPTIMISTIC: No upfront proof, relies on dispute mechanism
    TEE: Hardware-based attestation (Intel SGX, AMD SEV, NVIDIA H100)
    ZKML: Zero-knowledge proof of correct execution
    """
    OPTIMISTIC = auto()
    TEE = auto()
    ZKML = auto()


class AgentState(Enum):
    """Agent lifecycle states."""
    UNREGISTERED = auto()
    REGISTERED = auto()
    ACTIVE = auto()
    SLASHED = auto()
    INACTIVE = auto()


class TaskState(Enum):
    """Task lifecycle states."""
    SUBMITTED = auto()
    ASSIGNED = auto()
    EXECUTING = auto()
    VERIFYING = auto()
    COMPLETED = auto()
    DISPUTED = auto()
    RESOLVED = auto()


class ProposalState(Enum):
    """Governance proposal lifecycle states."""
    PROPOSED = auto()
    VOTING = auto()
    PASSED = auto()
    REJECTED = auto()
    EXECUTED = auto()
    EXPIRED = auto()


# =============================================================================
# VALUE OBJECTS
# =============================================================================

@dataclass(frozen=True)
class AgentID:
    """
    32-byte agent identifier derived from public key.
    
    Immutable and hashable for use as dict keys.
    """
    value: bytes
    
    def __post_init__(self) -> None:
        if len(self.value) != AGENT_ID_BYTES:
            raise ValueError(f"AgentID must be {AGENT_ID_BYTES} bytes, got {len(self.value)}")
    
    @classmethod
    def from_public_key(cls, public_key: bytes) -> AgentID:
        """Derive AgentID from public key via SHA-256."""
        digest = hashlib.sha256(public_key).digest()
        return cls(value=digest)
    
    @classmethod
    def from_hex(cls, hex_string: str) -> AgentID:
        """Create AgentID from hex string."""
        return cls(value=bytes.fromhex(hex_string))
    
    def to_hex(self) -> str:
        """Return hex representation."""
        return self.value.hex()
    
    def __str__(self) -> str:
        return f"Agent({self.to_hex()[:16]}...)"


@dataclass(frozen=True)
class Version:
    """
    Semantic version (major.minor.patch).
    
    Used for agent versioning and fork detection.
    """
    major: int
    minor: int
    patch: int
    
    def __post_init__(self) -> None:
        if self.major < 0 or self.minor < 0 or self.patch < 0:
            raise ValueError("Version components must be non-negative")
    
    def __str__(self) -> str:
        return f"{self.major}.{self.minor}.{self.patch}"
    
    def __lt__(self, other: Version) -> bool:
        return (self.major, self.minor, self.patch) < (other.major, other.minor, other.patch)
    
    def __le__(self, other: Version) -> bool:
        return (self.major, self.minor, self.patch) <= (other.major, other.minor, other.patch)
    
    def is_major_upgrade(self, other: Version) -> bool:
        """Check if this is a major upgrade from other."""
        return self.major > other.major
    
    def is_minor_upgrade(self, other: Version) -> bool:
        """Check if this is a minor upgrade from other."""
        return self.major == other.major and self.minor > other.minor


@dataclass(frozen=True)
class AgentFingerprint:
    """
    Unique identifier for a specific agent version.
    
    Hash of (AgentID || Version || ModelCommitment).
    Used as key for reputation records.
    """
    value: bytes
    
    def __post_init__(self) -> None:
        if len(self.value) != 32:
            raise ValueError("AgentFingerprint must be 32 bytes")
    
    @classmethod
    def compute(
        cls,
        agent_id: AgentID,
        version: Version,
        model_commitment: bytes
    ) -> AgentFingerprint:
        """
        Compute fingerprint from components.
        
        Deterministic: same inputs always produce same fingerprint.
        """
        version_bytes = struct.pack(">HHH", version.major, version.minor, version.patch)
        combined = agent_id.value + version_bytes + model_commitment
        digest = hashlib.sha256(combined).digest()
        return cls(value=digest)
    
    def to_hex(self) -> str:
        return self.value.hex()
    
    def __str__(self) -> str:
        return f"Fingerprint({self.to_hex()[:16]}...)"


@dataclass(frozen=True)
class TaskType:
    """
    Hierarchical task classification.
    
    L0: Domain (e.g., "inference", "training", "data")
    L1: Category (e.g., "nlp", "vision", "tabular")  
    L2: Specific (e.g., "sentiment", "classification", "ner")
    
    Patent Note: This taxonomy enables the novel task-type clustering
    algorithm for validator selection.
    """
    l0_domain: str
    l1_category: str
    l2_specific: str
    
    def __post_init__(self) -> None:
        # Normalize to lowercase
        object.__setattr__(self, 'l0_domain', self.l0_domain.lower().strip())
        object.__setattr__(self, 'l1_category', self.l1_category.lower().strip())
        object.__setattr__(self, 'l2_specific', self.l2_specific.lower().strip())
        
        if not self.l0_domain or not self.l1_category or not self.l2_specific:
            raise ValueError("All task type levels must be non-empty")
    
    def to_tuple(self) -> Tuple[str, str, str]:
        return (self.l0_domain, self.l1_category, self.l2_specific)
    
    def matches_domain(self, other: TaskType) -> bool:
        """Check if domains match (L0 level)."""
        return self.l0_domain == other.l0_domain
    
    def matches_category(self, other: TaskType) -> bool:
        """Check if domains and categories match (L0 + L1)."""
        return self.matches_domain(other) and self.l1_category == other.l1_category
    
    def matches_specific(self, other: TaskType) -> bool:
        """Check if fully matches (all levels)."""
        return self.to_tuple() == other.to_tuple()
    
    def similarity(self, other: TaskType) -> float:
        """
        Compute hierarchical similarity score.
        
        Returns:
            1.0 if fully matching
            0.7 if category matches (L0 + L1)
            0.3 if domain matches (L0 only)
            0.0 if no match
        """
        if self.matches_specific(other):
            return 1.0
        elif self.matches_category(other):
            return 0.7
        elif self.matches_domain(other):
            return 0.3
        else:
            return 0.0
    
    def __str__(self) -> str:
        return f"{self.l0_domain}/{self.l1_category}/{self.l2_specific}"


@dataclass(frozen=True)
class TaskTypeVector:
    """
    Embedding vector for task type.
    
    Used for computing similarity in validator selection.
    Fixed dimensionality for consistent comparisons.
    """
    values: Tuple[float, ...]
    
    def __post_init__(self) -> None:
        if len(self.values) != EMBEDDING_DIMENSIONS:
            raise ValueError(f"TaskTypeVector must have {EMBEDDING_DIMENSIONS} dimensions")
        # Verify all values are finite
        for v in self.values:
            if not math.isfinite(v):
                raise ValueError("TaskTypeVector values must be finite")
    
    def cosine_similarity(self, other: TaskTypeVector) -> float:
        """
        Compute cosine similarity with another vector.
        
        Returns value in [-1, 1], where 1 = identical direction.
        """
        dot = sum(a * b for a, b in zip(self.values, other.values))
        norm_self = math.sqrt(sum(v * v for v in self.values))
        norm_other = math.sqrt(sum(v * v for v in other.values))
        
        if norm_self == 0 or norm_other == 0:
            return 0.0
        
        return dot / (norm_self * norm_other)
    
    def magnitude(self) -> float:
        """Compute L2 norm."""
        return math.sqrt(sum(v * v for v in self.values))
    
    @classmethod
    def zero(cls) -> TaskTypeVector:
        """Create zero vector."""
        return cls(values=tuple([0.0] * EMBEDDING_DIMENSIONS))


@dataclass(frozen=True)
class ReputationScore:
    """
    Bounded reputation score in [0.0, 1.0].
    
    Immutable with automatic clamping to valid range.
    """
    value: float
    
    def __post_init__(self) -> None:
        clamped = max(REPUTATION_MIN, min(REPUTATION_MAX, self.value))
        object.__setattr__(self, 'value', clamped)
    
    @classmethod
    def default(cls) -> ReputationScore:
        """Create default reputation (0.5)."""
        return cls(value=REPUTATION_DEFAULT)
    
    @classmethod
    def minimum(cls) -> ReputationScore:
        """Create minimum reputation (0.0)."""
        return cls(value=REPUTATION_MIN)
    
    @classmethod
    def maximum(cls) -> ReputationScore:
        """Create maximum reputation (1.0)."""
        return cls(value=REPUTATION_MAX)
    
    def decay(self, factor: float) -> ReputationScore:
        """
        Apply decay factor and return new score.
        
        Factor should be in (0, 1] where 1 = no decay.
        """
        if factor <= 0 or factor > 1:
            raise ValueError(f"Decay factor must be in (0, 1], got {factor}")
        return ReputationScore(value=self.value * factor)
    
    def __float__(self) -> float:
        return self.value
    
    def __str__(self) -> str:
        return f"{self.value:.4f}"


@dataclass(frozen=True)
class Stake:
    """
    Stake amount in base units (wei-equivalent).
    
    Non-negative integer to avoid floating point issues.
    """
    value: int
    
    def __post_init__(self) -> None:
        if self.value < 0:
            raise ValueError("Stake must be non-negative")
    
    def __add__(self, other: Stake) -> Stake:
        return Stake(value=self.value + other.value)
    
    def __sub__(self, other: Stake) -> Stake:
        if self.value < other.value:
            raise ValueError("Cannot subtract to negative stake")
        return Stake(value=self.value - other.value)
    
    def __mul__(self, factor: float) -> Stake:
        return Stake(value=int(self.value * factor))
    
    def __lt__(self, other: Stake) -> bool:
        return self.value < other.value
    
    def __le__(self, other: Stake) -> bool:
        return self.value <= other.value
    
    def ratio(self, total: Stake) -> float:
        """Compute ratio of this stake to total."""
        if total.value == 0:
            return 0.0
        return self.value / total.value


@dataclass(frozen=True)
class Timestamp:
    """
    Unix timestamp in milliseconds.
    
    Millisecond precision for ordering within same second.
    """
    value: int
    
    def __post_init__(self) -> None:
        if self.value < 0:
            raise ValueError("Timestamp must be non-negative")
    
    def seconds_since(self, other: Timestamp) -> float:
        """Compute seconds elapsed since other timestamp."""
        return (self.value - other.value) / 1000.0
    
    def __lt__(self, other: Timestamp) -> bool:
        return self.value < other.value
    
    def __le__(self, other: Timestamp) -> bool:
        return self.value <= other.value


@dataclass(frozen=True)
class Duration:
    """
    Duration in milliseconds.
    
    Used for timeouts, decay periods, etc.
    """
    value: int
    
    def __post_init__(self) -> None:
        if self.value < 0:
            raise ValueError("Duration must be non-negative")
    
    @classmethod
    def from_seconds(cls, seconds: float) -> Duration:
        return cls(value=int(seconds * 1000))
    
    @classmethod
    def from_days(cls, days: float) -> Duration:
        return cls(value=int(days * 86400 * 1000))
    
    def to_seconds(self) -> float:
        return self.value / 1000.0


# =============================================================================
# ENTITY MODELS
# =============================================================================

@dataclass(frozen=True)
class ReputationRecord:
    """
    Reputation record for a specific agent-task combination.
    
    Patent Note: The task-type specific decay_half_life enables
    different trust decay rates for different task categories.
    """
    agent_fingerprint: AgentFingerprint
    task_type: TaskType
    score: ReputationScore
    successful_executions: int
    failed_executions: int
    last_updated: Timestamp
    decay_half_life: Duration
    
    def __post_init__(self) -> None:
        if self.successful_executions < 0 or self.failed_executions < 0:
            raise ValueError("Execution counts must be non-negative")
    
    def total_executions(self) -> int:
        return self.successful_executions + self.failed_executions
    
    def success_rate(self) -> float:
        """Compute success rate, or 0.5 if no executions."""
        total = self.total_executions()
        if total == 0:
            return 0.5
        return self.successful_executions / total
    
    def with_success(self, timestamp: Timestamp) -> ReputationRecord:
        """Return new record with one additional success."""
        # Bayesian update: move score toward 1.0
        new_score = ReputationScore(
            value=self.score.value + (1.0 - self.score.value) * 0.1
        )
        return ReputationRecord(
            agent_fingerprint=self.agent_fingerprint,
            task_type=self.task_type,
            score=new_score,
            successful_executions=self.successful_executions + 1,
            failed_executions=self.failed_executions,
            last_updated=timestamp,
            decay_half_life=self.decay_half_life,
        )
    
    def with_failure(self, timestamp: Timestamp) -> ReputationRecord:
        """Return new record with one additional failure."""
        # Bayesian update: move score toward 0.0
        new_score = ReputationScore(
            value=self.score.value * 0.8
        )
        return ReputationRecord(
            agent_fingerprint=self.agent_fingerprint,
            task_type=self.task_type,
            score=new_score,
            successful_executions=self.successful_executions,
            failed_executions=self.failed_executions + 1,
            last_updated=timestamp,
            decay_half_life=self.decay_half_life,
        )
    
    def decayed_score(self, current_time: Timestamp) -> ReputationScore:
        """
        Compute reputation score with time decay applied.
        
        Uses exponential decay with task-type specific half-life.
        """
        elapsed_seconds = current_time.seconds_since(self.last_updated)
        if elapsed_seconds <= 0:
            return self.score
        
        half_life_seconds = self.decay_half_life.to_seconds()
        decay_factor = math.pow(0.5, elapsed_seconds / half_life_seconds)
        
        return self.score.decay(decay_factor)


@dataclass(frozen=True)
class ValidatorInfo:
    """
    Information about a validator for selection purposes.
    
    Patent Note: The expertise_vector and operator_id enable
    the novel task-type clustering validator selection.
    """
    agent_fingerprint: AgentFingerprint
    stake: Stake
    reputation_records: FrozenSet[ReputationRecord]
    operator_id: bytes  # For anti-collusion (same operator = same bytes)
    is_active: bool
    
    def get_reputation_for_task(self, task_type: TaskType) -> Optional[ReputationRecord]:
        """Find reputation record for exact task type match."""
        for record in self.reputation_records:
            if record.task_type.matches_specific(task_type):
                return record
        return None
    
    def get_similar_reputations(self, task_type: TaskType) -> list:
        """Find reputation records with any similarity to task type."""
        similar = []
        for record in self.reputation_records:
            similarity = record.task_type.similarity(task_type)
            if similarity > 0:
                similar.append((record, similarity))
        return sorted(similar, key=lambda x: x[1], reverse=True)


@dataclass(frozen=True)
class Task:
    """
    Task submitted for execution.
    
    Contains all information needed for validator selection
    and verification tier selection.
    """
    task_id: bytes
    task_type: TaskType
    submitter: bytes
    stake_at_risk: Stake
    input_commitment: bytes  # Hash of input data
    created_at: Timestamp
    deadline: Timestamp
    sensitivity: float  # 0.0 to 1.0, affects verification tier
    
    def __post_init__(self) -> None:
        if len(self.task_id) != 32:
            raise ValueError("Task ID must be 32 bytes")
        if len(self.input_commitment) != 32:
            raise ValueError("Input commitment must be 32 bytes")
        if not 0.0 <= self.sensitivity <= 1.0:
            raise ValueError("Sensitivity must be in [0.0, 1.0]")


@dataclass(frozen=True)
class Vote:
    """
    Governance vote with execution-weighted power.
    
    Patent Note: The execution_weight field enables the novel
    execution-weighted governance algorithm.
    """
    voter: AgentFingerprint
    proposal_id: bytes
    in_favor: bool
    token_balance: Stake
    execution_weight: float  # Computed from execution history
    timestamp: Timestamp
    signature: bytes
    
    def voting_power(self) -> float:
        """
        Compute total voting power.
        
        Combines token balance (30%) with execution weight (70%).
        Token balance uses square root to reduce plutocracy.
        """
        token_score = math.sqrt(self.token_balance.value)
        return self.execution_weight * 0.7 + token_score * 0.3


@dataclass(frozen=True)
class ForkProof:
    """
    Cryptographic proof linking two agent versions.
    
    Patent Note: This structure enables the novel fork
    reputation inheritance algorithm.
    """
    old_fingerprint: AgentFingerprint
    new_fingerprint: AgentFingerprint
    old_version: Version
    new_version: Version
    model_diff_hash: bytes  # Hash of model differences
    code_diff_hash: bytes   # Hash of code differences
    signature: bytes        # Signature by agent owner
    
    def __post_init__(self) -> None:
        if self.new_version <= self.old_version:
            raise ValueError("New version must be greater than old version")


# =============================================================================
# RESULT TYPES
# =============================================================================

@dataclass(frozen=True)
class ValidatorSelection:
    """Result of validator selection algorithm."""
    validators: Tuple[ValidatorInfo, ...]
    task: Task
    selection_scores: Tuple[float, ...]  # Parallel to validators
    timestamp: Timestamp


@dataclass(frozen=True)
class VerificationDecision:
    """Result of verification tier selection."""
    tier: VerificationTier
    risk_score: float
    cost_estimate: Stake
    reasoning: str  # Human-readable explanation


@dataclass(frozen=True)
class InheritanceResult:
    """Result of fork reputation inheritance calculation."""
    inherited_score: ReputationScore
    inheritance_factor: float
    change_magnitude: float
    reasoning: str
