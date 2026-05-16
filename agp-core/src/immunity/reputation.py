"""
SWARM IMMUNITY - REPUTATION ENGINE
Implementation of Transitive Trust Decay Physics (IDF-006).

Equation: T(t) = T_0 * e^(-lambda * dt)
"""
import time
import math
from typing import Dict, List, Optional
from dataclasses import dataclass, field

@dataclass
class ReputationScore:
    """Reputation of a peer with timestamps for decay calculation."""
    value: float = 0.5  # 0.0 (Malicious) to 1.0 (Trusted)
    confidence: float = 0.1  # How many interactions back this score?
    last_update: float = field(default_factory=time.time)
    
    def decay(self, current_time: float, half_life_hours: float = 24.0) -> float:
        """
        Calculate current score based on radioactive decay.
        """
        dt_hours = (current_time - self.last_update) / 3600.0
        lambda_decay = math.log(2) / half_life_hours
        decay_factor = math.exp(-lambda_decay * dt_hours)
        
        # Decay tends towards neutral (0.5) over time, not 0.
        # Unknown agents are 0.5. Bad agents (0.1) decay back to 0.5 if inactive.
        # Good agents (0.9) decay back to 0.5 if inactive.
        neutral = 0.5
        decayed_value = neutral + (self.value - neutral) * decay_factor
        
        return decayed_value

    def update(self, new_signal: float, signal_conf: float, current_time: float):
        """
        Update score with new evidence (Bayesian-like update).
        """
        # First decay existing score to now
        current_val = self.decay(current_time)
        
        # Simple weighted average for update
        # Confidence increases with interaction
        alpha = signal_conf / (self.confidence + signal_conf)
        self.value = (1 - alpha) * current_val + alpha * new_signal
        
        self.confidence = min(0.95, self.confidence + signal_conf * 0.1)
        self.last_update = current_time


class ReputationManager:
    """
    Manages the 'Trust Graph' of the swarm.
    """
    def __init__(self, my_id: str, half_life_hours: float = 24.0):
        self.my_id = my_id
        self.peers: Dict[str, ReputationScore] = {}
        self.half_life = half_life_hours
        
    def get_trust(self, peer_id: str) -> float:
        """Get current decayed trust score for a peer."""
        if peer_id == self.my_id:
            return 1.0
            
        if peer_id not in self.peers:
            return 0.5 # Neutral
            
        return self.peers[peer_id].decay(time.time(), self.half_life)
        
    def direct_interaction(self, peer_id: str, outcome: float):
        """Record a direct observation (0.0=Bad, 1.0=Good)."""
        if peer_id not in self.peers:
            self.peers[peer_id] = ReputationScore(value=0.5, confidence=0.0)
            
        self.peers[peer_id].update(outcome, signal_conf=0.2, current_time=time.time())
        
    def process_gossip(self, reporter_id: str, subject_id: str, reported_score: float):
        """
        Process a trust report from another peer.
        Implements Transitive Trust: Trust(Subject) += Trust(Reporter) * Report
        """
        if reporter_id == self.my_id:
            return
            
        trust_in_reporter = self.get_trust(reporter_id)
        
        # If I don't trust the reporter, I ignore their gossip (Noise Filter)
        if trust_in_reporter < 0.4:
            return
            
        # The weight of this update depends on how much I trust the source
        signal_strength = (trust_in_reporter - 0.5) * 2.0 # 0.5->0.0, 1.0->1.0
        
        if signal_strength <= 0:
            return
            
        if subject_id not in self.peers:
            self.peers[subject_id] = ReputationScore(value=0.5, confidence=0.0)
            
        # Apply transitive update
        # We treat the report as a 'weak' direct interaction scaled by reporter trust
        self.peers[subject_id].update(
            reported_score, 
            signal_conf=0.1 * signal_strength, 
            current_time=time.time()
        )
