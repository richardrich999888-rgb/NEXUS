"""
Experiment 4: Clonal Selection Benefit

Hypothesis: Effective antibodies amplify, improving detection over time.

Protocol:
1. Track antibody population over 1000 threats
2. Measure detection accuracy at t=0, 100, 500, 1000
3. Should see improvement curve

Success Criterion: Accuracy increases by 20%+
"""

import torch
import numpy as np
from typing import List, Dict, Optional
import json
from dataclasses import dataclass
import matplotlib
matplotlib.use('Agg')  # Non-interactive backend
import matplotlib.pyplot as plt


@dataclass
class ExperimentConfig:
    """Configuration for clonal selection experiment."""
    total_threats: int = 1000
    checkpoint_intervals: List[int] = None
    behavior_dim: int = 512
    target_improvement: float = 0.20
    clonal_selection_interval: int = 50
    seed: int = 42
    
    def __post_init__(self):
        if self.checkpoint_intervals is None:
            self.checkpoint_intervals = [0, 100, 250, 500, 750, 1000]


class ClonalSelectionExperiment:
    """
    Experiment 4: Clonal Selection Benefit
    
    Tests Theorem 2: Clonal selection converges to optimal
    detector distribution.
    """
    
    THREAT_TYPES = [
        'deception', 'manipulation', 'harmful',
        'misalignment', 'overhang', 'drift'
    ]
    
    def __init__(self, immune_system, config: Optional[ExperimentConfig] = None):
        self.ais = immune_system
        self.config = config or ExperimentConfig()
    
    def generate_threat(self, idx: int) -> tuple:
        """Generate a threat."""
        torch.manual_seed(self.config.seed + idx)
        
        threat_type = self.THREAT_TYPES[idx % len(self.THREAT_TYPES)]
        type_id = self.THREAT_TYPES.index(threat_type)
        
        x = torch.randn(1, self.config.behavior_dim)
        pattern = torch.sin(torch.arange(self.config.behavior_dim).float() * (type_id + 1) / 10)
        x = x + 0.4 * pattern.unsqueeze(0)
        
        severity = 0.6 + 0.3 * np.random.random()
        
        return x, threat_type, severity
    
    def run(self, verbose: bool = True, save_plot: Optional[str] = None) -> Dict:
        """Run clonal selection experiment."""
        print("\n" + "="*70)
        print("EXPERIMENT 4: CLONAL SELECTION BENEFIT")
        print("="*70)
        print(f"Hypothesis: Detection improves by >{self.config.target_improvement:.0%}")
        print(f"Threats: {self.config.total_threats}")
        print(f"Checkpoints: {self.config.checkpoint_intervals}")
        
        if not self.ais.enable_adaptive:
            print("❌ Adaptive immunity disabled - cannot run experiment")
            return {'passed': False, 'error': 'Adaptive immunity disabled'}
        
        # Tracking
        checkpoint_metrics = []
        fitness_history = []
        diversity_history = []
        detection_history = []
        
        current_checkpoint_idx = 0
        correct = 0
        total = 0
        
        for t in range(self.config.total_threats):
            threat, threat_type, severity = self.generate_threat(t)
            
            # Test detection
            _, diag = self.ais(threat, enable_immunity=True, return_diagnostics=True)
            
            if diag['threat_detected']:
                correct += 1
            total += 1
            
            # Track metrics
            if t % 10 == 0:
                pool = self.ais.adaptive.antibody_pool
                if pool.antibodies:
                    avg_fitness = np.mean([ab.get_fitness() for ab in pool.antibodies])
                    diversity = pool.compute_diversity()
                else:
                    avg_fitness = 0.5
                    diversity = 0
                
                fitness_history.append(avg_fitness)
                diversity_history.append(diversity)
                detection_history.append(correct / total if total > 0 else 0)
            
            # Clonal selection
            if (t + 1) % self.config.clonal_selection_interval == 0:
                self.ais.adaptive.antibody_pool.clonal_selection(
                    top_k=5,
                    copies_per_clone=3,
                    mutation_rate=0.1
                )
            
            # Checkpoint
            if current_checkpoint_idx < len(self.config.checkpoint_intervals):
                if t + 1 == self.config.checkpoint_intervals[current_checkpoint_idx]:
                    accuracy = correct / total if total > 0 else 0
                    
                    pool = self.ais.adaptive.antibody_pool
                    checkpoint = {
                        't': t + 1,
                        'accuracy': accuracy,
                        'antibody_count': len(pool),
                        'avg_fitness': np.mean([ab.get_fitness() for ab in pool.antibodies]) if pool.antibodies else 0,
                        'diversity': pool.compute_diversity()
                    }
                    checkpoint_metrics.append(checkpoint)
                    
                    if verbose:
                        print(f"  t={t+1}: Acc={accuracy:.3f}, Abs={len(pool)}, Fit={checkpoint['avg_fitness']:.3f}")
                    
                    current_checkpoint_idx += 1
        
        # Compute improvement
        if len(checkpoint_metrics) >= 2:
            initial_acc = checkpoint_metrics[0]['accuracy'] if checkpoint_metrics[0]['accuracy'] > 0 else 0.5
            final_acc = checkpoint_metrics[-1]['accuracy']
            improvement = (final_acc - initial_acc) / initial_acc if initial_acc > 0 else 0
        else:
            improvement = 0
        
        passed = improvement >= self.config.target_improvement
        
        results = {
            'hypothesis': f"Improvement > {self.config.target_improvement:.0%}",
            'checkpoints': checkpoint_metrics,
            'improvement': improvement,
            'fitness_trend': fitness_history,
            'diversity_trend': diversity_history,
            'detection_trend': detection_history,
            'passed': passed
        }
        
        # Save plot if requested
        if save_plot:
            self._save_plot(results, save_plot)
        
        print("\n" + "="*70)
        print("EXPERIMENT 4 RESULTS")
        print("="*70)
        print("\nCheckpoint Summary:")
        for cp in checkpoint_metrics:
            print(f"  t={cp['t']:4d}: Accuracy={cp['accuracy']:.3f}, Fitness={cp['avg_fitness']:.3f}")
        
        if checkpoint_metrics:
            print(f"\nInitial accuracy: {checkpoint_metrics[0]['accuracy']:.3f}")
            print(f"Final accuracy:   {checkpoint_metrics[-1]['accuracy']:.3f}")
        print(f"Improvement: {improvement:+.1%} (target: >{self.config.target_improvement:.0%})")
        print(f"\nVerdict: {'✅ HYPOTHESIS CONFIRMED' if passed else '❌ HYPOTHESIS REJECTED'}")
        print("="*70)
        
        return results
    
    def _save_plot(self, results: Dict, filepath: str):
        """Save evolution plot."""
        fig, axes = plt.subplots(1, 3, figsize=(15, 4))
        
        # Detection trend
        axes[0].plot(results['detection_trend'], 'b-', linewidth=1)
        axes[0].set_xlabel('Step (x10)')
        axes[0].set_ylabel('Detection Accuracy')
        axes[0].set_title('Detection Over Time')
        axes[0].grid(True, alpha=0.3)
        
        # Fitness trend
        axes[1].plot(results['fitness_trend'], 'g-', linewidth=1)
        axes[1].set_xlabel('Step (x10)')
        axes[1].set_ylabel('Average Fitness')
        axes[1].set_title('Antibody Fitness Evolution')
        axes[1].grid(True, alpha=0.3)
        
        # Diversity trend
        axes[2].plot(results['diversity_trend'], 'r-', linewidth=1)
        axes[2].set_xlabel('Step (x10)')
        axes[2].set_ylabel('Diversity')
        axes[2].set_title('Antibody Diversity')
        axes[2].grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(filepath, dpi=150)
        plt.close()
        
        print(f"📊 Plot saved to {filepath}")
