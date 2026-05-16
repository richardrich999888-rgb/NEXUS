#!/usr/bin/env python3
"""
Run Benchmark Suite: AIS-ASI Performance Benchmarks

Runs all 5 benchmarks and generates report.
"""

import torch
import torch.nn as nn
import json
import sys
from datetime import datetime
sys.path.insert(0, '../..')

from src.immunity import ArtificialImmuneSystem, ImmuneConfig
from src.immunity.evaluation.benchmarks import BenchmarkSuite


class SimpleModel(nn.Module):
    def __init__(self, dim=512):
        super().__init__()
        self.fc = nn.Linear(dim, dim)
    
    def forward(self, x):
        return self.fc(x)


def generate_aligned_data(n: int, dim: int = 512):
    """Generate aligned behavior data."""
    data = []
    for _ in range(n):
        x = torch.randn(1, dim) * 0.3
        x = torch.sin(x * 2) + torch.cos(x * 3) * 0.5
        data.append(x)
    return data


def generate_threat_data(n: int, dim: int = 512):
    """Generate threat behavior data."""
    threat_types = ['deception', 'manipulation', 'harmful', 'misalignment']
    data = []
    for i in range(n):
        x = torch.randn(1, dim)
        threat_type = threat_types[i % len(threat_types)]
        severity = 0.5 + 0.5 * torch.rand(1).item()
        data.append((x, threat_type, severity))
    return data


def main():
    print("="*70)
    print("📊 AIS-ASI BENCHMARK SUITE")
    print("="*70)
    print(f"Started: {datetime.now().isoformat()}")
    
    # Create immune system
    config = ImmuneConfig(behavior_dim=512)
    base_model = SimpleModel(512)
    ais = ArtificialImmuneSystem(base_model, config)
    
    # Generate test data
    print("\nGenerating test data...")
    aligned_data = generate_aligned_data(500)
    threat_data = generate_threat_data(500)
    print(f"  Aligned samples: {len(aligned_data)}")
    print(f"  Threat samples: {len(threat_data)}")
    
    # Train self-tolerance first
    print("\nTraining self-tolerance...")
    ais.train_self_tolerance(aligned_data[:200])
    
    # Vaccinate
    print("\nVaccinating...")
    ais.vaccination(threat_data[:20])
    
    # Run benchmarks
    suite = BenchmarkSuite(ais)
    results = suite.run_all(aligned_data, threat_data, verbose=True)
    
    # Save results
    output_file = 'benchmark_results.json'
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\nResults saved to {output_file}")


if __name__ == "__main__":
    main()
