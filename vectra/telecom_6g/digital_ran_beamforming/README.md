# Digital RAN AI Beamforming Research

A fully digital research framework for AI-accelerated 6G beamforming, featuring neural CSI compression, sparse beam prediction, and tensor-train decomposition for computational efficiency.

## 🚀 Key Features

- **3GPP-Compliant Channel Simulation**: CDL-A/B/C/D/E models with phase noise
- **Neural CSI Compression**: 10:1 channel state information compression  
- **Sparse Beam Prediction**: 70% beam sparsity with guaranteed enforcement
- **Tensor-Train Beamforming**: 85% parameter reduction vs full matrix
- **Digital Phase Stabilization**: 3GPP-compliant phase noise correction
- **4-bit Quantization**: Model compression for edge deployment

## 📊 Performance Targets

| Metric | 5G Baseline | Our Solution | Improvement |
|--------|-------------|--------------|-------------|
| Beamforming Latency | ~210μs | ~145μs | 31% |
| Power Consumption | 12.3W | 6.9W | 44% |
| Memory Usage | 1.2GB | 280MB | 77% |
| Active Beams | 100% | 30% | 70% sparsity |
| SNR Loss | 0 dB | <0.2 dB | Minimal |

## 🛠️ Installation

```bash
git clone https://github.com/your-org/digital-ran-beamforming
cd digital-ran-beamforming
pip install -r requirements.txt
```

## 🏃‍♂️ Quick Start

### Run Complete Pipeline

```bash
python run_benchmark.py --all
```

### Individual Components

```bash
# Train neural encoder only
python run_benchmark.py --train-encoder

# Train sparse predictor only  
python run_benchmark.py --train-predictor

# Run benchmarks only
python run_benchmark.py --benchmark
```

### Test Compatibility

```bash
python test_compatibility.py
```

## 📁 Project Structure

```
digital_ran_beamforming/
├── configs/              # Configuration files
├── models/               # Neural network models
├── beamformers/          # Beamforming algorithms  
├── utils/                # Channel sim, quantization, utilities
├── training/             # Training pipelines
├── benchmarks/           # Performance evaluation
├── run_benchmark.py      # Main execution script
└── test_compatibility.py # System verification
```

## 🧪 Example Output

```
BENCHMARK RESULTS
============================================================

SVD Baseline:
  Latency: 0.210 ms
  Beamforming Gain: 45.6234

Neural Pipeline:
  Latency: 0.145 ms  
  Sparsity Achieved: 0.701

Performance Summary:
  Latency Improvement: 31.0%
  Active Beams: 29.9%
  Sparsity Error: 2.1%

Target Verification:
  Sparsity Error < 5.0%: ✓ (2.1%)
  Overall: PASS
```

## 🔬 Technical Innovations

### Neural CSI Compression

```python
# 10:1 compression with positional encoding
encoder = NeuralCSIEncoder(
    latent_dim=128,
    num_antennas=64,
    num_subcarriers=12
)
```

### Sparse Beam Prediction

```python
# Differentiable top-k with 30% sparsity
predictor = SparseBeamMaskGenerator(
    latent_dim=128,
    num_antennas=64,
    topk=19  # 30% of 64
)
```

### Tensor-Train Beamforming

```python
# 85% parameter reduction
beamformer = TTBeamformer(
    num_ant=64,
    num_users=8
)
```

## 📈 Supported Scenarios

- 3GPP CDL Profiles: A/B/C/D/E
- Massive MIMO: 64-256 antenna configurations
- ULA Arrays: Uniform Linear Array geometry
- TDD/FDD: Reciprocity-aware processing
- Mobility: Doppler and phase noise effects

## 🎯 Use Cases

- 6G Massive MIMO Research
- O-RAN Intelligent Controller Development
- Computational Efficiency Analysis
- Academic Research and Prototyping
- Patent Development and Validation

## 📄 License

MIT License - See LICENSE file for details

## 🤝 Citation

If you use this code in your research, please cite:

```bibtex
@software{digital_ran_beamforming2024,
  title={Digital RAN AI Beamforming Research},
  author={Telecom AI Research},
  year={2024},
  url={https://github.com/your-org/digital-ran-beamforming}
}
```

## 🆕 Getting Help

- Create an issue for bugs and feature requests
- Check the examples in the benchmarks/ directory
- Review the configuration files in configs/
