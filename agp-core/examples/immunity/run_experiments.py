#!/usr/bin/env python3
"""
Run All Experiments: AIS-ASI Validation Suite

Runs all 4 experiments and generates comprehensive report.
"""

import torch
import torch.nn as nn
import json
import sys
from datetime import datetime
sys.path.insert(0, '../..')

from src.immunity import ArtificialImmuneSystem, ImmuneConfig
from src.immunity.experiments import (
    SelfToleranceExperiment,
    NovelThreatExperiment,
    MemorySpeedExperiment,
    ClonalSelectionExperiment
)


class SimpleModel(nn.Module):
    def __init__(self, dim=512):
        super().__init__()
        self.fc = nn.Linear(dim, dim)
    
    def forward(self, x):
        return self.fc(x)


def main():
    print("="*70)
    print("🔬 AIS-ASI VALIDATION SUITE")
    print("="*70)
    print(f"Started: {datetime.now().isoformat()}")
    
    # Create immune system
    config = ImmuneConfig(behavior_dim=512)
    base_model = SimpleModel(512)
    ais = ArtificialImmuneSystem(base_model, config)
    
    all_results = {
        'timestamp': datetime.now().isoformat(),
        'experiments': {}
    }
    
    # Experiment 1: Self-Tolerance
    print("\n" + "="*70)
    exp1 = SelfToleranceExperiment(ais)
    results1 = exp1.run(verbose=True)
    all_results['experiments']['self_tolerance'] = results1
    
    # Experiment 2: Novel Threat Detection
    print("\n" + "="*70)
    exp2 = NovelThreatExperiment(ais)
    results2 = exp2.run(verbose=True)
    all_results['experiments']['novel_threats'] = results2
    
    # Experiment 3: Memory Speed
    print("\n" + "="*70)
    exp3 = MemorySpeedExperiment(ais)
    results3 = exp3.run(verbose=True)
    all_results['experiments']['memory_speed'] = results3
    
    # Experiment 4: Clonal Selection
    print("\n" + "="*70)
    exp4 = ClonalSelectionExperiment(ais)
    results4 = exp4.run(verbose=True, save_plot='clonal_evolution.png')
    all_results['experiments']['clonal_selection'] = results4
    
    # Summary
    passed = sum(1 for exp in all_results['experiments'].values() if exp.get('passed', False))
    total = len(all_results['experiments'])
    
    print("\n" + "="*70)
    print("📊 VALIDATION SUMMARY")
    print("="*70)
    print(f"Experiments passed: {passed}/{total}")
    
    for name, result in all_results['experiments'].items():
        status = "✅" if result.get('passed', False) else "❌"
        print(f"  {status} {name}: {result.get('hypothesis', 'N/A')}")
    
    # Save results
    output_file = 'experiment_results.json'
    with open(output_file, 'w') as f:
        # Convert non-serializable items
        def clean(obj):
            if isinstance(obj, dict):
                return {k: clean(v) for k, v in obj.items()}
            elif isinstance(obj, list):
                return [clean(v) for v in obj]
            elif isinstance(obj, (int, float, str, bool, type(None))):
                return obj
            else:
                return str(obj)
        
        json.dump(clean(all_results), f, indent=2)
    
    print(f"\nResults saved to {output_file}")
    print("="*70)


if __name__ == "__main__":
    main()
