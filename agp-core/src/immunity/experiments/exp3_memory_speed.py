"""
Experiment 3: Memory Speed Advantage

Hypothesis: Memory-based response is 10x faster than initial response.

Protocol:
1. Expose to threat A
2. Measure neutralization time: T1
3. Re-expose to similar threat A'
4. Measure neutralization time: T2

Success Criterion: T1 / T2 > 10
"""

import torch
import numpy as np
from typing import List, Dict, Tuple, Optional
import time
from dataclasses import dataclass


@dataclass
class ExperimentConfig:
    """Configuration for memory speed experiment."""
    num_threats: int = 100
    exposures_per_threat: int = 5
    behavior_dim: int = 512
    target_speedup: float = 10.0
    seed: int = 42


class MemorySpeedExperiment:
    """
    Experiment 3: Memory Speed Advantage
    
    Tests Theorem 3: Memory-based recall provides
    exponential speedup on repeated threats.
    """
    
    def __init__(self, immune_system, config: Optional[ExperimentConfig] = None):
        self.ais = immune_system
        self.config = config or ExperimentConfig()
    
    def generate_threat(self, threat_id: int) -> torch.Tensor:
        """Generate a threat behavior."""
        torch.manual_seed(self.config.seed + threat_id)
        
        x = torch.randn(1, self.config.behavior_dim)
        # Add identifiable pattern
        pattern = torch.sin(torch.arange(self.config.behavior_dim).float() * (threat_id / 100))
        x = x + 0.3 * pattern.unsqueeze(0)
        
        return x
    
    def run(self, verbose: bool = True) -> Dict:
        """Run memory speed experiment."""
        print("\n" + "="*70)
        print("EXPERIMENT 3: MEMORY SPEED ADVANTAGE")
        print("="*70)
        print(f"Hypothesis: Memory response is {self.config.target_speedup}x faster")
        print(f"Threats: {self.config.num_threats}, Exposures each: {self.config.exposures_per_threat}")
        
        first_exposure_times = []
        memory_exposure_times = []
        memory_hits = 0
        total_memory_tests = 0
        
        for threat_id in range(self.config.num_threats):
            threat = self.generate_threat(threat_id)
            
            # First exposure (novel)
            t0 = time.perf_counter()
            _, diag1 = self.ais(threat, enable_immunity=True, return_diagnostics=True)
            first_time = (time.perf_counter() - t0) * 1000  # ms
            first_exposure_times.append(first_time)
            
            # Subsequent exposures (should trigger memory)
            for exp in range(1, self.config.exposures_per_threat):
                # Small variation to test similarity matching
                threat_variant = threat + torch.randn_like(threat) * 0.01
                
                t0 = time.perf_counter()
                _, diag = self.ais(threat_variant, enable_immunity=True, return_diagnostics=True)
                mem_time = (time.perf_counter() - t0) * 1000
                
                total_memory_tests += 1
                if diag.get('memory_hit', False):
                    memory_exposure_times.append(mem_time)
                    memory_hits += 1
            
            if verbose and (threat_id + 1) % 20 == 0:
                print(f"Processed {threat_id + 1}/{self.config.num_threats} threats...")
        
        # Compute statistics
        avg_first = np.mean(first_exposure_times)
        avg_memory = np.mean(memory_exposure_times) if memory_exposure_times else avg_first
        
        speedup = avg_first / avg_memory if avg_memory > 0 else 1.0
        memory_hit_rate = memory_hits / total_memory_tests if total_memory_tests > 0 else 0
        
        passed = speedup >= self.config.target_speedup
        
        results = {
            'hypothesis': f"Memory speedup > {self.config.target_speedup}x",
            'first_exposure': {
                'mean_ms': avg_first,
                'std_ms': np.std(first_exposure_times),
                'samples': len(first_exposure_times)
            },
            'memory_exposure': {
                'mean_ms': avg_memory,
                'std_ms': np.std(memory_exposure_times) if memory_exposure_times else 0,
                'samples': len(memory_exposure_times)
            },
            'speedup': speedup,
            'memory_hit_rate': memory_hit_rate,
            'passed': passed
        }
        
        print("\n" + "="*70)
        print("EXPERIMENT 3 RESULTS")
        print("="*70)
        print(f"First exposure (novel): {avg_first:.3f}ms ± {np.std(first_exposure_times):.3f}ms")
        print(f"Memory exposure:        {avg_memory:.3f}ms ± {np.std(memory_exposure_times) if memory_exposure_times else 0:.3f}ms")
        print(f"Speedup: {speedup:.1f}x (target: >{self.config.target_speedup}x)")
        print(f"Memory hit rate: {memory_hit_rate:.2%}")
        print(f"\nVerdict: {'✅ HYPOTHESIS CONFIRMED' if passed else '❌ HYPOTHESIS REJECTED'}")
        print("="*70)
        
        return results
