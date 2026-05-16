"""
Agents API
Create, retrieve, and manage agents with endocrine-based reputation
"""

from typing import List, Optional
from uuid import UUID

from fastapi import APIRouter, HTTPException, Query
import structlog

from src.models import (
    Agent, AgentCreate, AgentResponse, AgentType, 
    Hormone, PrivilegeLevel
)
from src.core.database import create_agent, get_agent, list_agents, get_agent_by_fingerprint

router = APIRouter()
logger = structlog.get_logger()


@router.post("/", response_model=AgentResponse, status_code=201)
async def create_new_agent(agent: AgentCreate):
    """
    Create a new agent with initial endocrine state
    
    All 8 hormone levels start at baseline (0.5) unless specified.
    """
    logger.info("creating_agent", name=agent.name, type=agent.agent_type.value)
    
    created = await create_agent(agent)
    
    return AgentResponse(
        id=created.id,
        name=created.name,
        fingerprint=created.fingerprint,
        agent_type=created.agent_type,
        hormone_levels={h.value: v for h, v in created.endocrine_state.levels.items()},
        alignment=created.alignment,
        privilege_level=created.privilege_level,
        health_status=created.health_status
    )


@router.get("/{agent_id}", response_model=AgentResponse)
async def get_agent_by_id(agent_id: UUID):
    """
    Get agent by UUID
    
    Returns current endocrine state, alignment, and health status.
    """
    agent = await get_agent(agent_id)
    
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    
    return AgentResponse(
        id=agent.id,
        name=agent.name,
        fingerprint=agent.fingerprint,
        agent_type=agent.agent_type,
        hormone_levels={h.value: v for h, v in agent.endocrine_state.levels.items()},
        alignment=agent.alignment,
        privilege_level=agent.privilege_level,
        health_status=agent.health_status
    )


@router.get("/fingerprint/{fingerprint}", response_model=AgentResponse)
async def get_agent_by_fp(fingerprint: str):
    """
    Get agent by fingerprint (PQC-bound identity)
    """
    agent = await get_agent_by_fingerprint(fingerprint)
    
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    
    return AgentResponse(
        id=agent.id,
        name=agent.name,
        fingerprint=agent.fingerprint,
        agent_type=agent.agent_type,
        hormone_levels={h.value: v for h, v in agent.endocrine_state.levels.items()},
        alignment=agent.alignment,
        privilege_level=agent.privilege_level,
        health_status=agent.health_status
    )


@router.get("/", response_model=List[AgentResponse])
async def list_all_agents(
    limit: int = Query(default=100, ge=1, le=1000),
    offset: int = Query(default=0, ge=0),
    agent_type: Optional[AgentType] = None
):
    """
    List agents with pagination and optional type filter
    """
    agents = await list_agents(limit=limit, offset=offset, agent_type=agent_type)
    
    return [
        AgentResponse(
            id=a.id,
            name=a.name,
            fingerprint=a.fingerprint,
            agent_type=a.agent_type,
            hormone_levels={h.value: v for h, v in a.endocrine_state.levels.items()},
            alignment=a.alignment,
            privilege_level=a.privilege_level,
            health_status=a.health_status
        )
        for a in agents
    ]


@router.get("/{agent_id}/hormones", response_model=dict)
async def get_agent_hormones(agent_id: UUID):
    """
    Get detailed hormone levels for an agent
    
    Returns all 8 hormone levels with their dimension mappings.
    """
    agent = await get_agent(agent_id)
    
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    
    from src.config import HORMONE_CONFIG
    
    result = {}
    for hormone in Hormone:
        level = agent.endocrine_state.levels.get(hormone, 0.5)
        config = HORMONE_CONFIG.get(hormone.value, {})
        result[hormone.value] = {
            "level": level,
            "dimension": config.get("dimension", "unknown"),
            "half_life": config.get("half_life", 0),
            "km": config.get("km", 0.3),
        }
    
    return {
        "agent_id": str(agent_id),
        "hormones": result,
        "alignment": agent.alignment,
        "health_status": agent.health_status.value
    }


@router.get("/{agent_id}/privilege", response_model=dict)
async def get_agent_privilege(agent_id: UUID):
    """
    Get privilege level with receptor-based calculation
    """
    agent = await get_agent(agent_id)
    
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    
    from src.core.reputation_engine import reputation_engine
    
    privilege = reputation_engine.calculate_privilege_level(agent.endocrine_state)
    
    # Calculate receptor responses
    responses = {}
    for hormone in Hormone:
        level = agent.endocrine_state.levels.get(hormone, 0.5)
        response = reputation_engine.receptor_response(hormone, level)
        responses[hormone.value] = round(response, 4)
    
    return {
        "agent_id": str(agent_id),
        "privilege_level": privilege.value,
        "receptor_responses": responses,
        "average_response": sum(responses.values()) / len(responses)
    }
