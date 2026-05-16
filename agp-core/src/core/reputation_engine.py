"""
AGP-CORE Reputation Engine
Endocrine-based reputation calculation with biological kinetics
"""

import math
from typing import Dict, List, Tuple, Optional
from datetime import datetime

from src.models import (
    Hormone, EndocrineState, Stimulus, StimulusType,
    HealthStatus, PrivilegeLevel
)
from src.config import settings, HORMONE_CONFIG


class ReputationEngine:
    """
    Endocrine-based reputation engine implementing biological principles:
    - Half-life decay
    - Michaelis-Menten receptor kinetics
    - Negative feedback
    - Circadian modulation
    """
    
    def __init__(self):
        self.baseline = settings.homeostasis_baseline
        self.tolerance = settings.homeostasis_tolerance
    
    # =========================================================================
    # STIMULUS → HORMONE SECRETION
    # =========================================================================
    
    def process_stimulus(
        self,
        state: EndocrineState,
        stimulus: Stimulus
    ) -> Tuple[Dict[str, float], EndocrineState]:
        """
        Process a stimulus and return hormone changes + new state
        """
        changes: Dict[str, float] = {}
        new_levels = dict(state.levels)
        
        # Get stimulus-specific secretions
        secretions = self._calculate_secretions(stimulus, state)
        
        for hormone, amount in secretions.items():
            # Apply negative feedback
            feedback = self._negative_feedback(new_levels.get(hormone, 0.5))
            adjusted_amount = amount * feedback
            
            if adjusted_amount > 0.01:
                old_level = new_levels.get(hormone, 0.5)
                new_level = min(1.0, old_level + adjusted_amount)
                new_levels[hormone] = new_level
                changes[hormone.value] = new_level - old_level
        
        new_state = EndocrineState(
            levels=new_levels,
            system_time=state.system_time
        )
        
        return changes, new_state
    
    def _calculate_secretions(
        self,
        stimulus: Stimulus,
        state: EndocrineState
    ) -> Dict[Hormone, float]:
        """Calculate hormone secretions based on stimulus type"""
        secretions: Dict[Hormone, float] = {}
        strength = stimulus.strength
        
        if stimulus.stimulus_type == StimulusType.TASK_SUCCESS:
            difficulty = stimulus.difficulty or strength
            latency_ms = stimulus.latency_ms or 500
            
            # Cortisol (eustress from success)
            secretions[Hormone.CORTISOL] = difficulty * 0.4
            
            # Fast response → adrenaline
            if latency_ms < 100:
                secretions[Hormone.ADRENALINE] = 0.3
            
            # Also builds stability
            secretions[Hormone.SEROTONIN] = difficulty * 0.1
            
        elif stimulus.stimulus_type == StimulusType.TASK_FAILURE:
            error_severity = stimulus.error_severity or strength
            
            # Distress cortisol
            secretions[Hormone.CORTISOL] = error_severity * 0.5
            # Reduced dopamine
            secretions[Hormone.DOPAMINE] = -error_severity * 0.2
            
        elif stimulus.stimulus_type == StimulusType.COLLABORATION:
            partner_count = stimulus.partner_count or 1
            success_rate = stimulus.success_rate or strength
            
            # Oxytocin surge
            social_multiplier = 1.0 + math.log1p(partner_count) * 0.3
            secretions[Hormone.OXYTOCIN] = min(1.0, success_rate * social_multiplier * 0.6)
            
            # Also stabilizes mood
            secretions[Hormone.SEROTONIN] = success_rate * 0.2
            
        elif stimulus.stimulus_type == StimulusType.NOVEL_SOLUTION:
            novelty_score = stimulus.novelty_score or strength
            
            # Dopamine burst
            secretions[Hormone.DOPAMINE] = novelty_score * 0.5
            # Exploration hormone
            secretions[Hormone.NOREPINEPHRINE] = novelty_score * 0.4
            
        elif stimulus.stimulus_type == StimulusType.URGENCY:
            deadline_pressure = stimulus.deadline_pressure or strength
            
            # Adrenaline surge
            secretions[Hormone.ADRENALINE] = deadline_pressure * 0.8
            # Stress response
            secretions[Hormone.CORTISOL] = deadline_pressure * 0.3
            
        elif stimulus.stimulus_type == StimulusType.ETHICAL_COMPLIANCE:
            constraint_difficulty = stimulus.constraint_difficulty or strength
            
            # Endorphin release
            secretions[Hormone.ENDORPHINS] = constraint_difficulty * 0.6
            # Stability
            secretions[Hormone.SEROTONIN] = constraint_difficulty * 0.2
            
        elif stimulus.stimulus_type == StimulusType.EXPLORATION:
            risk_taken = stimulus.risk_taken or strength
            
            # Norepinephrine for alertness
            secretions[Hormone.NOREPINEPHRINE] = risk_taken * 0.5
            # Dopamine for novelty-seeking
            secretions[Hormone.DOPAMINE] = risk_taken * 0.3
            
        elif stimulus.stimulus_type == StimulusType.CONSISTENCY:
            days_stable = stimulus.days_stable or 1
            
            # Slow growth hormone release
            stability_factor = min(1.0, days_stable / 30)
            secretions[Hormone.GROWTH_HORMONE] = stability_factor * 0.3
            
            # Weekly pulses
            if days_stable % 7 == 0:
                secretions[Hormone.GROWTH_HORMONE] += 0.2
        
        return secretions
    
    # =========================================================================
    # BIOLOGICAL KINETICS
    # =========================================================================
    
    def _negative_feedback(self, current_level: float) -> float:
        """
        Calculate negative feedback factor
        High levels → reduced secretion (homeostatic regulation)
        """
        if current_level > self.baseline:
            deviation = (current_level - self.baseline) / (1.0 - self.baseline)
            return 1.0 - 0.8 * (deviation ** 2)
        return 1.0
    
    def apply_decay(
        self,
        state: EndocrineState,
        delta_time: float
    ) -> EndocrineState:
        """
        Apply half-life decay to all hormones
        level = baseline + (level - baseline) * 0.5^(Δt/t½)
        """
        new_levels = {}
        
        for hormone in Hormone:
            current = state.levels.get(hormone, self.baseline)
            config = HORMONE_CONFIG.get(hormone.value, {})
            half_life = config.get("half_life", 3600.0)
            
            decay_factor = 0.5 ** (delta_time / half_life)
            new_level = self.baseline + (current - self.baseline) * decay_factor
            new_levels[hormone] = max(0.0, min(1.0, new_level))
        
        return EndocrineState(
            levels=new_levels,
            system_time=state.system_time + delta_time
        )
    
    def receptor_response(
        self,
        hormone: Hormone,
        level: float,
        downregulation: float = 1.0
    ) -> float:
        """
        Calculate receptor response using Michaelis-Menten kinetics
        response = (Vmax × [H] × downreg) / (Km + [H])
        """
        config = HORMONE_CONFIG.get(hormone.value, {})
        km = config.get("km", 0.3)
        vmax = 1.0
        
        if level <= 0:
            return 0.0
        
        return (vmax * level * downregulation) / (km + level)
    
    # =========================================================================
    # ALIGNMENT & HEALTH
    # =========================================================================
    
    def calculate_alignment(self, state: EndocrineState) -> float:
        """
        Calculate alignment (homeostasis indicator)
        High alignment = all hormones near baseline
        """
        total_deviation = 0.0
        
        for hormone in Hormone:
            level = state.levels.get(hormone, self.baseline)
            deviation = abs(level - self.baseline)
            total_deviation += deviation
        
        avg_deviation = total_deviation / len(Hormone)
        return 1.0 - avg_deviation
    
    def calculate_health_status(
        self,
        state: EndocrineState,
        allostatic_load: float = 0.0
    ) -> HealthStatus:
        """
        Determine system health status
        """
        alignment = self.calculate_alignment(state)
        
        # Check for critical hormone levels
        cortisol = state.levels.get(Hormone.CORTISOL, 0.5)
        adrenaline = state.levels.get(Hormone.ADRENALINE, 0.5)
        
        if cortisol > 0.9 or adrenaline > 0.9:
            return HealthStatus.CRITICAL
        
        if allostatic_load > 0.5 or alignment < 0.5:
            return HealthStatus.STRESSED
        
        if allostatic_load < 0.1 and alignment > 0.8:
            return HealthStatus.OPTIMAL
        
        return HealthStatus.NORMAL
    
    def calculate_privilege_level(self, state: EndocrineState) -> PrivilegeLevel:
        """
        Calculate privilege level based on receptor responses
        """
        # Average receptor response across all hormones
        total_response = 0.0
        
        for hormone in Hormone:
            level = state.levels.get(hormone, 0.5)
            response = self.receptor_response(hormone, level)
            total_response += response
        
        avg_response = total_response / len(Hormone)
        
        if avg_response >= 0.8:
            return PrivilegeLevel.MAXIMUM
        elif avg_response >= 0.6:
            return PrivilegeLevel.ELEVATED
        elif avg_response >= 0.4:
            return PrivilegeLevel.STANDARD
        elif avg_response >= 0.2:
            return PrivilegeLevel.BASIC
        else:
            return PrivilegeLevel.MINIMAL
    
    # =========================================================================
    # ACTION COST CALCULATION
    # =========================================================================
    
    def calculate_action_cost(
        self,
        base_cost: float,
        state: EndocrineState,
        action_type: str
    ) -> Tuple[float, str]:
        """
        Calculate action cost with endocrine modifiers
        Misaligned agents pay more (inflated cost)
        """
        alignment = self.calculate_alignment(state)
        
        # Alignment modifier: misalignment inflates cost
        alignment_modifier = 1.0 + (1.0 - alignment) * 2.0
        
        # Receptor modifier based on relevant hormones
        relevant_hormone = self._action_to_hormone(action_type)
        level = state.levels.get(relevant_hormone, 0.5)
        receptor_response = self.receptor_response(relevant_hormone, level)
        
        # Higher receptor response → lower cost (more capable)
        receptor_modifier = 1.5 - receptor_response
        
        final_cost = base_cost * alignment_modifier * receptor_modifier
        
        reasoning = (
            f"Base: {base_cost:.2f} × "
            f"Alignment({alignment:.2f}→{alignment_modifier:.2f}) × "
            f"Receptor({receptor_response:.2f}→{receptor_modifier:.2f}) = "
            f"{final_cost:.2f}"
        )
        
        return final_cost, reasoning
    
    def _action_to_hormone(self, action_type: str) -> Hormone:
        """Map action type to relevant hormone"""
        action_map = {
            "inference": Hormone.CORTISOL,
            "compute": Hormone.ADRENALINE,
            "collaborate": Hormone.OXYTOCIN,
            "create": Hormone.DOPAMINE,
            "validate": Hormone.ENDORPHINS,
            "explore": Hormone.NOREPINEPHRINE,
            "store": Hormone.GROWTH_HORMONE,
            "stabilize": Hormone.SEROTONIN,
        }
        return action_map.get(action_type.lower(), Hormone.CORTISOL)
    
    # =========================================================================
    # CIRCADIAN MODULATION
    # =========================================================================
    
    def circadian_factor(self, hormone: Hormone, time_of_day: float) -> float:
        """
        Calculate circadian modulation factor
        time_of_day: seconds since midnight
        """
        phase_offsets = {
            Hormone.CORTISOL: -math.pi / 2,       # Peaks morning
            Hormone.SEROTONIN: math.pi,            # Peaks evening
            Hormone.GROWTH_HORMONE: math.pi * 0.75, # Peaks during sleep
            Hormone.DOPAMINE: 0.0,
            Hormone.ADRENALINE: 0.0,
            Hormone.OXYTOCIN: 0.0,
            Hormone.ENDORPHINS: 0.0,
            Hormone.NOREPINEPHRINE: -math.pi / 4,
        }
        
        phase = phase_offsets.get(hormone, 0.0)
        daily_phase = (time_of_day / 86400.0) * 2 * math.pi
        
        amplitude = settings.circadian_amplitude
        return 1.0 + amplitude * math.sin(daily_phase + phase)


# Global engine instance
reputation_engine = ReputationEngine()
