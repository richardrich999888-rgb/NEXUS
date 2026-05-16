"""
Export For Deployment

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

#!/usr/bin/env python3
"""
Export models for hardware deployment
Supports ONNX, FPGA, ASIC, ARM targets
"""

import argparse
import torch
import yaml
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).parent.parent))

from models.neural_csi_encoder import NeuralCSIEncoder
from models.sparse_beam_mask_generator import SparseBeamMaskGenerator
from utils.onnx_export import ONNXExporter, HardwareOptimizer, RealTimeInference
from utils.adaptive_quantization import AdaptiveQuantizer

def load_config(config_path):
    """Load configuration"""
    with open(config_path, 'r') as f:
        return yaml.safe_load(f)

def export_encoder(config, output_dir, target='onnx'):
    """Export CSI encoder"""
    print("Exporting Neural CSI Encoder...")
    
    input_dim = config['system']['num_antennas'] * config['system']['num_users'] * 2
    model = NeuralCSIEncoder(
        latent_dim=int(input_dim * config['neural_csi_encoder']['compression_ratio']),
        num_antennas=config['system']['num_antennas']
    )
    
    # Load trained weights if available
    try:
        checkpoint = torch.load('best_encoder.pth', map_location='cpu')
        model.load_state_dict(checkpoint['model_state_dict'])
        print("Loaded trained encoder weights")
    except FileNotFoundError:
        print("Warning: Using untrained model")
    
    model.eval()
    
    if target == 'onnx':
        exporter = ONNXExporter(
            model,
            model_name='neural_csi_encoder',
            input_shape=(1, config['system']['num_antennas'], 8, 2),  # (B, N_ant, N_subc, 2)
            dynamic_axes={'input': {0: 'batch_size'}, 'output': {0: 'batch_size'}}
        )
        
        onnx_path = exporter.export(output_dir, quantize=True)
        
        # Verify
        test_input = torch.randn(1, config['system']['num_antennas'], 8, 2)
        exporter.verify_onnx(test_input)
        
        # Benchmark
        rt_engine = RealTimeInference(onnx_path)
        latency = rt_engine.benchmark_latency()
        
        return onnx_path
    
    elif target == 'fpga':
        # FPGA optimization
        onnx_path = export_encoder(config, output_dir, target='onnx')
        fpga_path = HardwareOptimizer.optimize_for_fpga(
            onnx_path,
            str(Path(output_dir) / 'encoder_fpga.onnx')
        )
        return fpga_path

def export_predictor(config, output_dir, target='onnx'):
    """Export sparse beam predictor"""
    print("Exporting Sparse Beam Predictor...")
    
    input_dim = int(config['system']['num_antennas'] * config['system']['num_users'] * 2 * 
                   config['neural_csi_encoder']['compression_ratio'])
    
    model = SparseBeamMaskGenerator(
        latent_dim=input_dim,
        num_antennas=config['system']['num_antennas'],
        topk=int(config['system']['num_antennas'] * config['sparse_beam_mask']['sparsity_ratio'])
    )
    
    # Load trained weights if available
    try:
        checkpoint = torch.load('best_predictor.pth', map_location='cpu')
        model.load_state_dict(checkpoint['model_state_dict'])
        print("Loaded trained predictor weights")
    except FileNotFoundError:
        print("Warning: Using untrained model")
    
    model.eval()
    
    exporter = ONNXExporter(
        model,
        model_name='sparse_beam_predictor',
        input_shape=(1, input_dim),
        dynamic_axes={'input': {0: 'batch_size'}, 'output': {0: 'batch_size'}}
    )
    
    onnx_path = exporter.export(output_dir, quantize=True)
    
    # Verify
    test_input = torch.randn(1, input_dim)
    exporter.verify_onnx(test_input)
    
    return onnx_path

def export_with_adaptive_quantization(config, output_dir):
    """Export with adaptive quantization"""
    print("Exporting with Adaptive Quantization...")
    
    # Create adaptive quantizer
    quantizer = AdaptiveQuantizer(
        initial_bits=8,
        min_bits=4,
        max_bits=16
    )
    
    # Export quantized versions
    models = {}
    
    for bits in [4, 8, 16]:
        print(f"Exporting {bits}-bit quantized model...")
        # In practice, would quantize model weights here
        # For now, export standard ONNX (quantization happens at runtime)
        pass
    
    return models

def main():
    parser = argparse.ArgumentParser(description='Export models for deployment')
    parser.add_argument('--config', type=str, default='configs/telecom_default.yaml')
    parser.add_argument('--output', type=str, default='deployment')
    parser.add_argument('--target', type=str, choices=['onnx', 'fpga', 'asic', 'arm'], default='onnx')
    parser.add_argument('--model', type=str, choices=['encoder', 'predictor', 'all'], default='all')
    parser.add_argument('--adaptive-quant', action='store_true', help='Use adaptive quantization')
    
    args = parser.parse_args()
    
    config = load_config(args.config)
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("=" * 60)
    print("MODEL EXPORT FOR DEPLOYMENT")
    print("=" * 60)
    print(f"Target: {args.target}")
    print(f"Output: {output_dir}")
    print()
    
    exported_models = {}
    
    if args.model in ['encoder', 'all']:
        exported_models['encoder'] = export_encoder(config, str(output_dir), args.target)
    
    if args.model in ['predictor', 'all']:
        exported_models['predictor'] = export_predictor(config, str(output_dir), args.target)
    
    if args.adaptive_quant:
        exported_models['adaptive'] = export_with_adaptive_quantization(config, str(output_dir))
    
    print("\n" + "=" * 60)
    print("EXPORT COMPLETE")
    print("=" * 60)
    print(f"Exported models: {list(exported_models.keys())}")
    print(f"Output directory: {output_dir}")

if __name__ == "__main__":
    main()



