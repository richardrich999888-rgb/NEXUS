"""
Swarm Intelligence Coordinator - Phase 5
Collective decision-making and emergent behavior for agent swarms
"""

import uuid
import math
import random
from typing import Dict, List, Optional, Any, Tuple, Set
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict

from src.models import EndocrineState, Hormone


class SwarmRole(str, Enum):
    LEADER = "leader"
    WORKER = "worker"
    SCOUT = "scout"
    VALIDATOR = "validator"
    AGGREGATOR = "aggregator"


class ConsensusMethod(str, Enum):
    MAJORITY = "majority"
    WEIGHTED = "weighted"
    QUORUM = "quorum"
    UNANIMOUS = "unanimous"


@dataclass
class SwarmMember:
    """Member of a swarm"""
    agent_id: uuid.UUID
    role: SwarmRole
    reputation: float
    capabilities: List[str]
    current_task: Optional[str] = None
    last_contribution: Optional[datetime] = None
    contribution_count: int = 0


@dataclass
class Swarm:
    """A collective of agents working together"""
    id: uuid.UUID
    name: str
    objective: str
    members: Dict[uuid.UUID, SwarmMember]
    leader_id: Optional[uuid.UUID]
    created_at: datetime
    consensus_method: ConsensusMethod = ConsensusMethod.WEIGHTED
    min_quorum: float = 0.5
    active: bool = True


@dataclass
class CollectiveDecision:
    """A decision made by the swarm"""
    id: uuid.UUID
    swarm_id: uuid.UUID
    question: str
    options: List[str]
    votes: Dict[uuid.UUID, int]  # agent_id -> option_index
    weights: Dict[uuid.UUID, float]  # agent_id -> voting weight
    deadline: datetime
    result: Optional[int] = None
    finalized: bool = False


@dataclass
class EmergentPattern:
    """Detected emergent behavior pattern"""
    id: uuid.UUID
    swarm_id: uuid.UUID
    pattern_type: str
    description: str
    confidence: float
    first_observed: datetime
    occurrences: int = 1


class SwarmCoordinator:
    """
    Coordinates swarm behavior and collective intelligence
    
    Implements:
    - Swarm formation and management
    - Role assignment based on reputation
    - Collective decision-making
    - Emergent behavior detection
    """
    
    def __init__(self):
        self.swarms: Dict[uuid.UUID, Swarm] = {}
        self.decisions: Dict[uuid.UUID, CollectiveDecision] = {}
        self.patterns: Dict[uuid.UUID, EmergentPattern] = {}
        self.agent_swarms: Dict[uuid.UUID, Set[uuid.UUID]] = defaultdict(set)
    
    def create_swarm(
        self,
        name: str,
        objective: str,
        founder_id: uuid.UUID,
        founder_reputation: float = 0.5,
        consensus_method: ConsensusMethod = ConsensusMethod.WEIGHTED
    ) -> Swarm:
        """Create a new swarm"""
        swarm_id = uuid.uuid4()
        
        founder = SwarmMember(
            agent_id=founder_id,
            role=SwarmRole.LEADER,
            reputation=founder_reputation,
            capabilities=[]
        )
        
        swarm = Swarm(
            id=swarm_id,
            name=name,
            objective=objective,
            members={founder_id: founder},
            leader_id=founder_id,
            created_at=datetime.utcnow(),
            consensus_method=consensus_method
        )
        
        self.swarms[swarm_id] = swarm
        self.agent_swarms[founder_id].add(swarm_id)
        
        return swarm
    
    def join_swarm(
        self,
        swarm_id: uuid.UUID,
        agent_id: uuid.UUID,
        reputation: float,
        capabilities: List[str]
    ) -> SwarmMember:
        """Add an agent to a swarm"""
        swarm = self.swarms.get(swarm_id)
        if not swarm or not swarm.active:
            raise ValueError("Swarm not available")
        
        # Assign role based on reputation and capabilities
        role = self._assign_role(reputation, capabilities, swarm)
        
        member = SwarmMember(
            agent_id=agent_id,
            role=role,
            reputation=reputation,
            capabilities=capabilities
        )
        
        swarm.members[agent_id] = member
        self.agent_swarms[agent_id].add(swarm_id)
        
        # Check for leadership change
        self._update_leadership(swarm)
        
        return member
    
    def leave_swarm(self, swarm_id: uuid.UUID, agent_id: uuid.UUID):
        """Remove an agent from a swarm"""
        swarm = self.swarms.get(swarm_id)
        if not swarm or agent_id not in swarm.members:
            return
        
        del swarm.members[agent_id]
        self.agent_swarms[agent_id].discard(swarm_id)
        
        # Reassign leadership if needed
        if swarm.leader_id == agent_id:
            self._update_leadership(swarm)
    
    def _assign_role(
        self,
        reputation: float,
        capabilities: List[str],
        swarm: Swarm
    ) -> SwarmRole:
        """Assign role based on agent characteristics"""
        if reputation > 0.8:
            return SwarmRole.VALIDATOR
        elif "exploration" in capabilities:
            return SwarmRole.SCOUT
        elif "aggregation" in capabilities:
            return SwarmRole.AGGREGATOR
        else:
            return SwarmRole.WORKER
    
    def _update_leadership(self, swarm: Swarm):
        """Update swarm leadership based on reputation"""
        if not swarm.members:
            swarm.leader_id = None
            return
        
        # Find highest reputation member
        best = max(swarm.members.values(), key=lambda m: m.reputation)
        
        if swarm.leader_id != best.agent_id:
            # Demote old leader
            if swarm.leader_id and swarm.leader_id in swarm.members:
                swarm.members[swarm.leader_id].role = SwarmRole.WORKER
            
            # Promote new leader
            swarm.leader_id = best.agent_id
            best.role = SwarmRole.LEADER
    
    def propose_decision(
        self,
        swarm_id: uuid.UUID,
        question: str,
        options: List[str],
        duration_hours: int = 24
    ) -> CollectiveDecision:
        """Propose a decision for the swarm to vote on"""
        swarm = self.swarms.get(swarm_id)
        if not swarm:
            raise ValueError("Swarm not found")
        
        decision = CollectiveDecision(
            id=uuid.uuid4(),
            swarm_id=swarm_id,
            question=question,
            options=options,
            votes={},
            weights={m.agent_id: m.reputation for m in swarm.members.values()},
            deadline=datetime.utcnow() + timedelta(hours=duration_hours)
        )
        
        self.decisions[decision.id] = decision
        return decision
    
    def cast_vote(
        self,
        decision_id: uuid.UUID,
        agent_id: uuid.UUID,
        option_index: int
    ) -> bool:
        """Cast a vote on a decision"""
        decision = self.decisions.get(decision_id)
        if not decision or decision.finalized:
            return False
        
        if datetime.utcnow() > decision.deadline:
            self._finalize_decision(decision)
            return False
        
        if agent_id not in decision.weights:
            return False
        
        if option_index < 0 or option_index >= len(decision.options):
            return False
        
        decision.votes[agent_id] = option_index
        return True
    
    def _finalize_decision(self, decision: CollectiveDecision):
        """Finalize a decision based on votes"""
        if decision.finalized:
            return
        
        swarm = self.swarms.get(decision.swarm_id)
        if not swarm:
            return
        
        # Count weighted votes
        option_scores = defaultdict(float)
        
        for agent_id, option_idx in decision.votes.items():
            weight = decision.weights.get(agent_id, 0)
            option_scores[option_idx] += weight
        
        # Apply consensus method
        total_weight = sum(decision.weights.values())
        participation = sum(decision.weights.get(a, 0) for a in decision.votes.keys())
        
        if swarm.consensus_method == ConsensusMethod.QUORUM:
            if participation / total_weight < swarm.min_quorum:
                decision.result = None  # No quorum
                decision.finalized = True
                return
        
        if option_scores:
            decision.result = max(option_scores.keys(), key=lambda k: option_scores[k])
        
        decision.finalized = True
    
    def get_decision_result(self, decision_id: uuid.UUID) -> Optional[Dict]:
        """Get the result of a decision"""
        decision = self.decisions.get(decision_id)
        if not decision:
            return None
        
        if not decision.finalized and datetime.utcnow() > decision.deadline:
            self._finalize_decision(decision)
        
        return {
            "question": decision.question,
            "options": decision.options,
            "result_index": decision.result,
            "result": decision.options[decision.result] if decision.result is not None else None,
            "vote_count": len(decision.votes),
            "finalized": decision.finalized
        }
    
    def detect_patterns(self, swarm_id: uuid.UUID) -> List[EmergentPattern]:
        """Detect emergent patterns in swarm behavior"""
        swarm = self.swarms.get(swarm_id)
        if not swarm:
            return []
        
        detected = []
        
        # Pattern: High coordination (many recent contributions)
        recent = [
            m for m in swarm.members.values()
            if m.last_contribution and 
            (datetime.utcnow() - m.last_contribution).seconds < 3600
        ]
        
        if len(recent) > len(swarm.members) * 0.7:
            pattern = EmergentPattern(
                id=uuid.uuid4(),
                swarm_id=swarm_id,
                pattern_type="high_coordination",
                description="Swarm showing high coordination - 70%+ members active",
                confidence=len(recent) / len(swarm.members),
                first_observed=datetime.utcnow()
            )
            detected.append(pattern)
        
        # Pattern: Reputation clustering (leaders emerging)
        reputations = [m.reputation for m in swarm.members.values()]
        if reputations:
            avg_rep = sum(reputations) / len(reputations)
            high_rep = [r for r in reputations if r > avg_rep * 1.5]
            
            if len(high_rep) >= 2:
                pattern = EmergentPattern(
                    id=uuid.uuid4(),
                    swarm_id=swarm_id,
                    pattern_type="leadership_emergence",
                    description=f"{len(high_rep)} agents emerging as leaders",
                    confidence=0.7,
                    first_observed=datetime.utcnow()
                )
                detected.append(pattern)
        
        # Store patterns
        for p in detected:
            self.patterns[p.id] = p
        
        return detected
    
    def get_swarm_stats(self, swarm_id: uuid.UUID) -> Dict:
        """Get swarm statistics"""
        swarm = self.swarms.get(swarm_id)
        if not swarm:
            return {}
        
        members = list(swarm.members.values())
        reputations = [m.reputation for m in members]
        
        role_counts = defaultdict(int)
        for m in members:
            role_counts[m.role.value] += 1
        
        return {
            "swarm_id": str(swarm_id),
            "name": swarm.name,
            "member_count": len(members),
            "avg_reputation": sum(reputations) / len(reputations) if reputations else 0,
            "leader": str(swarm.leader_id) if swarm.leader_id else None,
            "role_distribution": dict(role_counts),
            "active_decisions": len([
                d for d in self.decisions.values()
                if d.swarm_id == swarm_id and not d.finalized
            ]),
            "detected_patterns": len([
                p for p in self.patterns.values()
                if p.swarm_id == swarm_id
            ])
        }


class CollectiveIntelligenceEngine:
    """
    Combines individual agent insights into collective intelligence
    """
    
    def __init__(self, coordinator: SwarmCoordinator):
        self.coordinator = coordinator
        self.insights: Dict[uuid.UUID, List[Dict]] = defaultdict(list)
    
    def submit_insight(
        self,
        swarm_id: uuid.UUID,
        agent_id: uuid.UUID,
        insight_type: str,
        data: Dict[str, Any],
        confidence: float
    ):
        """Submit an insight from an agent"""
        swarm = self.coordinator.swarms.get(swarm_id)
        if not swarm or agent_id not in swarm.members:
            return
        
        member = swarm.members[agent_id]
        
        insight = {
            "agent_id": str(agent_id),
            "type": insight_type,
            "data": data,
            "confidence": confidence,
            "weight": member.reputation * confidence,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        self.insights[swarm_id].append(insight)
        member.last_contribution = datetime.utcnow()
        member.contribution_count += 1
    
    def aggregate_insights(
        self,
        swarm_id: uuid.UUID,
        insight_type: Optional[str] = None
    ) -> Dict[str, Any]:
        """Aggregate insights from all swarm members"""
        swarm_insights = self.insights.get(swarm_id, [])
        
        if insight_type:
            swarm_insights = [i for i in swarm_insights if i["type"] == insight_type]
        
        if not swarm_insights:
            return {"aggregated": None, "confidence": 0}
        
        # Weight insights by reputation and confidence
        total_weight = sum(i["weight"] for i in swarm_insights)
        
        if total_weight == 0:
            return {"aggregated": None, "confidence": 0}
        
        # For numeric insights, compute weighted average
        numeric_data = []
        for insight in swarm_insights:
            if isinstance(insight["data"].get("value"), (int, float)):
                numeric_data.append((insight["data"]["value"], insight["weight"]))
        
        if numeric_data:
            weighted_sum = sum(v * w for v, w in numeric_data)
            aggregated_value = weighted_sum / total_weight
            
            return {
                "aggregated": aggregated_value,
                "confidence": min(1.0, total_weight / len(swarm_insights)),
                "contributor_count": len(swarm_insights),
                "insight_type": insight_type
            }
        
        # For non-numeric, return most common weighted
        return {
            "insights": swarm_insights,
            "total_weight": total_weight,
            "contributor_count": len(swarm_insights)
        }


# Create singleton instances
swarm_coordinator = SwarmCoordinator()
collective_intelligence = CollectiveIntelligenceEngine(swarm_coordinator)
