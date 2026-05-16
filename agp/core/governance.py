"""
AGP Execution-Weighted Governance

PATENT CLAIM: Novel algorithm for governance voting where voting power
is derived from verified execution history, not just token holdings.
"""

from __future__ import annotations

import math
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from enum import Enum, auto

from .types import (
    AgentFingerprint, TaskType, ReputationScore, ReputationRecord,
    Stake, Timestamp, Duration, Vote, ProposalState,
)

EXECUTION_WEIGHT_RATIO: float = 0.7
TOKEN_WEIGHT_RATIO: float = 0.3
MIN_TOKEN_BALANCE: int = 100
MIN_EXECUTION_SCORE_FOR_FULL_WEIGHT: float = 10.0
MIN_VOTING_PERIOD: int = 86400
MAX_VOTING_PERIOD: int = 86400 * 30
MIN_QUORUM_PERCENTAGE: float = 0.04
MAX_QUORUM_PERCENTAGE: float = 0.50
EXECUTION_DECAY_HALF_LIFE: int = 86400 * 30

class ProposalCategory(Enum):
    PARAMETER_CHANGE = auto()
    UPGRADE = auto()
    TREASURY = auto()
    EMERGENCY = auto()
    TASK_TYPE = auto()

@dataclass(frozen=True)
class ProposalConfig:
    category: ProposalCategory
    relevant_task_types: Tuple[TaskType, ...]
    voting_period: Duration
    quorum_percentage: float
    execution_delay: Duration

@dataclass(frozen=True)
class Proposal:
    proposal_id: bytes
    proposer: AgentFingerprint
    title: str
    description: str
    config: ProposalConfig
    actions: Tuple[bytes, ...]
    created_at: Timestamp
    voting_starts: Timestamp
    voting_ends: Timestamp
    state: ProposalState

@dataclass(frozen=True)
class ExecutionHistory:
    agent: AgentFingerprint
    reputation_records: Tuple[ReputationRecord, ...]
    total_successful: int
    total_failed: int
    unique_task_types: int

    @classmethod
    def from_records(cls, agent: AgentFingerprint, records: List[ReputationRecord]) -> ExecutionHistory:
        return cls(
            agent=agent,
            reputation_records=tuple(records),
            total_successful=sum(r.successful_executions for r in records),
            total_failed=sum(r.failed_executions for r in records),
            unique_task_types=len(set(r.task_type.to_tuple() for r in records)),
        )

    def total_executions(self) -> int:
        return self.total_successful + self.total_failed

    def success_rate(self) -> float:
        total = self.total_executions()
        return self.total_successful / total if total > 0 else 0.5

def compute_execution_weight(history: ExecutionHistory, relevant_task_types: Tuple[TaskType, ...], current_time: Timestamp) -> float:
    if not history.reputation_records:
        return 0.0
    relevant = [r for r in history.reputation_records if any(r.task_type.similarity(rt) > 0 for rt in relevant_task_types)]
    if not relevant:
        relevant = list(history.reputation_records)
        penalty = 0.5
    else:
        penalty = 1.0
    score = 0.0
    for record in relevant:
        net = max(0, record.successful_executions - record.failed_executions * 0.5)
        elapsed = current_time.seconds_since(record.last_updated)
        decay = math.pow(0.5, elapsed / EXECUTION_DECAY_HALF_LIFE)
        score += net * decay
    return math.log1p(score * penalty) * 10 if score > 0 else 0.0

@dataclass(frozen=True)
class VotingPower:
    voter: AgentFingerprint
    token_balance: Stake
    execution_weight: float
    token_score: float
    execution_score: float
    total_power: float

    @classmethod
    def compute(cls, voter: AgentFingerprint, token_balance: Stake, execution_history: ExecutionHistory, proposal: Proposal, current_time: Timestamp) -> VotingPower:
        token_score = math.sqrt(token_balance.value)
        exec_weight = compute_execution_weight(execution_history, proposal.config.relevant_task_types, current_time)
        target = math.sqrt(MIN_TOKEN_BALANCE)
        exec_score = target * (1 + math.log(exec_weight / MIN_EXECUTION_SCORE_FOR_FULL_WEIGHT)) if exec_weight > MIN_EXECUTION_SCORE_FOR_FULL_WEIGHT else target * (exec_weight / MIN_EXECUTION_SCORE_FOR_FULL_WEIGHT)
        total = EXECUTION_WEIGHT_RATIO * exec_score + TOKEN_WEIGHT_RATIO * token_score
        return cls(voter=voter, token_balance=token_balance, execution_weight=exec_weight, token_score=token_score, execution_score=exec_score, total_power=total)
