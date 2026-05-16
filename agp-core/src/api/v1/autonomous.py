"""
Autonomous Agents API - Phase 5
"""

import uuid
from typing import List, Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services import (
    messaging_service, task_protocol, MessageType,
    swarm_coordinator, collective_intelligence, SwarmRole,
    decision_engine, goal_optimizer
)
from src.models import EndocrineState, Hormone

router = APIRouter(prefix="/agents", tags=["autonomous-agents"])


# =============================================================================
# MODELS
# =============================================================================

class RegisterAgentRequest(BaseModel):
    agent_id: uuid.UUID
    public_key: str
    capabilities: List[str]
    endpoint_url: Optional[str] = None

class SendMessageRequest(BaseModel):
    sender_id: uuid.UUID
    recipient_id: uuid.UUID
    message_type: str
    payload: dict

class TaskOfferRequest(BaseModel):
    offerer_id: uuid.UUID
    task_description: str
    reward: float
    required_capabilities: List[str]
    deadline_hours: int = 24

class CreateSwarmRequest(BaseModel):
    name: str
    objective: str
    founder_id: uuid.UUID
    founder_reputation: float = 0.5

class JoinSwarmRequest(BaseModel):
    agent_id: uuid.UUID
    reputation: float
    capabilities: List[str]

class ProposeDecisionRequest(BaseModel):
    question: str
    options: List[str]
    duration_hours: int = 24

class EvaluateActionRequest(BaseModel):
    agent_id: uuid.UUID
    reputation: float
    stake_amount: float
    available_balance: float
    active_tasks: int
    recent_success_rate: float
    action: dict


# =============================================================================
# MESSAGING ENDPOINTS
# =============================================================================

@router.post("/messaging/register")
async def register_agent(request: RegisterAgentRequest):
    """Register an agent for communication"""
    endpoint = messaging_service.register_agent(
        request.agent_id,
        request.public_key,
        request.capabilities,
        request.endpoint_url
    )
    return {"agent_id": str(endpoint.agent_id), "registered": True}

@router.post("/messaging/send")
async def send_message(request: SendMessageRequest):
    """Send a message to another agent"""
    try:
        msg_type = MessageType(request.message_type)
        message = messaging_service.send_message(
            request.sender_id,
            request.recipient_id,
            msg_type,
            request.payload
        )
        return {"message_id": str(message.id), "sent": True}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/messaging/{agent_id}/inbox")
async def get_inbox(agent_id: uuid.UUID, message_type: Optional[str] = None):
    """Get pending messages for an agent"""
    msg_type = MessageType(message_type) if message_type else None
    messages = messaging_service.get_messages(agent_id, msg_type)
    return [{"id": str(m.id), "type": m.message_type.value, "from": str(m.sender_id), 
             "payload": m.payload, "timestamp": m.timestamp.isoformat()} for m in messages]

@router.get("/messaging/online")
async def get_online_agents(capability: Optional[str] = None):
    """Get online agents"""
    agents = messaging_service.get_online_agents(capability)
    return [{"id": str(a.agent_id), "capabilities": a.capabilities, 
             "reputation": a.reputation_score} for a in agents]


# =============================================================================
# TASK NEGOTIATION ENDPOINTS
# =============================================================================

@router.post("/tasks/offer")
async def offer_task(request: TaskOfferRequest):
    """Offer a task to the network"""
    from datetime import datetime, timedelta
    deadline = datetime.utcnow() + timedelta(hours=request.deadline_hours)
    
    offer_id = task_protocol.offer_task(
        request.offerer_id,
        request.task_description,
        request.reward,
        request.required_capabilities,
        deadline
    )
    return {"offer_id": str(offer_id)}

@router.post("/tasks/{offer_id}/accept")
async def accept_task(offer_id: uuid.UUID, agent_id: uuid.UUID):
    """Accept a task offer"""
    success = task_protocol.accept_task(agent_id, offer_id)
    if not success:
        raise HTTPException(status_code=400, detail="Could not accept task")
    return {"accepted": True}


# =============================================================================
# SWARM ENDPOINTS
# =============================================================================

@router.post("/swarms/create")
async def create_swarm(request: CreateSwarmRequest):
    """Create a new swarm"""
    swarm = swarm_coordinator.create_swarm(
        request.name,
        request.objective,
        request.founder_id,
        request.founder_reputation
    )
    return {"swarm_id": str(swarm.id), "name": swarm.name}

@router.post("/swarms/{swarm_id}/join")
async def join_swarm(swarm_id: uuid.UUID, request: JoinSwarmRequest):
    """Join a swarm"""
    try:
        member = swarm_coordinator.join_swarm(
            swarm_id,
            request.agent_id,
            request.reputation,
            request.capabilities
        )
        return {"role": member.role.value, "joined": True}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/swarms/{swarm_id}/stats")
async def get_swarm_stats(swarm_id: uuid.UUID):
    """Get swarm statistics"""
    return swarm_coordinator.get_swarm_stats(swarm_id)

@router.post("/swarms/{swarm_id}/propose")
async def propose_decision(swarm_id: uuid.UUID, request: ProposeDecisionRequest):
    """Propose a decision for swarm voting"""
    decision = swarm_coordinator.propose_decision(
        swarm_id,
        request.question,
        request.options,
        request.duration_hours
    )
    return {"decision_id": str(decision.id)}

@router.post("/swarms/decisions/{decision_id}/vote")
async def cast_vote(decision_id: uuid.UUID, agent_id: uuid.UUID, option_index: int):
    """Cast a vote on a decision"""
    success = swarm_coordinator.cast_vote(decision_id, agent_id, option_index)
    return {"voted": success}

@router.get("/swarms/decisions/{decision_id}/result")
async def get_decision_result(decision_id: uuid.UUID):
    """Get decision result"""
    return swarm_coordinator.get_decision_result(decision_id)

@router.get("/swarms/{swarm_id}/patterns")
async def detect_patterns(swarm_id: uuid.UUID):
    """Detect emergent patterns in a swarm"""
    patterns = swarm_coordinator.detect_patterns(swarm_id)
    return [{"type": p.pattern_type, "description": p.description, 
             "confidence": p.confidence} for p in patterns]


# =============================================================================
# AUTONOMOUS DECISION ENDPOINTS
# =============================================================================

@router.post("/decisions/evaluate")
async def evaluate_action(request: EvaluateActionRequest):
    """Evaluate an action for an agent"""
    from src.services.autonomous_decision import DecisionContext
    
    context = DecisionContext(
        agent_id=request.agent_id,
        agent_state=EndocrineState(),  # Default state
        reputation=request.reputation,
        stake_amount=request.stake_amount,
        available_balance=request.available_balance,
        active_tasks=request.active_tasks,
        recent_success_rate=request.recent_success_rate,
        swarm_memberships=0
    )
    
    score, scores = decision_engine.evaluate_action(context, request.action)
    return {"total_score": score, "breakdown": scores}

@router.post("/decisions/should-proceed")
async def should_proceed(request: EvaluateActionRequest, min_confidence: float = 0.4):
    """Determine if an action should proceed"""
    from src.services.autonomous_decision import DecisionContext
    
    context = DecisionContext(
        agent_id=request.agent_id,
        agent_state=EndocrineState(),
        reputation=request.reputation,
        stake_amount=request.stake_amount,
        available_balance=request.available_balance,
        active_tasks=request.active_tasks,
        recent_success_rate=request.recent_success_rate,
        swarm_memberships=0
    )
    
    should, decision = decision_engine.should_proceed(context, request.action, min_confidence)
    
    return {
        "should_proceed": should,
        "confidence": decision.confidence,
        "risk_level": decision.risk_assessment.risk_level.value,
        "reasoning": decision.reasoning
    }

@router.get("/decisions/{agent_id}/stats")
async def get_decision_stats(agent_id: uuid.UUID):
    """Get decision statistics for an agent"""
    return decision_engine.get_agent_decision_stats(agent_id)
