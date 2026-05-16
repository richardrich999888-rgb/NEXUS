"""
Autonomous Decision Framework - Phase 5
Goal optimization and risk assessment for autonomous agents
"""

import uuid
import math
from typing import Dict, List, Optional, Any, Tuple, Callable
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict

from src.models import EndocrineState, Hormone, HealthStatus


class DecisionType(str, Enum):
    ACCEPT_TASK = "accept_task"
    REJECT_TASK = "reject_task"
    COLLABORATE = "collaborate"
    STAKE = "stake"
    UNSTAKE = "unstake"
    VOTE = "vote"
    BRIDGE = "bridge"
    DELEGATE = "delegate"


class RiskLevel(str, Enum):
    MINIMAL = "minimal"
    LOW = "low"
    MODERATE = "moderate"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class Goal:
    """Agent goal"""
    id: uuid.UUID
    agent_id: uuid.UUID
    description: str
    priority: float  # 0-1
    target_value: float
    current_value: float
    deadline: Optional[datetime]
    achieved: bool = False


@dataclass
class DecisionContext:
    """Context for making a decision"""
    agent_id: uuid.UUID
    agent_state: EndocrineState
    reputation: float
    stake_amount: float
    available_balance: float
    active_tasks: int
    recent_success_rate: float
    swarm_memberships: int


@dataclass
class RiskAssessment:
    """Risk assessment result"""
    risk_level: RiskLevel
    risk_score: float  # 0-1
    factors: Dict[str, float]
    mitigations: List[str]
    recommendation: str


@dataclass
class Decision:
    """A decision made by an autonomous agent"""
    id: uuid.UUID
    agent_id: uuid.UUID
    decision_type: DecisionType
    context: DecisionContext
    risk_assessment: RiskAssessment
    outcome_prediction: float
    confidence: float
    reasoning: List[str]
    selected_action: Dict[str, Any]
    timestamp: datetime


class GoalOptimizer:
    """
    Optimizes agent goals and tracks progress
    """
    
    def __init__(self):
        self.goals: Dict[uuid.UUID, List[Goal]] = defaultdict(list)
    
    def add_goal(
        self,
        agent_id: uuid.UUID,
        description: str,
        target_value: float,
        priority: float = 0.5,
        deadline: Optional[datetime] = None
    ) -> Goal:
        """Add a new goal for an agent"""
        goal = Goal(
            id=uuid.uuid4(),
            agent_id=agent_id,
            description=description,
            priority=min(1.0, max(0.0, priority)),
            target_value=target_value,
            current_value=0.0,
            deadline=deadline
        )
        
        self.goals[agent_id].append(goal)
        return goal
    
    def update_progress(
        self,
        agent_id: uuid.UUID,
        goal_id: uuid.UUID,
        new_value: float
    ) -> Optional[Goal]:
        """Update progress towards a goal"""
        for goal in self.goals.get(agent_id, []):
            if goal.id == goal_id:
                goal.current_value = new_value
                if goal.current_value >= goal.target_value:
                    goal.achieved = True
                return goal
        return None
    
    def get_priority_goals(
        self,
        agent_id: uuid.UUID,
        top_n: int = 5
    ) -> List[Goal]:
        """Get top priority goals for an agent"""
        agent_goals = self.goals.get(agent_id, [])
        active = [g for g in agent_goals if not g.achieved]
        
        # Sort by priority, then by deadline urgency
        def urgency_score(g: Goal) -> float:
            score = g.priority
            if g.deadline:
                time_left = (g.deadline - datetime.utcnow()).total_seconds()
                if time_left > 0:
                    urgency = 1.0 / (1.0 + math.log1p(time_left / 3600))
                    score += urgency * 0.3
                else:
                    score += 0.5  # Overdue
            return score
        
        active.sort(key=urgency_score, reverse=True)
        return active[:top_n]
    
    def calculate_goal_alignment(
        self,
        agent_id: uuid.UUID,
        action: Dict[str, Any]
    ) -> float:
        """Calculate how well an action aligns with agent goals"""
        goals = self.get_priority_goals(agent_id)
        if not goals:
            return 0.5
        
        alignment = 0.0
        action_type = action.get("type", "")
        
        for goal in goals:
            # Simple keyword matching (in production, use embeddings)
            goal_keywords = goal.description.lower().split()
            action_desc = str(action).lower()
            
            matches = sum(1 for k in goal_keywords if k in action_desc)
            if goal_keywords:
                goal_alignment = matches / len(goal_keywords)
                alignment += goal_alignment * goal.priority
        
        return min(1.0, alignment / len(goals))


class RiskAssessor:
    """
    Assesses risks for autonomous decisions
    """
    
    RISK_THRESHOLDS = {
        RiskLevel.MINIMAL: 0.1,
        RiskLevel.LOW: 0.3,
        RiskLevel.MODERATE: 0.5,
        RiskLevel.HIGH: 0.7,
        RiskLevel.CRITICAL: 1.0
    }
    
    def __init__(self):
        self.risk_history: Dict[uuid.UUID, List[float]] = defaultdict(list)
    
    def assess_risk(
        self,
        context: DecisionContext,
        decision_type: DecisionType,
        action_params: Dict[str, Any]
    ) -> RiskAssessment:
        """Assess risk for a decision"""
        factors = {}
        mitigations = []
        
        # Factor 1: Reputation risk
        if context.reputation < 0.3:
            factors["low_reputation"] = 0.3
            mitigations.append("Build reputation before high-risk actions")
        elif context.reputation > 0.7:
            factors["high_reputation"] = -0.1  # Reduces risk
        
        # Factor 2: Financial exposure
        amount = action_params.get("amount", 0)
        if context.available_balance > 0:
            exposure = amount / context.available_balance
            if exposure > 0.5:
                factors["high_exposure"] = exposure * 0.5
                mitigations.append("Reduce position size")
            elif exposure > 0.2:
                factors["moderate_exposure"] = exposure * 0.2
        
        # Factor 3: Task overload
        if context.active_tasks > 5:
            factors["task_overload"] = (context.active_tasks - 5) * 0.05
            mitigations.append("Complete existing tasks first")
        
        # Factor 4: Recent performance
        if context.recent_success_rate < 0.5:
            factors["poor_performance"] = (0.5 - context.recent_success_rate) * 0.4
            mitigations.append("Focus on high-confidence tasks")
        
        # Factor 5: Hormone-based stress indicators
        cortisol = context.agent_state.levels.get(Hormone.CORTISOL, 0.5)
        if cortisol > 0.7:
            factors["elevated_stress"] = (cortisol - 0.7) * 0.3
            mitigations.append("Allow recovery before major decisions")
        
        # Factor 6: Decision-specific risks
        if decision_type == DecisionType.BRIDGE:
            factors["bridge_complexity"] = 0.15
            mitigations.append("Verify bridge contract before transfer")
        elif decision_type == DecisionType.STAKE:
            lock_days = action_params.get("lock_days", 30)
            if lock_days > 90:
                factors["long_lockup"] = 0.1
        
        # Calculate total risk score
        risk_score = sum(max(0, f) for f in factors.values())
        risk_score = min(1.0, max(0.0, risk_score))
        
        # Determine risk level
        risk_level = RiskLevel.MINIMAL
        for level, threshold in self.RISK_THRESHOLDS.items():
            if risk_score <= threshold:
                risk_level = level
                break
        
        # Generate recommendation
        if risk_level in [RiskLevel.HIGH, RiskLevel.CRITICAL]:
            recommendation = "Avoid or significantly reduce exposure"
        elif risk_level == RiskLevel.MODERATE:
            recommendation = "Proceed with caution and monitoring"
        else:
            recommendation = "Acceptable risk level"
        
        # Record for history
        self.risk_history[context.agent_id].append(risk_score)
        
        return RiskAssessment(
            risk_level=risk_level,
            risk_score=risk_score,
            factors=factors,
            mitigations=mitigations,
            recommendation=recommendation
        )
    
    def get_risk_trend(self, agent_id: uuid.UUID) -> str:
        """Get risk trend for an agent"""
        history = self.risk_history.get(agent_id, [])
        if len(history) < 3:
            return "insufficient_data"
        
        recent = history[-5:]
        avg_recent = sum(recent) / len(recent)
        avg_older = sum(history[:-5]) / max(1, len(history) - 5)
        
        if avg_recent > avg_older * 1.2:
            return "increasing"
        elif avg_recent < avg_older * 0.8:
            return "decreasing"
        return "stable"


class AutonomousDecisionEngine:
    """
    Makes autonomous decisions for agents
    
    Combines:
    - Goal alignment
    - Risk assessment
    - Outcome prediction
    - Multi-criteria optimization
    """
    
    def __init__(self):
        self.goal_optimizer = GoalOptimizer()
        self.risk_assessor = RiskAssessor()
        self.decision_history: Dict[uuid.UUID, List[Decision]] = defaultdict(list)
        
        # Decision weights
        self.weights = {
            "goal_alignment": 0.3,
            "risk_adjusted_return": 0.25,
            "reputation_impact": 0.2,
            "resource_efficiency": 0.15,
            "time_sensitivity": 0.1
        }
    
    def evaluate_action(
        self,
        context: DecisionContext,
        action: Dict[str, Any]
    ) -> Tuple[float, Dict[str, float]]:
        """Evaluate an action across multiple criteria"""
        scores = {}
        
        # Goal alignment
        scores["goal_alignment"] = self.goal_optimizer.calculate_goal_alignment(
            context.agent_id, action
        )
        
        # Risk-adjusted return
        expected_return = action.get("expected_return", 0.5)
        risk = self.risk_assessor.assess_risk(
            context,
            DecisionType(action.get("type", "accept_task")),
            action
        )
        scores["risk_adjusted_return"] = expected_return * (1 - risk.risk_score)
        
        # Reputation impact
        rep_impact = action.get("reputation_impact", 0)
        scores["reputation_impact"] = 0.5 + rep_impact * 0.5
        
        # Resource efficiency
        cost = action.get("cost", 0)
        if context.available_balance > 0:
            efficiency = 1 - (cost / context.available_balance)
            scores["resource_efficiency"] = max(0, efficiency)
        else:
            scores["resource_efficiency"] = 0.5
        
        # Time sensitivity
        deadline = action.get("deadline")
        if deadline:
            time_left = (datetime.fromisoformat(deadline) - datetime.utcnow()).total_seconds()
            if time_left > 0:
                scores["time_sensitivity"] = 1.0 / (1.0 + math.log1p(time_left / 3600))
            else:
                scores["time_sensitivity"] = 0  # Missed deadline
        else:
            scores["time_sensitivity"] = 0.5
        
        # Weighted sum
        total = sum(scores[k] * self.weights[k] for k in self.weights)
        
        return total, scores
    
    def make_decision(
        self,
        context: DecisionContext,
        options: List[Dict[str, Any]]
    ) -> Decision:
        """Make an autonomous decision from available options"""
        if not options:
            raise ValueError("No options provided")
        
        best_option = None
        best_score = -1
        best_scores = {}
        reasoning = []
        
        for option in options:
            score, scores = self.evaluate_action(context, option)
            
            reasoning.append(f"Option '{option.get('name', 'unknown')}': score={score:.3f}")
            
            if score > best_score:
                best_score = score
                best_option = option
                best_scores = scores
        
        # Get risk assessment for selected option
        decision_type = DecisionType(best_option.get("type", "accept_task"))
        risk = self.risk_assessor.assess_risk(context, decision_type, best_option)
        
        # Add reasoning from risk assessment
        if risk.mitigations:
            reasoning.append(f"Risk mitigations: {', '.join(risk.mitigations)}")
        
        # Calculate confidence based on score margin and risk
        confidence = best_score * (1 - risk.risk_score * 0.5)
        
        decision = Decision(
            id=uuid.uuid4(),
            agent_id=context.agent_id,
            decision_type=decision_type,
            context=context,
            risk_assessment=risk,
            outcome_prediction=best_score,
            confidence=confidence,
            reasoning=reasoning,
            selected_action=best_option,
            timestamp=datetime.utcnow()
        )
        
        self.decision_history[context.agent_id].append(decision)
        
        return decision
    
    def should_proceed(
        self,
        context: DecisionContext,
        action: Dict[str, Any],
        min_confidence: float = 0.4
    ) -> Tuple[bool, Decision]:
        """Determine if an action should proceed"""
        decision = self.make_decision(context, [action])
        
        should = (
            decision.confidence >= min_confidence and
            decision.risk_assessment.risk_level not in [RiskLevel.HIGH, RiskLevel.CRITICAL]
        )
        
        return should, decision
    
    def get_agent_decision_stats(self, agent_id: uuid.UUID) -> Dict:
        """Get decision statistics for an agent"""
        decisions = self.decision_history.get(agent_id, [])
        
        if not decisions:
            return {"total": 0}
        
        by_type = defaultdict(int)
        avg_confidence = 0
        avg_risk = 0
        
        for d in decisions:
            by_type[d.decision_type.value] += 1
            avg_confidence += d.confidence
            avg_risk += d.risk_assessment.risk_score
        
        return {
            "total": len(decisions),
            "by_type": dict(by_type),
            "avg_confidence": avg_confidence / len(decisions),
            "avg_risk": avg_risk / len(decisions),
            "risk_trend": self.risk_assessor.get_risk_trend(agent_id)
        }


# Create singleton instances
goal_optimizer = GoalOptimizer()
risk_assessor = RiskAssessor()
decision_engine = AutonomousDecisionEngine()
