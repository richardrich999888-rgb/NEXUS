"""
AGP-CORE Google ADK Integration
Creates AI agents with endocrine-based reputation using Google's Agent Development Kit
"""

from typing import Dict, List, Optional, Any, Callable
from datetime import datetime
import uuid
import asyncio

# Try to import Google ADK (requires Python 3.10+)
try:
    from google.adk.agents import Agent as ADKAgent
    ADK_AVAILABLE = True
except ImportError:
    ADKAgent = None
    ADK_AVAILABLE = False

from src.models import EndocrineState, Stimulus, StimulusType, Hormone
from src.core.reputation_engine import reputation_engine
from src.core.kairon_cache import kairon_cache


class AGPAgent:
    """
    AGP-CORE Agent wrapper around Google ADK Agent
    Combines Google's agent capabilities with endocrine-based reputation
    """
    
    def __init__(
        self,
        name: str,
        model: str = "gemini-2.0-flash",
        description: str = "",
        instruction: str = "",
        tools: Optional[List[Callable]] = None,
        initial_state: Optional[EndocrineState] = None
    ):
        self.id = uuid.uuid4()
        self.name = name
        self.model = model
        self.fingerprint = f"sha256:{uuid.uuid4().hex}"
        self.created_at = datetime.utcnow()
        
        # Endocrine state
        self.endocrine_state = initial_state or EndocrineState()
        
        # Create Google ADK agent if available
        if ADK_AVAILABLE and ADKAgent:
            self._adk_agent = ADKAgent(
                name=name,
                model=model,
                description=description or f"AGP Agent: {name}",
                instruction=instruction or "You are a helpful AI agent.",
                tools=tools or []
            )
        else:
            self._adk_agent = None
            self._description = description
            self._instruction = instruction
            self._tools = tools or []
        
        # Metrics
        self.total_tasks = 0
        self.successful_tasks = 0
        self.failed_tasks = 0
    
    @property
    def alignment(self) -> float:
        """Get current alignment score"""
        return reputation_engine.calculate_alignment(self.endocrine_state)
    
    @property
    def privilege_level(self) -> str:
        """Get current privilege level"""
        return reputation_engine.calculate_privilege_level(self.endocrine_state).value
    
    @property
    def health_status(self) -> str:
        """Get current health status"""
        return reputation_engine.calculate_health_status(self.endocrine_state).value
    
    @property
    def model_hash(self) -> str:
        """Alias for model - used by orchestrator for LLM selection"""
        return self.model
    
    async def execute(self, prompt: str) -> Dict[str, Any]:
        """
        Execute a task and update reputation based on outcome
        """
        self.total_tasks += 1
        start_time = datetime.utcnow()
        
        try:
            # Execute via Google ADK
            # Note: In production, this would use ADK's proper execution flow
            result = await self._execute_adk(prompt)
            
            # Calculate latency
            latency_ms = (datetime.utcnow() - start_time).total_seconds() * 1000
            
            # Success stimulus
            self.successful_tasks += 1
            stimulus = Stimulus(
                stimulus_type=StimulusType.TASK_SUCCESS,
                strength=0.8,
                difficulty=0.5,
                latency_ms=int(latency_ms)
            )
            
            changes, self.endocrine_state = reputation_engine.process_stimulus(
                self.endocrine_state, stimulus
            )
            
            return {
                "success": True,
                "result": result,
                "latency_ms": latency_ms,
                "hormone_changes": changes,
                "alignment": self.alignment
            }
            
        except Exception as e:
            # Failure stimulus
            self.failed_tasks += 1
            stimulus = Stimulus(
                stimulus_type=StimulusType.TASK_FAILURE,
                strength=0.5,
                error_severity=0.5
            )
            
            changes, self.endocrine_state = reputation_engine.process_stimulus(
                self.endocrine_state, stimulus
            )
            
            return {
                "success": False,
                "error": str(e),
                "hormone_changes": changes,
                "alignment": self.alignment
            }
    
    async def _execute_adk(self, prompt: str) -> str:
        """Execute prompt via Google ADK agent"""
        # This is a simplified execution
        # In production, use ADK's proper runner/session
        return f"[{self.name}] Processed: {prompt[:50]}..."
    
    async def collaborate(self, partner: "AGPAgent", task: str) -> Dict:
        """Collaborate with another agent"""
        # Record collaboration stimulus for both agents
        stimulus = Stimulus(
            stimulus_type=StimulusType.COLLABORATION,
            strength=0.6,
            partner_count=1,
            success_rate=0.8
        )
        
        _, self.endocrine_state = reputation_engine.process_stimulus(
            self.endocrine_state, stimulus
        )
        _, partner.endocrine_state = reputation_engine.process_stimulus(
            partner.endocrine_state, stimulus
        )
        
        return {
            "collaboration": True,
            "agents": [self.name, partner.name],
            "task": task,
            "self_alignment": self.alignment,
            "partner_alignment": partner.alignment
        }
    
    def get_state(self) -> Dict:
        """Get complete agent state"""
        return {
            "id": str(self.id),
            "name": self.name,
            "fingerprint": self.fingerprint,
            "alignment": self.alignment,
            "privilege_level": self.privilege_level,
            "health_status": self.health_status,
            "hormone_levels": {h.value: v for h, v in self.endocrine_state.levels.items()},
            "metrics": {
                "total_tasks": self.total_tasks,
                "successful_tasks": self.successful_tasks,
                "failed_tasks": self.failed_tasks,
                "success_rate": self.successful_tasks / max(1, self.total_tasks)
            }
        }


class AGPAgentRegistry:
    """Registry for managing AGP agents"""
    
    def __init__(self):
        self.agents: Dict[uuid.UUID, AGPAgent] = {}
    
    def create_agent(
        self,
        name: str,
        model: str = "gemini-2.0-flash",
        description: str = "",
        instruction: str = "",
        tools: Optional[List[Callable]] = None
    ) -> AGPAgent:
        """Create and register a new agent"""
        agent = AGPAgent(
            name=name,
            model=model,
            description=description,
            instruction=instruction,
            tools=tools
        )
        self.agents[agent.id] = agent
        return agent

    def register_agent(self, agent: AGPAgent):
        """Register an existing agent"""
        self.agents[agent.id] = agent
    
    def get_agent(self, agent_id: uuid.UUID) -> Optional[AGPAgent]:
        """Get agent by ID"""
        return self.agents.get(agent_id)
    
    def list_agents(self) -> List[Dict]:
        """List all agents"""
        return [agent.get_state() for agent in self.agents.values()]
    
    def get_top_agents(self, n: int = 10) -> List[Dict]:
        """Get top N agents by alignment"""
        sorted_agents = sorted(
            self.agents.values(),
            key=lambda a: a.alignment,
            reverse=True
        )
        return [agent.get_state() for agent in sorted_agents[:n]]


# Create AGP tool wrappers for Google ADK
def create_agp_tool(name: str, func: Callable, description: str = "") -> Callable:
    """Create a tool that tracks reputation"""
    async def wrapped_tool(*args, **kwargs):
        # Execute tool
        result = await func(*args, **kwargs) if asyncio.iscoroutinefunction(func) else func(*args, **kwargs)
        
        # Could track tool usage here for reputation
        return result
    
    wrapped_tool.__name__ = name
    wrapped_tool.__doc__ = description or func.__doc__
    return wrapped_tool


# Global registry
agent_registry = AGPAgentRegistry()


# Example tools for AGP agents
def get_agent_reputation(agent_name: str) -> dict:
    """
    Get the reputation status of an AGP agent.
    
    Args:
        agent_name: Name of the agent to query
        
    Returns:
        dict: Agent reputation data including alignment and privilege level
    """
    for agent in agent_registry.agents.values():
        if agent.name == agent_name:
            return {
                "status": "success",
                "agent": agent.get_state()
            }
    return {
        "status": "error",
        "error_message": f"Agent '{agent_name}' not found"
    }


def report_observation(agent_name: str, stimulus_type: str, magnitude: float) -> dict:
    """
    Report an observation about an agent's behavior.
    
    Args:
        agent_name: Name of the agent
        stimulus_type: Type of observation (task_success, task_failure, collaboration)
        magnitude: Magnitude of the stimulus (0.0 to 1.0)
        
    Returns:
        dict: Updated agent state
    """
    for agent in agent_registry.agents.values():
        if agent.name == agent_name:
            try:
                stype = StimulusType(stimulus_type)
                stimulus = Stimulus(stimulus_type=stype, strength=magnitude)
                _, agent.endocrine_state = reputation_engine.process_stimulus(
                    agent.endocrine_state, stimulus
                )
                return {
                    "status": "success",
                    "new_alignment": agent.alignment,
                    "new_privilege": agent.privilege_level
                }
            except ValueError:
                return {"status": "error", "error_message": f"Invalid stimulus type: {stimulus_type}"}
    return {"status": "error", "error_message": f"Agent '{agent_name}' not found"}
