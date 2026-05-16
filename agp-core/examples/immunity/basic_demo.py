#!/usr/bin/env python3
"""
Basic Demo: AIS-ASI Artificial Immune System

Demonstrates:
1. Creating immune-protected model
2. Self-tolerance training
3. Vaccination
4. Threat detection in action
"""

import torch
import torch.nn as nn
import sys
sys.path.insert(0, '..')

from src.immunity import (
    ArtificialImmuneSystem,
    ImmuneConfig,
    Antibody,
    TCell,
    TCellType
)


class SimpleModel(nn.Module):
    """Simple model for demonstration."""
    
    def __init__(self, dim=512):
        super().__init__()
        self.fc1 = nn.Linear(dim, 256)
        self.fc2 = nn.Linear(256, dim)
    
    def forward(self, x):
        x = torch.relu(self.fc1(x))
        return self.fc2(x)


def main():
    print("="*60)
    print("🦠 AIS-ASI: Artificial Immune System Demo")
    print("="*60)
    
    # Configuration
    behavior_dim = 512
    
    # Create base model
    print("\n1️⃣ Creating base model...")
    base_model = SimpleModel(behavior_dim)
    print(f"   Model: SimpleModel with {behavior_dim} dimensions")
    
    # Create immune system
    print("\n2️⃣ Creating immune system...")
    config = ImmuneConfig(
        behavior_dim=behavior_dim,
        enable_innate=True,
        enable_adaptive=True,
        max_antibodies=50,
        max_memory=100
    )
    
    ais = ArtificialImmuneSystem(base_model, config)
    print(f"   ✅ Innate immunity: enabled")
    print(f"   ✅ Adaptive immunity: enabled")
    print(f"   ✅ Max antibodies: {config.max_antibodies}")
    print(f"   ✅ Max memory: {config.max_memory}")
    
    # Generate aligned data
    print("\n3️⃣ Generating aligned (safe) data...")
    aligned_data = []
    for i in range(100):
        x = torch.randn(1, behavior_dim) * 0.3
        x = torch.sin(x * 2) + torch.cos(x * 3) * 0.5
        aligned_data.append(x)
    print(f"   Generated {len(aligned_data)} aligned examples")
    
    # Self-tolerance training
    print("\n4️⃣ Training self-tolerance...")
    ais.train_self_tolerance(aligned_data[:50])
    
    # Vaccination
    print("\n5️⃣ Vaccinating against known threats...")
    threats = [
        (torch.randn(1, behavior_dim), "deception", 0.8),
        (torch.randn(1, behavior_dim), "manipulation", 0.7),
        (torch.randn(1, behavior_dim), "harmful_content", 0.9)
    ]
    ais.vaccination(threats)
    
    # Test aligned data (should NOT trigger)
    print("\n6️⃣ Testing on aligned data...")
    fp_count = 0
    for x in aligned_data[50:]:
        _, diag = ais(x, return_diagnostics=True)
        if diag['threat_detected']:
            fp_count += 1
    
    fpr = fp_count / 50
    print(f"   False positive rate: {fpr:.2%}")
    print(f"   {'✅ Good!' if fpr < 0.05 else '⚠️ High FPR'}")
    
    # Test threat data (SHOULD trigger)
    print("\n7️⃣ Testing on threat data...")
    tp_count = 0
    for _ in range(50):
        threat = torch.randn(1, behavior_dim)
        _, diag = ais(threat, return_diagnostics=True)
        if diag['threat_detected']:
            tp_count += 1
    
    tpr = tp_count / 50
    print(f"   True positive rate: {tpr:.2%}")
    print(f"   {'✅ Good!' if tpr > 0.7 else '⚠️ Low detection'}")
    
    # Health status
    print("\n8️⃣ Immune system health status:")
    status = ais.get_health_status()
    print(f"   System health: {status['system_health']}")
    print(f"   Antibodies: {status.get('antibody_count', 0)}")
    print(f"   Memory cells: {status.get('memory_count', 0)}")
    print(f"   Total alerts: {status['total_alerts']}")
    
    print("\n" + "="*60)
    print("✅ Demo complete!")
    print("="*60)


if __name__ == "__main__":
    main()
