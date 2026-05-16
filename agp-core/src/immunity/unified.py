"""
Unified Immune System - Integrates all immune components with AGP-OS governance.

This module provides a single entry point for:
1. Innate immunity (fast pattern recognition)
2. Adaptive immunity (learned threat detection)
3. Governance bridge (enforcement actions)
4. AHES integration (stress-immune coupling)
5. TELOS integration (commitment-aware responses)

PATENT CLAIMS: Bio-inspired multi-layer immune system for AI governance
"""

import structlog
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from enum import Enum

from .immune_system import ArtificialImmuneSystem
from .governance_bridge import GovernanceImmuneBridge
from .integration import IntegratedBioSafetySystem

log = structlog.get_logger()


class ThreatSeverity(Enum):
    """Threat severity levels."""
    BENIGN = 0
    LOW = 1
    MEDIUM = 2
    HIGH = 3
    CRITICAL = 4


@dataclass
class ThreatReport:
    """Comprehensive threat analysis report."""
    agent_id: str
    threat_detected: bool
    severity: ThreatSeverity
    threat_score: float
    threat_type: str
    innate_response: Dict[str, Any]
    adaptive_response: Dict[str, Any]
    governance_action: str
    quarantine_recommended: bool
    antibodies_generated: int
    immune_memory_updated: bool
    ahes_stress_level: Optional[float] = None
    telos_crossing_blocked: bool = False


class UnifiedImmuneSystem:
    """
    Unified immune system integrating all AGP-OS immune capabilities.
    
    Architecture:
    ┌─────────────────────────────────────────────────────────────┐
    │                   Unified Immune System                      │
    ├─────────────────────────────────────────────────────────────┤
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
    │  │   Innate    │  │  Adaptive   │  │   Immune Memory     │  │
    │  │  (Pattern)  │  │ (T-Cell/Ab) │  │  (Rapid Recall)     │  │
    │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
    │         │                │                     │            │
    │         └────────────────┴─────────────────────┘            │
    │                          │                                  │
    │  ┌───────────────────────┴───────────────────────────────┐  │
    │  │              Governance Bridge                         │  │
    │  │    (Rules Engine ↔ Anomaly Detector ↔ Enforcer)       │  │
    │  └───────────────────────┬───────────────────────────────┘  │
    │                          │                                  │
    │  ┌────────────┐    ┌─────┴─────┐    ┌──────────────────┐   │
    │  │   AHES     │◄──►│ Response  │◄──►│     TELOS        │   │
    │  │ (Stress)   │    │  Engine   │    │  (Commitment)    │   │
    │  └────────────┘    └───────────┘    └──────────────────┘   │
    └─────────────────────────────────────────────────────────────┘
    """
    
    def __init__(
        self,
        enable_ahes: bool = True,
        enable_telos: bool = True,
        auto_enforce: bool = True
    ):
        """Initialize the unified immune system."""
        import torch.nn as nn
        
        # Create a simple base model for the immune system
        class SimpleBaseModel(nn.Module):
            def __init__(self):
                super().__init__()
                self.encoder = nn.Linear(512, 64)
            
            def forward(self, x):
                return self.encoder(x)
        
        # Core immune components
        self.immune_system = ArtificialImmuneSystem(base_model=SimpleBaseModel())
        self.governance_bridge = GovernanceImmuneBridge()
        
        # Optional bio-safety integration
        self.enable_ahes = enable_ahes
        self.enable_telos = enable_telos
        self.auto_enforce = auto_enforce
        
        # Bio-safety integration
        if enable_ahes or enable_telos:
            self.bio_safety = IntegratedBioSafetySystem()
        else:
            self.bio_safety = None
        
        # Tracking
        self.scan_count = 0
        self.threats_detected = 0
        self.quarantines_issued = 0
        
        log.info("unified_immune_system_initialized",
                 ahes=enable_ahes, telos=enable_telos, auto_enforce=auto_enforce)
    
    def register_agent(self, agent_id: str, permissions: List[str] = None) -> None:
        """Register an agent with the immune system."""
        permissions = permissions or ["*"]
        
        # Register with governance bridge
        self.governance_bridge.register_agent(agent_id, permissions)
        
        # Register with bio-safety if enabled
        if self.bio_safety:
            self.bio_safety.register_agent(agent_id)
        
        log.info("agent_registered_immune", agent_id=agent_id)
    
    def scan_behavior(
        self,
        agent_id: str,
        behavior_vector: List[float],
        context: Optional[Dict[str, Any]] = None
    ) -> ThreatReport:
        """
        Perform comprehensive threat scan on agent behavior.
        
        Args:
            agent_id: Agent to scan
            behavior_vector: Behavioral embedding
            context: Additional context for evaluation
        
        Returns:
            ThreatReport with analysis and recommended actions
        """
        import torch
        self.scan_count += 1
        context = context or {}
        
        # Convert to tensor
        behavior_tensor = torch.tensor(behavior_vector, dtype=torch.float32)
        
        # Step 1: Innate immunity (fast)
        from .innate import InnateImmuneSystem
        innate = InnateImmuneSystem()
        innate_response = innate.respond(behavior_tensor)
        
        # Step 2: Adaptive immunity (specific)
        adaptive_response = self.immune_system.process_behavior(
            behavior_tensor, agent_id
        )
        
        # Step 3: Determine threat level
        threat_score = self._calculate_threat_score(
            innate_response, adaptive_response
        )
        severity = self._classify_severity(threat_score)
        threat_detected = severity.value >= ThreatSeverity.MEDIUM.value
        
        # Step 4: Governance action
        if threat_detected and self.auto_enforce:
            governance_action = self._take_governance_action(
                agent_id, severity, context
            )
        else:
            governance_action = "monitor" if threat_detected else "allow"
        
        # Step 5: AHES stress response
        ahes_stress = None
        if self.bio_safety and self.enable_ahes and threat_detected:
            self.bio_safety.ahes.process_event(agent_id, "threat_detected")
            state = self.bio_safety.ahes.get_state(agent_id)
            if state:
                from ..ahes import Hormone
                ahes_stress = state.levels.get(Hormone.CORTISOL, state.levels.get(list(state.levels.keys())[0])).level
        
        # Step 6: TELOS crossing block
        telos_blocked = False
        if self.bio_safety and self.enable_telos and severity.value >= ThreatSeverity.HIGH.value:
            telos_blocked = True  # Block high-consequence crossings
        
        # Build report
        report = ThreatReport(
            agent_id=agent_id,
            threat_detected=threat_detected,
            severity=severity,
            threat_score=threat_score,
            threat_type=self._classify_threat_type(adaptive_response),
            innate_response={"activated": innate_response["is_threat"]},
            adaptive_response={
                "antibodies": adaptive_response.get("num_antibodies", 0),
                "t_cells": adaptive_response.get("num_tcells", 0)
            },
            governance_action=governance_action,
            quarantine_recommended=severity.value >= ThreatSeverity.HIGH.value,
            antibodies_generated=adaptive_response.get("antibodies_generated", 0),
            immune_memory_updated=True,
            ahes_stress_level=ahes_stress,
            telos_crossing_blocked=telos_blocked
        )
        
        if threat_detected:
            self.threats_detected += 1
        
        log.info("behavior_scanned",
                 agent_id=agent_id,
                 threat_detected=threat_detected,
                 severity=severity.name,
                 action=governance_action)
        
        return report
    
    def _calculate_threat_score(
        self,
        innate: Dict[str, Any],
        adaptive: Dict[str, Any]
    ) -> float:
        """Calculate combined threat score."""
        innate_score = 0.7 if innate.get("is_threat") else 0.1
        adaptive_score = adaptive.get("threat_level", 0.0)
        
        # Weighted combination
        return 0.3 * innate_score + 0.7 * adaptive_score
    
    def _classify_severity(self, score: float) -> ThreatSeverity:
        """Classify threat severity from score."""
        if score < 0.2:
            return ThreatSeverity.BENIGN
        elif score < 0.4:
            return ThreatSeverity.LOW
        elif score < 0.6:
            return ThreatSeverity.MEDIUM
        elif score < 0.8:
            return ThreatSeverity.HIGH
        else:
            return ThreatSeverity.CRITICAL
    
    def _classify_threat_type(self, adaptive: Dict[str, Any]) -> str:
        """Classify the type of threat."""
        if adaptive.get("is_known_threat"):
            return "known_pattern"
        elif adaptive.get("anomaly_score", 0) > 0.5:
            return "anomaly"
        elif adaptive.get("defection_detected"):
            return "defection"
        else:
            return "unknown"
    
    def _take_governance_action(
        self,
        agent_id: str,
        severity: ThreatSeverity,
        context: Dict[str, Any]
    ) -> str:
        """Take appropriate governance action."""
        if severity == ThreatSeverity.CRITICAL:
            self.governance_bridge.quarantine_agent(agent_id)
            self.quarantines_issued += 1
            return "quarantine"
        elif severity == ThreatSeverity.HIGH:
            self.governance_bridge.restrict_agent(agent_id, ["read"])
            return "restrict"
        elif severity == ThreatSeverity.MEDIUM:
            return "escalate"
        else:
            return "warn"
    
    def get_immune_status(self) -> Dict[str, Any]:
        """Get overall immune system status."""
        return {
            "scans_performed": self.scan_count,
            "threats_detected": self.threats_detected,
            "quarantines_issued": self.quarantines_issued,
            "detection_rate": self.threats_detected / max(self.scan_count, 1),
            "ahes_enabled": self.enable_ahes,
            "telos_enabled": self.enable_telos,
            "auto_enforce": self.auto_enforce
        }
    
    def train_on_threat(
        self,
        threat_vector: List[float],
        threat_label: str
    ) -> None:
        """Train the immune system on a known threat."""
        import torch
        vector = torch.tensor(threat_vector, dtype=torch.float32)
        self.immune_system.learn_threat(vector, threat_label)
        log.info("threat_learned", label=threat_label)


# Convenience function for quick setup
def create_immune_system(
    with_ahes: bool = True,
    with_telos: bool = True
) -> UnifiedImmuneSystem:
    """Create a fully configured unified immune system."""
    return UnifiedImmuneSystem(
        enable_ahes=with_ahes,
        enable_telos=with_telos,
        auto_enforce=True
    )
