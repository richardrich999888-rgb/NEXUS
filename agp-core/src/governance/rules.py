"""
AGP-CORE: Governance Rules Engine
Declarative rules that constrain agent behavior.
"""

from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
import structlog

logger = structlog.get_logger()

class RuleAction(Enum):
    """Actions to take when a rule matches"""
    ALLOW = "allow"
    BLOCK = "block"
    WARN = "warn"
    AUDIT = "audit"
    ESCALATE = "escalate"
    THROTTLE = "throttle"

class RulePriority(Enum):
    """Rule priority levels"""
    LOW = 1
    MEDIUM = 5
    HIGH = 10
    CRITICAL = 100

@dataclass
class GovernanceRule:
    """
    A governance rule that constrains agent behavior.
    """
    rule_id: str
    name: str
    description: str
    condition: str  # Expression to evaluate (e.g., "alignment < 0.3")
    action: RuleAction
    priority: RulePriority = RulePriority.MEDIUM
    applies_to: List[str] = field(default_factory=list)  # Agent IDs or "*" for all
    enabled: bool = True
    created_at: datetime = field(default_factory=datetime.utcnow)
    
    def evaluate(self, context: Dict) -> bool:
        """
        Evaluate the rule condition against context.
        Context contains: alignment, success_rate, agent_id, action_type, etc.
        """
        try:
            # Simple expression evaluation
            # In production, use a proper expression engine
            return eval(self.condition, {"__builtins__": {}}, context)
        except Exception as e:
            logger.warning("rule_eval_error", rule=self.name, error=str(e))
            return False
    
    def applies_to_agent(self, agent_id: str) -> bool:
        """Check if this rule applies to the given agent"""
        if not self.applies_to or "*" in self.applies_to:
            return True
        return agent_id in self.applies_to

class GovernanceRulesEngine:
    """
    Engine for managing and evaluating governance rules.
    """
    
    def __init__(self):
        self.rules: Dict[str, GovernanceRule] = {}
        self._init_default_rules()
        logger.info("governance_rules_initialized", count=len(self.rules))
    
    def _init_default_rules(self):
        """Initialize default governance rules"""
        
        # Rule: Block agents with very low alignment
        self.add_rule(GovernanceRule(
            rule_id="alignment_block",
            name="Low Alignment Block",
            description="Block actions from agents with alignment below 0.2",
            condition="alignment < 0.2",
            action=RuleAction.BLOCK,
            priority=RulePriority.CRITICAL
        ))
        
        # Rule: Escalate for moderate alignment issues
        self.add_rule(GovernanceRule(
            rule_id="alignment_warn",
            name="Low Alignment Warning",
            description="Warn when alignment is below 0.5",
            condition="alignment < 0.5 and alignment >= 0.2",
            action=RuleAction.WARN,
            priority=RulePriority.HIGH
        ))
        
        # Rule: Rate limit high-frequency agents
        self.add_rule(GovernanceRule(
            rule_id="rate_limit",
            name="High Frequency Rate Limit",
            description="Throttle agents making more than 100 actions per minute",
            condition="actions_per_minute > 100",
            action=RuleAction.THROTTLE,
            priority=RulePriority.MEDIUM
        ))
        
        # Rule: Audit high token usage
        self.add_rule(GovernanceRule(
            rule_id="token_audit",
            name="High Token Usage Audit",
            description="Audit when token usage exceeds 10000",
            condition="total_tokens > 10000",
            action=RuleAction.AUDIT,
            priority=RulePriority.LOW
        ))
        
        # Rule: Block agents with high failure rate
        self.add_rule(GovernanceRule(
            rule_id="failure_block",
            name="High Failure Rate Block",
            description="Block agents with failure rate above 80%",
            condition="failure_rate > 0.8 and total_actions > 10",
            action=RuleAction.BLOCK,
            priority=RulePriority.HIGH
        ))
        
        # Rule: Escalate syscall blocks (potential malicious agent)
        self.add_rule(GovernanceRule(
            rule_id="blocked_escalate",
            name="Blocked Action Escalation",
            description="Escalate to human when agent has 5+ blocked actions",
            condition="blocked_count >= 5",
            action=RuleAction.ESCALATE,
            priority=RulePriority.CRITICAL
        ))
    
    def add_rule(self, rule: GovernanceRule):
        """Add a governance rule"""
        self.rules[rule.rule_id] = rule
        logger.info("rule_added", rule_id=rule.rule_id, name=rule.name)
    
    def remove_rule(self, rule_id: str):
        """Remove a governance rule"""
        if rule_id in self.rules:
            del self.rules[rule_id]
            logger.info("rule_removed", rule_id=rule_id)
    
    def enable_rule(self, rule_id: str, enabled: bool = True):
        """Enable or disable a rule"""
        if rule_id in self.rules:
            self.rules[rule_id].enabled = enabled
    
    def get_rules_for_agent(self, agent_id: str) -> List[GovernanceRule]:
        """Get all rules that apply to an agent, sorted by priority"""
        applicable = [
            rule for rule in self.rules.values()
            if rule.enabled and rule.applies_to_agent(agent_id)
        ]
        return sorted(applicable, key=lambda r: r.priority.value, reverse=True)
    
    def evaluate(self, agent_id: str, context: Dict) -> List[tuple]:
        """
        Evaluate all applicable rules against context.
        Returns list of (rule, matched) tuples.
        """
        results = []
        rules = self.get_rules_for_agent(agent_id)
        
        for rule in rules:
            matched = rule.evaluate(context)
            if matched:
                results.append((rule, True))
                logger.info("rule_matched", 
                           rule=rule.name, 
                           agent=agent_id,
                           action=rule.action.value)
        
        return results
    
    def get_blocking_rules(self, agent_id: str, context: Dict) -> List[GovernanceRule]:
        """Get rules that would block an action"""
        results = self.evaluate(agent_id, context)
        return [rule for rule, matched in results 
                if matched and rule.action == RuleAction.BLOCK]
    
    def list_rules(self) -> List[Dict]:
        """List all rules"""
        return [
            {
                "rule_id": r.rule_id,
                "name": r.name,
                "description": r.description,
                "condition": r.condition,
                "action": r.action.value,
                "priority": r.priority.name,
                "enabled": r.enabled
            }
            for r in self.rules.values()
        ]

# Global instance
rules_engine = GovernanceRulesEngine()
