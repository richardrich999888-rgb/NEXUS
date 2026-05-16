"""
Treasury & Grant Management Service - Phase 4 Week 15
"""

import uuid
from typing import Dict, List, Optional, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum


class ProposalStatus(str, Enum):
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    EXECUTED = "executed"
    CANCELLED = "cancelled"


class GrantStatus(str, Enum):
    APPLIED = "applied"
    UNDER_REVIEW = "under_review"
    APPROVED = "approved"
    ACTIVE = "active"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class TreasuryProposal:
    """Multi-sig treasury proposal"""
    id: uuid.UUID
    title: str
    description: str
    target_address: str
    amount: Decimal
    token_address: Optional[str]
    proposer: uuid.UUID
    approvers: List[uuid.UUID] = field(default_factory=list)
    required_approvals: int = 2
    status: ProposalStatus = ProposalStatus.PENDING
    created_at: datetime = field(default_factory=datetime.utcnow)
    deadline: datetime = None
    executed_at: Optional[datetime] = None
    tx_hash: Optional[str] = None


@dataclass
class Grant:
    """Ecosystem grant"""
    id: uuid.UUID
    title: str
    description: str
    category: str  # 'development', 'research', 'community', 'marketing'
    applicant_id: uuid.UUID
    requested_amount: Decimal
    approved_amount: Optional[Decimal]
    milestones: List[Dict]
    status: GrantStatus = GrantStatus.APPLIED
    applied_at: datetime = field(default_factory=datetime.utcnow)
    approved_at: Optional[datetime] = None
    reviewer_notes: str = ""


class TreasuryService:
    """
    Manages protocol treasury with multi-sig style approvals
    """
    
    def __init__(self, required_approvals: int = 2):
        self.proposals: Dict[uuid.UUID, TreasuryProposal] = {}
        self.balances: Dict[str, Decimal] = {}  # token_address -> balance
        self.budgets: Dict[str, Decimal] = {}   # category -> budget
        self.required_approvals = required_approvals
        self.executors: List[uuid.UUID] = []
    
    def add_executor(self, executor_id: uuid.UUID):
        """Add an address to the executor list"""
        if executor_id not in self.executors:
            self.executors.append(executor_id)
    
    def create_proposal(
        self,
        title: str,
        description: str,
        target_address: str,
        amount: Decimal,
        proposer: uuid.UUID,
        token_address: Optional[str] = None,
        duration_days: int = 7
    ) -> TreasuryProposal:
        """Create a new treasury proposal"""
        proposal = TreasuryProposal(
            id=uuid.uuid4(),
            title=title,
            description=description,
            target_address=target_address,
            amount=amount,
            token_address=token_address,
            proposer=proposer,
            required_approvals=self.required_approvals,
            deadline=datetime.utcnow() + timedelta(days=duration_days)
        )
        
        self.proposals[proposal.id] = proposal
        return proposal
    
    def approve_proposal(self, proposal_id: uuid.UUID, approver: uuid.UUID) -> bool:
        """Approve a proposal"""
        proposal = self.proposals.get(proposal_id)
        if not proposal:
            return False
        
        if proposal.status != ProposalStatus.PENDING:
            return False
        
        if datetime.utcnow() > proposal.deadline:
            proposal.status = ProposalStatus.CANCELLED
            return False
        
        if approver not in self.executors:
            return False
        
        if approver in proposal.approvers:
            return False
        
        proposal.approvers.append(approver)
        
        if len(proposal.approvers) >= proposal.required_approvals:
            proposal.status = ProposalStatus.APPROVED
        
        return True
    
    def execute_proposal(self, proposal_id: uuid.UUID, executor: uuid.UUID) -> Dict:
        """Execute an approved proposal"""
        proposal = self.proposals.get(proposal_id)
        if not proposal:
            raise ValueError("Proposal not found")
        
        if proposal.status != ProposalStatus.APPROVED:
            raise ValueError("Proposal not approved")
        
        if executor not in self.executors:
            raise ValueError("Not an executor")
        
        # Check balance
        token = proposal.token_address or "native"
        if self.balances.get(token, Decimal("0")) < proposal.amount:
            raise ValueError("Insufficient treasury balance")
        
        # Deduct balance
        self.balances[token] -= proposal.amount
        
        proposal.status = ProposalStatus.EXECUTED
        proposal.executed_at = datetime.utcnow()
        
        return {
            "proposal_id": str(proposal.id),
            "amount": float(proposal.amount),
            "target": proposal.target_address,
            "status": proposal.status.value
        }
    
    def deposit(self, token_address: str, amount: Decimal):
        """Deposit funds to treasury"""
        current = self.balances.get(token_address, Decimal("0"))
        self.balances[token_address] = current + amount
    
    def get_balance(self, token_address: Optional[str] = None) -> Dict[str, Decimal]:
        """Get treasury balance(s)"""
        if token_address:
            return {token_address: self.balances.get(token_address, Decimal("0"))}
        return dict(self.balances)
    
    def set_budget(self, category: str, amount: Decimal):
        """Set budget for a category"""
        self.budgets[category] = amount
    
    def get_pending_proposals(self) -> List[TreasuryProposal]:
        """Get all pending proposals"""
        return [p for p in self.proposals.values() if p.status == ProposalStatus.PENDING]


class GrantService:
    """
    Manages ecosystem grants for developers, researchers, and community
    """
    
    CATEGORIES = ["development", "research", "community", "marketing", "infrastructure"]
    
    def __init__(self, treasury: TreasuryService):
        self.treasury = treasury
        self.grants: Dict[uuid.UUID, Grant] = {}
        self.reviewers: List[uuid.UUID] = []
    
    def add_reviewer(self, reviewer_id: uuid.UUID):
        """Add a grant reviewer"""
        if reviewer_id not in self.reviewers:
            self.reviewers.append(reviewer_id)
    
    def apply_for_grant(
        self,
        title: str,
        description: str,
        category: str,
        applicant_id: uuid.UUID,
        requested_amount: Decimal,
        milestones: List[Dict]
    ) -> Grant:
        """Submit a grant application"""
        if category not in self.CATEGORIES:
            raise ValueError(f"Invalid category. Must be one of: {self.CATEGORIES}")
        
        grant = Grant(
            id=uuid.uuid4(),
            title=title,
            description=description,
            category=category,
            applicant_id=applicant_id,
            requested_amount=requested_amount,
            milestones=milestones
        )
        
        self.grants[grant.id] = grant
        return grant
    
    def review_grant(
        self,
        grant_id: uuid.UUID,
        reviewer_id: uuid.UUID,
        approved: bool,
        approved_amount: Optional[Decimal] = None,
        notes: str = ""
    ) -> Grant:
        """Review a grant application"""
        grant = self.grants.get(grant_id)
        if not grant:
            raise ValueError("Grant not found")
        
        if reviewer_id not in self.reviewers:
            raise ValueError("Not a reviewer")
        
        if grant.status not in [GrantStatus.APPLIED, GrantStatus.UNDER_REVIEW]:
            raise ValueError("Grant cannot be reviewed in current status")
        
        grant.reviewer_notes = notes
        
        if approved:
            grant.status = GrantStatus.APPROVED
            grant.approved_amount = approved_amount or grant.requested_amount
            grant.approved_at = datetime.utcnow()
        else:
            grant.status = GrantStatus.FAILED
        
        return grant
    
    def activate_grant(self, grant_id: uuid.UUID) -> Dict:
        """Activate an approved grant and release initial funding"""
        grant = self.grants.get(grant_id)
        if not grant or grant.status != GrantStatus.APPROVED:
            raise ValueError("Grant not approved")
        
        grant.status = GrantStatus.ACTIVE
        
        # Create treasury proposal for first milestone
        first_milestone = grant.milestones[0] if grant.milestones else None
        initial_amount = grant.approved_amount * Decimal("0.2")  # 20% upfront
        
        return {
            "grant_id": str(grant.id),
            "status": grant.status.value,
            "initial_funding": float(initial_amount),
            "total_approved": float(grant.approved_amount)
        }
    
    def complete_milestone(
        self,
        grant_id: uuid.UUID,
        milestone_index: int,
        reviewer_id: uuid.UUID
    ) -> Dict:
        """Mark a milestone as complete and release funding"""
        grant = self.grants.get(grant_id)
        if not grant or grant.status != GrantStatus.ACTIVE:
            raise ValueError("Grant not active")
        
        if milestone_index >= len(grant.milestones):
            raise ValueError("Invalid milestone index")
        
        milestone = grant.milestones[milestone_index]
        milestone["completed"] = True
        milestone["completed_at"] = datetime.utcnow().isoformat()
        
        # Calculate payment (remaining funds / remaining milestones)
        remaining = len([m for m in grant.milestones if not m.get("completed", False)])
        payment = grant.approved_amount * Decimal("0.8") / Decimal(len(grant.milestones))
        
        # Check if all milestones complete
        if remaining == 0:
            grant.status = GrantStatus.COMPLETED
        
        return {
            "grant_id": str(grant.id),
            "milestone": milestone_index,
            "payment": float(payment),
            "remaining_milestones": remaining,
            "status": grant.status.value
        }
    
    def get_grants_by_status(self, status: GrantStatus) -> List[Grant]:
        """Get all grants with a specific status"""
        return [g for g in self.grants.values() if g.status == status]
    
    def get_grants_summary(self) -> Dict:
        """Get summary of all grants"""
        total_requested = sum(g.requested_amount for g in self.grants.values())
        total_approved = sum(
            g.approved_amount or Decimal("0") 
            for g in self.grants.values() 
            if g.status in [GrantStatus.APPROVED, GrantStatus.ACTIVE, GrantStatus.COMPLETED]
        )
        
        by_category = {}
        for cat in self.CATEGORIES:
            cat_grants = [g for g in self.grants.values() if g.category == cat]
            by_category[cat] = {
                "count": len(cat_grants),
                "requested": float(sum(g.requested_amount for g in cat_grants)),
                "approved": float(sum(g.approved_amount or Decimal("0") for g in cat_grants if g.approved_amount))
            }
        
        return {
            "total_applications": len(self.grants),
            "total_requested": float(total_requested),
            "total_approved": float(total_approved),
            "by_category": by_category,
            "by_status": {
                status.value: len([g for g in self.grants.values() if g.status == status])
                for status in GrantStatus
            }
        }


# Create singleton instances
treasury_service = TreasuryService(required_approvals=2)
grant_service = GrantService(treasury_service)
