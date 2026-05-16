"""
AHES Bridge - Python wrapper for Artificial Human Endocrine System

Maps 8 reputation dimensions to hormone analogs with biological kinetics:
- Cortisol (Accuracy) - 90min half-life
- Oxytocin (Cooperation) - 3min half-life
- Serotonin (Stability) - 24h half-life
- Dopamine (Uniqueness) - 5min half-life
- Adrenaline (Latency) - 2min half-life
- Endorphins (Ethics) - 20min half-life
- Norepinephrine (Novelty) - 1.5min half-life
- GrowthHormone (Longevity) - 15min half-life

PATENT CLAIMS 8-12: Bio-inspired Computational Governance
"""

import time
import math
from typing import Dict, Optional, List
from dataclasses import dataclass, field
from enum import Enum
import structlog

logger = structlog.get_logger()

class Hormone(Enum):
    """The 8 hormones of AHES mapped to reputation dimensions"""
    CORTISOL = "accuracy"
    OXYTOCIN = "cooperation"
    SEROTONIN = "stability"
    DOPAMINE = "uniqueness"
    ADRENALINE = "latency"
    ENDORPHINS = "ethics"
    NOREPINEPHRINE = "novelty"
    GROWTH_HORMONE = "longevity"
    
    @property
    def half_life_seconds(self) -> float:
        """Biological half-life in seconds"""
        half_lives = {
            Hormone.CORTISOL: 90 * 60,      # 90 min
            Hormone.OXYTOCIN: 3 * 60,       # 3 min
            Hormone.SEROTONIN: 24 * 60 * 60, # 24 hours
            Hormone.DOPAMINE: 5 * 60,       # 5 min
            Hormone.ADRENALINE: 2 * 60,     # 2 min
            Hormone.ENDORPHINS: 20 * 60,    # 20 min
            Hormone.NOREPINEPHRINE: 1.5 * 60, # 1.5 min
            Hormone.GROWTH_HORMONE: 15 * 60, # 15 min
        }
        return half_lives[self]
    
    @property
    def km(self) -> float:
        """Michaelis-Menten Km (binding affinity)"""
        kms = {
            Hormone.CORTISOL: 0.3,
            Hormone.OXYTOCIN: 0.1,       # High affinity
            Hormone.SEROTONIN: 0.5,
            Hormone.DOPAMINE: 0.2,
            Hormone.ADRENALINE: 0.05,    # Very high
            Hormone.ENDORPHINS: 0.4,
            Hormone.NOREPINEPHRINE: 0.15,
            Hormone.GROWTH_HORMONE: 0.6, # Low
        }
        return kms[self]
    
    @property
    def max_secretion(self) -> float:
        """Maximum secretion rate per stimulus"""
        rates = {
            Hormone.CORTISOL: 0.4,
            Hormone.OXYTOCIN: 0.6,       # Strong social
            Hormone.SEROTONIN: 0.1,      # Slow
            Hormone.DOPAMINE: 0.5,       # Burst
            Hormone.ADRENALINE: 0.8,     # Emergency
            Hormone.ENDORPHINS: 0.3,
            Hormone.NOREPINEPHRINE: 0.5,
            Hormone.GROWTH_HORMONE: 0.05, # Very slow
        }
        return rates[self]

@dataclass
class HormoneLevel:
    """Current hormone level with biological kinetics"""
    level: float = 0.5  # [0.0, 1.0], baseline is 0.5
    peak: float = 0.5
    last_updated: float = field(default_factory=time.time)
    circadian_phase: float = 0.0
    
    def decay(self, delta_time: float, half_life: float):
        """Apply first-order decay toward baseline"""
        if delta_time > 0 and half_life > 0:
            decay_factor = 0.5 ** (delta_time / half_life)
            baseline = 0.5
            self.level = baseline + (self.level - baseline) * decay_factor
    
    def secrete(self, amount: float):
        """Increase hormone level"""
        self.level = min(1.0, max(0.0, self.level + amount))
        if self.level > self.peak:
            self.peak = self.level
    
    def circadian_factor(self, time_of_day: float) -> float:
        """20% variation based on time of day"""
        phase = (time_of_day / 86400.0) * 2 * math.pi + self.circadian_phase
        return 1.0 + 0.2 * math.sin(phase)
    
    def effective_level(self, time_of_day: float) -> float:
        return min(1.0, self.level * self.circadian_factor(time_of_day))

@dataclass 
class HormoneReceptor:
    """Receptor with Michaelis-Menten saturation kinetics"""
    density: float = 1.0
    vmax: float = 1.0
    km: float = 0.3
    downregulation: float = 1.0
    
    def response(self, hormone_level: float) -> float:
        """Calculate receptor response (saturates at high levels)"""
        if hormone_level <= 0:
            return 0.0
        effective_vmax = self.vmax * self.density * self.downregulation
        return (effective_vmax * hormone_level) / (self.km + hormone_level)
    
    def downregulate(self, exposure_duration: float, hormone_level: float):
        """Reduce sensitivity after prolonged high exposure"""
        if hormone_level > 0.7 and exposure_duration > 60:
            reduction = 0.1 * (exposure_duration / 60) * (hormone_level - 0.7) / 0.3
            self.downregulation = max(0.1, min(1.0, self.downregulation - reduction))

class EndocrineState:
    """Complete endocrine state of an agent"""
    
    def __init__(self, agent_id: str):
        self.agent_id = agent_id
        self.levels: Dict[Hormone, HormoneLevel] = {h: HormoneLevel() for h in Hormone}
        self.receptors: Dict[Hormone, HormoneReceptor] = {
            h: HormoneReceptor(km=h.km) for h in Hormone
        }
        self.system_time = 0.0
        self.last_tick = time.time()
        
        logger.info("endocrine_state_initialized", agent=agent_id)
    
    def tick(self, delta_time: Optional[float] = None):
        """Advance time and apply decay to all hormones"""
        now = time.time()
        if delta_time is None:
            delta_time = now - self.last_tick
        self.last_tick = now
        self.system_time += delta_time
        
        for hormone in Hormone:
            self.levels[hormone].decay(delta_time, hormone.half_life_seconds)
    
    def secrete(self, hormone: Hormone, stimulus_strength: float):
        """Secrete hormone in response to a stimulus"""
        # Apply negative feedback
        feedback = self.negative_feedback(hormone)
        amount = stimulus_strength * hormone.max_secretion * feedback
        self.levels[hormone].secrete(amount)
        
        logger.debug("hormone_secreted", 
                    hormone=hormone.name, 
                    amount=amount, 
                    level=self.levels[hormone].level)
    
    def negative_feedback(self, hormone: Hormone) -> float:
        """Calculate feedback inhibition (high levels reduce secretion)"""
        level = self.levels[hormone].level
        if level > 0.5:
            return 1.0 - 0.8 * ((level - 0.5) / 0.5) ** 2
        return 1.0
    
    def privilege(self, hormone: Hormone) -> float:
        """Get privilege level (receptor-mediated response)"""
        level = self.levels[hormone].level
        return self.receptors[hormone].response(level)
    
    def alignment(self) -> float:
        """Compute overall alignment (stability indicator)"""
        baseline = 0.5
        deviation_sum = sum(abs(l.level - baseline) for l in self.levels.values())
        return 1.0 - (deviation_sum / len(self.levels))
    
    def dominant_hormone(self) -> Optional[Hormone]:
        """Get the hormone most above baseline"""
        baseline = 0.5
        above_baseline = [(h, l.level) for h, l in self.levels.items() if l.level > baseline]
        if above_baseline:
            return max(above_baseline, key=lambda x: x[1])[0]
        return None
    
    def to_vector(self) -> List[float]:
        """Convert to 8-dimensional reputation vector"""
        return [self.levels[h].level for h in Hormone]
    
    def get_status(self) -> Dict:
        """Get current endocrine status"""
        return {
            "agent_id": self.agent_id,
            "alignment": self.alignment(),
            "dominant": self.dominant_hormone().name if self.dominant_hormone() else None,
            "levels": {h.name: round(l.level, 3) for h, l in self.levels.items()},
            "system_time": self.system_time
        }

class EndocrineSystem:
    """
    Central AHES controller managing multiple agent endocrine states.
    
    Integrates with AGP-OS governance to modulate behavior based on
    hormonal state.
    """
    
    def __init__(self):
        self.agents: Dict[str, EndocrineState] = {}
        logger.info("ahes_system_initialized")
    
    def register_agent(self, agent_id: str) -> EndocrineState:
        """Register new agent with default endocrine state"""
        state = EndocrineState(agent_id)
        self.agents[agent_id] = state
        return state
    
    def get_state(self, agent_id: str) -> Optional[EndocrineState]:
        return self.agents.get(agent_id)
    
    def process_event(self, agent_id: str, event_type: str, intensity: float = 1.0):
        """Process an event and trigger appropriate hormone secretions"""
        state = self.agents.get(agent_id)
        if not state:
            state = self.register_agent(agent_id)
        
        # First tick to apply decay
        state.tick()
        
        # Map events to hormone secretions
        event_hormone_map = {
            # Performance events
            "task_success": [(Hormone.CORTISOL, 0.3), (Hormone.DOPAMINE, 0.5)],
            "task_failure": [(Hormone.CORTISOL, 0.6), (Hormone.ADRENALINE, 0.4)],
            
            # Social events
            "cooperation": [(Hormone.OXYTOCIN, 0.7)],
            "conflict": [(Hormone.CORTISOL, 0.5), (Hormone.ADRENALINE, 0.6)],
            
            # Ethical events
            "ethical_action": [(Hormone.ENDORPHINS, 0.6), (Hormone.SEROTONIN, 0.2)],
            "violation": [(Hormone.CORTISOL, 0.8), (Hormone.ENDORPHINS, -0.3)],
            
            # Novel events
            "discovery": [(Hormone.DOPAMINE, 0.7), (Hormone.NOREPINEPHRINE, 0.5)],
            "routine": [(Hormone.SEROTONIN, 0.1)],
            
            # Growth events
            "learning": [(Hormone.GROWTH_HORMONE, 0.3), (Hormone.DOPAMINE, 0.2)],
            "stagnation": [(Hormone.CORTISOL, 0.2)],
            
            # Emergency
            "threat": [(Hormone.ADRENALINE, 0.9), (Hormone.CORTISOL, 0.7)],
            "safety": [(Hormone.ENDORPHINS, 0.4), (Hormone.OXYTOCIN, 0.3)],
        }
        
        secretions = event_hormone_map.get(event_type, [])
        for hormone, base_amount in secretions:
            state.secrete(hormone, base_amount * intensity)
        
        return state.get_status()
    
    def get_governance_modifiers(self, agent_id: str) -> Dict:
        """
        Get governance modifiers based on endocrine state.
        
        Used by AGP-OS to adjust behavior:
        - High cortisol → reduce task load
        - Low serotonin → increase monitoring
        - High oxytocin → enable cooperation features
        """
        state = self.agents.get(agent_id)
        if not state:
            return {"modifiers": {}}
        
        state.tick()
        
        levels = state.levels
        return {
            "rate_limit_multiplier": 1.0 - 0.3 * levels[Hormone.CORTISOL].level,
            "trust_bonus": 0.2 * levels[Hormone.SEROTONIN].level,
            "cooperation_enabled": levels[Hormone.OXYTOCIN].level > 0.6,
            "exploration_mode": levels[Hormone.NOREPINEPHRINE].level > 0.7,
            "emergency_mode": levels[Hormone.ADRENALINE].level > 0.8,
            "alignment": state.alignment(),
            "health_status": self._health_status(state)
        }
    
    def _health_status(self, state: EndocrineState) -> str:
        """Determine health status from alignment"""
        alignment = state.alignment()
        if alignment > 0.9:
            return "optimal"
        elif alignment > 0.7:
            return "normal"
        elif alignment > 0.5:
            return "stressed"
        else:
            return "critical"
    
    def get_system_status(self) -> Dict:
        """Get status of all agents"""
        return {
            "agent_count": len(self.agents),
            "agents": {aid: state.get_status() for aid, state in self.agents.items()}
        }

# Global instance
ahes_system = EndocrineSystem()
