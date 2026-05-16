"""AGP Tiered Verification Selection - Patent Claim 4"""
from __future__ import annotations
import math
from dataclasses import dataclass
from .types import VerificationTier, VerificationDecision, Task, ReputationScore, Stake, Timestamp

VALUE_RISK_WEIGHT = 0.4
REPUTATION_RISK_WEIGHT = 0.3
SENSITIVITY_RISK_WEIGHT = 0.3
ZKML_RISK_THRESHOLD = 0.8
TEE_RISK_THRESHOLD = 0.4
BASE_ZKML_COST = 10000
BASE_TEE_COST = 1000
BASE_OPTIMISTIC_COST = 100

@dataclass(frozen=True)
class NetworkState:
    total_stake: Stake
    active_validators: int
    pending_zkml_proofs: int
    zkml_queue_capacity: int
    pending_tee_attestations: int
    tee_queue_capacity: int
    average_zkml_latency_ms: int
    average_tee_latency_ms: int
    current_timestamp: Timestamp

    def zkml_congestion(self) -> float:
        return self.pending_zkml_proofs / self.zkml_queue_capacity if self.zkml_queue_capacity > 0 else 1.0

    def is_zkml_congested(self) -> bool:
        return self.zkml_congestion() > 0.8

def select_verification_tier(task: Task, agent_reputation: ReputationScore, network_state: NetworkState) -> VerificationDecision:
    value_risk = min(1.0, task.stake_at_risk.ratio(network_state.total_stake) * 10)
    reputation_risk = 1.0 - agent_reputation.value
    sensitivity_risk = task.sensitivity
    weighted = VALUE_RISK_WEIGHT * value_risk + REPUTATION_RISK_WEIGHT * reputation_risk + SENSITIVITY_RISK_WEIGHT * sensitivity_risk
    risk = max(weighted, max(value_risk, reputation_risk, sensitivity_risk) * 0.9)
    risk = min(1.0, risk)

    if risk >= ZKML_RISK_THRESHOLD:
        tier = VerificationTier.ZKML
        cost = Stake(BASE_ZKML_COST)
        reason = "High risk requires zkML"
    elif risk >= TEE_RISK_THRESHOLD:
        tier = VerificationTier.TEE
        cost = Stake(BASE_TEE_COST)
        reason = "Medium risk uses TEE"
    else:
        tier = VerificationTier.OPTIMISTIC
        cost = Stake(BASE_OPTIMISTIC_COST)
        reason = "Low risk uses optimistic"

    if tier == VerificationTier.ZKML and network_state.is_zkml_congested() and risk < 0.9:
        tier = VerificationTier.TEE
        cost = Stake(BASE_TEE_COST)
        reason += " (downgraded due to congestion)"

    return VerificationDecision(tier=tier, risk_score=risk, cost_estimate=cost, reasoning=f"{reason}. Risk: {risk:.2f}")

def configure_verification(task: Task, agent_reputation: ReputationScore, network_state: NetworkState):
    return select_verification_tier(task, agent_reputation, network_state)

def explain_verification_decision(decision: VerificationDecision) -> str:
    return f"Tier: {decision.tier.name}, Risk: {decision.risk_score:.2f}, Cost: {decision.cost_estimate.value}"
