"""
Benchmark Runner

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited
"""

import torch
import time
import numpy as np
import json
from pathlib import Path
import sys
sys.path.append('..')

from models.neural_csi_encoder import NeuralCSIEncoder
from models.sparse_beam_mask_generator import SparseBeamMaskGenerator
from beamformers.baseline_svd import SVDBaseline
from beamformers.tt_beamformer import TTBeamformer
from utils.threegpp_channel_simulator import ThreeGPPChannelSimulator

class DigitalRANPipeline:
    """Complete digital RAN pipeline integrating all components"""
    
    def __init__(self, config):
        self.config = config
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        
        # Initialize components
        input_dim = config['system']['num_antennas'] * config['system']['num_users'] * 2
        self.encoder = NeuralCSIEncoder(
            latent_dim=int(input_dim * config['neural_csi_encoder']['compression_ratio']),
            num_antennas=config['system']['num_antennas']
        ).to(self.device)
        
        self.predictor = SparseBeamMaskGenerator(
            latent_dim=int(input_dim * config['neural_csi_encoder']['compression_ratio']),
            num_antennas=config['system']['num_antennas'],
            topk=int(config['system']['num_antennas'] * config['sparse_beam_mask']['sparsity_ratio'])
        ).to(self.device)
        
        self.beamformer = TTBeamformer(
            num_ant=config['system']['num_antennas'],
            num_users=config['system']['num_users']
        ).to(self.device)
        
        # Load trained models
        self.load_models()
        
    def load_models(self):
        """Load trained model weights"""
        try:
            encoder_checkpoint = torch.load('best_encoder.pth', map_location=self.device)
            self.encoder.load_state_dict(encoder_checkpoint['model_state_dict'])
            print("Loaded trained encoder")
        except FileNotFoundError:
            print("Warning: Using untrained encoder")
        
        try:
            predictor_checkpoint = torch.load('best_predictor.pth', map_location=self.device)
            self.predictor.load_state_dict(predictor_checkpoint['model_state_dict'])
            print("Loaded trained predictor")
        except FileNotFoundError:
            print("Warning: Using untrained predictor")
    
    def forward(self, H):
        """Complete forward pass"""
        with torch.no_grad():
            # Neural compression
            compressed = self.encoder.compress(H)
            
            # Sparse beam prediction
            beam_mask, _ = self.predictor(compressed, hard=True)
            
            # TT beamforming
            weights = self.beamformer.compute_beamweights(H, beam_mask)
            
        return weights, beam_mask

class BeamformingBenchmark:
    """Comprehensive benchmark suite"""
    
    def __init__(self, config):
        self.config = config
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        print(f"Benchmark running on: {self.device}")
        
        # Initialize components
        self.simulator = ThreeGPPChannelSimulator(
            num_antennas=config['system']['num_antennas'],
            num_users=config['system']['num_users'],
            scenario=config['system']['scenario'],
            carrier_freq=config['system']['carrier_freq']
        )
        
        self.baseline = SVDBaseline(
            num_antennas=config['system']['num_antennas'],
            num_users=config['system']['num_users']
        )
        
        self.pipeline = DigitalRANPipeline(config)
        
        self.results = {}
        
    def benchmark_svd_baseline(self, H, num_iterations=1000):
        """Benchmark SVD baseline"""
        print("Benchmarking SVD Baseline...")
        
        # Warmup
        _ = self.baseline.compute_beamweights(H[:10])
        
        # Timing
        start_time = time.time()
        for _ in range(num_iterations):
            weights = self.baseline.compute_beamweights(H)
        end_time = time.time()
        
        avg_latency = (end_time - start_time) * 1000 / num_iterations
        
        # Performance
        gains = []
        for i in range(H.shape[0]):
            weight = self.baseline.compute_beamweights(H[i:i+1])  # (1, Nt)
            # Proper gain calculation for MIMO: w^H * H * w using einsum
            # H[i] is (Nr, Nt), weight is (1, Nt)
            gain = torch.abs(torch.einsum('j,ij,j->', weight[0].conj(), H[i], weight[0]))
            gains.append(gain.item())
        
        results = {
            'latency_ms': avg_latency,
            'beamforming_gain': np.mean(gains),
            'complexity': 'O(min(Nr,Nt)^3)'
        }
        
        self.results['svd_baseline'] = results
        return results
    
    def benchmark_neural_pipeline(self, H, num_iterations=1000):
        """Benchmark neural pipeline"""
        print("Benchmarking Neural Pipeline...")
        
        # Warmup
        _, _ = self.pipeline.forward(H[:10])
        
        # Timing
        start_time = time.time()
        for _ in range(num_iterations):
            weights, beam_mask = self.pipeline.forward(H)
        end_time = time.time()
        
        avg_latency = (end_time - start_time) * 1000 / num_iterations
        
        # Performance metrics
        sparsity = 1.0 - beam_mask.float().mean().item()
        
        results = {
            'latency_ms': avg_latency,
            'sparsity_achieved': sparsity,
            'target_sparsity': self.config['sparse_beam_mask']['sparsity_ratio']
        }
        
        self.results['neural_pipeline'] = results
        return results
    
    def run_comprehensive_benchmark(self, batch_size=32, num_iterations=500):
        """Run comprehensive benchmark suite"""
        print("=" * 60)
        print("Running Comprehensive Beamforming Benchmark")
        print("=" * 60)
        
        # Generate test channels
        H = self.simulator.generate_cdl_channel(batch_size=batch_size)
        H = H.to(self.device)
        print(f"Test channels: {H.shape}")
        
        # Run benchmarks
        svd_results = self.benchmark_svd_baseline(H, num_iterations)
        neural_results = self.benchmark_neural_pipeline(H, num_iterations)
        
        # Calculate improvements
        latency_improvement = (svd_results['latency_ms'] - neural_results['latency_ms']) / svd_results['latency_ms'] * 100
        sparsity_error = abs(neural_results['sparsity_achieved'] - neural_results['target_sparsity']) / neural_results['target_sparsity'] * 100
        
        summary = {
            'latency_improvement_percent': latency_improvement,
            'sparsity_achieved_percent': neural_results['sparsity_achieved'] * 100,
            'sparsity_error_percent': sparsity_error,
        }
        
        self.results['summary'] = summary
        
        # Print results
        self.print_results()
        
        # Save results
        self.save_results()
        
        return self.results
    
    def print_results(self):
        """Print benchmark results"""
        print("\n" + "=" * 60)
        print("BENCHMARK RESULTS")
        print("=" * 60)
        
        svd = self.results['svd_baseline']
        neural = self.results['neural_pipeline']
        summary = self.results['summary']
        
        print(f"\nSVD Baseline:")
        print(f"  Latency: {svd['latency_ms']:.3f} ms")
        print(f"  Beamforming Gain: {svd['beamforming_gain']:.4f}")
        
        print(f"\nNeural Pipeline:")
        print(f"  Latency: {neural['latency_ms']:.3f} ms")
        print(f"  Sparsity Achieved: {neural['sparsity_achieved']:.3f}")
        
        print(f"\nPerformance Summary:")
        print(f"  Latency Improvement: {summary['latency_improvement_percent']:.1f}%")
        print(f"  Active Beams: {100 - summary['sparsity_achieved_percent']:.1f}%")
        print(f"  Sparsity Error: {summary['sparsity_error_percent']:.1f}%")
        
        # Check if targets are met
        print(f"\nTarget Verification:")
        sparsity_target = self.config['benchmark']['sparsity_tolerance'] * 100
        
        sparsity_ok = summary['sparsity_error_percent'] <= sparsity_target
        
        print(f"  Sparsity Error < {sparsity_target}%: {'✓' if sparsity_ok else '✗'} ({summary['sparsity_error_percent']:.1f}%)")
        print(f"  Overall: {'PASS' if sparsity_ok else 'FAIL'}")
    
    def save_results(self, filename="benchmark_results.json"):
        """Save results to JSON file"""
        results_serializable = {}
        for key, value in self.results.items():
            if isinstance(value, dict):
                results_serializable[key] = {
                    k: (v.item() if torch.is_tensor(v) else v)
                    for k, v in value.items()
                }
            else:
                results_serializable[key] = value.item() if torch.is_tensor(value) else value
        
        with open(filename, 'w') as f:
            json.dump(results_serializable, f, indent=2)
        
        print(f"\nResults saved to {filename}")

