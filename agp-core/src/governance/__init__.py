"""
AGP-CORE: Agent Governance Protocol
The core governance layer for AI agent behavior.
"""

from .behavioral_rag import (
    BehavioralRAG, BehaviorRecord, ActionType, Outcome,
    behavioral_rag
)
from .rules import (
    GovernanceRule, GovernanceRulesEngine, RuleAction, RulePriority,
    rules_engine
)
from .alignment import (
    AlignmentVerifier, AlignmentScore,
    alignment_verifier
)
from .enforcer import (
    ProtocolEnforcer, GovernanceDecision, Decision,
    protocol_enforcer
)
from .impact import (
    ImpactAnalyzer, ActionImpact, ActionCategory, RiskLevel,
    impact_analyzer
)
from .anomaly import (
    AnomalyDetector, AnomalyAlert, AnomalyType,
    anomaly_detector
)

__all__ = [
    # Behavioral RAG
    "BehavioralRAG",
    "BehaviorRecord",
    "ActionType",
    "Outcome",
    "behavioral_rag",
    
    # Rules Engine
    "GovernanceRule",
    "GovernanceRulesEngine",
    "RuleAction",
    "RulePriority",
    "rules_engine",
    
    # Alignment
    "AlignmentVerifier",
    "AlignmentScore",
    "alignment_verifier",
    
    # Enforcer
    "ProtocolEnforcer",
    "GovernanceDecision",
    "Decision",
    "protocol_enforcer",
]

# Convenience API
class AgentGovernanceProtocol:
    """
    High-level API for the Agent Governance Protocol.
    """
    
    def __init__(self):
        self.rag = behavioral_rag
        self.rules = rules_engine
        self.verifier = alignment_verifier
        self.enforcer = protocol_enforcer
    
    async def check(self, agent_id: str, agent_name: str,
                    action: str, details: dict = None) -> GovernanceDecision:
        """Check if an action is allowed"""
        return await self.enforcer.enforce(
            agent_id=agent_id,
            agent_name=agent_name,
            action_type=action,
            action_details=details or {}
        )
    
    def get_alignment(self, agent_id: str) -> float:
        """Get agent's current alignment score"""
        return self.verifier.get_alignment(agent_id)
    
    def get_history(self, agent_id: str, limit: int = 50) -> list:
        """Get agent's behavioral history"""
        return self.rag.retrieve_by_agent(agent_id, limit=limit)
    
    def add_rule(self, rule: GovernanceRule):
        """Add a governance rule"""
        self.rules.add_rule(rule)
    
    def get_stats(self) -> dict:
        """Get governance statistics"""
        return self.enforcer.get_governance_stats()

# Global AGP instance
agp = AgentGovernanceProtocol()
