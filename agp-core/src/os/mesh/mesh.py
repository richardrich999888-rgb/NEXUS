"""
AGP-CORE: Mesh Coordination System
Inter-agent communication and consensus for multi-robot coordination.
"""

import time
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
import structlog

logger = structlog.get_logger()

class MessageStatus(Enum):
    PENDING = "pending"
    READ = "read"
    EXPIRED = "expired"

class ProposalState(Enum):
    OPEN = "open"
    APPROVED = "approved"
    REJECTED = "rejected"
    EXPIRED = "expired"

@dataclass
class Message:
    """Peer-to-peer message between agents"""
    id: str
    sender_id: str
    recipient_id: str
    content: Any
    timestamp: float = field(default_factory=time.time)
    status: MessageStatus = MessageStatus.PENDING
    ttl: int = 300  # Time-to-live in seconds

@dataclass
class Proposal:
    """Collective action proposal for consensus voting"""
    id: str
    proposer_id: str
    action: str
    description: str
    timestamp: float = field(default_factory=time.time)
    state: ProposalState = ProposalState.OPEN
    votes_for: List[str] = field(default_factory=list)
    votes_against: List[str] = field(default_factory=list)
    required_approval: float = 0.5  # % of voters needed
    ttl: int = 60  # Voting window in seconds

class MeshCoordinator:
    """
    Central coordinator for inter-agent communication and consensus.
    """
    
    def __init__(self):
        # Mailboxes: agent_id -> List[Message]
        self.mailboxes: Dict[str, List[Message]] = {}
        
        # Active proposals
        self.proposals: Dict[str, Proposal] = {}
        
        # Known agents (for quorum calculation)
        self.known_agents: set = set()
        
        logger.info("mesh_coordinator_initialized")
    
    # ========== Message Bus ==========
    
    def register_agent(self, agent_id: str):
        """Register an agent in the mesh network"""
        if agent_id not in self.known_agents:
            self.known_agents.add(agent_id)
            self.mailboxes[agent_id] = []
            logger.info("mesh_agent_registered", agent_id=agent_id)
    
    def send_message(self, sender_id: str, recipient_id: str, content: Any) -> Dict:
        """Send a message to another agent"""
        if recipient_id not in self.mailboxes:
            return {"status": "error", "reason": "Recipient not found"}
        
        import uuid
        msg = Message(
            id=str(uuid.uuid4()),
            sender_id=sender_id,
            recipient_id=recipient_id,
            content=content
        )
        self.mailboxes[recipient_id].append(msg)
        logger.info("message_sent", from_=sender_id, to=recipient_id)
        return {"status": "sent", "message_id": msg.id}
    
    def receive_messages(self, agent_id: str, mark_read: bool = True) -> List[Dict]:
        """Retrieve pending messages for an agent"""
        if agent_id not in self.mailboxes:
            return []
        
        now = time.time()
        messages = []
        
        for msg in self.mailboxes[agent_id]:
            # Skip expired
            if now - msg.timestamp > msg.ttl:
                msg.status = MessageStatus.EXPIRED
                continue
            if msg.status == MessageStatus.PENDING:
                messages.append({
                    "id": msg.id,
                    "from": msg.sender_id,
                    "content": msg.content,
                    "timestamp": msg.timestamp
                })
                if mark_read:
                    msg.status = MessageStatus.READ
        
        return messages
    
    def broadcast(self, sender_id: str, content: Any) -> Dict:
        """Broadcast a message to all known agents"""
        count = 0
        for agent_id in self.known_agents:
            if agent_id != sender_id:
                self.send_message(sender_id, agent_id, content)
                count += 1
        return {"status": "broadcast", "recipients": count}
    
    # ========== Consensus Engine ==========
    
    def propose(self, proposer_id: str, action: str, description: str,
                required_approval: float = 0.5, ttl: int = 60) -> Dict:
        """Create a proposal for collective action"""
        import uuid
        proposal_id = str(uuid.uuid4())
        
        proposal = Proposal(
            id=proposal_id,
            proposer_id=proposer_id,
            action=action,
            description=description,
            required_approval=required_approval,
            ttl=ttl
        )
        self.proposals[proposal_id] = proposal
        
        # Broadcast proposal to all agents
        self.broadcast(proposer_id, {
            "type": "proposal",
            "proposal_id": proposal_id,
            "action": action,
            "description": description
        })
        
        logger.info("proposal_created", id=proposal_id, action=action)
        return {"status": "proposed", "proposal_id": proposal_id}
    
    def vote(self, voter_id: str, proposal_id: str, vote: bool) -> Dict:
        """Cast a vote on a proposal"""
        if proposal_id not in self.proposals:
            return {"status": "error", "reason": "Proposal not found"}
        
        proposal = self.proposals[proposal_id]
        
        # Check if voting is still open
        if proposal.state != ProposalState.OPEN:
            return {"status": "error", "reason": f"Proposal already {proposal.state.value}"}
        
        if time.time() - proposal.timestamp > proposal.ttl:
            proposal.state = ProposalState.EXPIRED
            return {"status": "error", "reason": "Proposal expired"}
        
        # Already voted?
        if voter_id in proposal.votes_for or voter_id in proposal.votes_against:
            return {"status": "error", "reason": "Already voted"}
        
        # Record vote
        if vote:
            proposal.votes_for.append(voter_id)
        else:
            proposal.votes_against.append(voter_id)
        
        logger.info("vote_cast", proposal=proposal_id, voter=voter_id, vote=vote)
        
        # Check for resolution
        self._check_resolution(proposal)
        
        return {
            "status": "voted",
            "proposal_id": proposal_id,
            "current_state": proposal.state.value,
            "votes_for": len(proposal.votes_for),
            "votes_against": len(proposal.votes_against)
        }
    
    def _check_resolution(self, proposal: Proposal):
        """Check if proposal has enough votes to be resolved"""
        total_voters = len(self.known_agents)
        if total_voters == 0:
            return
        
        votes_for = len(proposal.votes_for)
        votes_against = len(proposal.votes_against)
        total_votes = votes_for + votes_against
        
        # All agents voted or majority reached
        if votes_for / total_voters >= proposal.required_approval:
            proposal.state = ProposalState.APPROVED
            logger.info("proposal_approved", id=proposal.id)
        elif votes_against / total_voters > (1 - proposal.required_approval):
            proposal.state = ProposalState.REJECTED
            logger.info("proposal_rejected", id=proposal.id)
    
    def get_proposal_status(self, proposal_id: str) -> Dict:
        """Get the current status of a proposal"""
        if proposal_id not in self.proposals:
            return {"status": "not_found"}
        
        p = self.proposals[proposal_id]
        return {
            "id": p.id,
            "action": p.action,
            "state": p.state.value,
            "votes_for": len(p.votes_for),
            "votes_against": len(p.votes_against),
            "required_approval": p.required_approval
        }

# Global instance
mesh = MeshCoordinator()
