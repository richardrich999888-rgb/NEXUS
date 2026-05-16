"""
AGP-CORE: Alignment Verifier
Computes agent alignment from behavioral history with impact-weighted scoring.
"""

from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime, timedelta
import structlog

from .behavioral_rag import behavioral_rag, BehaviorRecord, Outcome, ActionType
from .impact import impact_analyzer

logger = structlog.get_logger()

@dataclass
class AlignmentScore:
    """Detailed alignment score breakdown"""
    overall: float
    success_component: float
    consistency_component: float
    compliance_component: float
    impact_distribution: Dict[str, int]  # NEW: Distribution of action impacts
    recency_weight: float
    sample_size: int
    computed_at: datetime

class AlignmentVerifier:
    """
    Verifies agent alignment based on behavioral history.
    This is the core of the governance reputation system.
    
    NEW: Uses impact-weighted scoring to differentiate between
    high-risk and low-risk actions.
    """
    
    def __init__(self):
        self.rag = behavioral_rag
        self.impact = impact_analyzer
        
        # Weights for alignment computation
        self.success_weight = 0.4
        self.consistency_weight = 0.3
        self.compliance_weight = 0.3
        
        # Recency decay (more recent behaviors weighted higher)
        self.recency_decay_hours = 24
        
        logger.info("alignment_verifier_initialized", impact_weighted=True)
    
    def compute_alignment(self, agent_id: str, 
                          sample_size: int = 100) -> AlignmentScore:
        """
        Compute alignment score from agent's behavioral history.
        
        Components:
        1. Success Rate (40%) - Impact-weighted ratio of successful actions
        2. Consistency (30%) - Similarity of behavior patterns
        3. Compliance (30%) - Adherence to governance rules
        """
        behaviors = self.rag.retrieve_by_agent(agent_id, limit=sample_size)
        
        if not behaviors:
            return AlignmentScore(
                overall=0.5,  # Neutral for new agents
                success_component=0.5,
                consistency_component=0.5,
                compliance_component=1.0,
                impact_distribution={},
                recency_weight=1.0,
                sample_size=0,
                computed_at=datetime.utcnow()
            )
        
        # 1. Compute IMPACT-WEIGHTED success rate
        success_component = self._compute_impact_weighted_success(behaviors)
        
        # 2. Compute consistency (embedding similarity)
        consistency_component = self._compute_consistency(behaviors)
        
        # 3. Compute compliance (violations count)
        compliance_component = self._compute_compliance(behaviors)
        
        # 4. Get impact distribution
        impact_distribution = self.impact.get_impact_category_distribution(behaviors)
        
        # Recency factor (boost for recent activity)
        recency_weight = self._compute_recency_weight(behaviors)
        
        # Final score
        overall = (
            self.success_weight * success_component +
            self.consistency_weight * consistency_component +
            self.compliance_weight * compliance_component
        ) * recency_weight
        
        # Clamp to [0, 1]
        overall = max(0.0, min(1.0, overall))
        
        score = AlignmentScore(
            overall=overall,
            success_component=success_component,
            consistency_component=consistency_component,
            compliance_component=compliance_component,
            impact_distribution=impact_distribution,
            recency_weight=recency_weight,
            sample_size=len(behaviors),
            computed_at=datetime.utcnow()
        )
        
        logger.info("alignment_computed", 
                   agent_id=agent_id,
                   alignment=f"{overall:.3f}",
                   impact_weighted=True,
                   sample_size=len(behaviors))
        
        return score
    
    def _compute_impact_weighted_success(self, behaviors: List[BehaviorRecord]) -> float:
        """
        Compute IMPACT-WEIGHTED success rate.
        
        Key difference from naive approach:
        - Successful high-risk actions LOWER the score (potential harm)
        - Failed high-risk actions RAISE the score (prevented harm)
        - Successful low-risk actions raise the score (good behavior)
        - Failed low-risk actions lower the score (incompetence)
        """
        if not behaviors:
            return 0.5
        
        now = datetime.utcnow()
        weighted_sum = 0.0
        total_weight = 0.0
        
        for b in behaviors:
            # Recency weight
            age_hours = (now - b.timestamp).total_seconds() / 3600
            recency_weight = max(0.1, 1.0 - (age_hours / (self.recency_decay_hours * 7)))
            
            # NEW: Use impact analyzer to compute weighted outcome
            is_success = (b.outcome == Outcome.SUCCESS)
            impact_score = self.impact.compute_weighted_outcome(
                action_description=b.input_summary,
                outcome_success=is_success
            )
            
            # Convert impact_score (-1 to 1) to alignment contribution (0 to 1)
            alignment_contribution = (impact_score + 1.0) / 2.0
            
            weighted_sum += alignment_contribution * recency_weight
            total_weight += recency_weight
        
        return weighted_sum / total_weight if total_weight > 0 else 0.5
    
    def _compute_consistency(self, behaviors: List[BehaviorRecord]) -> float:
        """
        Compute behavioral consistency using embedding similarity.
        Consistent agents have predictable behavioral patterns.
        """
        if len(behaviors) < 2:
            return 0.5
        
        # Get embeddings
        embeddings = [b.embedding for b in behaviors if b.embedding]
        
        if len(embeddings) < 2:
            return 0.5
        
        # Compute average pairwise similarity
        # (simplified - in production use proper cosine similarity)
        try:
            import numpy as np
            
            embeddings = np.array(embeddings[:20])  # Limit for performance
            
            # Compute centroid
            centroid = np.mean(embeddings, axis=0)
            
            # Compute distances from centroid
            distances = np.linalg.norm(embeddings - centroid, axis=1)
            avg_distance = np.mean(distances)
            
            # Convert to similarity (lower distance = higher consistency)
            max_distance = 2.0  # Normalized embeddings have max distance ~2
            consistency = 1.0 - (avg_distance / max_distance)
            
            return max(0.0, min(1.0, consistency))
            
        except Exception:
            # Fallback without numpy
            return 0.7  # Assume reasonable consistency
    
    def _compute_compliance(self, behaviors: List[BehaviorRecord]) -> float:
        """
        Compute compliance score based on rejected or failed actions.

        Failures are treated as governance violations here because repeated
        execution failures should reduce alignment before an action is
        formally blocked.
        """
        if not behaviors:
            return 1.0
        
        violations = sum(
            1 for b in behaviors
            if b.outcome in (Outcome.BLOCKED, Outcome.FAILURE)
        )
        total = len(behaviors)
        
        # Violation rate
        violation_rate = violations / total if total > 0 else 0.0
        
        # Compliance score (inverse of violation rate)
        return 1.0 - violation_rate
    
    def _compute_recency_weight(self, behaviors: List[BehaviorRecord]) -> float:
        """
        Compute recency weight - agents with recent activity get slight boost.
        """
        if not behaviors:
            return 0.8
        
        most_recent = max(b.timestamp for b in behaviors)
        age_hours = (datetime.utcnow() - most_recent).total_seconds() / 3600
        
        # No penalty for recent activity, slight decay for stale agents
        if age_hours < 1:
            return 1.0
        elif age_hours < 24:
            return 0.95
        elif age_hours < 168:  # 1 week
            return 0.9
        else:
            return 0.8
    
    def get_alignment(self, agent_id: str) -> float:
        """Quick method to get just the overall alignment"""
        score = self.compute_alignment(agent_id)
        return score.overall
    
    def compare_agents(self, agent_ids: List[str]) -> Dict[str, float]:
        """Compare alignment scores across multiple agents"""
        return {
            agent_id: self.get_alignment(agent_id)
            for agent_id in agent_ids
        }
    
    def get_top_aligned(self, limit: int = 10) -> List[tuple]:
        """Get agents with highest alignment"""
        all_agents = list(self.rag.behaviors.keys())
        scores = [(agent_id, self.get_alignment(agent_id)) for agent_id in all_agents]
        return sorted(scores, key=lambda x: x[1], reverse=True)[:limit]

# Global instance
alignment_verifier = AlignmentVerifier()
