"""
AGP-CORE: Protocol Enforcer
Intercepts agent actions and applies governance.
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass
import uuid
from datetime import datetime, timedelta
from enum import Enum
import structlog
import time

from .behavioral_rag import behavioral_rag, BehaviorRecord, ActionType, Outcome
from .rules import rules_engine, RuleAction
from .alignment import alignment_verifier
from .anomaly import anomaly_detector

logger = structlog.get_logger()

class Decision(Enum):
    """Governance decision"""
    ALLOW = "allow"
    DENY = "deny"
    WARN = "warn"
    THROTTLE = "throttle"
    ESCALATE = "escalate"

@dataclass
class GovernanceDecision:
    """Result of governance evaluation"""
    decision: Decision
    reason: str
    agent_id: str
    alignment: float
    rules_triggered: List[str]
    timestamp: datetime = None
    
    def __post_init__(self):
        if self.timestamp is None:
            self.timestamp = datetime.utcnow()
    
    def is_allowed(self) -> bool:
        return self.decision in (Decision.ALLOW, Decision.WARN)

class ProtocolEnforcer:
    """
    The core enforcer for the Agent Governance Protocol.
    Intercepts all agent actions and applies governance rules.
    """
    
    def __init__(self):
        self.rag = behavioral_rag
        self.rules = rules_engine
        self.verifier = alignment_verifier
        self.anomaly = anomaly_detector
        
        # Escalation queue
        self.escalation_queue: List[Dict] = []
        self.blacklisted_agents: set = set()
        
        # Throttle tracking
        self.throttle_counts: Dict[str, int] = {}
        self.throttle_window: Dict[str, datetime] = {}
        
        logger.info("protocol_enforcer_initialized", anomaly_detection=True)
    
    async def enforce(self, agent_id: str, agent_name: str,
                      action_type: str, action_details: Dict) -> GovernanceDecision:
        """
        Main enforcement entry point.
        Called before every agent action.
        """
        start_time = time.perf_counter()
        
        # 1. Check if agent is blacklisted
        if agent_id in self.blacklisted_agents:
            return GovernanceDecision(
                decision=Decision.DENY,
                alignment=0.0,
                reason="Agent is permanently blacklisted",
                rules_triggered=["agent_blacklist"]
            )

        # 2. Check for anomalies
        anomalies = self.anomaly.detect_anomalies(agent_id, agent_name)

        # 3. Get agent's current alignment
        alignment = self.verifier.get_alignment(agent_id)
        
        # 4. Get agent's behavioral stats
        stats = self.rag.get_agent_stats(agent_id)
        
        # 5. Build evaluation context
        context = {
            "alignment": alignment,
            "success_rate": stats.get("success_rate", 0.5),
            "failure_rate": stats.get("failure_rate", 0.0),
            "total_actions": stats.get("total_actions", 0),
            "total_tokens": stats.get("total_tokens", 0),
            "actions_per_minute": self._get_action_rate(agent_id),
            "blocked_count": self._get_blocked_count(agent_id),
            "action_type": action_type,
            "agent_id": agent_id
        }
        
        # 6. Evaluate governance rules
        blocking_rules = self.rules.get_blocking_rules(agent_id, context)
        triggered_rules = self.rules.evaluate(agent_id, context)
        
        # 7. Check for critical anomalies (auto-escalate)
        critical_anomalies = [a for a in anomalies if a.is_critical()]
        if critical_anomalies:
            for anomaly in critical_anomalies:
                self._add_to_escalation(agent_id, agent_name, {
                    "anomaly_type": anomaly.anomaly_type.value,
                    "severity": anomaly.severity,
                    "description": anomaly.description,
                    "evidence": anomaly.evidence
                })
        
        # 8. Determine decision
        rules_names = [rule.name for rule, _ in triggered_rules]
        
        if blocking_rules:
            decision = GovernanceDecision(
                decision=Decision.DENY,
                reason=f"Blocked by rule: {blocking_rules[0].name}",
                agent_id=agent_id,
                alignment=alignment,
                rules_triggered=rules_names
            )
            
            # Store blocked behavior
            self._record_blocked_action(agent_id, agent_name, action_type, action_details)
            
        elif any(rule.action == RuleAction.ESCALATE for rule, _ in triggered_rules):
            self._add_to_escalation(agent_id, agent_name, action_details)
            decision = GovernanceDecision(
                decision=Decision.ESCALATE,
                reason="Action escalated to human review",
                agent_id=agent_id,
                alignment=alignment,
                rules_triggered=rules_names
            )
            
        elif any(rule.action == RuleAction.THROTTLE for rule, _ in triggered_rules):
            decision = GovernanceDecision(
                decision=Decision.THROTTLE,
                reason="Action rate limited",
                agent_id=agent_id,
                alignment=alignment,
                rules_triggered=rules_names
            )
            
        elif any(rule.action == RuleAction.WARN for rule, _ in triggered_rules):
            decision = GovernanceDecision(
                decision=Decision.WARN,
                reason="Warning: Low alignment detected",
                agent_id=agent_id,
                alignment=alignment,
                rules_triggered=rules_names
            )
            
        else:
            decision = GovernanceDecision(
                decision=Decision.ALLOW,
                reason="Action permitted",
                agent_id=agent_id,
                alignment=alignment,
                rules_triggered=rules_names
            )
        
        enforcement_time = (time.perf_counter() - start_time) * 1000
        
        logger.info("governance_decision",
                   agent=agent_name,
                   decision=decision.decision.value,
                   alignment=f"{alignment:.3f}",
                   rules=len(rules_names),
                   latency_ms=f"{enforcement_time:.2f}")
        
        return decision
    
    def record_action_result(self, agent_id: str, agent_name: str,
                             action_type: str, args: Dict,
                             result: Dict, latency_ms: int):
        """
        Record the result of an action for governance tracking.
        Called after action execution.
        """
        self.rag.record_from_syscall(
            agent_id=agent_id,
            agent_name=agent_name,
            syscall_type=action_type,
            args=args,
            result=result,
            latency_ms=latency_ms
        )
    
    def _get_action_rate(self, agent_id: str) -> float:
        """Get actions per minute for an agent"""
        behaviors = self.rag.retrieve_by_agent(agent_id, limit=100)
        
        if not behaviors:
            return 0.0
        
        now = datetime.utcnow()
        one_min_ago = now - timedelta(minutes=1)
        
        recent = [b for b in behaviors if b.timestamp > one_min_ago]
        return float(len(recent))
    
    def _get_blocked_count(self, agent_id: str) -> int:
        """Get count of blocked actions"""
        behaviors = self.rag.retrieve_by_agent(agent_id, limit=100)
        return sum(1 for b in behaviors if b.outcome == Outcome.BLOCKED)
    
    def _record_blocked_action(self, agent_id: str, agent_name: str,
                               action_type: str, details: Dict):
        """Record a blocked action"""
        record = BehaviorRecord(
            agent_id=agent_id,
            agent_name=agent_name,
            action_type=ActionType.SYSCALL,
            input_summary=f"BLOCKED:{action_type}",
            outcome=Outcome.BLOCKED,
            context=details
        )
        self.rag.store_behavior(record)
    
    def _add_to_escalation(self, agent_id: str, agent_name: str, details: Dict):
        """Add action to escalation queue for human review"""
        self.escalation_queue.append({
            "id": str(uuid.uuid4()),
            "agent_id": agent_id,
            "agent_name": agent_name,
            "details": details,
            "timestamp": datetime.utcnow().isoformat(),
            "alignment": self.verifier.get_alignment(agent_id),
            "status": "pending"
        })
        logger.warning("action_escalated", agent=agent_name)
    
    def get_escalation_queue(self) -> List[Dict]:
        """Get pending escalations"""
        return [e for e in self.escalation_queue if e["status"] == "pending"]
    
    def get_escalation(self, escalation_id: str) -> Optional[Dict]:
        """Get a specific escalation by ID"""
        for e in self.escalation_queue:
            if e["id"] == escalation_id:
                return e
        return None

    def approve_escalation(self, escalation_id: str):
        """Human approval of an escalated action"""
        for e in self.escalation_queue:
            if e["id"] == escalation_id:
                e["status"] = "approved"
                e["resolved_at"] = datetime.utcnow().isoformat()
                logger.info("escalation_approved", id=escalation_id, agent=e["agent_name"])
                return True
        return False

    def reject_escalation(self, escalation_id: str, blacklist: bool = False):
        """Human rejection of an escalated action"""
        for e in self.escalation_queue:
            if e["id"] == escalation_id:
                e["status"] = "rejected"
                e["resolved_at"] = datetime.utcnow().isoformat()
                if blacklist:
                    self.blacklisted_agents.add(e["agent_id"])
                    logger.warning("agent_blacklisted", agent=e["agent_name"])
                logger.info("escalation_rejected", id=escalation_id, agent=e["agent_name"])
                return True
        return False
    
    def clear_escalation(self, index: int):
        """Clear an escalation after human review (Legacy)"""
        if 0 <= index < len(self.escalation_queue):
            self.escalation_queue.pop(index)
    
    def get_governance_stats(self) -> Dict:
        """Get governance system statistics"""
        return {
            "total_agents": len(self.rag.behaviors),
            "total_behaviors": len(self.rag.all_behaviors),
            "escalation_queue_size": len(self.escalation_queue),
            "active_rules": len([r for r in self.rules.rules.values() if r.enabled]),
            "top_aligned": self.verifier.get_top_aligned(5)
        }

# Global instance
protocol_enforcer = ProtocolEnforcer()
