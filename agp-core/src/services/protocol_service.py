"""
Protocol Service - Phase 2
Multi-protocol support with similarity calculation and cross-protocol projection
"""

import uuid
import math
from typing import Dict, List, Optional, Tuple
from datetime import datetime
from dataclasses import dataclass

from src.models import Hormone, EndocrineState


@dataclass
class Protocol:
    """Protocol definition"""
    id: uuid.UUID
    name: str
    description: str
    version: str
    is_active: bool
    action_types: Dict[str, Dict]  # action_name -> {hormone_weights, category}
    config: Dict
    created_at: datetime
    
    
@dataclass
class ActionType:
    """Action type with hormone weights"""
    name: str
    category: str
    hormone_weights: Dict[str, float]  # hormone -> weight
    base_cost: float
    description: str


class ProtocolService:
    """
    Manages protocols and their action types
    
    Each protocol defines:
    - Which actions it supports
    - How actions map to hormones (weights)
    - Similarity metrics with other protocols
    """
    
    def __init__(self):
        self.protocols: Dict[uuid.UUID, Protocol] = {}
        self.similarity_cache: Dict[Tuple[uuid.UUID, uuid.UUID], float] = {}
    
    def register_protocol(
        self,
        name: str,
        description: str,
        action_types: List[ActionType],
        config: Optional[Dict] = None
    ) -> Protocol:
        """Register a new protocol"""
        protocol_id = uuid.uuid4()
        
        action_map = {
            at.name: {
                "hormone_weights": at.hormone_weights,
                "category": at.category,
                "base_cost": at.base_cost,
                "description": at.description
            }
            for at in action_types
        }
        
        protocol = Protocol(
            id=protocol_id,
            name=name,
            description=description,
            version="1.0.0",
            is_active=True,
            action_types=action_map,
            config=config or {},
            created_at=datetime.utcnow()
        )
        
        self.protocols[protocol_id] = protocol
        
        # Invalidate similarity cache
        self._invalidate_cache(protocol_id)
        
        return protocol
    
    def get_protocol(self, protocol_id: uuid.UUID) -> Optional[Protocol]:
        """Get protocol by ID"""
        return self.protocols.get(protocol_id)
    
    def list_protocols(self, active_only: bool = True) -> List[Protocol]:
        """List all protocols"""
        if active_only:
            return [p for p in self.protocols.values() if p.is_active]
        return list(self.protocols.values())
    
    def _invalidate_cache(self, protocol_id: uuid.UUID):
        """Invalidate similarity cache for a protocol"""
        keys_to_remove = [
            k for k in self.similarity_cache
            if protocol_id in k
        ]
        for k in keys_to_remove:
            del self.similarity_cache[k]


class SimilarityService:
    """
    Calculates similarity between protocols for cross-protocol reputation projection
    
    Similarity is based on:
    - Action space overlap
    - Hormone weight correlation
    - Outcome correlation (from observations)
    - Attestation consistency
    """
    
    def __init__(self, protocol_service: ProtocolService):
        self.protocol_service = protocol_service
    
    def calculate_similarity(
        self,
        protocol_a: Protocol,
        protocol_b: Protocol
    ) -> float:
        """
        Calculate similarity score [0.0, 1.0] between two protocols
        """
        # 1. Action space overlap
        action_overlap = self._action_overlap(protocol_a, protocol_b)
        
        # 2. Hormone weight correlation
        hormone_correlation = self._hormone_correlation(protocol_a, protocol_b)
        
        # Weighted combination
        similarity = 0.4 * action_overlap + 0.6 * hormone_correlation
        
        return max(0.0, min(1.0, similarity))
    
    def _action_overlap(self, a: Protocol, b: Protocol) -> float:
        """Calculate Jaccard similarity of action spaces"""
        actions_a = set(a.action_types.keys())
        actions_b = set(b.action_types.keys())
        
        if not actions_a or not actions_b:
            return 0.0
        
        intersection = len(actions_a & actions_b)
        union = len(actions_a | actions_b)
        
        return intersection / union if union > 0 else 0.0
    
    def _hormone_correlation(self, a: Protocol, b: Protocol) -> float:
        """Calculate correlation of hormone weight profiles"""
        # Get common actions
        common_actions = set(a.action_types.keys()) & set(b.action_types.keys())
        
        if not common_actions:
            return 0.0
        
        correlations = []
        
        for action in common_actions:
            weights_a = a.action_types[action].get("hormone_weights", {})
            weights_b = b.action_types[action].get("hormone_weights", {})
            
            # Calculate cosine similarity of weight vectors
            all_hormones = set(weights_a.keys()) | set(weights_b.keys())
            
            if not all_hormones:
                continue
            
            dot_product = sum(
                weights_a.get(h, 0) * weights_b.get(h, 0)
                for h in all_hormones
            )
            
            mag_a = math.sqrt(sum(v**2 for v in weights_a.values()))
            mag_b = math.sqrt(sum(v**2 for v in weights_b.values()))
            
            if mag_a > 0 and mag_b > 0:
                correlations.append(dot_product / (mag_a * mag_b))
        
        return sum(correlations) / len(correlations) if correlations else 0.0
    
    def find_similar_protocols(
        self,
        protocol: Protocol,
        min_similarity: float = 0.5
    ) -> List[Tuple[Protocol, float]]:
        """Find protocols similar to the given one"""
        results = []
        
        for other in self.protocol_service.list_protocols():
            if other.id == protocol.id:
                continue
            
            similarity = self.calculate_similarity(protocol, other)
            
            if similarity >= min_similarity:
                results.append((other, similarity))
        
        # Sort by similarity descending
        results.sort(key=lambda x: x[1], reverse=True)
        
        return results


class ProjectionService:
    """
    Projects agent reputation across similar protocols
    
    When an agent has reputation in Protocol A, we can estimate
    their reputation in Protocol B based on protocol similarity.
    """
    
    def __init__(
        self,
        protocol_service: ProtocolService,
        similarity_service: SimilarityService
    ):
        self.protocol_service = protocol_service
        self.similarity_service = similarity_service
    
    def project_reputation(
        self,
        source_state: EndocrineState,
        source_protocol: Protocol,
        target_protocol: Protocol,
        source_experience: int = 0
    ) -> Tuple[EndocrineState, float]:
        """
        Project reputation from source to target protocol
        
        Returns:
            - Projected endocrine state
            - Confidence score [0.0, 1.0]
        """
        similarity = self.similarity_service.calculate_similarity(
            source_protocol, target_protocol
        )
        
        if similarity < 0.1:
            # Too dissimilar - return baseline with low confidence
            return EndocrineState(), 0.0
        
        # Build action mapping
        action_mapping = self._build_action_mapping(
            source_protocol, target_protocol
        )
        
        # Project each hormone level
        projected_levels = {}
        
        for hormone in Hormone:
            source_level = source_state.levels.get(hormone, 0.5)
            
            # Weight by similarity and action overlap
            weight = self._calculate_hormone_weight(
                hormone.value, source_protocol, target_protocol, action_mapping
            )
            
            # Project towards baseline weighted by similarity
            baseline = 0.5
            projected = baseline + (source_level - baseline) * similarity * weight
            projected_levels[hormone] = max(0.0, min(1.0, projected))
        
        projected_state = EndocrineState(levels=projected_levels)
        
        # Confidence based on similarity and experience
        experience_factor = math.log1p(source_experience) / math.log1p(100)
        confidence = similarity * min(1.0, experience_factor + 0.3)
        
        return projected_state, confidence
    
    def _build_action_mapping(
        self,
        source: Protocol,
        target: Protocol
    ) -> Dict[str, str]:
        """Build mapping of source actions to target actions"""
        mapping = {}
        
        source_actions = list(source.action_types.keys())
        target_actions = list(target.action_types.keys())
        
        # Direct matches
        for action in source_actions:
            if action in target_actions:
                mapping[action] = action
        
        # Category-based matches for unmatched
        for source_action in source_actions:
            if source_action in mapping:
                continue
            
            source_cat = source.action_types[source_action].get("category", "")
            
            for target_action in target_actions:
                if target_action in mapping.values():
                    continue
                
                target_cat = target.action_types[target_action].get("category", "")
                
                if source_cat == target_cat and source_cat:
                    mapping[source_action] = target_action
                    break
        
        return mapping
    
    def _calculate_hormone_weight(
        self,
        hormone: str,
        source: Protocol,
        target: Protocol,
        action_mapping: Dict[str, str]
    ) -> float:
        """Calculate weight for projecting a specific hormone"""
        if not action_mapping:
            return 0.5
        
        weights = []
        
        for source_action, target_action in action_mapping.items():
            source_weights = source.action_types[source_action].get("hormone_weights", {})
            target_weights = target.action_types[target_action].get("hormone_weights", {})
            
            if hormone in source_weights or hormone in target_weights:
                source_w = source_weights.get(hormone, 0)
                target_w = target_weights.get(hormone, 0)
                
                # Weight similarity
                if source_w + target_w > 0:
                    weight = 1 - abs(source_w - target_w) / (source_w + target_w)
                    weights.append(weight)
        
        return sum(weights) / len(weights) if weights else 0.5


# Create default instances
protocol_service = ProtocolService()
similarity_service = SimilarityService(protocol_service)
projection_service = ProjectionService(protocol_service, similarity_service)
