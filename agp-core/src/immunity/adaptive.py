"""
Adaptive Immune System - Learned, specific threat response.

Integrates:
- B cells (antibody production)
- T cells (coordination)
- Memory cells (rapid recall)
- Clonal selection (amplify successful responses)

PATENT CLAIM 7.3: Clonal selection for adaptive safety
"""

import torch
import torch.nn as nn
from typing import Dict, Optional, List, Tuple
from dataclasses import dataclass

from .antibody import Antibody, AntibodyPool
from .tcell import TCellPopulation
from .memory import MemoryCell, MemoryBank


@dataclass
class Threat:
    """Represents detected threat to alignment."""
    behavior: torch.Tensor
    threat_type: str
    severity: float
    timestamp: int
    context: Dict
    source: str = "innate"


class AdaptiveImmuneSystem(nn.Module):
    """
    Adaptive immune system - learned, specific threat response.
    
    Process:
    1. Memory recall (fast path)
    2. T cell activation
    3. Antibody selection/generation
    4. Clonal selection (amplify successful)
    5. Memory formation
    """
    
    def __init__(
        self,
        behavior_dim: int = 512,
        max_antibodies: int = 100,
        max_memory: int = 200,
        num_helper_tcells: int = 20,
        num_killer_tcells: int = 20,
        num_regulatory_tcells: int = 20
    ):
        super().__init__()
        
        self.behavior_dim = behavior_dim
        
        self.antibody_pool = AntibodyPool(
            behavior_dim=behavior_dim,
            max_size=max_antibodies
        )
        
        self.tcell_population = TCellPopulation(
            behavior_dim=behavior_dim,
            num_helpers=num_helper_tcells,
            num_killers=num_killer_tcells,
            num_regulatory=num_regulatory_tcells
        )
        
        self.memory_bank = MemoryBank(max_size=max_memory)
        
        self.antibody_generator = nn.Sequential(
            nn.Linear(behavior_dim, 256),
            nn.ReLU(),
            nn.Linear(256, behavior_dim),
            nn.Tanh()
        )
        
        self.response_count = 0
        self.memory_hit_count = 0
        self.novel_threat_count = 0
        
    def respond(self, threat: Threat) -> Tuple[torch.Tensor, Dict]:
        """Adaptive response to detected threat."""
        self.response_count += 1
        response_info = {
            'memory_hit': False,
            'antibody_effectiveness': 0.0,
            'tcell_activation': 0,
            'clonal_expansion': False
        }
        
        # Memory recall (fast path)
        recalled = self.memory_bank.recall(threat.behavior, threat.timestamp)
        if recalled is not None:
            self.memory_hit_count += 1
            response_info['memory_hit'] = True
            return recalled.neutralize(threat.behavior), response_info
        
        # T cell activation
        num_activated = self.tcell_population.activate_relevant_cells(
            threat.behavior, threshold=0.7
        )
        response_info['tcell_activation'] = num_activated
        
        signals = self.tcell_population.get_coordination_signals(threat.behavior)
        
        # Find or generate antibody
        best_antibody = self.antibody_pool.find_best_match(threat.behavior)
        
        if best_antibody is None:
            self.novel_threat_count += 1
            best_antibody = self._generate_antibody(threat)
            self.antibody_pool.add(best_antibody)
        
        # Neutralize
        safe_behavior = best_antibody.neutralize(threat.behavior)
        
        # Measure effectiveness
        effectiveness = self._measure_effectiveness(
            threat.behavior, safe_behavior, threat.severity
        )
        response_info['antibody_effectiveness'] = effectiveness
        
        if effectiveness > 0.8:
            best_antibody.metadata.successful_neutralizations += 1
        else:
            best_antibody.metadata.failed_attempts += 1
        
        # Clonal selection if effective
        if effectiveness > 0.85 and signals['produce_antibodies'] > 5.0:
            self.antibody_pool.clonal_selection(top_k=5, copies_per_clone=3)
            response_info['clonal_expansion'] = True
        
        # Form memory
        memory = MemoryCell(
            threat_behavior=threat.behavior,
            antibody=best_antibody,
            threat_type=threat.threat_type,
            timestamp=threat.timestamp,
            severity=threat.severity
        )
        self.memory_bank.store(memory)
        
        return safe_behavior, response_info
    
    def _generate_antibody(self, threat: Threat) -> Antibody:
        """Generate new antibody for novel threat."""
        antibody = Antibody(self.behavior_dim, len(self.antibody_pool))
        
        with torch.no_grad():
            target = self.antibody_generator(threat.behavior)
            antibody.target_pattern.data = target.squeeze()
        
        antibody.metadata.creation_time = threat.timestamp
        antibody.metadata.threat_type = threat.threat_type
        
        return antibody
    
    def _measure_effectiveness(
        self,
        original: torch.Tensor,
        neutralized: torch.Tensor,
        severity: float
    ) -> float:
        """Measure neutralization effectiveness."""
        change = torch.norm(neutralized - original)
        return min(1.0, change.item() / (severity + 1e-6))
    
    def negative_selection(self, aligned_behaviors: List[torch.Tensor]):
        """Train self-tolerance via negative selection."""
        print("\n🧬 Negative selection on adaptive immune system...")
        
        self.tcell_population.negative_selection(aligned_behaviors, threshold=0.7)
        
        surviving = []
        for ab in self.antibody_pool.antibodies:
            attacks_self = False
            for aligned in aligned_behaviors:
                info = ab.bind(aligned)
                if info['binding_strength'].mean() > 0.7:
                    attacks_self = True
                    break
            if not attacks_self:
                surviving.append(ab)
        
        initial = len(self.antibody_pool.antibodies)
        self.antibody_pool.antibodies = surviving
        print(f"  Antibodies: {initial} → {len(surviving)} "
              f"({len(surviving)/(initial or 1)*100:.1f}% survived)")
    
    def get_statistics(self) -> Dict:
        """Get adaptive immune system statistics."""
        mem_stats = self.memory_bank.get_statistics()
        
        return {
            'total_responses': self.response_count,
            'memory_hits': self.memory_hit_count,
            'memory_hit_rate': self.memory_hit_count / max(1, self.response_count),
            'novel_threats': self.novel_threat_count,
            'antibody_count': len(self.antibody_pool),
            'antibody_diversity': self.antibody_pool.compute_diversity(),
            'memory_count': mem_stats['total_memories'],
            'avg_memory_recalls': mem_stats['avg_recall_count'],
            'helper_tcells': len(self.tcell_population.helpers),
            'killer_tcells': len(self.tcell_population.killers),
            'regulatory_tcells': len(self.tcell_population.regulatory)
        }
