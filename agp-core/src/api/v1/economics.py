"""
Token Economics API - Phase 4 Week 14-16
"""

import uuid
from decimal import Decimal
from typing import List, Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services import (
    distribution_service, economic_simulator, AllocationCategory,
    treasury_service, grant_service,
    bridge_service, ChainType
)

router = APIRouter(prefix="/economics", tags=["token-economics"])

# =============================================================================
# MODELS
# =============================================================================

class CreateVestingRequest(BaseModel):
    beneficiary_id: uuid.UUID
    category: str
    amount: float
    revocable: bool = True

class TreasuryProposalRequest(BaseModel):
    title: str
    description: str
    target_address: str
    amount: float
    token_address: Optional[str] = None

class GrantApplicationRequest(BaseModel):
    title: str
    description: str
    category: str
    requested_amount: float
    milestones: List[dict]

class BridgeRequest(BaseModel):
    source_chain: str
    target_chain: str
    sender: str
    recipient: str
    amount: float

# =============================================================================
# VESTING ENDPOINTS
# =============================================================================

@router.post("/vesting/create")
async def create_vesting(request: CreateVestingRequest):
    """Create a new vesting schedule"""
    try:
        category = AllocationCategory(request.category)
        schedule = distribution_service.create_vesting_schedule(
            beneficiary_id=request.beneficiary_id,
            category=category,
            amount=Decimal(str(request.amount)),
            revocable=request.revocable
        )
        return {
            "schedule_id": str(schedule.id),
            "category": schedule.category.value,
            "total_amount": float(schedule.total_amount),
            "cliff_days": schedule.cliff_duration_days,
            "vesting_days": schedule.vesting_duration_days
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/vesting/{schedule_id}/releasable")
async def get_releasable(schedule_id: uuid.UUID):
    """Get releasable amount for a vesting schedule"""
    releasable = distribution_service.calculate_releasable(schedule_id)
    vested = distribution_service.calculate_vested_amount(schedule_id)
    return {"schedule_id": str(schedule_id), "vested": float(vested), "releasable": float(releasable)}

@router.post("/vesting/{schedule_id}/release")
async def release_vested(schedule_id: uuid.UUID):
    """Release vested tokens"""
    released = distribution_service.release(schedule_id)
    return {"schedule_id": str(schedule_id), "released": float(released)}

@router.get("/distribution/summary")
async def get_distribution_summary():
    """Get token distribution summary"""
    return distribution_service.get_distribution_summary()

@router.get("/simulation/supply")
async def simulate_supply(months: int = 48):
    """Simulate circulating supply over time"""
    return economic_simulator.simulate_supply_schedule(months)

@router.get("/simulation/staking")
async def simulate_staking(initial_stake: float = 30, months: int = 24):
    """Simulate staking dynamics"""
    return economic_simulator.simulate_staking_scenario(
        initial_stake_percent=Decimal(str(initial_stake)),
        months=months
    )

# =============================================================================
# TREASURY ENDPOINTS
# =============================================================================

@router.post("/treasury/propose")
async def create_treasury_proposal(request: TreasuryProposalRequest, proposer_id: uuid.UUID):
    """Create a treasury spending proposal"""
    proposal = treasury_service.create_proposal(
        title=request.title,
        description=request.description,
        target_address=request.target_address,
        amount=Decimal(str(request.amount)),
        proposer=proposer_id,
        token_address=request.token_address
    )
    return {"proposal_id": str(proposal.id), "status": proposal.status.value, "deadline": proposal.deadline.isoformat()}

@router.post("/treasury/proposal/{proposal_id}/approve")
async def approve_proposal(proposal_id: uuid.UUID, approver_id: uuid.UUID):
    """Approve a treasury proposal"""
    success = treasury_service.approve_proposal(proposal_id, approver_id)
    if not success:
        raise HTTPException(status_code=400, detail="Approval failed")
    return {"approved": True}

@router.get("/treasury/balance")
async def get_treasury_balance():
    """Get treasury balances"""
    return treasury_service.get_balance()

@router.get("/treasury/proposals/pending")
async def get_pending_proposals():
    """Get pending treasury proposals"""
    return [{"id": str(p.id), "title": p.title, "amount": float(p.amount), "approvals": len(p.approvers)} 
            for p in treasury_service.get_pending_proposals()]

# =============================================================================
# GRANT ENDPOINTS
# =============================================================================

@router.post("/grants/apply")
async def apply_for_grant(request: GrantApplicationRequest, applicant_id: uuid.UUID):
    """Submit a grant application"""
    grant = grant_service.apply_for_grant(
        title=request.title,
        description=request.description,
        category=request.category,
        applicant_id=applicant_id,
        requested_amount=Decimal(str(request.requested_amount)),
        milestones=request.milestones
    )
    return {"grant_id": str(grant.id), "status": grant.status.value}

@router.get("/grants/summary")
async def get_grants_summary():
    """Get grants program summary"""
    return grant_service.get_grants_summary()

# =============================================================================
# BRIDGE ENDPOINTS
# =============================================================================

@router.get("/bridge/routes")
async def get_bridge_routes(source_chain: Optional[str] = None):
    """Get available bridge routes"""
    source = ChainType(source_chain) if source_chain else None
    routes = bridge_service.get_available_routes(source)
    return [{"source": r.source_chain.value, "target": r.target_chain.value, 
             "fee_percent": float(r.fee_percent), "min": float(r.min_amount),
             "est_minutes": r.estimated_time_minutes} for r in routes]

@router.post("/bridge/initiate")
async def initiate_bridge(request: BridgeRequest):
    """Initiate a cross-chain bridge transfer"""
    try:
        tx = bridge_service.initiate_bridge(
            source_chain=ChainType(request.source_chain),
            target_chain=ChainType(request.target_chain),
            sender=request.sender,
            recipient=request.recipient,
            amount=Decimal(str(request.amount))
        )
        return {
            "tx_id": str(tx.id),
            "status": tx.status.value,
            "fee": float(tx.fee),
            "estimated_completion": tx.estimated_completion.isoformat()
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/bridge/{tx_id}")
async def get_bridge_transaction(tx_id: uuid.UUID):
    """Get bridge transaction status"""
    tx = bridge_service.get_transaction(tx_id)
    if not tx:
        raise HTTPException(status_code=404, detail="Transaction not found")
    return {
        "tx_id": str(tx.id),
        "status": tx.status.value,
        "source_chain": tx.source_chain.value,
        "target_chain": tx.target_chain.value,
        "amount": float(tx.amount),
        "source_tx_hash": tx.source_tx_hash,
        "target_tx_hash": tx.target_tx_hash
    }

@router.get("/bridge/stats")
async def get_bridge_stats():
    """Get bridge statistics"""
    return bridge_service.get_bridge_stats()
