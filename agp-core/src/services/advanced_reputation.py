"""
Advanced Reputation Services - Phase 3
Staking, governance, derivatives, ML integration
"""

import math
import uuid
from typing import Dict, List, Optional, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from enum import Enum

from src.models import Hormone, EndocrineState, HealthStatus


# =============================================================================
# STAKING SYSTEM
# =============================================================================

@dataclass
class Stake:
    """Agent stake for reputation bonding"""
    id: uuid.UUID
    agent_id: uuid.UUID
    amount: float
    locked_until: datetime
    multiplier: float  # earned from stake duration
    created_at: datetime


class StakingService:
    """
    Manages reputation staking and bonding curves
    
    Staking affects:
    - Receptor density (higher stake = more sensitive receptors)
    - Privilege levels
    - Action cost discounts
    """
    
    def __init__(self):
        self.stakes: Dict[uuid.UUID, List[Stake]] = {}
        self.total_staked: float = 0.0
        
        # Bonding curve parameters
        self.base_rate = 1.0
        self.curve_exponent = 0.5  # Sublinear (diminishing returns)
    
    def stake(
        self,
        agent_id: uuid.UUID,
        amount: float,
        lock_days: int = 30
    ) -> Stake:
        """Create a new stake"""
        # Calculate multiplier based on lock duration
        multiplier = 1.0 + math.log1p(lock_days) * 0.1
        
        stake = Stake(
            id=uuid.uuid4(),
            agent_id=agent_id,
            amount=amount,
            locked_until=datetime.utcnow() + timedelta(days=lock_days),
            multiplier=multiplier,
            created_at=datetime.utcnow()
        )
        
        if agent_id not in self.stakes:
            self.stakes[agent_id] = []
        
        self.stakes[agent_id].append(stake)
        self.total_staked += amount
        
        return stake
    
    def unstake(self, agent_id: uuid.UUID, stake_id: uuid.UUID) -> Optional[float]:
        """Unstake if lock period expired"""
        if agent_id not in self.stakes:
            return None
        
        for stake in self.stakes[agent_id]:
            if stake.id == stake_id:
                if datetime.utcnow() < stake.locked_until:
                    return None  # Still locked
                
                self.stakes[agent_id].remove(stake)
                self.total_staked -= stake.amount
                
                # Return amount with multiplier
                return stake.amount * stake.multiplier
        
        return None
    
    def get_stake_power(self, agent_id: uuid.UUID) -> float:
        """Calculate total stake power for an agent"""
        if agent_id not in self.stakes:
            return 0.0
        
        total = 0.0
        now = datetime.utcnow()
        
        for stake in self.stakes[agent_id]:
            # Active stakes count
            if stake.locked_until > now:
                total += stake.amount * stake.multiplier
        
        return total
    
    def bonding_curve_price(self, supply: float) -> float:
        """
        Calculate token price based on bonding curve
        price = base_rate × (total_supply ^ exponent)
        """
        return self.base_rate * (supply ** self.curve_exponent)
    
    def receptor_multiplier(self, agent_id: uuid.UUID) -> float:
        """
        Calculate receptor sensitivity multiplier based on stake
        More stake = higher receptor density = more sensitive
        """
        stake_power = self.get_stake_power(agent_id)
        
        # Logarithmic scaling to prevent extreme values
        return 1.0 + math.log1p(stake_power) * 0.1


# =============================================================================
# GOVERNANCE SYSTEM
# =============================================================================

class VoteType(str, Enum):
    YES = "yes"
    NO = "no"
    ABSTAIN = "abstain"


@dataclass
class Proposal:
    """Governance proposal"""
    id: uuid.UUID
    title: str
    description: str
    proposer_id: uuid.UUID
    parameter_key: Optional[str]
    parameter_value: Optional[float]
    votes: Dict[uuid.UUID, VoteType] = field(default_factory=dict)
    vote_weights: Dict[uuid.UUID, float] = field(default_factory=dict)
    status: str = "active"
    created_at: datetime = field(default_factory=datetime.utcnow)
    expires_at: Optional[datetime] = None


class GovernanceService:
    """
    Manages governance proposals and voting
    
    Voting power is based on:
    - Stake amount
    - Reputation (endocrine state)
    - Protocol participation
    """
    
    def __init__(self, staking_service: StakingService):
        self.staking_service = staking_service
        self.proposals: Dict[uuid.UUID, Proposal] = {}
        
        # Governance parameters
        self.quorum = 0.1  # 10% of voting power must participate
        self.threshold = 0.6  # 60% majority to pass
    
    def create_proposal(
        self,
        title: str,
        description: str,
        proposer_id: uuid.UUID,
        parameter_key: Optional[str] = None,
        parameter_value: Optional[float] = None,
        duration_days: int = 7
    ) -> Proposal:
        """Create a new governance proposal"""
        proposal = Proposal(
            id=uuid.uuid4(),
            title=title,
            description=description,
            proposer_id=proposer_id,
            parameter_key=parameter_key,
            parameter_value=parameter_value,
            expires_at=datetime.utcnow() + timedelta(days=duration_days)
        )
        
        self.proposals[proposal.id] = proposal
        return proposal
    
    def vote(
        self,
        proposal_id: uuid.UUID,
        voter_id: uuid.UUID,
        vote: VoteType,
        voter_state: EndocrineState
    ) -> bool:
        """Cast a vote on a proposal"""
        if proposal_id not in self.proposals:
            return False
        
        proposal = self.proposals[proposal_id]
        
        if proposal.status != "active":
            return False
        
        if proposal.expires_at and datetime.utcnow() > proposal.expires_at:
            proposal.status = "expired"
            return False
        
        # Calculate voting power
        stake_power = self.staking_service.get_stake_power(voter_id)
        
        # Reputation-weighted voting power
        alignment = self._calculate_alignment(voter_state)
        reputation_factor = 0.5 + alignment * 0.5
        
        voting_power = (1 + stake_power) * reputation_factor
        
        proposal.votes[voter_id] = vote
        proposal.vote_weights[voter_id] = voting_power
        
        return True
    
    def tally_votes(self, proposal_id: uuid.UUID) -> Dict:
        """Tally votes and determine outcome"""
        if proposal_id not in self.proposals:
            return {"error": "Proposal not found"}
        
        proposal = self.proposals[proposal_id]
        
        yes_weight = sum(
            w for v, w in zip(proposal.votes.values(), proposal.vote_weights.values())
            if v == VoteType.YES
        )
        no_weight = sum(
            w for v, w in zip(proposal.votes.values(), proposal.vote_weights.values())
            if v == VoteType.NO
        )
        total_weight = sum(proposal.vote_weights.values())
        
        passed = False
        if total_weight > 0:
            yes_ratio = yes_weight / total_weight
            passed = yes_ratio >= self.threshold
        
        return {
            "proposal_id": str(proposal_id),
            "yes_weight": yes_weight,
            "no_weight": no_weight,
            "total_weight": total_weight,
            "passed": passed,
            "votes_count": len(proposal.votes)
        }
    
    def _calculate_alignment(self, state: EndocrineState) -> float:
        """Calculate alignment from endocrine state"""
        baseline = 0.5
        total_deviation = sum(
            abs(v - baseline) for v in state.levels.values()
        )
        return 1.0 - (total_deviation / len(Hormone))


# =============================================================================
# INCENTIVE PROGRAMS
# =============================================================================

@dataclass
class IncentiveProgram:
    """Incentive/rewards program"""
    id: uuid.UUID
    name: str
    reward_hormone: Hormone
    reward_amount: float
    conditions: Dict  # e.g., {"min_stake": 100, "action_type": "inference"}
    participants: List[uuid.UUID] = field(default_factory=list)
    created_at: datetime = field(default_factory=datetime.utcnow)
    expires_at: Optional[datetime] = None


class IncentiveService:
    """
    Manages incentive programs like:
    - Liquidity mining (stake rewards)
    - Referral bonuses
    - Task completion bonuses
    """
    
    def __init__(self):
        self.programs: Dict[uuid.UUID, IncentiveProgram] = {}
    
    def create_program(
        self,
        name: str,
        reward_hormone: Hormone,
        reward_amount: float,
        conditions: Dict,
        duration_days: int = 30
    ) -> IncentiveProgram:
        """Create an incentive program"""
        program = IncentiveProgram(
            id=uuid.uuid4(),
            name=name,
            reward_hormone=reward_hormone,
            reward_amount=reward_amount,
            conditions=conditions,
            expires_at=datetime.utcnow() + timedelta(days=duration_days)
        )
        
        self.programs[program.id] = program
        return program
    
    def join_program(
        self,
        program_id: uuid.UUID,
        agent_id: uuid.UUID,
        agent_state: EndocrineState,
        stake_amount: float = 0.0
    ) -> bool:
        """Join an incentive program if conditions are met"""
        if program_id not in self.programs:
            return False
        
        program = self.programs[program_id]
        
        if program.expires_at and datetime.utcnow() > program.expires_at:
            return False
        
        # Check conditions
        conditions = program.conditions
        
        if "min_stake" in conditions and stake_amount < conditions["min_stake"]:
            return False
        
        if "min_alignment" in conditions:
            alignment = 1.0 - sum(
                abs(v - 0.5) for v in agent_state.levels.values()
            ) / len(Hormone)
            if alignment < conditions["min_alignment"]:
                return False
        
        if agent_id not in program.participants:
            program.participants.append(agent_id)
        
        return True
    
    def calculate_rewards(
        self,
        program_id: uuid.UUID,
        agent_id: uuid.UUID,
        participation_score: float
    ) -> float:
        """Calculate rewards for an agent in a program"""
        if program_id not in self.programs:
            return 0.0
        
        program = self.programs[program_id]
        
        if agent_id not in program.participants:
            return 0.0
        
        # Reward proportional to participation
        return program.reward_amount * participation_score


# =============================================================================
# INTEGRATION
# =============================================================================

# Create service instances
staking_service = StakingService()
governance_service = GovernanceService(staking_service)
incentive_service = IncentiveService()
