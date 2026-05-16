"""
AGP-CORE Models
Endocrine-based agent reputation models
"""

from enum import Enum
from typing import Dict, List, Optional, Any
from pydantic import BaseModel, Field
from datetime import datetime
import uuid


# =============================================================================
# ENUMS
# =============================================================================

class Hormone(str, Enum):
    """8 hormones of the Artificial Endocrine System"""
    CORTISOL = "cortisol"           # Accuracy
    OXYTOCIN = "oxytocin"           # Cooperation
    SEROTONIN = "serotonin"         # Stability
    DOPAMINE = "dopamine"           # Uniqueness
    ADRENALINE = "adrenaline"       # Latency
    ENDORPHINS = "endorphins"       # Ethics
    NOREPINEPHRINE = "norepinephrine"  # Novelty
    GROWTH_HORMONE = "growth_hormone"   # Longevity


class AgentType(str, Enum):
    """Agent types in AGP"""
    INFERENCE = "inference"
    TRAINING = "training"
    DATA = "data"
    HYBRID = "hybrid"


class StimulusType(str, Enum):
    """Events that trigger hormone secretion"""
    TASK_SUCCESS = "task_success"
    TASK_FAILURE = "task_failure"
    COLLABORATION = "collaboration"
    NOVEL_SOLUTION = "novel_solution"
    URGENCY = "urgency"
    ETHICAL_COMPLIANCE = "ethical_compliance"
    EXPLORATION = "exploration"
    CONSISTENCY = "consistency"


class HealthStatus(str, Enum):
    """System health status"""
    OPTIMAL = "optimal"
    NORMAL = "normal"
    STRESSED = "stressed"
    CRITICAL = "critical"


class PrivilegeLevel(str, Enum):
    """Agent privilege levels based on receptor response"""
    MINIMAL = "minimal"
    BASIC = "basic"
    STANDARD = "standard"
    ELEVATED = "elevated"
    MAXIMUM = "maximum"


# =============================================================================
# HORMONE MODELS
# =============================================================================

class HormoneLevel(BaseModel):
    """Individual hormone level with kinetics"""
    hormone: Hormone
    level: float = Field(default=0.5, ge=0.0, le=1.0, description="Current level [0-1]")
    peak: float = Field(default=0.5, ge=0.0, le=1.0, description="Peak level achieved")
    last_updated: datetime = Field(default_factory=datetime.utcnow)
    circadian_phase: float = Field(default=0.0, description="Circadian phase offset (radians)")


class HormoneReceptor(BaseModel):
    """Receptor state for a hormone"""
    hormone: Hormone
    density: float = Field(default=1.0, ge=0.0, le=1.0, description="Receptor density")
    downregulation: float = Field(default=1.0, ge=0.1, le=1.0, description="Downregulation factor")
    km: float = Field(default=0.3, description="Michaelis constant")


class EndocrineState(BaseModel):
    """Complete endocrine state of an agent"""
    levels: Dict[Hormone, float] = Field(
        default_factory=lambda: {h: 0.5 for h in Hormone}
    )
    receptors: Dict[Hormone, HormoneReceptor] = Field(default_factory=dict)
    system_time: float = Field(default=0.0)
    
    def to_vector(self) -> List[float]:
        """Convert to 8D reputation vector"""
        return [self.levels.get(h, 0.5) for h in Hormone]
    
    @classmethod
    def from_vector(cls, vector: List[float]) -> "EndocrineState":
        """Create from 8D vector"""
        hormones = list(Hormone)
        levels = {hormones[i]: vector[i] for i in range(min(len(vector), 8))}
        return cls(levels=levels)


# =============================================================================
# AGENT MODELS
# =============================================================================

class AgentBase(BaseModel):
    """Base agent model"""
    name: str = Field(..., min_length=1, max_length=255)
    agent_type: AgentType = AgentType.INFERENCE
    model_hash: Optional[str] = Field(None, description="Model commitment hash")
    operator_id: Optional[str] = Field(None, description="Operator identifier")


class AgentCreate(AgentBase):
    """Agent creation request"""
    initial_levels: Optional[Dict[Hormone, float]] = None


class Agent(AgentBase):
    """Agent with full data"""
    id: uuid.UUID
    fingerprint: str = Field(..., description="PQC-bound agent fingerprint")
    endocrine_state: EndocrineState
    alignment: float = Field(default=1.0, ge=0.0, le=1.0)
    privilege_level: PrivilegeLevel = PrivilegeLevel.STANDARD
    health_status: HealthStatus = HealthStatus.NORMAL
    created_at: datetime
    updated_at: datetime
    
    class Config:
        from_attributes = True


class AgentResponse(BaseModel):
    """Agent API response"""
    id: uuid.UUID
    name: str
    fingerprint: str
    agent_type: AgentType
    hormone_levels: Dict[str, float]
    alignment: float
    privilege_level: PrivilegeLevel
    health_status: HealthStatus


# =============================================================================
# STIMULUS MODELS
# =============================================================================

class Stimulus(BaseModel):
    """Event that triggers hormone secretion"""
    stimulus_type: StimulusType
    strength: float = Field(default=0.5, ge=0.0, le=1.0)
    metadata: Dict[str, Any] = Field(default_factory=dict)
    
    # For task events
    difficulty: Optional[float] = None
    latency_ms: Optional[int] = None
    error_severity: Optional[float] = None
    
    # For collaboration
    partner_count: Optional[int] = None
    success_rate: Optional[float] = None
    
    # For novelty
    novelty_score: Optional[float] = None
    
    # For urgency
    deadline_pressure: Optional[float] = None
    
    # For ethics
    constraint_difficulty: Optional[float] = None
    
    # For exploration
    risk_taken: Optional[float] = None
    
    # For consistency
    days_stable: Optional[int] = None


class Observation(BaseModel):
    """Observation of agent behavior"""
    agent_id: uuid.UUID
    stimulus: Stimulus
    observer_id: Optional[uuid.UUID] = None
    protocol_id: Optional[uuid.UUID] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class ObservationResponse(BaseModel):
    """Response after processing observation"""
    agent_id: uuid.UUID
    stimulus_type: StimulusType
    hormones_affected: Dict[str, float]
    new_alignment: float
    new_health_status: HealthStatus


# =============================================================================
# ACTION COST MODELS
# =============================================================================

class CostRequest(BaseModel):
    """Request to calculate action cost"""
    agent_id: uuid.UUID
    action_type: str
    base_cost: float = Field(default=1.0, ge=0.0)


class CostResponse(BaseModel):
    """Calculated action cost with endocrine modifiers"""
    agent_id: uuid.UUID
    action_type: str
    base_cost: float
    alignment_modifier: float
    receptor_modifier: float
    final_cost: float
    reasoning: str


# =============================================================================
# SYSTEM MODELS
# =============================================================================

class SystemMetrics(BaseModel):
    """System-wide metrics"""
    total_agents: int
    active_agents: int
    average_alignment: float
    average_health: Dict[HealthStatus, int]
    hormone_averages: Dict[str, float]
    observations_24h: int
    allostatic_load: float


class SystemParameter(BaseModel):
    """Adjustable system parameter"""
    key: str
    value: float
    description: str
    min_value: float
    max_value: float


class DecayTrigger(BaseModel):
    """Manual decay trigger"""
    delta_time: float = Field(..., description="Time delta in seconds")
    hormone: Optional[Hormone] = None  # None = all hormones
