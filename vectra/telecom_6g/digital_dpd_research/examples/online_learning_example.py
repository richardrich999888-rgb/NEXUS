"""
Online Learning Example

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Online Learning for DPD Example
Demonstrates real-time DPD adaptation
"""

import torch
import numpy as np
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent.parent))

from models.neural_dpd import NeuralDPD
from models.pa_behavioral import RappModel
from training.online_learning import OnlineDPDLearner, RealTimeDPDAdaptation
from utils.metrics import DPDEvaluator
from utils.signal_generation import SignalGenerator

def main():
    print("=" * 60)
    print("Online DPD Learning Example")
    print("=" * 60)
    
    # Initialize models
    print("\n1. Initializing models...")
    dpd_model = NeuralDPD(memory_depth=5, hidden_dims=[32, 32])
    pa_model = RappModel(smoothness_factor=3.0)
    
    # Create online learner
    learner = OnlineDPDLearner(
        dpd_model,
        learning_rate=1e-4,
        adaptation_rate=0.1,
        memory_size=1000,
        update_frequency=100
    )
    
    print("   ✓ Online learner initialized")
    
    # Generate test signals
    print("\n2. Generating test signals...")
    signal_gen = SignalGenerator()
    test_signal = signal_gen.generate_ofdm_signal(num_symbols=100, modulation='64qam')
    
    # Simulate online adaptation
    print("\n3. Simulating online adaptation...")
    evaluator = DPDEvaluator()
    
    num_samples = 500
    evm_history = []
    
    for i in range(num_samples):
        # Get signal sample
        if i < len(test_signal):
            input_sample = test_signal[i:i+1].unsqueeze(0)  # (1, 1)
        else:
            input_sample = torch.randn(1, 1, dtype=torch.cfloat)
        
        # Convert to I/Q
        input_iq = torch.stack([input_sample.real, input_sample.imag], dim=-1)
        
        # Apply DPD
        with torch.no_grad():
            dpd_output = dpd_model(input_iq)
            dpd_complex = torch.complex(dpd_output[..., 0], dpd_output[..., 1])
        
        # Apply PA
        pa_input_iq = torch.stack([dpd_complex.real, dpd_complex.imag], dim=-1)
        pa_output = pa_model(pa_input_iq)
        pa_output_complex = torch.complex(pa_output[..., 0], pa_output[..., 1])
        
        # Compute metrics
        evm = evaluator.calculate_evm(input_sample, pa_output_complex)
        evm_history.append(evm)
        
        # Add to experience buffer
        learner.add_experience(
            input_iq,
            pa_output,
            input_iq,
            {'evm': evm, 'aclr': -40.0}  # Simplified
        )
        
        # Update model periodically
        if learner.should_update():
            loss = learner.update_model()
            if i % 100 == 0:
                print(f"   Update {learner.update_count}: loss={loss:.6f}, EVM={evm:.2f}%")
    
    # Get final statistics
    print("\n4. Final statistics...")
    stats = learner.get_performance_stats()
    print(f"   Average EVM: {stats['avg_evm']:.2f}%")
    print(f"   Updates performed: {stats['update_count']}")
    print(f"   Memory size: {stats['memory_size']}")
    
    # Show improvement
    initial_evm = np.mean(evm_history[:100])
    final_evm = np.mean(evm_history[-100:])
    improvement = initial_evm - final_evm
    
    print(f"\n   Initial EVM: {initial_evm:.2f}%")
    print(f"   Final EVM: {final_evm:.2f}%")
    print(f"   Improvement: {improvement:.2f}%")
    
    print("\n" + "=" * 60)
    print("Online learning example completed!")
    print("=" * 60)

if __name__ == "__main__":
    main()



