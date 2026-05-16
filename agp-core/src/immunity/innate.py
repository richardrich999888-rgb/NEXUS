"""
Innate Immune System - First line of defense.

Fast, non-specific threat detection using pattern matching.
"""

import torch
import torch.nn as nn
from typing import Dict, Tuple, List


class PatternDetector(nn.Module):
    """Fast pattern-based detector for specific threat type."""
    
    def __init__(self, behavior_dim: int, hidden_dim: int = 128):
        super().__init__()
        self.detector = nn.Sequential(
            nn.Linear(behavior_dim, hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.1),
            nn.Linear(hidden_dim, hidden_dim // 2),
            nn.ReLU(),
            nn.Linear(hidden_dim // 2, 1),
            nn.Sigmoid()
        )
        
    def forward(self, behavior: torch.Tensor) -> torch.Tensor:
        if behavior.dim() == 1:
            behavior = behavior.unsqueeze(0)
        return self.detector(behavior).squeeze(-1)


class InnateImmuneSystem(nn.Module):
    """
    Innate immune system - rapid, non-specific threat detection.
    
    Components:
    1. Pattern detectors for known threat classes
    2. Immediate neutralization for severe threats
    3. Inflammatory signaling to activate adaptive system
    """
    
    def __init__(
        self,
        behavior_dim: int = 512,
        threat_patterns: List[str] = None
    ):
        super().__init__()
        
        if threat_patterns is None:
            threat_patterns = [
                'deception',
                'harmful_content',
                'value_drift',
                'capability_overhang',
                'goal_misalignment',
                'manipulation',
                'privacy_violation',
                'bias_amplification'
            ]
        
        self.behavior_dim = behavior_dim
        self.threat_patterns = threat_patterns
        
        self.detectors = nn.ModuleDict({
            pattern: PatternDetector(behavior_dim)
            for pattern in threat_patterns
        })
        
        self.neutralizer = nn.Sequential(
            nn.Linear(behavior_dim, 256),
            nn.ReLU(),
            nn.Linear(256, behavior_dim),
            nn.Tanh()
        )
        
        self.detection_threshold = 0.5
        self.alert_threshold = 0.8
        
    def scan(self, behavior: torch.Tensor) -> Dict[str, float]:
        """Rapid scan for known threat patterns."""
        threats = {}
        
        with torch.no_grad():
            for pattern, detector in self.detectors.items():
                confidence = detector(behavior)
                if confidence.dim() > 0:
                    confidence = confidence.mean()
                
                if confidence.item() > self.detection_threshold:
                    threats[pattern] = confidence.item()
        
        return threats
    
    def immediate_response(
        self,
        behavior: torch.Tensor,
        threats: Dict[str, float]
    ) -> Tuple[torch.Tensor, bool]:
        """Take immediate action if threat is severe."""
        if not threats:
            return behavior, False
        
        max_threat = max(threats.values())
        
        if max_threat > self.alert_threshold:
            with torch.no_grad():
                modified = self.neutralizer(behavior)
                blend = min(0.7, (max_threat - self.alert_threshold) / 0.2)
                modified_behavior = (1 - blend) * behavior + blend * modified
            return modified_behavior, True
        
        return behavior, True
    
    def forward(
        self,
        behavior: torch.Tensor
    ) -> Tuple[torch.Tensor, Dict[str, float], bool]:
        """Complete innate response."""
        threats = self.scan(behavior)
        safe_behavior, alert = self.immediate_response(behavior, threats)
        return safe_behavior, threats, alert
