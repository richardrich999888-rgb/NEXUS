# Digital Predistortion (DPD) for Massive MIMO

A complete research framework for Machine Learning-based Digital Predistortion in 6G massive MIMO systems. This repository implements joint beamforming and DPD optimization for power amplifier linearization.

## 🚀 Key Features

- **Neural Network DPD**: RVTDNN2L architecture for power amplifier linearization
- **Beam-Aware DPD**: Shared coefficients across antenna clusters conditioned on beam patterns
- **Joint Optimization**: Simultaneous beamforming and DPD optimization
- **PA Behavioral Models**: Rapp, Saleh, and Ghorbani models with array variations
- **Quantization**: INT8/INT4 model compression for deployment
- **Complete Simulation**: End-to-end simulation with performance metrics

## 📊 Performance Targets

| Metric | Without DPD | With DPD | Improvement |
|--------|-------------|----------|-------------|
| EVM | 3.5-4.5% | 1.5-2.5% | 40-60% |
| ACLR | -35 to -40 dBc | -45 to -50 dBc | 5-10 dB |
| PA Efficiency | 20-30% | 50-65% | 2-3x |
| Model Size | N/A | 5-10 KB | Deployable |

## 🏗️ Architecture

```
Baseband Signal → [Beamforming] → [Neural DPD] → PA Array → Antenna
↑               ↑              ↑
[CSI Compression] [Beam Weights] [Beam Conditioning]
```

### Core Innovations:
1. **Beam-aware DPD coefficients** conditioned on beamforming weights
2. **Shared DPD models** across antenna clusters (8:1 compression)
3. **Joint optimization** of beamforming and linearization
4. **Quantized deployment** for embedded implementation

## 🛠️ Installation

```bash
git clone https://github.com/your-org/digital-dpd-research
cd digital-dpd-research
pip install -r requirements.txt
```

## 🏃 Quick Start

### Run Complete Demo

```bash
python demo_joint_optimization.py
```

### Run Simulation

```bash
python run_dpd_experiment.py --simulate
```

### Train Model

```bash
python run_dpd_experiment.py --train
```

### Run Tests

```bash
python run_dpd_experiment.py --test
```

## 📁 Project Structure

```
digital_dpd_research/
├── configs/              # Configuration files
├── models/               # Neural DPD and PA models
├── beamformers/          # Tensor-train beamformer
├── utils/                # Metrics, quantization, signal generation
├── training/             # Training pipelines
├── simulation/           # Complete simulation environment
├── tests/                # Unit tests
└── run_dpd_experiment.py # Main execution script
```

## 🔬 Technical Details

### Neural DPD Architecture

```python
class NeuralDPD(nn.Module):
    """Real-Valued Time-Delay Neural Network (RVTDNN2L)"""
    def __init__(self, memory_depth=5, hidden_dims=[64, 64]):
        # Time-delay memory taps
        # 2 hidden layers with residual connections
        # Linear output for I/Q predistortion
```

### Beam-Aware Conditioning

```python
class BeamAwareDPD(nn.Module):
    """DPD with beam pattern conditioning"""
    def forward(self, x, beam_weights):
        # Encode beam pattern to cluster assignments
        # Apply cluster-specific DPD
        # Cache coefficients for fast inference
```

### Joint Optimization

```python
class JointBeamformingDPD(nn.Module):
    """Joint beamforming and DPD"""
    def forward(self, channel_state, data_symbols):
        # Generate beam weights from CSI
        # Condition DPD on beam pattern
        # Optimize both simultaneously
```

## 📈 Performance Metrics

The system calculates:

- **EVM (Error Vector Magnitude)**: Signal quality metric
- **ACLR (Adjacent Channel Leakage Ratio)**: Spectral regrowth
- **NMSE (Normalized Mean Square Error)**: Distortion measure
- **PA Efficiency**: DC-RF conversion efficiency
- **Model Size**: Deployment feasibility (KB)

## 🎯 Use Cases

- **6G Massive MIMO**: Linearize 64+ antenna arrays
- **mmWave/THz Systems**: Compensate PA nonlinearity at high frequencies
- **O-RAN Integration**: Deploy as xApp for real-time linearization
- **ASIC/FPGA Implementation**: Quantized models for hardware deployment
- **Research & Development**: PA behavioral modeling and linearization

## 📄 License

MIT License - See LICENSE file for details

## 🤝 Citation

If you use this code in your research, please cite:

```bibtex
@software{digital_dpd_2024,
  title={Digital Predistortion for Massive MIMO},
  author={Telecom Research Lab},
  year={2024},
  url={https://github.com/your-org/digital-dpd-research}
}
```

## 📚 References

1. 3GPP TR 38.803: Power Amplifier Requirements
2. "Neural Network DPD for 5G NR" (IEEE TMTT 2023)
3. "Beam-Space DPD for Massive MIMO" (IEEE VTC 2022)
4. "Joint Beamforming and Linearization" (IEEE SPAWC 2023)

