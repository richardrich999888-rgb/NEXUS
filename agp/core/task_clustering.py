"""
AGP Task-Type Clustering for Validator Selection

PATENT CLAIM: Novel algorithm for selecting validators based on
demonstrated expertise in similar task types.

Prior Art Gap: Existing systems use random selection or stake-weighted
selection. No system clusters validators by task-type expertise.

Algorithm:
1. Embed task type into vector space
2. Compute expertise similarity for each validator
3. Score = reputation * expertise_similarity * stake_weight
4. Select top-k validators with anti-collusion filter

This module has ZERO external dependencies.
"""

from __future__ import annotations

import math
import hashlib
from typing import List, Tuple, Optional, Set
from dataclasses import dataclass

from .types import (
    TaskType,
    TaskTypeVector,
    ValidatorInfo,
    Task,
    ValidatorSelection,
    ReputationScore,
    Stake,
    Timestamp,
    AgentFingerprint,
    EMBEDDING_DIMENSIONS,
)


# =============================================================================
# CONSTANTS
# =============================================================================

# Minimum validators required for task assignment
MIN_VALIDATORS: int = 3

# Maximum validators to select
MAX_VALIDATORS: int = 21

# Weight factors for validator scoring
REPUTATION_WEIGHT: float = 0.4
EXPERTISE_WEIGHT: float = 0.4
STAKE_WEIGHT: float = 0.2

# Minimum stake required to be considered
MIN_STAKE_THRESHOLD: int = 1000

# Maximum validators from same operator (anti-collusion)
MAX_SAME_OPERATOR: int = 1


# =============================================================================
# TASK TYPE EMBEDDING
# =============================================================================

def _deterministic_hash(data: str) -> int:
    """
    Generate deterministic integer from string.
    
    Uses SHA-256 truncated to 64 bits for stability.
    """
    digest = hashlib.sha256(data.encode('utf-8')).digest()
    return int.from_bytes(digest[:8], 'big')


def _hash_to_unit_float(hash_val: int, index: int) -> float:
    """
    Convert hash to float in [-1, 1] using deterministic mixing.
    
    Creates unique deterministic value for each (hash_val, index) pair.
    """
    # Create unique seed by combining hash_val and index
    combined = hashlib.sha256(f"{hash_val}:{index}".encode()).digest()
    # Use first 8 bytes as unsigned int
    int_val = int.from_bytes(combined[:8], 'big')
    # Map to [0, 1]
    unit = int_val / (2**64)
    # Map to [-1, 1]
    return 2.0 * unit - 1.0


def embed_task_type(task_type: TaskType) -> TaskTypeVector:
    """
    Embed task type into vector space.
    
    Uses deterministic hashing to generate embedding from task type string.
    Each level (L0, L1, L2) contributes to different parts of the vector.
    
    Patent Note: This embedding enables similarity computation between
    tasks and validator expertise profiles.
    
    Args:
        task_type: The task type to embed
        
    Returns:
        64-dimensional embedding vector
        
    Determinism: Same input always produces same output.
    """
    values = []
    dims_per_level = EMBEDDING_DIMENSIONS // 3
    
    # L0 domain contributes to first third
    l0_hash = _deterministic_hash(task_type.l0_domain)
    for i in range(dims_per_level):
        values.append(_hash_to_unit_float(l0_hash, i))
    
    # L1 category contributes to second third
    l1_combined = f"{task_type.l0_domain}/{task_type.l1_category}"
    l1_hash = _deterministic_hash(l1_combined)
    for i in range(dims_per_level):
        values.append(_hash_to_unit_float(l1_hash, i))
    
    # L2 specific contributes to final third
    l2_combined = str(task_type)
    l2_hash = _deterministic_hash(l2_combined)
    remaining = EMBEDDING_DIMENSIONS - 2 * dims_per_level
    for i in range(remaining):
        values.append(_hash_to_unit_float(l2_hash, i))
    
    # Normalize to unit vector
    magnitude = math.sqrt(sum(v * v for v in values))
    if magnitude > 0:
        values = [v / magnitude for v in values]
    
    return TaskTypeVector(values=tuple(values))


# =============================================================================
# EXPERTISE PROFILING
# =============================================================================

def compute_expertise_vector(
    validator: ValidatorInfo,
    current_time: Timestamp
) -> TaskTypeVector:
    """
    Compute validator's expertise profile as weighted embedding.
    
    Weights each task type by:
    - Recency (exponential decay from last activity)
    - Success rate in that task type
    - Volume of executions
    
    Patent Note: This expertise profiling enables matching validators
    to tasks based on demonstrated competence.
    
    Args:
        validator: Validator to profile
        current_time: Current timestamp for decay calculation
        
    Returns:
        Expertise vector (weighted combination of task type embeddings)
    """
    if not validator.reputation_records:
        return TaskTypeVector.zero()
    
    weighted_sum = [0.0] * EMBEDDING_DIMENSIONS
    total_weight = 0.0
    
    for record in validator.reputation_records:
        # Skip if no executions
        if record.total_executions() == 0:
            continue
        
        # Compute weight factors
        decayed_score = record.decayed_score(current_time)
        volume_factor = math.log1p(record.total_executions())  # Diminishing returns
        success_factor = record.success_rate()
        
        weight = float(decayed_score.value) * volume_factor * success_factor
        if weight <= 0:
            continue
        
        # Add weighted embedding
        embedding = embed_task_type(record.task_type)
        for i, v in enumerate(embedding.values):
            weighted_sum[i] += v * weight
        total_weight += weight
    
    # Normalize
    if total_weight > 0:
        weighted_sum = [v / total_weight for v in weighted_sum]
    
    # Ensure unit vector
    magnitude = math.sqrt(sum(v * v for v in weighted_sum))
    if magnitude > 0:
        weighted_sum = [v / magnitude for v in weighted_sum]
    
    return TaskTypeVector(values=tuple(weighted_sum))


# =============================================================================
# VALIDATOR SCORING
# =============================================================================

@dataclass(frozen=True)
class ValidatorScore:
    """Intermediate scoring result for a validator."""
    validator: ValidatorInfo
    reputation_score: float
    expertise_score: float
    stake_score: float
    combined_score: float


def score_validator(
    validator: ValidatorInfo,
    task: Task,
    task_embedding: TaskTypeVector,
    total_stake: Stake,
    current_time: Timestamp
) -> ValidatorScore:
    """
    Compute comprehensive score for a validator on a specific task.
    
    Score = reputation_weight * reputation_score
          + expertise_weight * expertise_similarity
          + stake_weight * stake_ratio
    
    Patent Note: This multi-factor scoring enables task-appropriate
    validator selection beyond simple stake-weighting.
    
    Args:
        validator: Validator to score
        task: Task being assigned
        task_embedding: Pre-computed task type embedding
        total_stake: Total stake in the network (for ratio calculation)
        current_time: Current timestamp
        
    Returns:
        ValidatorScore with component scores and combined score
    """
    # 1. Reputation score for this task type
    reputation_score = 0.5  # Default for new validators
    
    # Check for exact match first
    exact_record = validator.get_reputation_for_task(task.task_type)
    if exact_record:
        reputation_score = float(exact_record.decayed_score(current_time).value)
    else:
        # Use similar task types with weighted average
        similar = validator.get_similar_reputations(task.task_type)
        if similar:
            weighted_sum = 0.0
            weight_total = 0.0
            for record, similarity in similar:
                score = float(record.decayed_score(current_time).value)
                weighted_sum += score * similarity
                weight_total += similarity
            if weight_total > 0:
                reputation_score = weighted_sum / weight_total
    
    # 2. Expertise similarity
    expertise_vector = compute_expertise_vector(validator, current_time)
    expertise_score = task_embedding.cosine_similarity(expertise_vector)
    # Map from [-1, 1] to [0, 1]
    expertise_score = (expertise_score + 1.0) / 2.0
    
    # 3. Stake ratio (square root to reduce plutocracy)
    if total_stake.value > 0:
        stake_ratio = validator.stake.ratio(total_stake)
        stake_score = math.sqrt(stake_ratio)
    else:
        stake_score = 0.0
    
    # 4. Combined score
    combined_score = (
        REPUTATION_WEIGHT * reputation_score +
        EXPERTISE_WEIGHT * expertise_score +
        STAKE_WEIGHT * stake_score
    )
    
    return ValidatorScore(
        validator=validator,
        reputation_score=reputation_score,
        expertise_score=expertise_score,
        stake_score=stake_score,
        combined_score=combined_score,
    )


# =============================================================================
# ANTI-COLLUSION FILTER
# =============================================================================

def apply_anti_collusion_filter(
    scored_validators: List[ValidatorScore],
    max_same_operator: int = MAX_SAME_OPERATOR
) -> List[ValidatorScore]:
    """
    Filter validators to prevent operator collusion.
    
    Ensures no more than max_same_operator validators from the
    same operator are selected.
    
    Patent Note: This filter is part of the novel validator selection
    system but builds on standard anti-collusion principles.
    
    Args:
        scored_validators: Validators sorted by score (descending)
        max_same_operator: Maximum validators per operator
        
    Returns:
        Filtered list maintaining score order
    """
    operator_counts: dict = {}
    filtered = []
    
    for sv in scored_validators:
        operator = sv.validator.operator_id
        current_count = operator_counts.get(operator, 0)
        
        if current_count < max_same_operator:
            filtered.append(sv)
            operator_counts[operator] = current_count + 1
    
    return filtered


# =============================================================================
# MAIN SELECTION ALGORITHM
# =============================================================================

def select_validators(
    task: Task,
    available_validators: List[ValidatorInfo],
    k: int,
    current_time: Timestamp,
    total_network_stake: Optional[Stake] = None
) -> ValidatorSelection:
    """
    Select k validators for a task using task-type clustering.
    
    PATENT CLAIM: This algorithm selects validators based on demonstrated
    expertise in similar task types, not just stake or random selection.
    
    Algorithm:
    1. Filter to active validators meeting minimum stake
    2. Embed task type into vector space
    3. Score each validator on reputation, expertise, and stake
    4. Apply anti-collusion filter
    5. Select top-k validators
    
    Args:
        task: Task requiring validators
        available_validators: All validators in the network
        k: Number of validators to select
        current_time: Current timestamp
        total_network_stake: Total stake (computed if not provided)
        
    Returns:
        ValidatorSelection with selected validators and scores
        
    Raises:
        ValueError: If insufficient validators available
        
    Determinism: Same inputs always produce same selection.
    """
    # Validate k
    if k < MIN_VALIDATORS:
        raise ValueError(f"Must select at least {MIN_VALIDATORS} validators")
    if k > MAX_VALIDATORS:
        raise ValueError(f"Cannot select more than {MAX_VALIDATORS} validators")
    
    # 1. Filter to eligible validators
    eligible = [
        v for v in available_validators
        if v.is_active and v.stake.value >= MIN_STAKE_THRESHOLD
    ]
    
    if len(eligible) < k:
        raise ValueError(
            f"Insufficient validators: need {k}, have {len(eligible)} eligible"
        )
    
    # 2. Compute total stake if not provided
    if total_network_stake is None:
        total_network_stake = Stake(
            value=sum(v.stake.value for v in eligible)
        )
    
    # 3. Embed task type
    task_embedding = embed_task_type(task.task_type)
    
    # 4. Score all validators
    scored = [
        score_validator(
            validator=v,
            task=task,
            task_embedding=task_embedding,
            total_stake=total_network_stake,
            current_time=current_time,
        )
        for v in eligible
    ]
    
    # 5. Sort by combined score (descending)
    scored.sort(key=lambda x: x.combined_score, reverse=True)
    
    # 6. Apply anti-collusion filter
    filtered = apply_anti_collusion_filter(scored)
    
    if len(filtered) < k:
        raise ValueError(
            f"Insufficient validators after anti-collusion: need {k}, have {len(filtered)}"
        )
    
    # 7. Select top-k
    selected = filtered[:k]
    
    return ValidatorSelection(
        validators=tuple(sv.validator for sv in selected),
        task=task,
        selection_scores=tuple(sv.combined_score for sv in selected),
        timestamp=current_time,
    )


# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

def compute_selection_diversity(selection: ValidatorSelection) -> float:
    """
    Compute diversity score for a validator selection.
    
    Higher diversity = validators from different operators and
    with different expertise profiles.
    
    Returns value in [0, 1] where 1 = maximum diversity.
    """
    if len(selection.validators) <= 1:
        return 1.0
    
    # Operator diversity
    operators = set(v.operator_id for v in selection.validators)
    operator_diversity = len(operators) / len(selection.validators)
    
    # Score diversity (entropy-based)
    scores = selection.selection_scores
    total = sum(scores)
    if total > 0:
        probs = [s / total for s in scores]
        entropy = -sum(p * math.log(p) for p in probs if p > 0)
        max_entropy = math.log(len(scores))
        score_diversity = entropy / max_entropy if max_entropy > 0 else 1.0
    else:
        score_diversity = 0.0
    
    return 0.6 * operator_diversity + 0.4 * score_diversity


def explain_selection(
    selection: ValidatorSelection,
    current_time: Timestamp
) -> str:
    """
    Generate human-readable explanation of validator selection.
    
    Useful for auditing and debugging.
    """
    lines = [
        f"Validator Selection for Task {selection.task.task_id.hex()[:16]}...",
        f"Task Type: {selection.task.task_type}",
        f"Selected {len(selection.validators)} validators:",
        "",
    ]
    
    for i, (validator, score) in enumerate(
        zip(selection.validators, selection.selection_scores)
    ):
        lines.append(f"  {i+1}. {validator.agent_fingerprint}")
        lines.append(f"     Score: {score:.4f}")
        lines.append(f"     Stake: {validator.stake.value}")
        
        rep = validator.get_reputation_for_task(selection.task.task_type)
        if rep:
            decayed = rep.decayed_score(current_time)
            lines.append(f"     Reputation: {decayed.value:.4f} (decayed)")
        else:
            lines.append("     Reputation: N/A (no history for task type)")
        lines.append("")
    
    diversity = compute_selection_diversity(selection)
    lines.append(f"Selection Diversity: {diversity:.4f}")
    
    return "\n".join(lines)
