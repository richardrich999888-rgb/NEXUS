"""
Deploy Dpd

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Deploy DPD models for hardware
Supports FPGA, ASIC, ARM, GPU targets
"""

import argparse
import torch
import yaml
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent.parent))

from models.neural_dpd import BeamAwareDPD
from utils.hardware_deployment import HardwareDeploymentPipeline, RealTimeDPDEngine
from utils.onnx_export import DPDONNXExporter
from training.online_learning import OnlineDPDLearner

def load_config(config_path):
    """Load configuration"""
    with open(config_path, 'r') as f:
        return yaml.safe_load(f)

def deploy_dpd_model(config, output_dir, target='fpga'):
    """Deploy DPD model for hardware"""
    print(f"Deploying DPD model for {target.upper()}...")
    
    # Initialize model
    model = BeamAwareDPD(
        num_clusters=config['system']['num_clusters'],
        memory_depth=config['neural_dpd']['memory_depth'],
        hidden_dims=config['neural_dpd']['hidden_layers']
    )
    
    # Load trained weights if available
    try:
        checkpoint = torch.load('best_dpd_model.pth', map_location='cpu')
        model.load_state_dict(checkpoint['model_state_dict'])
        print("Loaded trained DPD weights")
    except FileNotFoundError:
        print("Warning: Using untrained model")
    
    model.eval()
    
    # Create deployment pipeline
    pipeline = HardwareDeploymentPipeline(model, target=target)
    
    # Deploy
    deployment_info = pipeline.deploy(output_dir)
    
    print(f"\nDeployment Summary:")
    print(f"  Model size: {deployment_info['metadata']['model_size_kb']:.1f} KB")
    print(f"  Target: {target}")
    print(f"  Artifacts: {list(deployment_info['artifacts'].keys())}")
    
    return deployment_info

def test_real_time_engine(onnx_path, num_samples=10000):
    """Test real-time inference engine"""
    print("\nTesting Real-Time Inference Engine...")
    
    engine = RealTimeDPDEngine(onnx_path, device='cpu', use_fixed_point=False)
    
    # Benchmark
    results = engine.benchmark_latency(num_samples)
    
    print(f"\nReal-Time Performance:")
    print(f"  Latency per sample: {results['latency_per_sample_us']:.3f} μs")
    print(f"  Throughput: {results['throughput_samples_per_sec']/1e6:.2f} MSamples/s")
    print(f"  Real-time factor: {results['real_time_factor']:.3f}")
    print(f"  Can process real-time: {'✓' if results['can_process_real_time'] else '✗'}")
    
    return results

def main():
    parser = argparse.ArgumentParser(description='Deploy DPD models')
    parser.add_argument('--config', type=str, default='configs/dpd_config.yaml')
    parser.add_argument('--output', type=str, default='deployment')
    parser.add_argument('--target', type=str, 
                       choices=['fpga', 'asic', 'arm', 'gpu'], 
                       default='fpga')
    parser.add_argument('--test', action='store_true', help='Test real-time engine')
    
    args = parser.parse_args()
    
    config = load_config(args.config)
    output_dir = Path(args.output)
    
    print("=" * 60)
    print("DPD MODEL DEPLOYMENT")
    print("=" * 60)
    
    # Deploy
    deployment_info = deploy_dpd_model(config, str(output_dir), args.target)
    
    # Test if requested
    if args.test and 'onnx_model' in deployment_info['artifacts']:
        test_real_time_engine(deployment_info['artifacts']['onnx_model'])
    
    print("\n" + "=" * 60)
    print("DEPLOYMENT COMPLETE")
    print("=" * 60)

if __name__ == "__main__":
    main()



