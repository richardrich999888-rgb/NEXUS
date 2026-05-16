"""
AGP-CORE Agent Factory
High-leverage agent templates for startup creation and scaling.
"""

from typing import Dict, Optional
import uuid
from datetime import datetime

from src.models import (
    Agent, AgentType, Hormone, EndocrineState, 
    PrivilegeLevel, HealthStatus
)
from src.agents.orchestrator import AgentOrchestrator

class AgentFactory:
    """
    Creates specialized agents with optimized endocrine baselines for specific roles.
    """
    
    @staticmethod
    def create_growth_agent(name: str = "GrowthLead") -> Agent:
        """
        Creates a Growth/Marketing agent.
        Optimized for: Dopamine (Novelty) + Oxytocin (Communication)
        """
        base_levels = {
            Hormone.DOPAMINE: 0.8,      # High drive for novelty/reward
            Hormone.OXYTOCIN: 0.7,      # High social capabilities
            Hormone.NOREPINEPHRINE: 0.6,  # Moderate dominance/action
            Hormone.ADRENALINE: 0.6,    # High activity
            Hormone.SEROTONIN: 0.4,     # Lower stability (needs change)
            Hormone.CORTISOL: 0.3       # Resilient to rejection
        }
        return AgentFactory._build_agent(
            name, AgentType.HYBRID, base_levels,
            "Specialized for user acquisition, viral marketing, and rapid experimentation."
        )

    @staticmethod
    def create_engineer_agent(name: str = "TechLead") -> Agent:
        """
        Creates an Engineering agent.
        Optimized for: Serotonin (Stability) + Endorphins (Focus/Pain tolerance)
        """
        base_levels = {
            Hormone.DOPAMINE: 0.4,      # Less distracted by shiny things
            Hormone.OXYTOCIN: 0.5,      # Balanced collaboration
            Hormone.SEROTONIN: 0.9,     # Extreme stability/consistency
            Hormone.CORTISOL: 0.4,      # Moderate stress tolerance
            Hormone.ENDORPHINS: 0.8,    # High endurance for debugging
            Hormone.GROWTH_HORMONE: 0.7 # Long-term architectural thinking
        }
        return AgentFactory._build_agent(
            name, AgentType.INFERENCE, base_levels,
            "Specialized for system architecture, reliable code generation, and debugging."
        )

    @staticmethod
    def create_product_agent(name: str = "ProductOwner") -> Agent:
        """
        Creates a Product agent.
        Optimized for: Balance (Allostasis) and Empathy (Oxytocin)
        """
        base_levels = {
            Hormone.DOPAMINE: 0.6,
            Hormone.OXYTOCIN: 0.9,      # Maximum user empathy
            Hormone.SEROTONIN: 0.7,     # Needs to be stable for the team
            Hormone.CORTISOL: 0.5,      # Handles pressure well
            Hormone.NOREPINEPHRINE: 0.6 # Alert to market changes
        }
        return AgentFactory._build_agent(
            name, AgentType.HYBRID, base_levels,
            "Specialized for user needs analysis, prioritization, and team coordination."
        )

    @staticmethod
    def create_system_agent(name: str = "System_Init") -> Agent:
        """
        Creates a System/Kernel agent.
        Optimized for: Maximum stability, low stress tolerance.
        """
        base_levels = {
            Hormone.SEROTONIN: 0.95,     # Extreme stability
            Hormone.CORTISOL: 0.1,       # Very low stress
            Hormone.DOPAMINE: 0.3,       # Low novelty seeking
            Hormone.NOREPINEPHRINE: 0.4, # Calm alertness
            Hormone.ENDORPHINS: 0.9,     # High endurance
        }
        return AgentFactory._build_agent(
            name, AgentType.INFERENCE, base_levels,
            "System agent for kernel initialization and daemon management."
        )

    @staticmethod
    def _build_agent(
        name: str, 
        agent_type: AgentType, 
        levels: Dict[Hormone, float],
        description: str
    ) -> Agent:
        """Internal builder"""
        # Fill missing hormones with baseline
        full_levels = {h: 0.5 for h in Hormone}
        full_levels.update(levels)
        
        state = EndocrineState(levels=full_levels)
        
        return Agent(
            id=uuid.uuid4(),
            name=name,
            fingerprint=f"startup_pqc_{name}_{uuid.uuid4().hex[:8]}",
            agent_type=agent_type,
            model_hash="gemini-2.0-flash", # Default startup model
            operator_id=None,
            endocrine_state=state,
            alignment=1.0,
            privilege_level=PrivilegeLevel.STANDARD,
            health_status=HealthStatus.NORMAL,
            created_at=datetime.utcnow(),
            updated_at=datetime.utcnow()
        )
