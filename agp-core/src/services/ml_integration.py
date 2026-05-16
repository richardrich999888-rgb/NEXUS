"""
ML Integration Services - Phase 3
Prediction, clustering, anomaly detection for endocrine system
"""

import math
import uuid
import random
from typing import Dict, List, Optional, Tuple
from datetime import datetime
from dataclasses import dataclass

from src.models import Hormone, EndocrineState, HealthStatus


@dataclass
class Prediction:
    """Prediction result"""
    predicted_value: float
    confidence: float
    features_used: List[str]
    model_version: str
    created_at: datetime


@dataclass
class AgentCluster:
    """Cluster of similar agents"""
    id: uuid.UUID
    centroid: List[float]  # 8D hormone vector
    agent_ids: List[uuid.UUID]
    label: str
    metadata: Dict


class OutcomePredictionService:
    """
    Predicts outcomes based on endocrine state
    
    Uses historical data to predict:
    - Task success probability
    - Collaboration compatibility
    - Risk of adverse events
    """
    
    def __init__(self):
        self.model_version = "1.0.0"
        # Simple heuristic weights (in production, use ML models)
        self.hormone_weights = {
            "task_success": {
                Hormone.CORTISOL: 0.25,
                Hormone.ADRENALINE: 0.15,
                Hormone.SEROTONIN: 0.20,
                Hormone.DOPAMINE: 0.15,
                Hormone.GROWTH_HORMONE: 0.25
            },
            "collaboration": {
                Hormone.OXYTOCIN: 0.40,
                Hormone.SEROTONIN: 0.25,
                Hormone.CORTISOL: -0.10,
                Hormone.ADRENALINE: -0.05
            },
            "risk": {
                Hormone.CORTISOL: 0.30,
                Hormone.ADRENALINE: 0.25,
                Hormone.NOREPINEPHRINE: 0.20,
                Hormone.SEROTONIN: -0.25
            }
        }
    
    def predict_task_success(
        self,
        state: EndocrineState,
        task_difficulty: float = 0.5
    ) -> Prediction:
        """Predict probability of task success"""
        weights = self.hormone_weights["task_success"]
        
        # Weighted sum of hormone levels
        score = 0.0
        for hormone, weight in weights.items():
            level = state.levels.get(hormone, 0.5)
            score += level * weight
        
        # Adjust for difficulty
        difficulty_factor = 1.0 - (task_difficulty - 0.5) * 0.4
        probability = min(1.0, max(0.0, score * difficulty_factor))
        
        # Confidence based on hormone stability
        variance = sum(
            (v - 0.5) ** 2 for v in state.levels.values()
        ) / len(Hormone)
        confidence = 1.0 - min(1.0, variance * 2)
        
        return Prediction(
            predicted_value=probability,
            confidence=confidence,
            features_used=list(weights.keys()),
            model_version=self.model_version,
            created_at=datetime.utcnow()
        )
    
    def predict_collaboration_compatibility(
        self,
        state_a: EndocrineState,
        state_b: EndocrineState
    ) -> Prediction:
        """Predict compatibility between two agents for collaboration"""
        weights = self.hormone_weights["collaboration"]
        
        # Similarity in social hormones
        similarity = 0.0
        for hormone, weight in weights.items():
            level_a = state_a.levels.get(hormone, 0.5)
            level_b = state_b.levels.get(hormone, 0.5)
            
            # Positive weight = similarity matters
            # Negative weight = both should be low
            if weight > 0:
                hormone_sim = 1.0 - abs(level_a - level_b)
                similarity += weight * hormone_sim
            else:
                avg_level = (level_a + level_b) / 2
                similarity += abs(weight) * (1.0 - avg_level)
        
        compatibility = min(1.0, max(0.0, 0.5 + similarity))
        
        return Prediction(
            predicted_value=compatibility,
            confidence=0.7,  # Fixed for now
            features_used=list(weights.keys()),
            model_version=self.model_version,
            created_at=datetime.utcnow()
        )
    
    def predict_risk_score(self, state: EndocrineState) -> Prediction:
        """Predict risk level based on stress hormones"""
        weights = self.hormone_weights["risk"]
        
        risk_score = 0.5
        for hormone, weight in weights.items():
            level = state.levels.get(hormone, 0.5)
            risk_score += (level - 0.5) * weight
        
        risk_score = min(1.0, max(0.0, risk_score))
        
        # Confidence higher when stress hormones are clearly elevated or low
        cortisol = state.levels.get(Hormone.CORTISOL, 0.5)
        adrenaline = state.levels.get(Hormone.ADRENALINE, 0.5)
        confidence = 0.5 + 0.5 * abs((cortisol + adrenaline) / 2 - 0.5) * 2
        
        return Prediction(
            predicted_value=risk_score,
            confidence=confidence,
            features_used=list(weights.keys()),
            model_version=self.model_version,
            created_at=datetime.utcnow()
        )


class ClusteringService:
    """
    Clusters agents based on endocrine profiles
    
    Useful for:
    - Finding similar agents
    - Behavioral pattern analysis
    - Anomaly detection baseline
    """
    
    def __init__(self, n_clusters: int = 5):
        self.n_clusters = n_clusters
        self.clusters: Dict[uuid.UUID, AgentCluster] = {}
        self.agent_assignments: Dict[uuid.UUID, uuid.UUID] = {}
    
    def fit(self, agents: Dict[uuid.UUID, EndocrineState]):
        """Fit clusters using simple k-means-like algorithm"""
        if len(agents) < self.n_clusters:
            return
        
        agent_list = list(agents.keys())
        
        # Initialize centroids randomly
        centroids = [
            self._state_to_vector(agents[random.choice(agent_list)])
            for _ in range(self.n_clusters)
        ]
        
        # Simple k-means iteration
        for _ in range(10):
            # Assign agents to clusters
            assignments = {i: [] for i in range(self.n_clusters)}
            
            for agent_id, state in agents.items():
                vector = self._state_to_vector(state)
                distances = [
                    self._euclidean_distance(vector, c)
                    for c in centroids
                ]
                nearest = distances.index(min(distances))
                assignments[nearest].append(agent_id)
            
            # Update centroids
            for i, members in assignments.items():
                if members:
                    vectors = [
                        self._state_to_vector(agents[m])
                        for m in members
                    ]
                    centroids[i] = [
                        sum(v[j] for v in vectors) / len(vectors)
                        for j in range(8)
                    ]
        
        # Create cluster objects
        labels = ["High Performance", "Social", "Stable", "Exploratory", "Developing"]
        self.clusters = {}
        self.agent_assignments = {}
        
        for i, (centroid, members) in enumerate(zip(centroids, assignments.values())):
            cluster_id = uuid.uuid4()
            self.clusters[cluster_id] = AgentCluster(
                id=cluster_id,
                centroid=centroid,
                agent_ids=members,
                label=labels[i] if i < len(labels) else f"Cluster {i}",
                metadata={"size": len(members)}
            )
            for agent_id in members:
                self.agent_assignments[agent_id] = cluster_id
    
    def predict_cluster(self, state: EndocrineState) -> Optional[AgentCluster]:
        """Predict which cluster a new agent belongs to"""
        if not self.clusters:
            return None
        
        vector = self._state_to_vector(state)
        
        best_cluster = None
        best_distance = float('inf')
        
        for cluster in self.clusters.values():
            distance = self._euclidean_distance(vector, cluster.centroid)
            if distance < best_distance:
                best_distance = distance
                best_cluster = cluster
        
        return best_cluster
    
    def _state_to_vector(self, state: EndocrineState) -> List[float]:
        """Convert endocrine state to 8D vector"""
        return [state.levels.get(h, 0.5) for h in Hormone]
    
    def _euclidean_distance(self, a: List[float], b: List[float]) -> float:
        """Calculate Euclidean distance"""
        return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


class AnomalyDetectionService:
    """
    Detects anomalous agent behavior based on endocrine patterns
    
    Flags agents with:
    - Unusual hormone combinations
    - Rapid state changes
    - Deviation from cluster norms
    """
    
    def __init__(self, clustering_service: ClusteringService):
        self.clustering_service = clustering_service
        self.threshold = 2.0  # Standard deviations for anomaly
    
    def detect_anomaly(
        self,
        state: EndocrineState,
        agent_id: Optional[uuid.UUID] = None
    ) -> Tuple[bool, float, str]:
        """
        Detect if state is anomalous
        
        Returns:
            - is_anomaly: bool
            - anomaly_score: float [0.0, 1.0]
            - reason: str
        """
        scores = []
        reasons = []
        
        # 1. Check for extreme hormone levels
        for hormone in Hormone:
            level = state.levels.get(hormone, 0.5)
            if level > 0.95 or level < 0.05:
                scores.append(1.0)
                reasons.append(f"Extreme {hormone.value}: {level:.2f}")
        
        # 2. Check for impossible combinations
        cortisol = state.levels.get(Hormone.CORTISOL, 0.5)
        serotonin = state.levels.get(Hormone.SEROTONIN, 0.5)
        
        # High cortisol with very high serotonin is unusual
        if cortisol > 0.8 and serotonin > 0.8:
            scores.append(0.7)
            reasons.append("Unusual cortisol-serotonin combination")
        
        # 3. Check deviation from cluster centroid
        cluster = self.clustering_service.predict_cluster(state)
        if cluster:
            vector = [state.levels.get(h, 0.5) for h in Hormone]
            distance = self._euclidean_distance(vector, cluster.centroid)
            
            # High distance = potential anomaly
            if distance > 0.5:
                scores.append(distance / 1.0)  # Normalize
                reasons.append(f"Far from cluster centroid ({cluster.label})")
        
        if not scores:
            return False, 0.0, "Normal"
        
        avg_score = sum(scores) / len(scores)
        is_anomaly = avg_score > 0.6
        
        return is_anomaly, avg_score, "; ".join(reasons)
    
    def _euclidean_distance(self, a: List[float], b: List[float]) -> float:
        return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


# Create service instances
outcome_predictor = OutcomePredictionService()
clustering_service = ClusteringService()
anomaly_detector = AnomalyDetectionService(clustering_service)
