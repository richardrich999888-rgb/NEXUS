"""
Run Benchmark

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Main execution script for Digital RAN AI Beamforming Research
Runs complete training and benchmarking pipeline
"""

import argparse
import sys
import os
import yaml
from pathlib import Path

# Add project root to path
sys.path.append(str(Path(__file__).parent))

def load_config(config_path="configs/telecom_default.yaml"):
    """Load configuration file"""
    with open(config_path, 'r') as f:
        return yaml.safe_load(f)

def main():
    parser = argparse.ArgumentParser(description='Digital RAN AI Beamforming Research')
    parser.add_argument('--train-encoder', action='store_true', help='Train neural CSI encoder')
    parser.add_argument('--train-predictor', action='store_true', help='Train sparse beam predictor')
    parser.add_argument('--benchmark', action='store_true', help='Run comprehensive benchmark')
    parser.add_argument('--all', action='store_true', help='Run complete pipeline')
    parser.add_argument('--config', type=str, default='configs/telecom_default.yaml', help='Config file path')
    
    args = parser.parse_args()
    config = load_config(args.config)
    
    if args.all or (not any(vars(args).values())):
        args.train_encoder = True
        args.train_predictor = True
        args.benchmark = True
    
    if args.train_encoder:
        print("=" * 60)
        print("TRAINING NEURAL CSI ENCODER")
        print("=" * 60)
        from training.train_encoder import EncoderTrainer
        trainer = EncoderTrainer(config)
        trainer.train()
    
    if args.train_predictor:
        print("=" * 60)
        print("TRAINING SPARSE BEAM PREDICTOR")
        print("=" * 60)
        from training.train_predictor import PredictorTrainer
        trainer = PredictorTrainer(config)
        trainer.train()
    
    if args.benchmark:
        print("=" * 60)
        print("RUNNING COMPREHENSIVE BENCHMARK")
        print("=" * 60)
        from benchmarks.benchmark_runner import BeamformingBenchmark
        benchmark = BeamformingBenchmark(config)
        results = benchmark.run_comprehensive_benchmark()
        
        print("\n" + "=" * 60)
        print("BENCHMARK COMPLETED SUCCESSFULLY!")
        print("=" * 60)

if __name__ == "__main__":
    main()
