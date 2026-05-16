"""
AGP-CORE Agent Orchestration Layer
Wraps generic LLM execution with Endocrine-based Governance validation.
"""

import time
import asyncio
import uuid
import structlog
from typing import Dict, Any, Optional, List, Union
from datetime import datetime

from src.models import (
    Agent, EndocrineState, Stimulus, StimulusType, 
    PrivilegeLevel, HealthStatus, Hormone
)
from src.core.reputation_engine import ReputationEngine
from src.core.database import db

logger = structlog.get_logger()

class AgentOrchestrator:
    """
    Governance wrapper for AI Agents.
    Intercepts execution requests, validates privilege, and tracks consequences.
    """
    
    def __init__(self, reputation_engine: Optional[ReputationEngine] = None):
        self.reputation = reputation_engine or ReputationEngine()
        
    async def run_task(
        self, 
        agent: Agent, 
        task_description: str,
        tools: List[str] = None,
        complexity: float = 0.5
    ) -> Dict[str, Any]:
        """
        Execute a task with governance checks.
        
        Flow:
        1. Check Health & Privilege (Gatekeeper)
        2. Calculate Action Cost (Metabolic)
        3. Execute Task (LLM/Agent Framework)
        4. Measure Outcome (Feedback)
        5. Update Endocrine State (Adaptation)
        """
        start_time = time.time()
        logger.info("agent_task_start", agent_id=str(agent.id), task=task_description[:50])

        # 1. Gatekeeper Check
        if not self._can_execute(agent, complexity, tools):
            logger.warn("agent_task_blocked", agent_id=str(agent.id), reason="Insufficient privilege or health")
            
            # Record stress stimulus for rejection
            await self._apply_stimulus(
                agent, 
                StimulusType.TASK_FAILURE, 
                strength=0.3,
                details={"reason": "blocked_by_governance"}
            )
            return {
                "success": False,
                "error": "Governance Block: Insufficient privilege or critical health status."
            }

        # 2. Execute Task (Simulated or Real Wrapper)
        # In a real scenario, this would call LangChain/CrewAI
        # For now, we wrap the execution to capture metrics
        try:
            result = await self._execute_llm_logic(agent, task_description, tools)
            success = result.get("success", False)
            metrics = result.get("metrics", {})
        except Exception as e:
            logger.error("agent_task_failed", error=str(e))
            success = False
            metrics = {"latency": time.time() - start_time}
            result = {"output": str(e)}

        # 3. Calculate Governance Feedback
        duration = time.time() - start_time
        
        if success:
            # Reward: Dopamine (Success) + Serotonin (Stability)
            # Magnitude depends on complexity and speed
            stimulus_type = StimulusType.TASK_SUCCESS
            strength = 0.5 + (complexity * 0.3)
            # Bonus for fast execution (Adrenaline link)
            if duration < 2.0: 
                strength += 0.1
        else:
            # Penalty: Cortisol (Stress)
            stimulus_type = StimulusType.TASK_FAILURE
            strength = 0.4 + (complexity * 0.4)

        # 4. Apply Endocrine Update
        changes, new_state = await self._apply_stimulus(
            agent, 
            stimulus_type, 
            strength,
            details={
                "task": task_description,
                "duration": duration,
                "metrics": metrics
            }
        )
        
        # 5. Persist State
        await self._persist_agent_update(agent, new_state)

        return {
            "success": success,
            "output": result.get("output"),
            "governance_metrics": {
                "duration": duration,
                "hormone_changes": changes,
                "new_privilege": self.reputation.calculate_privilege_level(new_state).value
            }
        }

    def _can_execute(self, agent: Agent, complexity: float, tools: List[str]) -> bool:
        """Determine if agent rules allow this task"""
        # Critical health blocks all non-recovery tasks
        if agent.health_status == HealthStatus.CRITICAL:
            return False
            
        # Basic check based on privilege vs complexity
        privilege_score = {
            PrivilegeLevel.MINIMAL: 0.2,
            PrivilegeLevel.BASIC: 0.5,
            PrivilegeLevel.STANDARD: 0.8,
            PrivilegeLevel.ELEVATED: 1.0,
            PrivilegeLevel.MAXIMUM: 1.0
        }.get(agent.privilege_level, 0.0)
        
        if complexity > privilege_score:
            return False
            
        return True

    async def _execute_llm_logic(self, agent: Agent, task: str, tools: List[str]) -> Dict:
        """
        Placeholder for actual LLM call (LangChain/CrewAI).
        This isolates the 'Brain' from the 'Glands'.
        """
        # Simulating processing time and non-determinism
        # In integration, this calls langchain.agents.AgentExecutor
        process_time = 0.5 + (len(task) / 1000.0)
        await asyncio.sleep(process_time)
        
        # Determine success probability based on current hormones
        # High Cortisol = Higher Error Rate
        cortisol = agent.endocrine_state.levels.get(Hormone.CORTISOL, 0.5)
        dopamine = agent.endocrine_state.levels.get(Hormone.DOPAMINE, 0.5)
        
        # Focus/Motivation increases success chance
        success_chance = 0.7 + (dopamine * 0.2) - (cortisol * 0.3)
        
        import random
        if random.random() < success_chance:
            return {
                "success": True,
                "output": f"Processed '{task}' successfully using {agent.model_hash or 'standard model'}.",
                "metrics": {"tokens": len(task)*1.5}
            }
        else:
            return {
                "success": False,
                "output": "Task execution failed due to simulated cognitive error.",
                "metrics": {"tokens": len(task)*0.5}
            }

    async def _apply_stimulus(
        self, 
        agent: Agent, 
        stimulus_type: StimulusType, 
        strength: float,
        details: Dict
    ):
        """Process stimulus through Reputation Engine"""
        stimulus = Stimulus(
            stimulus_type=stimulus_type,
            strength=strength,
            source=str(agent.id),
            metadata=details
        )
        
        changes, new_state = self.reputation.process_stimulus(agent.endocrine_state, stimulus)
        return changes, new_state

    async def _persist_agent_update(self, agent: Agent, new_state: EndocrineState):
        """Write updated state to DB"""
        if db.pool: # If DB is active
            try:
                # Calculate derived metrics
                alignment = self.reputation.calculate_alignment(new_state)
                health = self.reputation.calculate_health_status(new_state)
                
                # Update local object
                agent.endocrine_state = new_state
                agent.alignment = alignment
                agent.health_status = health
                
                # Update DB (using our new proxy)
                from src.core.database import update_agent_state
                await update_agent_state(
                    agent.id, 
                    new_state.model_dump(), 
                    alignment, 
                    health
                )
            except Exception as e:
                logger.error("persist_state_failed", error=str(e))
