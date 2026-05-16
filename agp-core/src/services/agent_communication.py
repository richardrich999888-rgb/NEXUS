"""
Agent Communication Protocol - Phase 5
Secure messaging between autonomous agents
"""

import uuid
import hashlib
import hmac
import time
from typing import Dict, List, Optional, Any, Callable
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
import asyncio


class MessageType(str, Enum):
    REQUEST = "request"
    RESPONSE = "response"
    BROADCAST = "broadcast"
    HEARTBEAT = "heartbeat"
    HANDSHAKE = "handshake"
    TASK_OFFER = "task_offer"
    TASK_ACCEPT = "task_accept"
    TASK_COMPLETE = "task_complete"
    REPUTATION_QUERY = "reputation_query"
    REPUTATION_ATTESTATION = "reputation_attestation"


class ChannelState(str, Enum):
    PENDING = "pending"
    ESTABLISHED = "established"
    CLOSED = "closed"


@dataclass
class AgentMessage:
    """Message between agents"""
    id: uuid.UUID
    sender_id: uuid.UUID
    recipient_id: Optional[uuid.UUID]  # None for broadcast
    message_type: MessageType
    payload: Dict[str, Any]
    timestamp: datetime
    signature: str
    nonce: int
    reply_to: Optional[uuid.UUID] = None
    ttl_seconds: int = 300


@dataclass
class SecureChannel:
    """Secure communication channel between two agents"""
    id: uuid.UUID
    agent_a: uuid.UUID
    agent_b: uuid.UUID
    shared_secret: str
    state: ChannelState
    created_at: datetime
    last_activity: datetime
    message_count: int = 0


@dataclass
class AgentEndpoint:
    """Agent communication endpoint"""
    agent_id: uuid.UUID
    public_key: str
    endpoint_url: Optional[str]
    capabilities: List[str]
    reputation_score: float
    last_seen: datetime
    is_online: bool = True


class AgentMessagingService:
    """
    Handles agent-to-agent communication
    
    Features:
    - Secure channel establishment
    - Message signing and verification
    - Broadcast messaging
    - Message routing
    """
    
    def __init__(self):
        self.endpoints: Dict[uuid.UUID, AgentEndpoint] = {}
        self.channels: Dict[uuid.UUID, SecureChannel] = {}
        self.message_queue: Dict[uuid.UUID, List[AgentMessage]] = defaultdict(list)
        self.message_handlers: Dict[MessageType, List[Callable]] = defaultdict(list)
        self.nonce_tracker: Dict[uuid.UUID, int] = defaultdict(int)
    
    def register_agent(
        self,
        agent_id: uuid.UUID,
        public_key: str,
        capabilities: List[str],
        endpoint_url: Optional[str] = None
    ) -> AgentEndpoint:
        """Register an agent for communication"""
        endpoint = AgentEndpoint(
            agent_id=agent_id,
            public_key=public_key,
            endpoint_url=endpoint_url,
            capabilities=capabilities,
            reputation_score=0.5,
            last_seen=datetime.utcnow()
        )
        self.endpoints[agent_id] = endpoint
        return endpoint
    
    def establish_channel(
        self,
        agent_a: uuid.UUID,
        agent_b: uuid.UUID
    ) -> SecureChannel:
        """Establish secure channel between two agents"""
        if agent_a not in self.endpoints or agent_b not in self.endpoints:
            raise ValueError("Both agents must be registered")
        
        # Generate shared secret (simplified - in production use DH key exchange)
        shared_secret = hashlib.sha256(
            f"{agent_a}{agent_b}{time.time()}".encode()
        ).hexdigest()
        
        channel = SecureChannel(
            id=uuid.uuid4(),
            agent_a=agent_a,
            agent_b=agent_b,
            shared_secret=shared_secret,
            state=ChannelState.ESTABLISHED,
            created_at=datetime.utcnow(),
            last_activity=datetime.utcnow()
        )
        
        self.channels[channel.id] = channel
        return channel
    
    def send_message(
        self,
        sender_id: uuid.UUID,
        recipient_id: uuid.UUID,
        message_type: MessageType,
        payload: Dict[str, Any],
        reply_to: Optional[uuid.UUID] = None
    ) -> AgentMessage:
        """Send a message to another agent"""
        if sender_id not in self.endpoints:
            raise ValueError("Sender not registered")
        
        # Increment nonce for replay protection
        self.nonce_tracker[sender_id] += 1
        nonce = self.nonce_tracker[sender_id]
        
        # Create signature
        signature = self._sign_message(sender_id, recipient_id, payload, nonce)
        
        message = AgentMessage(
            id=uuid.uuid4(),
            sender_id=sender_id,
            recipient_id=recipient_id,
            message_type=message_type,
            payload=payload,
            timestamp=datetime.utcnow(),
            signature=signature,
            nonce=nonce,
            reply_to=reply_to
        )
        
        # Queue message for recipient
        self.message_queue[recipient_id].append(message)
        
        # Trigger handlers
        self._trigger_handlers(message)
        
        return message
    
    def broadcast(
        self,
        sender_id: uuid.UUID,
        message_type: MessageType,
        payload: Dict[str, Any],
        capability_filter: Optional[str] = None
    ) -> List[AgentMessage]:
        """Broadcast message to multiple agents"""
        messages = []
        
        for agent_id, endpoint in self.endpoints.items():
            if agent_id == sender_id:
                continue
            
            if capability_filter and capability_filter not in endpoint.capabilities:
                continue
            
            msg = self.send_message(sender_id, agent_id, message_type, payload)
            messages.append(msg)
        
        return messages
    
    def get_messages(
        self,
        agent_id: uuid.UUID,
        message_type: Optional[MessageType] = None,
        since: Optional[datetime] = None
    ) -> List[AgentMessage]:
        """Get pending messages for an agent"""
        messages = self.message_queue.get(agent_id, [])
        
        if message_type:
            messages = [m for m in messages if m.message_type == message_type]
        
        if since:
            messages = [m for m in messages if m.timestamp > since]
        
        return messages
    
    def acknowledge_message(self, agent_id: uuid.UUID, message_id: uuid.UUID):
        """Acknowledge receipt of a message"""
        self.message_queue[agent_id] = [
            m for m in self.message_queue[agent_id] if m.id != message_id
        ]
    
    def register_handler(self, message_type: MessageType, handler: Callable):
        """Register a handler for a message type"""
        self.message_handlers[message_type].append(handler)
    
    def _sign_message(
        self,
        sender_id: uuid.UUID,
        recipient_id: uuid.UUID,
        payload: Dict,
        nonce: int
    ) -> str:
        """Sign a message (simplified HMAC)"""
        sender = self.endpoints.get(sender_id)
        if not sender:
            return ""
        
        message_bytes = f"{sender_id}{recipient_id}{payload}{nonce}".encode()
        return hmac.new(
            sender.public_key.encode(),
            message_bytes,
            hashlib.sha256
        ).hexdigest()
    
    def verify_message(self, message: AgentMessage) -> bool:
        """Verify message signature"""
        expected = self._sign_message(
            message.sender_id,
            message.recipient_id,
            message.payload,
            message.nonce
        )
        return hmac.compare_digest(expected, message.signature)
    
    def _trigger_handlers(self, message: AgentMessage):
        """Trigger registered handlers for a message"""
        for handler in self.message_handlers.get(message.message_type, []):
            try:
                handler(message)
            except Exception:
                pass  # Log in production
    
    def get_online_agents(self, capability: Optional[str] = None) -> List[AgentEndpoint]:
        """Get list of online agents"""
        agents = [e for e in self.endpoints.values() if e.is_online]
        
        if capability:
            agents = [a for a in agents if capability in a.capabilities]
        
        return agents
    
    def update_reputation(self, agent_id: uuid.UUID, score: float):
        """Update agent reputation score"""
        if agent_id in self.endpoints:
            self.endpoints[agent_id].reputation_score = max(0.0, min(1.0, score))


class TaskNegotiationProtocol:
    """
    Protocol for agents to negotiate task assignments
    """
    
    def __init__(self, messaging: AgentMessagingService):
        self.messaging = messaging
        self.open_offers: Dict[uuid.UUID, Dict] = {}
        self.accepted_tasks: Dict[uuid.UUID, Dict] = {}
    
    def offer_task(
        self,
        offerer_id: uuid.UUID,
        task_description: str,
        reward: float,
        required_capabilities: List[str],
        deadline: datetime
    ) -> uuid.UUID:
        """Offer a task to the network"""
        offer_id = uuid.uuid4()
        
        offer = {
            "offer_id": str(offer_id),
            "task": task_description,
            "reward": reward,
            "capabilities": required_capabilities,
            "deadline": deadline.isoformat(),
            "offerer": str(offerer_id)
        }
        
        self.open_offers[offer_id] = offer
        
        # Broadcast to capable agents
        for cap in required_capabilities:
            self.messaging.broadcast(
                offerer_id,
                MessageType.TASK_OFFER,
                offer,
                capability_filter=cap
            )
        
        return offer_id
    
    def accept_task(
        self,
        agent_id: uuid.UUID,
        offer_id: uuid.UUID,
        bid_modifier: float = 1.0
    ) -> bool:
        """Accept a task offer"""
        if offer_id not in self.open_offers:
            return False
        
        offer = self.open_offers[offer_id]
        offerer_id = uuid.UUID(offer["offerer"])
        
        self.messaging.send_message(
            agent_id,
            offerer_id,
            MessageType.TASK_ACCEPT,
            {
                "offer_id": str(offer_id),
                "acceptor": str(agent_id),
                "bid_modifier": bid_modifier
            }
        )
        
        self.accepted_tasks[offer_id] = {
            "offer": offer,
            "acceptor": agent_id,
            "status": "accepted"
        }
        
        del self.open_offers[offer_id]
        return True
    
    def complete_task(
        self,
        agent_id: uuid.UUID,
        offer_id: uuid.UUID,
        result: Dict[str, Any]
    ) -> bool:
        """Mark a task as complete"""
        if offer_id not in self.accepted_tasks:
            return False
        
        task = self.accepted_tasks[offer_id]
        offerer_id = uuid.UUID(task["offer"]["offerer"])
        
        self.messaging.send_message(
            agent_id,
            offerer_id,
            MessageType.TASK_COMPLETE,
            {
                "offer_id": str(offer_id),
                "result": result
            }
        )
        
        task["status"] = "completed"
        task["result"] = result
        
        return True


# Create singleton instances
messaging_service = AgentMessagingService()
task_protocol = TaskNegotiationProtocol(messaging_service)
