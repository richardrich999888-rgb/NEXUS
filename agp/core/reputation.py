"""
AGP Reputation System with Fork Inheritance

PATENT CLAIM: Novel algorithm for inheriting reputation when an agent
is updated to a new version (forked).

Prior Art Gap: No existing system handles agent versioning with
reputation inheritance. Agents either start fresh or keep full reputation
regardless of changes.

Algorithm:
1. Verify cryptographic fork proof linking versions
2. Compute change magnitude from model/code diffs
3. Apply inheritance factor based on change magnitude
4. Enforce minimum floor for verified forks

This module has ZERO external dependencies.
"""

from __future__ import annotations

import math
import hashlib
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass

from .types import (
    AgentFingerprint,
    AgentID,
    Version,
    TaskType,
    ReputationScore,
    ReputationRecord,
    Timestamp,
    Duration,
    Stake,
    ForkProof,
    InheritanceResult,
    REPUTATION_DEFAULT,
    REPUTATION_FLOOR_VERIFIED_FORK,
    DEFAULT_DECAY_HALF_LIFE,
    HIGH_SENSITIVITY_DECAY_HALF_LIFE,
    LOW_SENSITIVITY_DECAY_HALF_LIFE,
)


# =============================================================================
# CONSTANTS
# =============================================================================

# Lambda values for inheritance decay (per unit change magnitude)
MAJOR_VERSION_LAMBDA: float = 2.0  # Aggressive decay for major versions
MINOR_VERSION_LAMBDA: float = 0.5  # Moderate decay for minor versions
PATCH_VERSION_LAMBDA: float = 0.1  # Minimal decay for patches

# Weight factors for change magnitude calculation
MODEL_DIFF_WEIGHT: float = 0.7
CODE_DIFF_WEIGHT: float = 0.3

# Maximum reputation boost per successful execution
MAX_BOOST_PER_SUCCESS: float = 0.1

# Maximum reputation penalty per failure
MAX_PENALTY_PER_FAILURE: float = 0.2

# Minimum executions before reputation is considered reliable
MIN_EXECUTIONS_FOR_RELIABILITY: int = 10


# =============================================================================
# DECAY HALF-LIFE SELECTION
# =============================================================================

# Task domain sensitivity mapping
DOMAIN_SENSITIVITY: Dict[str, str] = {
    "inference": "medium",
    "training": "high",
    "data": "medium",
    "financial": "high",
    "medical": "high",
    "creative": "low",
    "utility": "low",
}


def get_decay_half_life(task_type: TaskType) -> Duration:
    """
    Determine decay half-life based on task type sensitivity.
    
    High-sensitivity tasks (financial, medical, training) have faster
    decay to require more recent proof of competence.
    
    Low-sensitivity tasks (creative, utility) have slower decay to
    allow longer periods between activity.
    
    Args:
        task_type: Task type to determine sensitivity for
        
    Returns:
        Appropriate decay half-life duration
    """
    sensitivity = DOMAIN_SENSITIVITY.get(task_type.l0_domain, "medium")
    
    if sensitivity == "high":
        return Duration(value=HIGH_SENSITIVITY_DECAY_HALF_LIFE * 1000)
    elif sensitivity == "low":
        return Duration(value=LOW_SENSITIVITY_DECAY_HALF_LIFE * 1000)
    else:
        return Duration(value=DEFAULT_DECAY_HALF_LIFE * 1000)


# =============================================================================
# CHANGE MAGNITUDE COMPUTATION
# =============================================================================

def compute_hash_distance(hash1: bytes, hash2: bytes) -> float:
    """
    Compute normalized Hamming distance between two hashes.
    
    Returns value in [0, 1] where:
    - 0 = identical hashes
    - 1 = maximally different hashes
    
    This provides a coarse measure of how different two artifacts are.
    """
    if len(hash1) != len(hash2):
        raise ValueError("Hashes must be same length")
    
    if len(hash1) == 0:
        return 0.0
    
    # Count differing bits
    diff_bits = 0
    total_bits = len(hash1) * 8
    
    for b1, b2 in zip(hash1, hash2):
        xor = b1 ^ b2
        diff_bits += bin(xor).count('1')
    
    return diff_bits / total_bits


def compute_change_magnitude(fork_proof: ForkProof) -> float:
    """
    Compute magnitude of changes between agent versions.
    
    PATENT CLAIM: This computation enables graduated reputation
    inheritance based on actual code/model changes.
    
    Factors:
    - Model diff: How different is the new model?
    - Code diff: How different is the agent code?
    - Version jump: Major vs minor vs patch
    
    Args:
        fork_proof: Proof linking old and new versions
        
    Returns:
        Change magnitude in [0, 1] where:
        - 0 = identical (patch with no changes)
        - 1 = completely different (major rewrite)
    """
    # Model difference contribution
    model_distance = compute_hash_distance(
        fork_proof.old_fingerprint.value,
        fork_proof.model_diff_hash
    )
    
    # Code difference contribution
    code_distance = compute_hash_distance(
        fork_proof.old_fingerprint.value,
        fork_proof.code_diff_hash
    )
    
    # Version jump multiplier
    if fork_proof.new_version.is_major_upgrade(fork_proof.old_version):
        version_multiplier = 1.0
    elif fork_proof.new_version.is_minor_upgrade(fork_proof.old_version):
        version_multiplier = 0.5
    else:  # Patch
        version_multiplier = 0.2
    
    # Weighted combination
    raw_magnitude = (
        MODEL_DIFF_WEIGHT * model_distance +
        CODE_DIFF_WEIGHT * code_distance
    )
    
    # Apply version multiplier
    magnitude = raw_magnitude * version_multiplier
    
    # Clamp to [0, 1]
    return max(0.0, min(1.0, magnitude))


# =============================================================================
# FORK INHERITANCE ALGORITHM
# =============================================================================

def compute_inheritance_factor(
    change_magnitude: float,
    old_version: Version,
    new_version: Version
) -> float:
    """
    Compute reputation inheritance factor based on change magnitude.
    
    Uses exponential decay: factor = exp(-lambda * magnitude)
    
    Lambda varies by version type:
    - Major version: Aggressive decay (lambda = 2.0)
    - Minor version: Moderate decay (lambda = 0.5)
    - Patch version: Minimal decay (lambda = 0.1)
    
    Args:
        change_magnitude: Computed change magnitude [0, 1]
        old_version: Previous agent version
        new_version: New agent version
        
    Returns:
        Inheritance factor in (0, 1]
    """
    if new_version.is_major_upgrade(old_version):
        lambda_val = MAJOR_VERSION_LAMBDA
    elif new_version.is_minor_upgrade(old_version):
        lambda_val = MINOR_VERSION_LAMBDA
    else:
        lambda_val = PATCH_VERSION_LAMBDA
    
    return math.exp(-lambda_val * change_magnitude)


def inherit_reputation(
    old_reputation: ReputationScore,
    fork_proof: ForkProof,
    is_verified_fork: bool = True
) -> InheritanceResult:
    """
    Compute inherited reputation for a forked agent.
    
    PATENT CLAIM: This algorithm enables agents to preserve reputation
    through updates while scaling inheritance by change magnitude.
    
    Algorithm:
    1. Compute change magnitude from model/code diffs
    2. Determine inheritance factor via exponential decay
    3. Apply factor to old reputation
    4. Enforce minimum floor for verified forks
    
    Args:
        old_reputation: Reputation of the old agent version
        fork_proof: Cryptographic proof linking versions
        is_verified_fork: Whether fork proof has been verified
        
    Returns:
        InheritanceResult with inherited score and explanation
        
    Determinism: Same inputs always produce same output.
    """
    # Step 1: Compute change magnitude
    change_magnitude = compute_change_magnitude(fork_proof)
    
    # Step 2: Compute inheritance factor
    inheritance_factor = compute_inheritance_factor(
        change_magnitude,
        fork_proof.old_version,
        fork_proof.new_version,
    )
    
    # Step 3: Apply to old reputation
    raw_inherited = old_reputation.value * inheritance_factor
    
    # Step 4: Apply minimum floor for verified forks
    if is_verified_fork:
        inherited_value = max(raw_inherited, REPUTATION_FLOOR_VERIFIED_FORK)
    else:
        # Unverified forks start at default
        inherited_value = REPUTATION_DEFAULT
    
    inherited_score = ReputationScore(value=inherited_value)
    
    # Generate reasoning
    version_type = "major" if fork_proof.new_version.is_major_upgrade(fork_proof.old_version) else \
                   "minor" if fork_proof.new_version.is_minor_upgrade(fork_proof.old_version) else "patch"
    
    reasoning = (
        f"Inherited reputation for {version_type} upgrade "
        f"({fork_proof.old_version} -> {fork_proof.new_version}). "
        f"Change magnitude: {change_magnitude:.4f}, "
        f"Inheritance factor: {inheritance_factor:.4f}, "
        f"Old score: {old_reputation.value:.4f}, "
        f"New score: {inherited_score.value:.4f}"
    )
    
    return InheritanceResult(
        inherited_score=inherited_score,
        inheritance_factor=inheritance_factor,
        change_magnitude=change_magnitude,
        reasoning=reasoning,
    )


# =============================================================================
# REPUTATION UPDATE OPERATIONS
# =============================================================================

@dataclass(frozen=True)
class ReputationUpdate:
    """Record of a reputation update operation."""
    old_score: ReputationScore
    new_score: ReputationScore
    reason: str
    timestamp: Timestamp


def update_on_success(
    record: ReputationRecord,
    timestamp: Timestamp,
    task_value: Stake
) -> Tuple[ReputationRecord, ReputationUpdate]:
    """
    Update reputation after successful task execution.
    
    Boost amount scales with task value (higher stakes = higher reward).
    Uses Bayesian update moving score toward 1.0.
    
    Args:
        record: Current reputation record
        timestamp: When the success occurred
        task_value: Value at stake in the task
        
    Returns:
        Tuple of (new_record, update_info)
    """
    # Scale boost by task value (log scale to prevent gaming)
    value_factor = math.log1p(task_value.value / 1000) / 10
    boost = min(MAX_BOOST_PER_SUCCESS, 0.05 + 0.05 * value_factor)
    
    # Bayesian update toward 1.0
    old_score = record.score
    new_value = old_score.value + (1.0 - old_score.value) * boost
    new_score = ReputationScore(value=new_value)
    
    new_record = ReputationRecord(
        agent_fingerprint=record.agent_fingerprint,
        task_type=record.task_type,
        score=new_score,
        successful_executions=record.successful_executions + 1,
        failed_executions=record.failed_executions,
        last_updated=timestamp,
        decay_half_life=record.decay_half_life,
    )
    
    update = ReputationUpdate(
        old_score=old_score,
        new_score=new_score,
        reason=f"Success on task (value={task_value.value}, boost={boost:.4f})",
        timestamp=timestamp,
    )
    
    return new_record, update


def update_on_failure(
    record: ReputationRecord,
    timestamp: Timestamp,
    is_slashable: bool = False
) -> Tuple[ReputationRecord, ReputationUpdate]:
    """
    Update reputation after failed task execution.
    
    Penalty is more severe for slashable failures (provable misbehavior)
    vs non-slashable failures (honest mistakes).
    
    Args:
        record: Current reputation record
        timestamp: When the failure occurred
        is_slashable: Whether this is provable misbehavior
        
    Returns:
        Tuple of (new_record, update_info)
    """
    if is_slashable:
        # Severe penalty for provable misbehavior
        penalty = MAX_PENALTY_PER_FAILURE
    else:
        # Moderate penalty for honest failures
        penalty = MAX_PENALTY_PER_FAILURE * 0.4
    
    old_score = record.score
    new_value = old_score.value * (1.0 - penalty)
    new_score = ReputationScore(value=new_value)
    
    new_record = ReputationRecord(
        agent_fingerprint=record.agent_fingerprint,
        task_type=record.task_type,
        score=new_score,
        successful_executions=record.successful_executions,
        failed_executions=record.failed_executions + 1,
        last_updated=timestamp,
        decay_half_life=record.decay_half_life,
    )
    
    update = ReputationUpdate(
        old_score=old_score,
        new_score=new_score,
        reason=f"Failure (slashable={is_slashable}, penalty={penalty:.4f})",
        timestamp=timestamp,
    )
    
    return new_record, update


def update_with_decay(
    record: ReputationRecord,
    current_time: Timestamp
) -> ReputationRecord:
    """
    Apply time-based decay to reputation record.
    
    Returns a new record with decayed score and updated timestamp.
    """
    decayed_score = record.decayed_score(current_time)
    
    return ReputationRecord(
        agent_fingerprint=record.agent_fingerprint,
        task_type=record.task_type,
        score=decayed_score,
        successful_executions=record.successful_executions,
        failed_executions=record.failed_executions,
        last_updated=current_time,
        decay_half_life=record.decay_half_life,
    )


# =============================================================================
# REPUTATION INITIALIZATION
# =============================================================================

def create_initial_record(
    agent_fingerprint: AgentFingerprint,
    task_type: TaskType,
    timestamp: Timestamp,
    initial_score: Optional[ReputationScore] = None
) -> ReputationRecord:
    """
    Create initial reputation record for a new agent-task combination.
    
    Args:
        agent_fingerprint: The agent's fingerprint
        task_type: Task type this record is for
        timestamp: Creation timestamp
        initial_score: Optional initial score (defaults to 0.5)
        
    Returns:
        New reputation record
    """
    if initial_score is None:
        initial_score = ReputationScore.default()
    
    return ReputationRecord(
        agent_fingerprint=agent_fingerprint,
        task_type=task_type,
        score=initial_score,
        successful_executions=0,
        failed_executions=0,
        last_updated=timestamp,
        decay_half_life=get_decay_half_life(task_type),
    )


def create_inherited_record(
    old_record: ReputationRecord,
    new_fingerprint: AgentFingerprint,
    fork_proof: ForkProof,
    timestamp: Timestamp
) -> Tuple[ReputationRecord, InheritanceResult]:
    """
    Create reputation record for forked agent inheriting from old version.
    
    PATENT CLAIM: This function implements the complete fork inheritance
    workflow for a single task type.
    
    Args:
        old_record: Reputation record of old agent version
        new_fingerprint: Fingerprint of new agent version
        fork_proof: Proof linking old and new versions
        timestamp: When inheritance is being computed
        
    Returns:
        Tuple of (new_record, inheritance_result)
    """
    # Compute decayed old reputation first
    decayed_old = old_record.decayed_score(timestamp)
    
    # Compute inheritance
    inheritance = inherit_reputation(
        old_reputation=decayed_old,
        fork_proof=fork_proof,
        is_verified_fork=True,
    )
    
    # Create new record with inherited score
    # Note: Execution counts reset to 0 for new version
    new_record = ReputationRecord(
        agent_fingerprint=new_fingerprint,
        task_type=old_record.task_type,
        score=inheritance.inherited_score,
        successful_executions=0,
        failed_executions=0,
        last_updated=timestamp,
        decay_half_life=old_record.decay_half_life,
    )
    
    return new_record, inheritance


# =============================================================================
# AGGREGATE REPUTATION
# =============================================================================

def compute_aggregate_reputation(
    records: List[ReputationRecord],
    current_time: Timestamp
) -> ReputationScore:
    """
    Compute aggregate reputation across all task types.
    
    Uses weighted average where weight = execution count * recency.
    
    Args:
        records: All reputation records for an agent
        current_time: Current timestamp for decay
        
    Returns:
        Aggregate reputation score
    """
    if not records:
        return ReputationScore.default()
    
    weighted_sum = 0.0
    weight_total = 0.0
    
    for record in records:
        # Weight by execution count (diminishing returns)
        exec_weight = math.log1p(record.total_executions())
        
        # Apply time decay
        decayed = record.decayed_score(current_time)
        
        weighted_sum += float(decayed.value) * exec_weight
        weight_total += exec_weight
    
    if weight_total == 0:
        return ReputationScore.default()
    
    return ReputationScore(value=weighted_sum / weight_total)


def is_reputation_reliable(records: List[ReputationRecord]) -> bool:
    """
    Check if an agent has enough history for reliable reputation.
    
    Returns True if total executions across all task types exceeds
    the minimum threshold.
    """
    total_executions = sum(r.total_executions() for r in records)
    return total_executions >= MIN_EXECUTIONS_FOR_RELIABILITY
