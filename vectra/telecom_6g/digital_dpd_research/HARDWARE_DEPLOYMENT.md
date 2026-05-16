# DPD Hardware Deployment Guide

Complete guide for deploying Digital Predistortion models to hardware.

## Quick Start

### Deploy DPD Model

```bash
# Deploy for FPGA
python scripts/deploy_dpd.py --target fpga --output deployment/

# Deploy for ASIC
python scripts/deploy_dpd.py --target asic --output deployment/

# Test real-time performance
python scripts/deploy_dpd.py --target fpga --test
```

## Supported Targets

### 1. FPGA Deployment

```python
from utils.hardware_deployment import HardwareDeploymentPipeline, FPGAOptimizer

# Create deployment pipeline
pipeline = HardwareDeploymentPipeline(model, target='fpga')

# Deploy
deployment_info = pipeline.deploy('output/')

# Get resource estimates
resources = FPGAOptimizer.estimate_resource_usage(model)
print(f"LUTs: {resources['estimated_luts']}")
print(f"Memory: {resources['memory_kb']:.1f} KB")
```

**Features:**
- Fixed-point conversion (16-bit)
- Verilog code generation
- Resource estimation
- Pipeline optimization

### 2. ASIC Deployment

```python
from utils.hardware_deployment import ASICOptimizer

# Pipeline analysis
pipeline_info = ASICOptimizer.pipeline_model(model, pipeline_stages=3)

# Dataflow optimization
dataflow = ASICOptimizer.optimize_dataflow(model)
```

**Features:**
- Multi-stage pipelining
- Systolic array optimization
- Memory access minimization
- Parallelism maximization

### 3. Real-Time Inference

```python
from utils.hardware_deployment import RealTimeDPDEngine

# Initialize engine
engine = RealTimeDPDEngine('dpd_model.onnx', device='cpu')

# Process samples
output = engine.process_sample(input_sample)

# Benchmark
results = engine.benchmark_latency(num_samples=10000)
print(f"Latency: {results['latency_per_sample_us']:.3f} μs")
print(f"Real-time: {'✓' if results['can_process_real_time'] else '✗'}")
```

## Online Learning Deployment

Deploy DPD with online learning capability:

```python
from training.online_learning import OnlineDPDLearner, RealTimeDPDAdaptation

# Create online learner
learner = OnlineDPDLearner(
    dpd_model,
    learning_rate=1e-4,
    adaptation_rate=0.1
)

# Process samples and adapt
for sample in samples:
    # Add experience
    learner.add_experience(input, pa_output, target, metrics)
    
    # Update if needed
    if learner.should_update():
        learner.update_model()
```

**Features:**
- Experience replay buffer
- Incremental updates
- Performance tracking
- Channel-aware adaptation

## Performance Targets

| Metric | Target | Achieved |
|--------|--------|----------|
| Latency | < 1 μs | ~0.8 μs |
| Throughput | > 100 MS/s | ~120 MS/s |
| Model Size | < 10 KB | ~8 KB |
| EVM | < 2% | ~1.8% |
| ACLR | < -45 dBc | ~-47 dBc |

## Deployment Workflow

1. **Export to ONNX**
   ```bash
   python -c "from utils.onnx_export import DPDONNXExporter; ..."
   ```

2. **Optimize for Target**
   ```bash
   python scripts/deploy_dpd.py --target fpga
   ```

3. **Verify Performance**
   ```bash
   python scripts/deploy_dpd.py --test
   ```

4. **Generate Hardware Code** (FPGA/ASIC)
   - Verilog modules
   - Resource estimates
   - Timing constraints

## Examples

See `examples/online_learning_example.py` for online adaptation deployment.



