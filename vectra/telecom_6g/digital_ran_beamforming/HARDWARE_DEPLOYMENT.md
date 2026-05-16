# Hardware Deployment Guide

This guide covers deploying models to various hardware targets: FPGA, ASIC, ARM, and GPU.

## Quick Start

### Export Models for Deployment

```bash
# Export to ONNX
python scripts/export_for_deployment.py --target onnx --model all

# Export for FPGA
python scripts/export_for_deployment.py --target fpga --model all

# Export with adaptive quantization
python scripts/export_for_deployment.py --target onnx --adaptive-quant
```

## Supported Targets

### 1. ONNX (Universal)

Export models to ONNX format for deployment on any ONNX-compatible runtime:

```python
from utils.onnx_export import ONNXExporter

exporter = ONNXExporter(
    model,
    model_name='neural_csi_encoder',
    input_shape=(1, 64, 8, 2)
)

onnx_path = exporter.export('deployment/', quantize=True)
```

**Features:**
- INT8 quantization support
- Dynamic batch sizes
- Model verification
- Latency benchmarking

### 2. FPGA

Optimize models for FPGA deployment:

```python
from utils.onnx_export import HardwareOptimizer

# Optimize ONNX model
fpga_path = HardwareOptimizer.optimize_for_fpga(
    'model.onnx',
    'model_fpga.onnx'
)

# Generate Verilog wrapper
HardwareOptimizer.generate_verilog_wrapper(
    'model.onnx',
    'model_wrapper.v'
)
```

**Features:**
- Fixed-point conversion
- Verilog code generation
- Resource estimation
- Operation fusion

### 3. ARM Processors

Optimize for ARM mobile/edge devices:

```python
arm_path = HardwareOptimizer.optimize_for_arm(
    'model.onnx',
    'model_arm.onnx'
)
```

**Features:**
- ARM-specific operators
- Memory layout optimization
- Mobile-friendly quantization

### 4. Real-Time Inference

Deploy optimized inference engine:

```python
from utils.onnx_export import RealTimeInference

engine = RealTimeInference('model.onnx', device='cpu')
output = engine.infer(input_data)

# Benchmark latency
latency = engine.benchmark_latency(num_iterations=1000)
```

## Multi-User MIMO

Deploy multi-user beamforming:

```python
from utils.multi_user_mimo import MultiUserBeamformingPipeline

pipeline = MultiUserBeamformingPipeline(num_antennas=64, num_users=8)
output = pipeline.forward(H, method='mmse')
```

**Methods:**
- Zero-forcing: Simple, low complexity
- MMSE: Better performance, higher complexity
- Dirty Paper Coding: Optimal, highest complexity

## Adaptive Quantization

Deploy with adaptive quantization:

```python
from utils.adaptive_quantization import AdaptiveQuantizer

quantizer = AdaptiveQuantizer(
    initial_bits=8,
    min_bits=4,
    max_bits=16
)

quantized, metadata = quantizer(x, channel_snr=snr)
```

**Features:**
- Channel-aware precision adjustment
- Performance-based adaptation
- Mixed precision support

## Performance Targets

| Target | Latency | Throughput | Model Size |
|--------|---------|------------|------------|
| FPGA | < 10 μs | > 100 MS/s | < 1 MB |
| ASIC | < 5 μs | > 200 MS/s | < 500 KB |
| ARM | < 50 μs | > 20 MS/s | < 5 MB |
| GPU | < 1 μs | > 500 MS/s | < 10 MB |

## Deployment Checklist

- [ ] Export model to ONNX
- [ ] Quantize to INT8
- [ ] Verify model accuracy
- [ ] Benchmark latency
- [ ] Generate hardware-specific code (if FPGA/ASIC)
- [ ] Test on target hardware
- [ ] Optimize for target constraints

## Examples

See `examples/multi_user_example.py` for multi-user MIMO deployment.



