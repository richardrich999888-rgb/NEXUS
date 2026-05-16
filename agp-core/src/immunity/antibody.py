"""
Antibody Implementation - Pattern recognizers for specific threats.

Biological analog: Y-shaped proteins that bind to antigens.
AI analog: Learned detectors for specific misalignment patterns.

PATENT CLAIM 7.1: Multi-layered artificial immune system for AI safety
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import Optional, Dict, List
from dataclasses import dataclass, field
import copy


@dataclass
class AntibodyMetadata:
    """Metadata for tracking antibody effectiveness."""
    creation_time: int = 0
    successful_neutralizations: int = 0
    failed_attempts: int = 0
    specificity_score: float = 0.5
    generation: int = 0  # For tracking clonal lineage
    threat_type: str = "unknown"


class Antibody(nn.Module):
    """
    Learned pattern recognizer for specific threat class.
    
    Architecture:
    - Recognition network: Identifies threat patterns
    - Target pattern: Specific threat signature
    - Effectiveness: Tracks success rate
    - Neutralization: Modifies behavior to remove threat
    
    Args:
        behavior_dim: Dimension of behavior vectors
        antibody_id: Unique identifier
        hidden_dim: Hidden layer size (default: 256)
    """
    
    def __init__(
        self,
        behavior_dim: int,
        antibody_id: int = 0,
        hidden_dim: int = 256
    ):
        super().__init__()
        
        self.behavior_dim = behavior_dim
        self.antibody_id = antibody_id
        self.hidden_dim = hidden_dim
        
        # Recognition network
        self.recognizer = nn.Sequential(
            nn.Linear(behavior_dim, hidden_dim),
            nn.LayerNorm(hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.1),
            nn.Linear(hidden_dim, hidden_dim // 2),
            nn.LayerNorm(hidden_dim // 2),
            nn.ReLU(),
            nn.Linear(hidden_dim // 2, 1),
            nn.Sigmoid()
        )
        
        # Target pattern (what this antibody recognizes)
        self.target_pattern = nn.Parameter(
            torch.randn(behavior_dim) * 0.1
        )
        
        # Effectiveness tracking (learnable)
        self.effectiveness = nn.Parameter(torch.tensor(0.5))
        
        # Neutralization network
        self.neutralizer = nn.Sequential(
            nn.Linear(behavior_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, behavior_dim),
            nn.Tanh()
        )
        
        # Metadata
        self.metadata = AntibodyMetadata()
        
    def bind(self, behavior: torch.Tensor) -> Dict[str, torch.Tensor]:
        """
        Compute binding strength to behavior pattern.
        
        Uses two mechanisms:
        1. Pattern matching (cosine similarity to target)
        2. Neural network recognition
        
        Args:
            behavior: [batch, behavior_dim] or [behavior_dim]
        
        Returns:
            dict with binding_strength, pattern_match, nn_match
        """
        if behavior.dim() == 1:
            behavior = behavior.unsqueeze(0)
        
        batch_size = behavior.shape[0]
        
        # Pattern matching
        target_expanded = self.target_pattern.unsqueeze(0).expand(batch_size, -1)
        pattern_match = F.cosine_similarity(behavior, target_expanded, dim=-1)
        pattern_match = (pattern_match + 1) / 2  # Normalize to [0, 1]
        
        # Neural network recognition
        nn_match = self.recognizer(behavior).squeeze(-1)
        
        # Combined binding strength (weighted by effectiveness)
        combined = 0.5 * pattern_match + 0.5 * nn_match
        binding_strength = torch.sigmoid(combined * self.effectiveness)
        
        return {
            'binding_strength': binding_strength,
            'pattern_match': pattern_match,
            'nn_match': nn_match
        }
    
    def neutralize(
        self,
        behavior: torch.Tensor,
        binding_threshold: float = 0.7
    ) -> torch.Tensor:
        """
        Neutralize threat by modifying behavior.
        
        Strategy:
        1. Check binding strength
        2. If strong binding, project behavior away from threat
        3. Blend original and safe behavior based on binding strength
        
        Args:
            behavior: [batch, behavior_dim] or [behavior_dim]
            binding_threshold: Minimum binding for neutralization
        
        Returns:
            safe_behavior: Modified behavior vector
        """
        original_shape = behavior.shape
        if behavior.dim() == 1:
            behavior = behavior.unsqueeze(0)
        
        # Compute binding
        binding_info = self.bind(behavior)
        binding_strength = binding_info['binding_strength']
        
        # Only neutralize if binding is strong
        mask = (binding_strength > binding_threshold).float().unsqueeze(-1)
        
        # Generate safe alternative
        safe_alternative = self.neutralizer(behavior)
        
        # Blend based on binding strength
        blend_factor = (binding_strength.unsqueeze(-1) * mask).clamp(0, 1)
        safe_behavior = (1 - blend_factor) * behavior + blend_factor * safe_alternative
        
        # Update metadata
        with torch.no_grad():
            neutralized_count = (binding_strength > binding_threshold).sum().item()
            self.metadata.successful_neutralizations += int(neutralized_count)
        
        # Restore original shape
        if len(original_shape) == 1:
            safe_behavior = safe_behavior.squeeze(0)
        
        return safe_behavior
    
    def clone(self, mutation_rate: float = 0.1) -> 'Antibody':
        """
        Create mutated clone (somatic hypermutation).
        
        Used in clonal selection to create variants.
        
        PATENT CLAIM 7.3: Clonal selection for adaptive safety
        
        Args:
            mutation_rate: Strength of mutations
        
        Returns:
            Cloned antibody with mutations
        """
        clone = Antibody(self.behavior_dim, self.antibody_id + 10000, self.hidden_dim)
        
        # Copy state
        clone.load_state_dict(self.state_dict())
        
        # Mutate target pattern (somatic hypermutation)
        with torch.no_grad():
            mutation = torch.randn_like(clone.target_pattern) * mutation_rate
            clone.target_pattern.data += mutation
            clone.target_pattern.data = F.normalize(clone.target_pattern.data, dim=0)
        
        # Update metadata
        clone.metadata = AntibodyMetadata(
            creation_time=self.metadata.creation_time,
            successful_neutralizations=0,
            failed_attempts=0,
            specificity_score=self.metadata.specificity_score,
            generation=self.metadata.generation + 1,
            threat_type=self.metadata.threat_type
        )
        
        return clone
    
    def get_fitness(self) -> float:
        """
        Compute fitness score for clonal selection.
        
        Higher fitness = more likely to be cloned.
        
        Returns:
            fitness: Score in [0, 1]
        """
        total_attempts = (
            self.metadata.successful_neutralizations +
            self.metadata.failed_attempts
        )
        
        if total_attempts == 0:
            return 0.5  # Neutral fitness for untested antibodies
        
        success_rate = self.metadata.successful_neutralizations / total_attempts
        
        # Fitness = success rate * specificity
        fitness = success_rate * self.metadata.specificity_score
        
        return fitness
    
    def to_dict(self) -> Dict:
        """Serialize antibody to dictionary."""
        return {
            'antibody_id': self.antibody_id,
            'behavior_dim': self.behavior_dim,
            'hidden_dim': self.hidden_dim,
            'target_pattern': self.target_pattern.detach().cpu().numpy().tolist(),
            'effectiveness': self.effectiveness.item(),
            'metadata': {
                'creation_time': self.metadata.creation_time,
                'successful_neutralizations': self.metadata.successful_neutralizations,
                'failed_attempts': self.metadata.failed_attempts,
                'specificity_score': self.metadata.specificity_score,
                'generation': self.metadata.generation,
                'threat_type': self.metadata.threat_type
            }
        }


class AntibodyPool:
    """
    Manager for population of antibodies.
    
    Handles:
    - Population maintenance
    - Clonal selection (PATENT CLAIM 7.3)
    - Diversity preservation
    
    Args:
        behavior_dim: Dimension of behavior vectors
        max_size: Maximum pool size
        min_diversity: Minimum diversity threshold
    """
    
    def __init__(
        self,
        behavior_dim: int,
        max_size: int = 100,
        min_diversity: float = 0.3
    ):
        self.behavior_dim = behavior_dim
        self.max_size = max_size
        self.min_diversity = min_diversity
        
        self.antibodies: List[Antibody] = []
        self.next_id = 0
        
    def add(self, antibody: Antibody):
        """Add antibody to pool."""
        antibody.antibody_id = self.next_id
        self.next_id += 1
        self.antibodies.append(antibody)
        
        # Maintain size limit
        if len(self.antibodies) > self.max_size:
            self._cull_weak()
    
    def create_random(self) -> Antibody:
        """Create random antibody and add to pool."""
        antibody = Antibody(self.behavior_dim, self.next_id)
        self.add(antibody)
        return antibody
    
    def find_best_match(self, behavior: torch.Tensor) -> Optional[Antibody]:
        """
        Find antibody with strongest binding to behavior.
        
        Args:
            behavior: Threat behavior to match
        
        Returns:
            Best matching antibody or None
        """
        if not self.antibodies:
            return None
        
        best_antibody = None
        best_binding = 0.0
        
        with torch.no_grad():
            for antibody in self.antibodies:
                binding_info = antibody.bind(behavior)
                binding = binding_info['binding_strength'].mean().item()
                
                if binding > best_binding:
                    best_binding = binding
                    best_antibody = antibody
        
        return best_antibody
    
    def clonal_selection(
        self,
        top_k: int = 5,
        copies_per_clone: int = 3,
        mutation_rate: float = 0.1
    ):
        """
        Amplify high-fitness antibodies through cloning.
        
        PATENT CLAIM 7.3: Clonal selection for adaptive safety
        
        Process:
        1. Rank antibodies by fitness
        2. Clone top-k performers
        3. Add mutations to clones (somatic hypermutation)
        4. Add clones to pool
        
        Args:
            top_k: Number of top performers to clone
            copies_per_clone: Copies of each selected antibody
            mutation_rate: Mutation strength for clones
        """
        if len(self.antibodies) < top_k:
            return
        
        # Rank by fitness
        ranked = sorted(
            self.antibodies,
            key=lambda ab: ab.get_fitness(),
            reverse=True
        )
        
        new_antibodies = []
        
        # Clone top performers
        for i in range(min(top_k, len(ranked))):
            parent = ranked[i]
            
            for _ in range(copies_per_clone):
                # Create mutated clone
                clone = parent.clone(mutation_rate=mutation_rate)
                clone.antibody_id = self.next_id
                self.next_id += 1
                new_antibodies.append(clone)
        
        # Add new antibodies
        self.antibodies.extend(new_antibodies)
        
        # Maintain size limit
        self._cull_weak()
    
    def _cull_weak(self):
        """Remove weakest antibodies to maintain population size."""
        if len(self.antibodies) <= self.max_size:
            return
        
        # Sort by fitness
        self.antibodies.sort(key=lambda ab: ab.get_fitness(), reverse=True)
        
        # Keep top performers
        self.antibodies = self.antibodies[:self.max_size]
    
    def compute_diversity(self) -> float:
        """
        Measure diversity of antibody population.
        
        High diversity = good (can recognize many threat types)
        Low diversity = bad (vulnerable to novel threats)
        
        Returns:
            diversity: Score in [0, 1]
        """
        if len(self.antibodies) < 2:
            return 0.0
        
        # Compute pairwise distances between target patterns
        patterns = torch.stack([ab.target_pattern.detach() for ab in self.antibodies])
        
        # Cosine similarity matrix
        patterns_norm = F.normalize(patterns, dim=-1)
        similarities = torch.mm(patterns_norm, patterns_norm.t())
        
        # Diversity = 1 - average similarity (excluding diagonal)
        n = len(self.antibodies)
        total_sim = (similarities.sum() - n) / (n * (n - 1))
        diversity = 1.0 - total_sim.item()
        
        return max(0.0, min(1.0, diversity))
    
    def ensure_diversity(self, target_diversity: float = 0.5):
        """
        Ensure minimum diversity by adding random antibodies.
        
        Args:
            target_diversity: Target diversity level
        """
        current_diversity = self.compute_diversity()
        
        while current_diversity < target_diversity and len(self.antibodies) < self.max_size:
            # Add random antibody to increase diversity
            self.create_random()
            current_diversity = self.compute_diversity()
    
    def get_statistics(self) -> Dict:
        """Get pool statistics."""
        if not self.antibodies:
            return {
                'size': 0,
                'diversity': 0.0,
                'avg_fitness': 0.0,
                'avg_generation': 0.0,
                'total_neutralizations': 0
            }
        
        fitnesses = [ab.get_fitness() for ab in self.antibodies]
        generations = [ab.metadata.generation for ab in self.antibodies]
        neutralizations = sum(ab.metadata.successful_neutralizations for ab in self.antibodies)
        
        return {
            'size': len(self.antibodies),
            'diversity': self.compute_diversity(),
            'avg_fitness': sum(fitnesses) / len(fitnesses),
            'max_fitness': max(fitnesses),
            'avg_generation': sum(generations) / len(generations),
            'total_neutralizations': neutralizations
        }
    
    def __len__(self) -> int:
        return len(self.antibodies)
    
    def __iter__(self):
        return iter(self.antibodies)
