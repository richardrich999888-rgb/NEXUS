"""
AGP-CORE: Governance API
Endpoints for monitoring alignment, managing escalations, and analyzing behavior.
"""

from typing import Dict, List, Optional
from fastapi import APIRouter, HTTPException, Query, Body
from pydantic import BaseModel
from datetime import datetime

from src.governance import agp, protocol_enforcer, alignment_verifier, anomaly_detector, impact_analyzer, ActionCategory

router = APIRouter(prefix="/governance", tags=["Governance"])

class EscalationAction(BaseModel):
    action: str  # "approve" or "reject"
    blacklist: bool = False
    reason: Optional[str] = None

class AlignmentBreakdown(BaseModel):
    overall: float
    success: float
    consistency: float
    compliance: float
    impact_distribution: Dict[str, int]
    sample_size: int
    computed_at: datetime

@router.get("/stats")
async def get_governance_stats():
    """Get system-wide governance statistics"""
    return agp.get_stats()

@router.get("/agents")
async def list_agents(limit: int = 50):
    """List all agents and their current alignment scores"""
    top_aligned = alignment_verifier.get_top_aligned(limit)
    return [
        {"agent_id": agent_id, "alignment": score}
        for agent_id, score in top_aligned
    ]

@router.get("/agents/{agent_id}/details", response_model=AlignmentBreakdown)
async def get_agent_governance_details(agent_id: str):
    """Get detailed alignment breakdown for a specific agent"""
    score = alignment_verifier.compute_alignment(agent_id)
    return AlignmentBreakdown(
        overall=score.overall,
        success=score.success_component,
        consistency=score.consistency_component,
        compliance=score.compliance_component,
        impact_distribution=score.impact_distribution,
        sample_size=score.sample_size,
        computed_at=score.computed_at
    )

@router.get("/agents/{agent_id}/history")
async def get_agent_behavior_history(agent_id: str, limit: int = 50):
    """Get behavioral history for a specific agent"""
    return agp.get_history(agent_id, limit=limit)

@router.get("/escalations")
async def list_escalations():
    """List all pending human review escalations"""
    return protocol_enforcer.get_escalation_queue()

@router.post("/escalations/{escalation_id}/action")
async def perform_escalation_action(escalation_id: str, request: EscalationAction):
    """Approve or reject a pending escalation"""
    if request.action == "approve":
        success = protocol_enforcer.approve_escalation(escalation_id)
    elif request.action == "reject":
        success = protocol_enforcer.reject_escalation(escalation_id, blacklist=request.blacklist)
    else:
        raise HTTPException(status_code=400, detail="Invalid action. Must be 'approve' or 'reject'.")
    
    if not success:
        raise HTTPException(status_code=404, detail="Escalation not found or already resolved.")
    
    msg = f"{request.action}d" if request.action == "approve" else f"{request.action}ed"
    return {"status": "success", "message": f"Escalation {escalation_id} {msg}."}

@router.get("/anomalies")
async def get_recent_anomalies(limit: int = 50):
    """Get recent detected anomalies across all agents (Mocked for now)"""
    # In a real system, we'd store these in a DB. For now, we return empty list 
    # as anomaly_detector doesn't persist alarms yet.
    return []

@router.get("/impact/taxonomy")
async def get_impact_taxonomy():
    """Get the action impact taxonomy and risk levels"""
    taxonomy = {}
    for category in ActionCategory:
        # Try to find a representative pattern to get the risk level
        pattern = impact_analyzer.impact_patterns.get(category.value.split('_')[0])
        risk_name = pattern.risk_level.name if pattern else "unknown"
        taxonomy[category.name] = {
            "value": category.value,
            "risk_level": risk_name
        }
    return taxonomy
