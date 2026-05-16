"""
AGP-CORE: Semantic Anomaly Detection
Detects abnormal behavioral patterns in agents using embedding analysis.
"""

from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
import structlog

from .behavioral_rag import behavioral_rag, BehaviorRecord

logger = structlog.get_logger()

class AnomalyType(Enum):
    """Types of anomalies detected"""
    BEHAVIORAL_DRIFT = "behavioral_drift"        # Gradual change from baseline
    SUDDEN_SHIFT = "sudden_shift"                # Abrupt change in behavior
    CATEGORY_SHIFT = "category_shift"            # Changed action categories
    FREQUENCY_SPIKE = "frequency_spike"          # Unusual activity rate
    HIGH_RISK_PATTERN = "high_risk_pattern"      # Pattern of risky actions

@dataclass
class AnomalyAlert:
    """Alert for detected anomaly"""
    agent_id: str
    agent_name: str
    anomaly_type: AnomalyType
    severity: float  # 0.0 to 1.0
    description: str
    evidence: Dict
    detected_at: datetime
    
    def is_critical(self) -> bool:
        return self.severity >= 0.8

class AnomalyDetector:
    """
    Detects semantic anomalies in agent behavior using embedding analysis.
    
    Key techniques:
    1. Baseline profiling - Learn normal behavior patterns
    2. Drift detection - Compare recent vs historical embeddings
    3. Category analysis - Detect shifts in action types
    4. Frequency analysis - Identify unusual activity rates
    """
    
    def __init__(self):
        self.rag = behavioral_rag
        
        # Thresholds for anomaly detection
        self.drift_threshold = 0.5          # Cosine distance threshold
        self.sudden_shift_threshold = 0.7   # For abrupt changes
        self.frequency_multiplier = 3.0     # X times normal rate
        self.high_risk_threshold = 0.5      # % of high-risk actions
        
        # Sliding window sizes
        self.baseline_window = 50           # Historical behaviors for baseline
        self.recent_window = 10             # Recent behaviors to analyze
        
        logger.info("anomaly_detector_initialized",
                   drift_threshold=self.drift_threshold,
                   baseline_window=self.baseline_window)
    
    def detect_anomalies(self, agent_id: str, agent_name: str) -> List[AnomalyAlert]:
        """
        Detect all types of anomalies for an agent.
        Returns list of alerts (empty if no anomalies).
        """
        behaviors = self.rag.retrieve_by_agent(agent_id, limit=100)
        
        if len(behaviors) < self.baseline_window:
            # Not enough data for anomaly detection
            return []
        
        alerts = []
        
        # 1. Check for behavioral drift
        drift_alert = self._detect_drift(agent_id, agent_name, behaviors)
        if drift_alert:
            alerts.append(drift_alert)
        
        # 2. Check for sudden shift
        shift_alert = self._detect_sudden_shift(agent_id, agent_name, behaviors)
        if shift_alert:
            alerts.append(shift_alert)
        
        # 3. Check for category shift
        category_alert = self._detect_category_shift(agent_id, agent_name, behaviors)
        if category_alert:
            alerts.append(category_alert)
        
        # 4. Check for frequency spike
        freq_alert = self._detect_frequency_spike(agent_id, agent_name, behaviors)
        if freq_alert:
            alerts.append(freq_alert)
        
        # 5. Check for high-risk pattern
        risk_alert = self._detect_high_risk_pattern(agent_id, agent_name, behaviors)
        if risk_alert:
            alerts.append(risk_alert)
        
        if alerts:
            logger.warning("anomalies_detected",
                         agent=agent_name,
                         count=len(alerts),
                         types=[a.anomaly_type.value for a in alerts])
        
        return alerts
    
    def _detect_drift(self, agent_id: str, agent_name: str, 
                     behaviors: List[BehaviorRecord]) -> Optional[AnomalyAlert]:
        """Detect gradual behavioral drift from baseline"""
        
        # Split into baseline and recent
        baseline = behaviors[self.recent_window:][:self.baseline_window]
        recent = behaviors[:self.recent_window]
        
        # Get embeddings
        baseline_embeddings = [b.embedding for b in baseline if b.embedding]
        recent_embeddings = [b.embedding for b in recent if b.embedding]
        
        if not baseline_embeddings or not recent_embeddings:
            return None
        
        # Compute distance between centroids
        try:
            import numpy as np
            
            baseline_centroid = np.mean(baseline_embeddings, axis=0)
            recent_centroid = np.mean(recent_embeddings, axis=0)
            
            # Cosine distance
            distance = np.linalg.norm(baseline_centroid - recent_centroid)
            
            if distance > self.drift_threshold:
                severity = min(1.0, distance / 2.0)
                
                return AnomalyAlert(
                    agent_id=agent_id,
                    agent_name=agent_name,
                    anomaly_type=AnomalyType.BEHAVIORAL_DRIFT,
                    severity=severity,
                    description=f"Behavioral drift detected (distance: {distance:.3f})",
                    evidence={
                        "distance": float(distance),
                        "baseline_size": len(baseline),
                        "recent_size": len(recent)
                    },
                    detected_at=datetime.utcnow()
                )
        except Exception as e:
            logger.error("drift_detection_error", error=str(e))
            return None
        
        return None
    
    def _detect_sudden_shift(self, agent_id: str, agent_name: str,
                            behaviors: List[BehaviorRecord]) -> Optional[AnomalyAlert]:
        """Detect abrupt shift in recent behavior"""
        
        if len(behaviors) < 20:
            return None
        
        # Compare last 5 vs previous 15
        very_recent = behaviors[:5]
        previous = behaviors[5:20]
        
        recent_embeddings = [b.embedding for b in very_recent if b.embedding]
        previous_embeddings = [b.embedding for b in previous if b.embedding]
        
        if not recent_embeddings or not previous_embeddings:
            return None
        
        try:
            import numpy as np
            
            recent_centroid = np.mean(recent_embeddings, axis=0)
            previous_centroid = np.mean(previous_embeddings, axis=0)
            
            distance = np.linalg.norm(recent_centroid - previous_centroid)
            
            if distance > self.sudden_shift_threshold:
                severity = min(1.0, distance / 1.5)
                
                return AnomalyAlert(
                    agent_id=agent_id,
                    agent_name=agent_name,
                    anomaly_type=AnomalyType.SUDDEN_SHIFT,
                    severity=severity,
                    description=f"Sudden behavioral shift in last 5 actions (distance: {distance:.3f})",
                    evidence={
                        "distance": float(distance),
                        "very_recent": len(very_recent),
                        "previous": len(previous)
                    },
                    detected_at=datetime.utcnow()
                )
        except Exception:
            return None
        
        return None
    
    def _detect_category_shift(self, agent_id: str, agent_name: str,
                               behaviors: List[BehaviorRecord]) -> Optional[AnomalyAlert]:
        """Detect shift in action categories"""
        
        from .impact import impact_analyzer
        
        baseline = behaviors[self.recent_window:][:self.baseline_window]
        recent = behaviors[:self.recent_window]
        
        if not baseline or not recent:
            return None
        
        # Get category distributions
        baseline_dist = impact_analyzer.get_impact_category_distribution(baseline)
        recent_dist = impact_analyzer.get_impact_category_distribution(recent)
        
        # Normalize
        baseline_total = sum(baseline_dist.values()) or 1
        recent_total = sum(recent_dist.values()) or 1
        
        baseline_norm = {k: v / baseline_total for k, v in baseline_dist.items()}
        recent_norm = {k: v / recent_total for k, v in recent_dist.items()}
        
        # Compute distribution difference (KL-divergence approximation)
        max_diff = 0.0
        shifted_category = None
        
        for category in baseline_norm.keys():
            diff = abs(recent_norm.get(category, 0) - baseline_norm.get(category, 0))
            if diff > max_diff:
                max_diff = diff
                shifted_category = category
        
        if max_diff > 0.3:  # 30% shift in any category
            severity = min(1.0, max_diff)
            
            return AnomalyAlert(
                agent_id=agent_id,
                agent_name=agent_name,
                anomaly_type=AnomalyType.CATEGORY_SHIFT,
                severity=severity,
                description=f"Category shift detected: {shifted_category} changed by {max_diff*100:.1f}%",
                evidence={
                    "shifted_category": shifted_category,
                    "baseline_dist": baseline_dist,
                    "recent_dist": recent_dist,
                    "max_diff": float(max_diff)
                },
                detected_at=datetime.utcnow()
            )
        
        return None
    
    def _detect_frequency_spike(self, agent_id: str, agent_name: str,
                               behaviors: List[BehaviorRecord]) -> Optional[AnomalyAlert]:
        """Detect unusual activity frequency"""
        
        if len(behaviors) < 20:
            return None
        
        now = datetime.utcnow()
        hour_ago = now - timedelta(hours=1)
        day_ago = now - timedelta(days=1)
        
        # Count actions in last hour vs typical hourly rate
        last_hour = [b for b in behaviors if b.timestamp > hour_ago]
        last_day = [b for b in behaviors if b.timestamp > day_ago]
        
        if not last_day:
            return None
        
        # Typical hourly rate
        typical_rate = len(last_day) / 24.0
        
        # Current rate
        current_rate = len(last_hour)
        
        if current_rate > typical_rate * self.frequency_multiplier:
            severity = min(1.0, current_rate / (typical_rate * 5))
            
            return AnomalyAlert(
                agent_id=agent_id,
                agent_name=agent_name,
                anomaly_type=AnomalyType.FREQUENCY_SPIKE,
                severity=severity,
                description=f"Activity spike: {current_rate:.1f}/hr vs typical {typical_rate:.1f}/hr",
                evidence={
                    "current_rate": float(current_rate),
                    "typical_rate": float(typical_rate),
                    "multiplier": float(current_rate / (typical_rate or 1))
                },
                detected_at=datetime.utcnow()
            )
        
        return None
    
    def _detect_high_risk_pattern(self, agent_id: str, agent_name: str,
                                  behaviors: List[BehaviorRecord]) -> Optional[AnomalyAlert]:
        """Detect pattern of high-risk actions"""
        
        from .impact import impact_analyzer, RiskLevel
        
        recent = behaviors[:self.recent_window]
        
        if not recent:
            return None
        
        # Count high-risk actions
        high_risk_count = 0
        for b in recent:
            impact = impact_analyzer.analyze(b.input_summary)
            if impact.risk_level.value >= RiskLevel.HIGH.value:
                high_risk_count += 1
        
        high_risk_ratio = high_risk_count / len(recent)
        
        if high_risk_ratio > self.high_risk_threshold:
            severity = min(1.0, high_risk_ratio * 1.5)
            
            return AnomalyAlert(
                agent_id=agent_id,
                agent_name=agent_name,
                anomaly_type=AnomalyType.HIGH_RISK_PATTERN,
                severity=severity,
                description=f"High-risk pattern: {high_risk_ratio*100:.1f}% of recent actions are high-risk",
                evidence={
                    "high_risk_count": high_risk_count,
                    "total_recent": len(recent),
                    "ratio": float(high_risk_ratio)
                },
                detected_at=datetime.utcnow()
            )
        
        return None

# Global instance
anomaly_detector = AnomalyDetector()
