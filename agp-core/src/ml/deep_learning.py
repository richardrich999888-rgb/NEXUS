"""
AGP-CORE Deep Learning Module
Neural network models for agent behavior prediction
"""

import uuid
from typing import Dict, List, Optional, Tuple, Any
from datetime import datetime
from dataclasses import dataclass

# Try to import PyTorch
try:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False

# Try to import sklearn
try:
    from sklearn.ensemble import RandomForestClassifier, IsolationForest
    from sklearn.preprocessing import StandardScaler
    import numpy as np
    SKLEARN_AVAILABLE = True
except ImportError:
    SKLEARN_AVAILABLE = False

from src.models import Hormone, EndocrineState


@dataclass
class ModelMetrics:
    """Model performance metrics"""
    accuracy: float
    precision: float
    recall: float
    f1_score: float
    training_samples: int
    last_trained: datetime


# =============================================================================
# PyTorch Models
# =============================================================================

if TORCH_AVAILABLE:
    class EndocrineEncoder(nn.Module):
        """
        Encodes 8D endocrine state to latent representation
        """
        def __init__(self, input_dim: int = 8, latent_dim: int = 32):
            super().__init__()
            self.encoder = nn.Sequential(
                nn.Linear(input_dim, 64),
                nn.ReLU(),
                nn.BatchNorm1d(64),
                nn.Linear(64, latent_dim),
                nn.ReLU()
            )
        
        def forward(self, x):
            return self.encoder(x)
    
    class BehaviorPredictor(nn.Module):
        """
        Predicts agent behavior outcome from endocrine state
        Outputs: [success_prob, collaboration_prob, risk_level]
        """
        def __init__(self, input_dim: int = 8, hidden_dim: int = 64):
            super().__init__()
            self.network = nn.Sequential(
                nn.Linear(input_dim, hidden_dim),
                nn.ReLU(),
                nn.Dropout(0.2),
                nn.Linear(hidden_dim, hidden_dim),
                nn.ReLU(),
                nn.Dropout(0.2),
                nn.Linear(hidden_dim, 3),
                nn.Sigmoid()
            )
        
        def forward(self, x):
            return self.network(x)
    
    class AnomalyAutoEncoder(nn.Module):
        """
        Autoencoder for anomaly detection
        High reconstruction error = anomalous agent
        """
        def __init__(self, input_dim: int = 8, latent_dim: int = 4):
            super().__init__()
            
            self.encoder = nn.Sequential(
                nn.Linear(input_dim, 16),
                nn.ReLU(),
                nn.Linear(16, latent_dim),
                nn.ReLU()
            )
            
            self.decoder = nn.Sequential(
                nn.Linear(latent_dim, 16),
                nn.ReLU(),
                nn.Linear(16, input_dim),
                nn.Sigmoid()
            )
        
        def forward(self, x):
            z = self.encoder(x)
            return self.decoder(z)
        
        def reconstruction_error(self, x):
            recon = self.forward(x)
            return F.mse_loss(recon, x, reduction='none').mean(dim=1)


# =============================================================================
# ML Services
# =============================================================================

class DeepLearningService:
    """
    Deep learning service for AGP-CORE
    Provides neural network-based predictions
    """
    
    def __init__(self):
        self.behavior_model = None
        self.anomaly_model = None
        self.scaler = None
        self._initialized = False
        
        if TORCH_AVAILABLE:
            self._init_models()
    
    def _init_models(self):
        """Initialize PyTorch models"""
        self.behavior_model = BehaviorPredictor()
        self.anomaly_model = AnomalyAutoEncoder()
        self._initialized = True
        
        # Set to eval mode
        self.behavior_model.eval()
        self.anomaly_model.eval()
    
    def _state_to_tensor(self, state: EndocrineState) -> Optional[Any]:
        """Convert endocrine state to tensor"""
        if not TORCH_AVAILABLE:
            return None
        
        vector = [state.levels.get(h, 0.5) for h in Hormone]
        return torch.tensor([vector], dtype=torch.float32)
    
    def predict_behavior(self, state: EndocrineState) -> Dict[str, float]:
        """
        Predict agent behavior outcomes
        Returns probabilities for success, collaboration, and risk
        """
        if not self._initialized:
            return self._fallback_predict(state)
        
        tensor = self._state_to_tensor(state)
        if tensor is None:
            return self._fallback_predict(state)
        
        with torch.no_grad():
            output = self.behavior_model(tensor)
            probs = output.squeeze().tolist()
        
        return {
            "success_probability": probs[0],
            "collaboration_probability": probs[1],
            "risk_level": probs[2]
        }
    
    def detect_anomaly(self, state: EndocrineState) -> Dict[str, Any]:
        """
        Detect if agent state is anomalous
        Uses autoencoder reconstruction error
        """
        if not self._initialized:
            return self._fallback_anomaly(state)
        
        tensor = self._state_to_tensor(state)
        if tensor is None:
            return self._fallback_anomaly(state)
        
        with torch.no_grad():
            error = self.anomaly_model.reconstruction_error(tensor)
            anomaly_score = error.item()
        
        # Threshold for anomaly (tuneable)
        threshold = 0.1
        is_anomaly = anomaly_score > threshold
        
        return {
            "is_anomaly": is_anomaly,
            "anomaly_score": anomaly_score,
            "threshold": threshold,
            "confidence": 1.0 - min(1.0, anomaly_score / threshold) if not is_anomaly else min(1.0, anomaly_score / threshold)
        }
    
    def _fallback_predict(self, state: EndocrineState) -> Dict[str, float]:
        """Fallback prediction without PyTorch"""
        dopamine = state.levels.get(Hormone.DOPAMINE, 0.5)
        oxytocin = state.levels.get(Hormone.OXYTOCIN, 0.5)
        cortisol = state.levels.get(Hormone.CORTISOL, 0.5)
        
        return {
            "success_probability": dopamine * 0.6 + (1 - cortisol) * 0.4,
            "collaboration_probability": oxytocin * 0.8 + dopamine * 0.2,
            "risk_level": cortisol * 0.5 + (1 - dopamine) * 0.3 + state.levels.get(Hormone.ADRENALINE, 0.5) * 0.2
        }
    
    def _fallback_anomaly(self, state: EndocrineState) -> Dict[str, Any]:
        """Fallback anomaly detection"""
        vector = [state.levels.get(h, 0.5) for h in Hormone]
        
        # Simple deviation from baseline
        baseline = 0.5
        deviations = [abs(v - baseline) for v in vector]
        avg_deviation = sum(deviations) / len(deviations)
        
        is_anomaly = avg_deviation > 0.3
        
        return {
            "is_anomaly": is_anomaly,
            "anomaly_score": avg_deviation,
            "threshold": 0.3,
            "confidence": 0.7
        }
    
    def get_embedding(self, state: EndocrineState) -> List[float]:
        """Get latent embedding of agent state"""
        if not TORCH_AVAILABLE:
            return [state.levels.get(h, 0.5) for h in Hormone]
        
        encoder = EndocrineEncoder()
        tensor = self._state_to_tensor(state)
        
        with torch.no_grad():
            embedding = encoder(tensor)
        
        return embedding.squeeze().tolist()


class SklearnService:
    """
    Scikit-learn based ML service
    Traditional ML for interpretable predictions
    """
    
    def __init__(self):
        self.classifier = None
        self.isolation_forest = None
        self.scaler = None
        self._fitted = False
        
        if SKLEARN_AVAILABLE:
            self._init_models()
    
    def _init_models(self):
        """Initialize sklearn models"""
        self.classifier = RandomForestClassifier(n_estimators=100, random_state=42)
        self.isolation_forest = IsolationForest(contamination=0.1, random_state=42)
        self.scaler = StandardScaler()
    
    def _state_to_array(self, state: EndocrineState) -> Any:
        """Convert state to numpy array"""
        if not SKLEARN_AVAILABLE:
            return None
        vector = [state.levels.get(h, 0.5) for h in Hormone]
        return np.array(vector).reshape(1, -1)
    
    def fit(
        self,
        states: List[EndocrineState],
        labels: Optional[List[int]] = None
    ):
        """Fit models on training data"""
        if not SKLEARN_AVAILABLE:
            return
        
        X = np.array([[s.levels.get(h, 0.5) for h in Hormone] for s in states])
        X_scaled = self.scaler.fit_transform(X)
        
        # Fit isolation forest for anomaly detection
        self.isolation_forest.fit(X_scaled)
        
        # Fit classifier if labels provided
        if labels:
            self.classifier.fit(X_scaled, labels)
        
        self._fitted = True
    
    def predict_anomaly(self, state: EndocrineState) -> Dict[str, Any]:
        """Detect anomaly using Isolation Forest"""
        if not SKLEARN_AVAILABLE or not self._fitted:
            return {"is_anomaly": False, "score": 0.0, "method": "unavailable"}
        
        X = self._state_to_array(state)
        X_scaled = self.scaler.transform(X)
        
        prediction = self.isolation_forest.predict(X_scaled)[0]
        score = self.isolation_forest.score_samples(X_scaled)[0]
        
        return {
            "is_anomaly": prediction == -1,
            "score": float(-score),  # Higher = more anomalous
            "method": "isolation_forest"
        }
    
    def feature_importance(self) -> Dict[str, float]:
        """Get feature importance from Random Forest"""
        if not self._fitted or not hasattr(self.classifier, 'feature_importances_'):
            return {}
        
        hormones = list(Hormone)
        return {h.value: float(imp) for h, imp in zip(hormones, self.classifier.feature_importances_)}


# =============================================================================
# Global Instances
# =============================================================================

deep_learning_service = DeepLearningService()
sklearn_service = SklearnService()


def predict_behavior(state: EndocrineState) -> Dict[str, float]:
    """Predict agent behavior"""
    return deep_learning_service.predict_behavior(state)

def detect_anomaly(state: EndocrineState) -> Dict[str, Any]:
    """Detect anomalous agent state"""
    return deep_learning_service.detect_anomaly(state)
