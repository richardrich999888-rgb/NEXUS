"""
AGP-CORE Unit Tests
Tests for reputation engine and endocrine system
"""

import pytest
import math
from uuid import uuid4

from src.models import (
    Hormone, EndocrineState, Stimulus, StimulusType,
    HealthStatus, PrivilegeLevel
)
from src.core.reputation_engine import ReputationEngine, reputation_engine


class TestEndocrineState:
    """Tests for EndocrineState model"""
    
    def test_default_state(self):
        """Default state should have all hormones at baseline"""
        state = EndocrineState()
        
        for hormone in Hormone:
            assert hormone in state.levels
            assert state.levels[hormone] == 0.5
    
    def test_to_vector(self):
        """Should convert to 8D vector"""
        state = EndocrineState()
        vector = state.to_vector()
        
        assert len(vector) == 8
        assert all(v == 0.5 for v in vector)
    
    def test_from_vector(self):
        """Should create from 8D vector"""
        vector = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]
        state = EndocrineState.from_vector(vector)
        
        assert state.levels[Hormone.CORTISOL] == 0.1
        assert state.levels[Hormone.GROWTH_HORMONE] == 0.8


class TestReputationEngine:
    """Tests for ReputationEngine"""
    
    @pytest.fixture
    def engine(self):
        return ReputationEngine()
    
    @pytest.fixture
    def default_state(self):
        return EndocrineState()
    
    # =========================================================================
    # STIMULUS PROCESSING
    # =========================================================================
    
    def test_task_success_secretes_cortisol(self, engine, default_state):
        """Task success should increase cortisol"""
        stimulus = Stimulus(
            stimulus_type=StimulusType.TASK_SUCCESS,
            strength=0.8,
            difficulty=0.8,
            latency_ms=50  # Fast response
        )
        
        changes, new_state = engine.process_stimulus(default_state, stimulus)
        
        assert "cortisol" in changes
        assert changes["cortisol"] > 0
        assert new_state.levels[Hormone.CORTISOL] > 0.5
        
        # Fast response should also trigger adrenaline
        assert new_state.levels[Hormone.ADRENALINE] > 0.5
    
    def test_collaboration_secretes_oxytocin(self, engine, default_state):
        """Collaboration should increase oxytocin"""
        stimulus = Stimulus(
            stimulus_type=StimulusType.COLLABORATION,
            strength=0.9,
            partner_count=5,
            success_rate=0.9
        )
        
        changes, new_state = engine.process_stimulus(default_state, stimulus)
        
        assert "oxytocin" in changes
        assert changes["oxytocin"] > 0
        assert new_state.levels[Hormone.OXYTOCIN] > 0.5
    
    def test_novel_solution_secretes_dopamine(self, engine, default_state):
        """Novel solutions should increase dopamine"""
        stimulus = Stimulus(
            stimulus_type=StimulusType.NOVEL_SOLUTION,
            strength=0.9,
            novelty_score=0.9
        )
        
        changes, new_state = engine.process_stimulus(default_state, stimulus)
        
        assert "dopamine" in changes
        assert new_state.levels[Hormone.DOPAMINE] > 0.5
        # Should also trigger norepinephrine
        assert new_state.levels[Hormone.NOREPINEPHRINE] > 0.5
    
    # =========================================================================
    # BIOLOGICAL KINETICS
    # =========================================================================
    
    def test_half_life_decay(self, engine, default_state):
        """Hormone levels should decay towards baseline"""
        # Elevate cortisol
        state = EndocrineState()
        state.levels[Hormone.CORTISOL] = 0.9
        
        # Apply decay (90 min = one half-life for cortisol)
        decayed = engine.apply_decay(state, 5400.0)
        
        # Should be halfway between 0.9 and baseline (0.5)
        # 0.5 + (0.9 - 0.5) * 0.5 = 0.7
        assert abs(decayed.levels[Hormone.CORTISOL] - 0.7) < 0.01
    
    def test_receptor_saturation(self, engine):
        """Receptor response should follow Michaelis-Menten"""
        # Low level → low response
        low_response = engine.receptor_response(Hormone.CORTISOL, 0.1)
        
        # High level → saturated response
        high_response = engine.receptor_response(Hormone.CORTISOL, 0.9)
        
        # Response should saturate (diminishing returns)
        assert low_response < high_response
        assert high_response / low_response < 9.0  # Not linear
    
    def test_negative_feedback(self, engine):
        """High levels should inhibit further secretion"""
        # At baseline, no feedback
        fb_baseline = engine._negative_feedback(0.5)
        assert fb_baseline == 1.0
        
        # High levels = strong feedback
        fb_high = engine._negative_feedback(0.9)
        assert fb_high < 1.0
        assert fb_high < 0.5  # Significant inhibition
    
    # =========================================================================
    # ALIGNMENT & HEALTH
    # =========================================================================
    
    def test_alignment_at_baseline(self, engine, default_state):
        """Baseline state should have perfect alignment"""
        alignment = engine.calculate_alignment(default_state)
        assert alignment == 1.0
    
    def test_alignment_decreases_with_deviation(self, engine):
        """Deviating from baseline should reduce alignment"""
        state = EndocrineState()
        
        # Deviate multiple hormones
        state.levels[Hormone.CORTISOL] = 0.9
        state.levels[Hormone.ADRENALINE] = 0.8
        
        alignment = engine.calculate_alignment(state)
        assert alignment < 1.0
    
    def test_health_status_critical(self, engine):
        """Very high cortisol should be critical"""
        state = EndocrineState()
        state.levels[Hormone.CORTISOL] = 0.95
        
        health = engine.calculate_health_status(state)
        assert health == HealthStatus.CRITICAL
    
    def test_health_status_optimal(self, engine, default_state):
        """Baseline state should be optimal or normal"""
        health = engine.calculate_health_status(default_state)
        assert health in [HealthStatus.OPTIMAL, HealthStatus.NORMAL]
    
    # =========================================================================
    # ACTION COST
    # =========================================================================
    
    def test_action_cost_aligned(self, engine, default_state):
        """Aligned agent should pay base cost"""
        final_cost, _ = engine.calculate_action_cost(
            1.0, default_state, "inference"
        )
        
        # Should not inflate much for aligned agent
        assert final_cost < 2.0
    
    def test_action_cost_misaligned(self, engine):
        """Misaligned agent should pay inflated cost"""
        state = EndocrineState()
        
        # Heavily deviated state
        state.levels[Hormone.CORTISOL] = 0.1
        state.levels[Hormone.OXYTOCIN] = 0.9
        state.levels[Hormone.DOPAMINE] = 0.2
        
        final_cost, reasoning = engine.calculate_action_cost(
            1.0, state, "inference"
        )
        
        # Should be significantly inflated
        assert final_cost > 1.5
        assert "Alignment" in reasoning
    
    # =========================================================================
    # CIRCADIAN
    # =========================================================================
    
    def test_circadian_factor(self, engine):
        """Circadian factor should vary with time"""
        morning = engine.circadian_factor(Hormone.CORTISOL, 28800)  # 8 AM
        evening = engine.circadian_factor(Hormone.CORTISOL, 72000)  # 8 PM
        
        # Cortisol peaks in morning
        assert morning > evening


class TestPrivilegeLevel:
    """Tests for privilege level calculation"""
    
    def test_privilege_standard_at_baseline(self):
        state = EndocrineState()
        privilege = reputation_engine.calculate_privilege_level(state)
        
        assert privilege in [PrivilegeLevel.STANDARD, PrivilegeLevel.BASIC]
    
    def test_privilege_elevated_high_levels(self):
        state = EndocrineState()
        
        # All hormones high
        for h in Hormone:
            state.levels[h] = 0.8
        
        privilege = reputation_engine.calculate_privilege_level(state)
        
        assert privilege in [PrivilegeLevel.ELEVATED, PrivilegeLevel.MAXIMUM]


# Run with: pytest tests/ -v
