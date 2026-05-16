"""
TELOS Bridge - Python wrapper for Rust TELOS Protocol

This module provides Python access to the TELOS commitment membrane,
enabling accountability for all governed agent actions.

In production, this would use PyO3/maturin for Rust FFI.
For now, implements the core TELOS concepts in Python.
"""

import time
import hashlib
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from enum import Enum
import structlog

logger = structlog.get_logger()


class ExecutionBlocked(Exception):
    """Raised when TELOS membrane rejects execution (entropy, authority, or trust)."""
    pass


class ConsequenceTier(Enum):
    """Consequence tiers - higher = more entropy required"""
    TRIVIAL = 1      # Read-only operations
    LOW = 2          # Reversible writes
    MEDIUM = 3       # Significant changes
    HIGH = 4         # Critical operations
    CRITICAL = 5     # Irreversible, high-impact

@dataclass
class Decision:
    """A decision to cross the commitment membrane"""
    decision_id: str
    action: str
    agent_id: str
    tier: ConsequenceTier
    context: Dict[str, Any] = field(default_factory=dict)
    timestamp: float = field(default_factory=time.time)
    
    @property
    def content_hash(self) -> str:
        """Content-addressed ID for this decision"""
        content = f"{self.action}:{self.agent_id}:{self.tier.value}:{self.timestamp}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]

@dataclass
class CrossingResult:
    """Result of attempting to cross the commitment membrane"""
    allowed: bool
    decision_id: str
    entropy_spent: int
    authority_verified: bool
    attestations: int
    reason: Optional[str] = None
    trust_delta: float = 0.0

@dataclass
class Authority:
    """Agent authority scope"""
    agent_id: str
    scopes: List[str]  # e.g., ["read:*", "write:models", "execute:safe"]
    delegated_from: Optional[str] = None
    constraints: List[str] = field(default_factory=list)
    revoked: bool = False

class EntropyMeter:
    """
    Tracks entropy budget - actions cost entropy proportional to consequence.
    
    Entropy refills over time (like a rate limiter with consequence scaling).
    """
    
    def __init__(self, budget: int = 10000, refill_rate: int = 100):
        self.budget = budget
        self.max_budget = budget
        self.refill_rate = refill_rate  # per minute
        self.last_refill = time.time()
        
        # Entropy costs by tier
        self.tier_costs = {
            ConsequenceTier.TRIVIAL: 1,
            ConsequenceTier.LOW: 10,
            ConsequenceTier.MEDIUM: 100,
            ConsequenceTier.HIGH: 500,
            ConsequenceTier.CRITICAL: 2000,
        }
    
    def refill(self):
        """Refill entropy based on time elapsed"""
        now = time.time()
        elapsed_minutes = (now - self.last_refill) / 60
        refill_amount = int(elapsed_minutes * self.refill_rate)
        if refill_amount > 0:
            self.budget = min(self.max_budget, self.budget + refill_amount)
            self.last_refill = now
    
    def cost(self, tier: ConsequenceTier) -> int:
        """Get entropy cost for a consequence tier"""
        return self.tier_costs.get(tier, 100)
    
    def spend(self, tier: ConsequenceTier) -> bool:
        """Attempt to spend entropy. Returns True if successful."""
        self.refill()
        cost = self.cost(tier)
        if self.budget >= cost:
            self.budget -= cost
            return True
        return False
    
    def get_status(self) -> Dict:
        self.refill()
        return {
            "budget": self.budget,
            "max_budget": self.max_budget,
            "refill_rate_per_min": self.refill_rate
        }

class AuthorityRegistry:
    """Registry of agent authorities and delegation chains"""
    
    def __init__(self):
        self.authorities: Dict[str, Authority] = {}
    
    def register(self, agent_id: str, scopes: List[str], 
                 delegated_from: Optional[str] = None) -> Authority:
        auth = Authority(
            agent_id=agent_id,
            scopes=scopes,
            delegated_from=delegated_from
        )
        self.authorities[agent_id] = auth
        return auth
    
    def verify(self, agent_id: str, required_scope: str) -> bool:
        """Verify agent has required authority scope"""
        if agent_id not in self.authorities:
            return False
        
        auth = self.authorities[agent_id]
        if auth.revoked:
            return False
        
        for scope in auth.scopes:
            if scope == "*" or scope == required_scope:
                return True
            # Wildcard matching (e.g., "write:*" matches "write:models")
            if scope.endswith(":*"):
                prefix = scope[:-1]
                if required_scope.startswith(prefix):
                    return True
        
        return False
    
    def revoke(self, agent_id: str):
        """Revoke an agent's authority"""
        if agent_id in self.authorities:
            self.authorities[agent_id].revoked = True

class TrustAccumulator:
    """Accumulates trust based on commitment history"""
    
    def __init__(self):
        self.scores: Dict[str, float] = {}  # agent_id -> trust score
        self.history: Dict[str, List[CrossingResult]] = {}  # agent_id -> history
    
    def get_trust(self, agent_id: str) -> float:
        return self.scores.get(agent_id, 0.5)  # Default 0.5
    
    def record(self, agent_id: str, result: CrossingResult):
        """Record a crossing result and update trust"""
        if agent_id not in self.history:
            self.history[agent_id] = []
        self.history[agent_id].append(result)
        
        # Update trust based on result
        current = self.get_trust(agent_id)
        if result.allowed:
            # Successful crossing increases trust slightly
            delta = 0.01 * result.attestations  # More attestations = more trust
        else:
            # Blocked crossing decreases trust
            delta = -0.05
        
        self.scores[agent_id] = max(0.0, min(1.0, current + delta))
        result.trust_delta = delta

class CommitmentMembrane:
    """
    The core TELOS commitment membrane.
    
    Separates reversible reasoning from irreversible action.
    Crossing requires: entropy + authority + (optional) attestation.
    """
    
    def __init__(self):
        self.entropy_meter = EntropyMeter()
        self.authority_registry = AuthorityRegistry()
        self.trust_accumulator = TrustAccumulator()
        
        # Pending decisions awaiting attestation
        self.pending: Dict[str, Decision] = {}
        
        # Completed crossings
        self.crossings: List[CrossingResult] = []
        
        logger.info("telos_membrane_initialized")
    
    def request_crossing(self, decision: Decision, 
                        required_scope: str = "execute:*",
                        require_attestation: bool = False) -> CrossingResult:
        """
        Request to cross the commitment membrane.
        
        This is THE enforcement point - all actions must pass through here.
        """
        
        # 1. Check entropy budget
        if not self.entropy_meter.spend(decision.tier):
            result = CrossingResult(
                allowed=False,
                decision_id=decision.decision_id,
                entropy_spent=0,
                authority_verified=False,
                attestations=0,
                reason="ENTROPY_EXHAUSTED: Insufficient entropy budget"
            )
            self.trust_accumulator.record(decision.agent_id, result)
            logger.warning("crossing_denied_entropy", decision=decision.decision_id)
            return result
        
        entropy_cost = self.entropy_meter.cost(decision.tier)
        
        # 2. Check authority
        authority_ok = self.authority_registry.verify(decision.agent_id, required_scope)
        if not authority_ok:
            result = CrossingResult(
                allowed=False,
                decision_id=decision.decision_id,
                entropy_spent=entropy_cost,
                authority_verified=False,
                attestations=0,
                reason=f"AUTHORITY_DENIED: Agent lacks scope '{required_scope}'"
            )
            self.trust_accumulator.record(decision.agent_id, result)
            logger.warning("crossing_denied_authority", 
                          decision=decision.decision_id, 
                          scope=required_scope)
            return result
        
        # 3. Check trust threshold for high-consequence actions
        trust = self.trust_accumulator.get_trust(decision.agent_id)
        if decision.tier.value >= ConsequenceTier.HIGH.value and trust < 0.6:
            result = CrossingResult(
                allowed=False,
                decision_id=decision.decision_id,
                entropy_spent=entropy_cost,
                authority_verified=True,
                attestations=0,
                reason=f"TRUST_INSUFFICIENT: {trust:.2f} < 0.6 for tier {decision.tier.name}"
            )
            self.trust_accumulator.record(decision.agent_id, result)
            logger.warning("crossing_denied_trust", 
                          decision=decision.decision_id,
                          trust=trust)
            return result
        
        # 4. Success - crossing allowed
        result = CrossingResult(
            allowed=True,
            decision_id=decision.decision_id,
            entropy_spent=entropy_cost,
            authority_verified=True,
            attestations=1,  # Self-attestation
            reason=None
        )
        
        self.crossings.append(result)
        self.trust_accumulator.record(decision.agent_id, result)
        
        logger.info("crossing_allowed", 
                   decision=decision.decision_id,
                   tier=decision.tier.name,
                   entropy=entropy_cost)
        
        return result
    
    def register_agent(self, agent_id: str, scopes: List[str]):
        """Register an agent with authority scopes"""
        self.authority_registry.register(agent_id, scopes)
    
    def get_status(self) -> Dict:
        """Get membrane status"""
        return {
            "entropy": self.entropy_meter.get_status(),
            "registered_agents": len(self.authority_registry.authorities),
            "total_crossings": len(self.crossings),
            "successful_crossings": sum(1 for c in self.crossings if c.allowed)
        }

# Global instance
telos_membrane = CommitmentMembrane()
