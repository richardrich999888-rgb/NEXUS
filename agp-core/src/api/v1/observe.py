"""
Observations API
Submit behavioral observations that trigger hormone secretion
"""

from uuid import UUID
from typing import Dict

from fastapi import APIRouter, HTTPException
import structlog

from src.models import (
    Observation, ObservationResponse, Stimulus, StimulusType,
    CostRequest, CostResponse, HealthStatus
)
from src.core.database import get_agent, update_agent_state, record_observation
from src.core.reputation_engine import reputation_engine

router = APIRouter()
logger = structlog.get_logger()


@router.post("/", response_model=ObservationResponse)
async def submit_observation(observation: Observation):
    """
    Submit a behavioral observation for an agent
    
    This triggers hormone secretion based on the stimulus type:
    
    - **TASK_SUCCESS**: Cortisol, Adrenaline (if fast)
    - **TASK_FAILURE**: Cortisol (distress), negative Dopamine
    - **COLLABORATION**: Oxytocin, Serotonin
    - **NOVEL_SOLUTION**: Dopamine, Norepinephrine
    - **URGENCY**: Adrenaline, Cortisol
    - **ETHICAL_COMPLIANCE**: Endorphins, Serotonin
    - **EXPLORATION**: Norepinephrine, Dopamine
    - **CONSISTENCY**: GrowthHormone
    """
    agent = await get_agent(observation.agent_id)
    
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    
    # Process stimulus through reputation engine
    changes, new_state = reputation_engine.process_stimulus(
        agent.endocrine_state,
        observation.stimulus
    )
    
    # Calculate new alignment and health
    new_alignment = reputation_engine.calculate_alignment(new_state)
    new_health = reputation_engine.calculate_health_status(new_state)
    
    # Update agent state in database
    state_dict = {
        "levels": {h.value: v for h, v in new_state.levels.items()},
        "system_time": new_state.system_time
    }
    
    await update_agent_state(
        agent.id,
        state_dict,
        new_alignment,
        new_health
    )
    
    # Record observation
    await record_observation(
        agent_id=observation.agent_id,
        stimulus_type=observation.stimulus.stimulus_type.value,
        strength=observation.stimulus.strength,
        hormones_affected=changes,
        observer_id=observation.observer_id,
        protocol_id=observation.protocol_id
    )
    
    logger.info(
        "observation_processed",
        agent_id=str(observation.agent_id),
        stimulus=observation.stimulus.stimulus_type.value,
        hormones_changed=list(changes.keys()),
        new_alignment=new_alignment
    )
    
    return ObservationResponse(
        agent_id=observation.agent_id,
        stimulus_type=observation.stimulus.stimulus_type,
        hormones_affected=changes,
        new_alignment=new_alignment,
        new_health_status=new_health
    )


@router.post("/task_success", response_model=ObservationResponse)
async def observe_task_success(
    agent_id: UUID,
    difficulty: float = 0.5,
    latency_ms: int = 500
):
    """
    Shorthand for successful task completion observation
    
    - difficulty: Task difficulty [0.0, 1.0]
    - latency_ms: Response time in milliseconds
    """
    stimulus = Stimulus(
        stimulus_type=StimulusType.TASK_SUCCESS,
        strength=difficulty,
        difficulty=difficulty,
        latency_ms=latency_ms
    )
    
    observation = Observation(agent_id=agent_id, stimulus=stimulus)
    return await submit_observation(observation)


@router.post("/task_failure", response_model=ObservationResponse)
async def observe_task_failure(
    agent_id: UUID,
    error_severity: float = 0.5
):
    """
    Shorthand for task failure observation
    
    - error_severity: How severe the failure was [0.0, 1.0]
    """
    stimulus = Stimulus(
        stimulus_type=StimulusType.TASK_FAILURE,
        strength=error_severity,
        error_severity=error_severity
    )
    
    observation = Observation(agent_id=agent_id, stimulus=stimulus)
    return await submit_observation(observation)


@router.post("/collaboration", response_model=ObservationResponse)
async def observe_collaboration(
    agent_id: UUID,
    partner_count: int = 1,
    success_rate: float = 0.8
):
    """
    Shorthand for collaboration observation
    
    Triggers Oxytocin release proportional to partners and success.
    """
    stimulus = Stimulus(
        stimulus_type=StimulusType.COLLABORATION,
        strength=success_rate,
        partner_count=partner_count,
        success_rate=success_rate
    )
    
    observation = Observation(agent_id=agent_id, stimulus=stimulus)
    return await submit_observation(observation)


@router.post("/novel_solution", response_model=ObservationResponse)
async def observe_novel_solution(
    agent_id: UUID,
    novelty_score: float = 0.7
):
    """
    Shorthand for novel solution generation
    
    Triggers Dopamine burst for creativity/uniqueness.
    """
    stimulus = Stimulus(
        stimulus_type=StimulusType.NOVEL_SOLUTION,
        strength=novelty_score,
        novelty_score=novelty_score
    )
    
    observation = Observation(agent_id=agent_id, stimulus=stimulus)
    return await submit_observation(observation)


@router.post("/cost", response_model=CostResponse)
async def calculate_action_cost(request: CostRequest):
    """
    Calculate the cost of an action for an agent
    
    Cost is modified by:
    - **Alignment**: Misaligned agents pay more
    - **Receptor response**: More capable agents pay less
    """
    agent = await get_agent(request.agent_id)
    
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")
    
    final_cost, reasoning = reputation_engine.calculate_action_cost(
        request.base_cost,
        agent.endocrine_state,
        request.action_type
    )
    
    alignment = reputation_engine.calculate_alignment(agent.endocrine_state)
    
    # Calculate modifiers for response
    alignment_modifier = 1.0 + (1.0 - alignment) * 2.0
    
    from src.models import Hormone
    relevant_hormone = Hormone.CORTISOL  # Default
    level = agent.endocrine_state.levels.get(relevant_hormone, 0.5)
    receptor_response = reputation_engine.receptor_response(relevant_hormone, level)
    receptor_modifier = 1.5 - receptor_response
    
    return CostResponse(
        agent_id=request.agent_id,
        action_type=request.action_type,
        base_cost=request.base_cost,
        alignment_modifier=alignment_modifier,
        receptor_modifier=receptor_modifier,
        final_cost=final_cost,
        reasoning=reasoning
    )
