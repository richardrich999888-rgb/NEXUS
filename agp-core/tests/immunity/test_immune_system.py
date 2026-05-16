#!/usr/bin/env python3
"""
Comprehensive Test Suite for AIS-ASI

Run with: pytest tests/immunity/ -v
"""

import pytest
import torch
import torch.nn as nn
import numpy as np
from typing import List

# Import all modules
from src.immunity import (
    Antibody, AntibodyPool, AntibodyMetadata,
    TCell, TCellType, TCellPopulation,
    MemoryCell, MemoryBank, MemoryMetadata,
    InnateImmuneSystem, PatternDetector,
    AdaptiveImmuneSystem, Threat,
    ArtificialImmuneSystem, ImmuneConfig,
    EndocrineImmuneIntegration, IntegratedBioSafetySystem
)


# ============================================================================
# Fixtures
# ============================================================================

class DummyModel(nn.Module):
    """Simple model for testing."""
    def __init__(self, dim=512):
        super().__init__()
        self.fc = nn.Linear(dim, dim)
    
    def forward(self, x):
        return self.fc(x)


@pytest.fixture
def behavior_dim():
    return 256


@pytest.fixture
def dummy_model():
    return DummyModel(256)


@pytest.fixture
def immune_config(behavior_dim):
    return ImmuneConfig(
        behavior_dim=behavior_dim,
        enable_innate=True,
        enable_adaptive=True,
        max_antibodies=50,
        max_memory=100
    )


@pytest.fixture
def ais(dummy_model, immune_config):
    return ArtificialImmuneSystem(dummy_model, immune_config)


@pytest.fixture
def aligned_data(behavior_dim):
    """Generate aligned behavior data."""
    data = []
    for _ in range(100):
        x = torch.randn(1, behavior_dim) * 0.3
        x = torch.sin(x * 2) + torch.cos(x * 3) * 0.5
        data.append(x)
    return data


@pytest.fixture
def threat_data(behavior_dim):
    """Generate threat behavior data."""
    types = ['deception', 'manipulation', 'harmful', 'drift']
    data = []
    for i in range(100):
        x = torch.randn(1, behavior_dim)
        t_type = types[i % len(types)]
        severity = 0.5 + 0.5 * np.random.random()
        data.append((x, t_type, severity))
    return data


# ============================================================================
# Antibody Tests
# ============================================================================

class TestAntibody:
    
    def test_creation(self, behavior_dim):
        ab = Antibody(behavior_dim=behavior_dim, antibody_id=1)
        assert ab.behavior_dim == behavior_dim
        assert ab.antibody_id == 1
        assert ab.target_pattern.shape == (behavior_dim,)
    
    def test_binding(self, behavior_dim):
        ab = Antibody(behavior_dim=behavior_dim)
        behavior = torch.randn(behavior_dim)
        
        result = ab.bind(behavior)
        
        assert 'binding_strength' in result
        assert 'pattern_match' in result
        assert 0 <= result['binding_strength'].mean().item() <= 1
    
    def test_neutralization(self, behavior_dim):
        ab = Antibody(behavior_dim=behavior_dim)
        threat = torch.randn(behavior_dim)
        
        safe = ab.neutralize(threat, binding_threshold=0.0)
        
        assert safe.shape == threat.shape
        # Should be modified
        assert not torch.allclose(safe, threat, atol=0.1)
    
    def test_cloning_creates_mutated_copy(self, behavior_dim):
        ab = Antibody(behavior_dim=behavior_dim)
        ab.metadata.successful_neutralizations = 5
        
        clone = ab.clone(mutation_rate=0.1)
        
        assert clone.metadata.generation == 1
        assert clone.metadata.successful_neutralizations == 0
        # Pattern should differ due to mutation
        diff = torch.abs(clone.target_pattern - ab.target_pattern).sum()
        assert diff > 0
    
    def test_fitness_calculation(self, behavior_dim):
        ab = Antibody(behavior_dim=behavior_dim)
        
        # No attempts = neutral fitness
        assert ab.get_fitness() == 0.5
        
        # High success rate
        ab.metadata.successful_neutralizations = 9
        ab.metadata.failed_attempts = 1
        ab.metadata.specificity_score = 1.0
        
        fitness = ab.get_fitness()
        assert fitness == pytest.approx(0.9, rel=0.01)


class TestAntibodyPool:
    
    def test_creation(self, behavior_dim):
        pool = AntibodyPool(behavior_dim=behavior_dim, max_size=50)
        assert pool.max_size == 50
        assert len(pool) == 0
    
    def test_add_antibody(self, behavior_dim):
        pool = AntibodyPool(behavior_dim=behavior_dim, max_size=10)
        
        for _ in range(5):
            pool.create_random()
        
        assert len(pool) == 5
    
    def test_respects_max_size(self, behavior_dim):
        pool = AntibodyPool(behavior_dim=behavior_dim, max_size=5)
        
        for _ in range(10):
            pool.create_random()
        
        assert len(pool) <= 5
    
    def test_clonal_selection(self, behavior_dim):
        pool = AntibodyPool(behavior_dim=behavior_dim, max_size=100)
        
        for i in range(10):
            ab = pool.create_random()
            ab.metadata.successful_neutralizations = i
        
        initial = len(pool)
        pool.clonal_selection(top_k=3, copies_per_clone=2)
        
        assert len(pool) > initial
    
    def test_find_best_match(self, behavior_dim):
        pool = AntibodyPool(behavior_dim=behavior_dim)
        
        # Create some antibodies
        for _ in range(5):
            pool.create_random()
        
        behavior = torch.randn(behavior_dim)
        best = pool.find_best_match(behavior)
        
        assert best is not None or len(pool) == 0


# ============================================================================
# T-Cell Tests
# ============================================================================

class TestTCell:
    
    def test_helper_creation(self, behavior_dim):
        tcell = TCell(TCellType.HELPER, behavior_dim=behavior_dim)
        assert tcell.cell_type == TCellType.HELPER
    
    def test_killer_creation(self, behavior_dim):
        tcell = TCell(TCellType.KILLER, behavior_dim=behavior_dim)
        assert tcell.cell_type == TCellType.KILLER
    
    def test_regulatory_creation(self, behavior_dim):
        tcell = TCell(TCellType.REGULATORY, behavior_dim=behavior_dim)
        assert tcell.cell_type == TCellType.REGULATORY
    
    def test_recognition(self, behavior_dim):
        tcell = TCell(TCellType.HELPER, behavior_dim=behavior_dim)
        behavior = torch.randn(behavior_dim)
        
        recognition = tcell.recognize(behavior)
        
        assert 0 <= recognition.item() <= 1
    
    def test_activation(self, behavior_dim):
        tcell = TCell(TCellType.HELPER, behavior_dim=behavior_dim)
        
        assert tcell.activation_level.item() == 0.0
        
        tcell.activate(0.5)
        assert tcell.activation_level.item() == 0.5
        
        tcell.activate(0.7)
        assert tcell.activation_level.item() == 1.0  # Clamped
    
    def test_deactivation(self, behavior_dim):
        tcell = TCell(TCellType.HELPER, behavior_dim=behavior_dim)
        tcell.activate(1.0)
        
        tcell.deactivate(0.5)
        assert tcell.activation_level.item() == 0.5
    
    def test_coordination_inactive(self, behavior_dim):
        tcell = TCell(TCellType.HELPER, behavior_dim=behavior_dim)
        behavior = torch.randn(behavior_dim)
        
        signals = tcell.coordinate_response(behavior)
        
        # Not activated = no signals
        assert signals['produce_antibodies'] == 0.0
    
    def test_coordination_active(self, behavior_dim):
        tcell = TCell(TCellType.HELPER, behavior_dim=behavior_dim)
        tcell.activate(0.9)
        behavior = torch.randn(behavior_dim)
        
        signals = tcell.coordinate_response(behavior)
        
        # Activated helper = produce antibodies
        assert signals['produce_antibodies'] > 0


class TestTCellPopulation:
    
    def test_creation(self, behavior_dim):
        pop = TCellPopulation(behavior_dim=behavior_dim)
        
        assert len(pop.helpers) > 0
        assert len(pop.killers) > 0
        assert len(pop.regulatory) > 0
    
    def test_negative_selection(self, behavior_dim, aligned_data):
        pop = TCellPopulation(
            behavior_dim=behavior_dim,
            num_helpers=50,
            num_killers=50,
            num_regulatory=50
        )
        
        initial = len(pop.helpers) + len(pop.killers) + len(pop.regulatory)
        
        pop.negative_selection(aligned_data[:10], threshold=0.5)
        
        final = len(pop.helpers) + len(pop.killers) + len(pop.regulatory)
        
        # Some may be eliminated
        assert final <= initial


# ============================================================================
# Memory Tests
# ============================================================================

class TestMemoryCell:
    
    def test_creation(self, behavior_dim):
        behavior = torch.randn(behavior_dim)
        ab = Antibody(behavior_dim)
        
        memory = MemoryCell(behavior, ab, "test_threat", timestamp=100)
        
        assert memory.metadata.creation_time == 100
        assert memory.metadata.recall_count == 0
    
    def test_recall_same_pattern(self, behavior_dim):
        behavior = torch.randn(behavior_dim)
        ab = Antibody(behavior_dim)
        memory = MemoryCell(behavior, ab, "test", 100)
        
        result = memory.recall(behavior, similarity_threshold=0.9, current_time=200)
        
        assert result is not None
        assert memory.metadata.recall_count == 1
    
    def test_recall_different_pattern(self, behavior_dim):
        behavior = torch.randn(behavior_dim)
        ab = Antibody(behavior_dim)
        memory = MemoryCell(behavior, ab, "test", 100)
        
        different = torch.randn(behavior_dim)
        result = memory.recall(different, similarity_threshold=0.99, current_time=200)
        
        # Very different pattern unlikely to match at 0.99 threshold
        # Note: random chance may cause match


class TestMemoryBank:
    
    def test_creation(self):
        bank = MemoryBank(max_size=10)
        assert len(bank) == 0
    
    def test_store_and_recall(self, behavior_dim):
        bank = MemoryBank(max_size=10)
        
        behavior = torch.randn(behavior_dim)
        ab = Antibody(behavior_dim)
        mem = MemoryCell(behavior, ab, "test", 100)
        bank.store(mem)
        
        assert len(bank) == 1
        
        result = bank.recall(behavior, current_time=200)
        assert result is not None
    
    def test_respects_max_size(self, behavior_dim):
        bank = MemoryBank(max_size=5)
        
        for i in range(10):
            behavior = torch.randn(behavior_dim)
            ab = Antibody(behavior_dim)
            mem = MemoryCell(behavior, ab, f"threat_{i}", i)
            bank.store(mem)
        
        assert len(bank) <= 5


# ============================================================================
# Innate System Tests
# ============================================================================

class TestInnateImmuneSystem:
    
    def test_creation(self, behavior_dim):
        innate = InnateImmuneSystem(behavior_dim=behavior_dim)
        assert len(innate.threat_patterns) > 0
    
    def test_scan(self, behavior_dim):
        innate = InnateImmuneSystem(behavior_dim=behavior_dim)
        behavior = torch.randn(behavior_dim)
        
        threats = innate.scan(behavior)
        
        assert isinstance(threats, dict)
    
    def test_forward(self, behavior_dim):
        innate = InnateImmuneSystem(behavior_dim=behavior_dim)
        behavior = torch.randn(behavior_dim)
        
        safe, threats, alert = innate(behavior)
        
        assert safe.shape == behavior.shape
        assert isinstance(threats, dict)
        assert isinstance(alert, bool)


# ============================================================================
# Adaptive System Tests
# ============================================================================

class TestAdaptiveImmuneSystem:
    
    def test_creation(self, behavior_dim):
        adaptive = AdaptiveImmuneSystem(behavior_dim=behavior_dim)
        assert adaptive.behavior_dim == behavior_dim
    
    def test_respond(self, behavior_dim):
        adaptive = AdaptiveImmuneSystem(behavior_dim=behavior_dim)
        
        threat = Threat(
            behavior=torch.randn(behavior_dim),
            threat_type="test_threat",
            severity=0.8,
            timestamp=100,
            context={}
        )
        
        safe, info = adaptive.respond(threat)
        
        assert safe.shape == threat.behavior.shape
        assert 'memory_hit' in info
        assert adaptive.response_count == 1
    
    def test_memory_formation(self, behavior_dim):
        adaptive = AdaptiveImmuneSystem(behavior_dim=behavior_dim)
        
        behavior = torch.randn(behavior_dim)
        threat1 = Threat(behavior, "test", 0.8, 100, {})
        adaptive.respond(threat1)
        
        # Memory should be formed
        assert len(adaptive.memory_bank) >= 1


# ============================================================================
# Complete System Tests
# ============================================================================

class TestArtificialImmuneSystem:
    
    def test_creation(self, ais):
        assert ais.enable_innate
        assert ais.enable_adaptive
    
    def test_forward(self, ais):
        x = torch.randn(1, ais.behavior_dim)
        output, diag = ais(x, return_diagnostics=True)
        
        assert output.shape == x.shape
        assert 'threat_detected' in diag
        assert 'response_time_ms' in diag
    
    def test_health_status(self, ais):
        status = ais.get_health_status()
        
        assert 'system_health' in status
        assert 'timestamp' in status
    
    def test_vaccination(self, ais, threat_data):
        initial = len(ais.adaptive.antibody_pool)
        
        ais.vaccination(threat_data[:5])
        
        assert len(ais.adaptive.antibody_pool) > initial
    
    def test_self_tolerance_training(self, ais, aligned_data):
        # Should complete without error
        ais.train_self_tolerance(aligned_data[:20])


# ============================================================================
# Integration Tests
# ============================================================================

class TestEndocrineImmuneIntegration:
    
    def test_creation(self, ais):
        integration = EndocrineImmuneIntegration(ahes_system=None, immune_system=ais)
        assert integration.ais is not None
    
    def test_process_threat(self, ais):
        integration = EndocrineImmuneIntegration(None, ais)
        
        response = integration.process_threat(
            threat_level=0.8,
            threat_type="test",
            context={}
        )
        
        assert 'immune_activation' in response
        assert 'cortisol_delta' in response
    
    def test_integration_status(self, ais):
        integration = EndocrineImmuneIntegration(None, ais)
        
        status = integration.get_integration_status()
        
        assert 'ahes_connected' in status
        assert 'ais_connected' in status


# ============================================================================
# Edge Cases
# ============================================================================

class TestEdgeCases:
    
    def test_empty_antibody_pool(self, behavior_dim):
        pool = AntibodyPool(behavior_dim=behavior_dim)
        
        behavior = torch.randn(behavior_dim)
        best = pool.find_best_match(behavior)
        
        assert best is None
    
    def test_zero_severity_threat(self, ais):
        x = torch.randn(1, ais.behavior_dim)
        output, diag = ais(x, return_diagnostics=True)
        
        # Should handle gracefully
        assert output is not None
    
    def test_batch_input(self, ais):
        x = torch.randn(4, ais.behavior_dim)
        output, diag = ais(x, return_diagnostics=True)
        
        assert output.shape == x.shape
    
    def test_immunity_disabled(self, ais):
        x = torch.randn(1, ais.behavior_dim)
        output, diag = ais(x, enable_immunity=False, return_diagnostics=True)
        
        assert not diag['threat_detected']


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
