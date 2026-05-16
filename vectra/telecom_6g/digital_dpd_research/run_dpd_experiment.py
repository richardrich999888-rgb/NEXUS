"""
Run Dpd Experiment

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Main script to run DPD experiments
"""

import yaml
import torch
import argparse
import sys
import os
sys.path.append('.')

from simulation.dpd_simulator import DPDSimulator
from training.train_joint_dpd import JointOptimizationTrainer
from models.neural_dpd import JointBeamformingDPD
from models.pa_behavioral import PAArrayModel
from beamformers.tt_beamformer import TTBeamformer

def load_config(config_path="configs/dpd_config.yaml"):
    """Load configuration file"""
    with open(config_path, 'r') as f:
        return yaml.safe_load(f)

def run_simulation(config):
    """Run DPD simulation"""
    print("=" * 60)
    print("Running DPD Simulation")
    print("=" * 60)
    
    simulator = DPDSimulator(config)
    results = simulator.run_simulation()
    
    # Plot results
    simulator.plot_results()
    
    # Generate report
    simulator.generate_report()
    
    return results

def train_model(config):
    """Train DPD model"""
    print("=" * 60)
    print("Training Joint Beamforming + DPD Model")
    print("=" * 60)
    
    # Initialize models
    pa_model = PAArrayModel(
        num_antennas=config['system']['num_antennas'],
        model_type=config['pa_model']['type']
    )
    
    dpd_model = JointBeamformingDPD(
        tt_beamformer=TTBeamformer(),
        neural_dpd=None,  # Will be created inside
        num_antennas=config['system']['num_antennas']
    )
    
    beamformer = TTBeamformer()
    
    # Create trainer
    trainer = JointOptimizationTrainer(
        config=config,
        joint_model=dpd_model,
        pa_model=pa_model,
        beamformer=beamformer
    )
    
    # Train
    history = trainer.train()
    
    return history

def main():
    parser = argparse.ArgumentParser(description='DPD Research System')
    parser.add_argument('--simulate', action='store_true', help='Run simulation')
    parser.add_argument('--train', action='store_true', help='Train model')
    parser.add_argument('--test', action='store_true', help='Run tests')
    parser.add_argument('--all', action='store_true', help='Run complete pipeline')
    parser.add_argument('--config', type=str, default='configs/dpd_config.yaml', 
                       help='Config file path')
    
    args = parser.parse_args()
    config = load_config(args.config)
    
    if args.all or (not any(vars(args).values())):
        args.simulate = True
        args.train = True
        args.test = True
    
    if args.test:
        from tests.test_dpd import run_all_tests
        run_all_tests()
    
    if args.simulate:
        results = run_simulation(config)
        print(f"\nSimulation Results:")
        print(f"  EVM without DPD: {results['evm']['without_dpd']:.2f}%")
        print(f"  EVM with DPD: {results['evm']['with_dpd']:.2f}%")
        print(f"  ACLR without DPD: {results['aclr']['without_dpd']:.2f} dBc")
        print(f"  ACLR with DPD: {results['aclr']['with_dpd']:.2f} dBc")
        print(f"  Model size: {results.get('model_size_kb', 'N/A'):.1f} KB")
    
    if args.train:
        history = train_model(config)
        print(f"\nTraining completed.")
        print(f"  Final EVM: {history['evm'][-1]:.2f}%")
        print(f"  Final NMSE: {history['nmse'][-1]:.2f} dB")
    
    print("\n" + "=" * 60)
    print("DPD Experiment Completed Successfully!")
    print("=" * 60)

if __name__ == "__main__":
    main()

