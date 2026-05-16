"""
FYNTRAX O-RAN RIC Adapter

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Adapter for O-RAN Near-RT RIC integration.
Exposes FYNTRAX control logic via A1, E2, and O1 interfaces.
"""

from typing import Dict, Any, Optional, List
from dataclasses import dataclass
import time


@dataclass
class RICMessage:
    """RIC message structure."""
    message_type: str
    payload: Dict[str, Any]
    timestamp: float
    source: str
    target: str


class RICAdapter:
    """
    O-RAN RIC Adapter.
    
    Provides interface between FYNTRAX control and O-RAN RIC.
    Filters xApp recommendations through Lyapunov supervisor.
    """
    
    def __init__(self, ric_endpoint: str = "localhost:8080"):
        """
        Initialize RIC adapter.
        
        Args:
            ric_endpoint: RIC API endpoint
        """
        self.ric_endpoint = ric_endpoint
        self.connected = False
        self.message_queue: List[RICMessage] = []
        self.processed_count = 0
        self.filtered_count = 0

    def connect(self) -> bool:
        """Connect to RIC (simulated)."""
        # In production, this would establish gRPC/REST connection
        self.connected = True
        return True

    def disconnect(self) -> None:
        """Disconnect from RIC."""
        self.connected = False

    def send_policy(self, policy: Dict[str, Any]) -> bool:
        """
        Send policy to RIC.
        
        Args:
            policy: Policy dictionary
            
        Returns:
            True if successful
        """
        if not self.connected:
            return False
        
        msg = RICMessage(
            message_type="POLICY_UPDATE",
            payload=policy,
            timestamp=time.time(),
            source="FYNTRAX",
            target="RIC",
        )
        
        self.message_queue.append(msg)
        return True

    def receive_recommendation(self) -> Optional[Dict[str, Any]]:
        """
        Receive recommendation from xApp.
        
        Returns:
            Recommendation dictionary or None
        """
        if not self.connected:
            return None
        
        # Simulated recommendation
        return {
            "action": "adjust_power",
            "target_cells": ["cell_001", "cell_002"],
            "parameters": {
                "power_reduction_db": 3.0,
                "timestamp": time.time(),
            },
        }

    def filter_recommendation(
        self,
        recommendation: Dict[str, Any],
        is_safe: bool,
    ) -> Dict[str, Any]:
        """
        Filter xApp recommendation through Lyapunov supervisor.
        
        Args:
            recommendation: Original recommendation
            is_safe: Result of Lyapunov safety check
            
        Returns:
            Filtered recommendation
        """
        self.processed_count += 1
        
        if is_safe:
            return recommendation
        
        # Recommendation was unsafe - apply conservative action
        self.filtered_count += 1
        
        return {
            "action": "maintain_current",
            "reason": "lyapunov_constraint_violation",
            "original_action": recommendation.get("action"),
            "timestamp": time.time(),
        }

    def report_kpi(self, kpis: Dict[str, float]) -> bool:
        """
        Report KPIs to RIC.
        
        Args:
            kpis: KPI dictionary
            
        Returns:
            True if successful
        """
        if not self.connected:
            return False
        
        msg = RICMessage(
            message_type="KPI_REPORT",
            payload=kpis,
            timestamp=time.time(),
            source="FYNTRAX",
            target="RIC",
        )
        
        self.message_queue.append(msg)
        return True

    def statistics(self) -> dict:
        """Get adapter statistics."""
        return {
            "connected": self.connected,
            "endpoint": self.ric_endpoint,
            "messages_queued": len(self.message_queue),
            "recommendations_processed": self.processed_count,
            "recommendations_filtered": self.filtered_count,
            "filter_rate": self.filtered_count / self.processed_count if self.processed_count > 0 else 0,
        }
