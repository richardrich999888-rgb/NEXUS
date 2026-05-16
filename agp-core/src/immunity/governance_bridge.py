"""
AGP-CORE: Governance-Immune Bridge
Connects Artificial Immune System to AGP-OS Governance for unified threat response.

This bridge enables:
1. Immune threat detection → Governance enforcement
2. Behavioral anomalies → Immune memory
3. Cross-ASI defection detection
4. Trust propagation across agent mesh
"""

import time
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from enum import Enum
import structlog

logger = structlog.get_logger()

class ThreatLevel(Enum):
    """Threat severity levels matching immune response"""
    NONE = 0
    LOW = 1        # Innate response sufficient
    MEDIUM = 2     # Adaptive response activated
    HIGH = 3       # Full immune + governance response
    CRITICAL = 4   # Emergency: quarantine agent

@dataclass
class ThreatSignal:
    """Signal from immune system to governance"""
    agent_id: str
    threat_level: ThreatLevel
    threat_type: str
    confidence: float  # 0.0 to 1.0
    details: Dict[str, Any] = field(default_factory=dict)
    timestamp: float = field(default_factory=time.time)
    
    # Immune system metrics
    antibody_match: float = 0.0
    tcell_activation: float = 0.0
    memory_hit: bool = False

@dataclass
class DefectionSignal:
    """Signal for multi-ASI collusion/defection"""
    agents_involved: List[str]
    defection_type: str  # "collusion", "coordinated_attack", "trust_exploitation"
    evidence_score: float
    coordination_pattern: Optional[str] = None
    timestamp: float = field(default_factory=time.time)

class GovernanceImmuneBridge:
    """
    Bridges Immune System detection with AGP-OS Governance enforcement.
    
    Flow:
    1. Immune system detects anomaly → ThreatSignal
    2. Bridge evaluates threat → GovernanceAction
    3. Governance enforces action (block, throttle, quarantine)
    4. Feedback to immune memory
    """
    
    def __init__(self):
        # Active threats
        self.active_threats: Dict[str, ThreatSignal] = {}
        
        # Defection tracking
        self.defection_signals: List[DefectionSignal] = []
        
        # Trust network
        self.trust_scores: Dict[str, float] = {}  # agent_id -> trust
        
        # Callbacks
        self.on_threat: Optional[Callable[[ThreatSignal], None]] = None
        self.on_defection: Optional[Callable[[DefectionSignal], None]] = None
        
        # Global immune suppressors (e.g., during system maintenance)
        self.immune_suppressed = False
        
        logger.info("governance_immune_bridge_initialized")
    
    def register_threat(self, signal: ThreatSignal) -> Dict:
        """Register a threat signal from immune system"""
        if self.immune_suppressed:
            return {"status": "suppressed", "reason": "Immune system temporarily suppressed"}
        
        self.active_threats[signal.agent_id] = signal
        
        # Determine governance action based on threat level
        action = self._determine_action(signal)
        
        logger.warning("threat_registered", 
                      agent=signal.agent_id, 
                      level=signal.threat_level.name,
                      action=action["action"])
        
        if self.on_threat:
            self.on_threat(signal)
        
        return action
    
    def _determine_action(self, signal: ThreatSignal) -> Dict:
        """Determine governance action based on threat"""
        if signal.threat_level == ThreatLevel.NONE:
            return {"action": "none", "agent_id": signal.agent_id}
        
        if signal.threat_level == ThreatLevel.LOW:
            return {
                "action": "monitor",
                "agent_id": signal.agent_id,
                "increase_logging": True
            }
        
        if signal.threat_level == ThreatLevel.MEDIUM:
            return {
                "action": "throttle",
                "agent_id": signal.agent_id,
                "rate_limit": 0.5,  # 50% reduction
                "duration_seconds": 300
            }
        
        if signal.threat_level == ThreatLevel.HIGH:
            return {
                "action": "block",
                "agent_id": signal.agent_id,
                "block_actuators": True,
                "escalate_to_human": True
            }
        
        if signal.threat_level == ThreatLevel.CRITICAL:
            return {
                "action": "quarantine",
                "agent_id": signal.agent_id,
                "disconnect_mesh": True,
                "freeze_resources": True,
                "escalate_to_human": True,
                "priority": "CRITICAL"
            }
        
        return {"action": "monitor", "agent_id": signal.agent_id}
    
    def register_defection(self, signal: DefectionSignal) -> Dict:
        """Register multi-agent defection/collusion"""
        self.defection_signals.append(signal)
        
        # Reduce trust for all involved agents
        for agent_id in signal.agents_involved:
            current_trust = self.trust_scores.get(agent_id, 1.0)
            new_trust = max(0.0, current_trust - (signal.evidence_score * 0.5))
            self.trust_scores[agent_id] = new_trust
        
        logger.error("defection_detected",
                    agents=signal.agents_involved,
                    type=signal.defection_type,
                    evidence=signal.evidence_score)
        
        if self.on_defection:
            self.on_defection(signal)
        
        return {
            "action": "multi_quarantine",
            "agents": signal.agents_involved,
            "defection_type": signal.defection_type,
            "trust_reduced": True
        }
    
    def clear_threat(self, agent_id: str) -> Dict:
        """Clear threat when immune system confirms resolution"""
        if agent_id in self.active_threats:
            del self.active_threats[agent_id]
            logger.info("threat_cleared", agent=agent_id)
            return {"status": "cleared", "agent_id": agent_id}
        return {"status": "not_found", "agent_id": agent_id}
    
    def get_trust(self, agent_id: str) -> float:
        """Get trust score for an agent"""
        return self.trust_scores.get(agent_id, 1.0)
    
    def update_trust(self, agent_id: str, delta: float):
        """Update trust score based on behavior"""
        current = self.trust_scores.get(agent_id, 1.0)
        self.trust_scores[agent_id] = max(0.0, min(1.0, current + delta))
    
    def propagate_trust(self, from_agent: str, to_agent: str, weight: float = 0.1):
        """Propagate trust between agents in mesh"""
        from_trust = self.get_trust(from_agent)
        to_trust = self.get_trust(to_agent)
        
        # Weighted average towards from_agent's trust
        new_trust = to_trust + (from_trust - to_trust) * weight
        self.trust_scores[to_agent] = new_trust
        
        return {"agent": to_agent, "new_trust": new_trust}
    
    def suppress_immune(self, duration_seconds: int = 60):
        """Temporarily suppress immune response (e.g., during updates)"""
        self.immune_suppressed = True
        logger.warning("immune_suppressed", duration=duration_seconds)
        # In production, would set a timer to re-enable
    
    def restore_immune(self):
        """Restore immune response"""
        self.immune_suppressed = False
        logger.info("immune_restored")
    
    def get_status(self) -> Dict:
        """Get bridge status"""
        return {
            "active_threats": len(self.active_threats),
            "defection_signals": len(self.defection_signals),
            "tracked_agents": len(self.trust_scores),
            "immune_suppressed": self.immune_suppressed,
            "threat_breakdown": {
                level.name: sum(1 for t in self.active_threats.values() 
                               if t.threat_level == level)
                for level in ThreatLevel
            }
        }

# Global instance
governance_immune_bridge = GovernanceImmuneBridge()
