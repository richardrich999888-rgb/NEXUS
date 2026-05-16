# AIS-ASI: Artificial Immune System for ASI Safety

<p align="center">
  <strong>Bio-inspired multi-layered defense for AI alignment</strong>
</p>

---

## Overview

AIS-ASI implements an **Artificial Immune System** for AI safety, inspired by the biological immune system. It provides continuous, adaptive protection against alignment failures, novel threats, and behavioral drift.

### Key Features

| Feature | Description |
|---------|-------------|
| **Multi-layered Defense** | Innate (fast) + Adaptive (learned) + Memory (recall) |
| **Self-Tolerance** | Negative selection prevents false positives on aligned behavior |
| **Clonal Selection** | Effective antibodies are amplified evolutionarily |
| **Memory Recall** | 10x faster response on previously-seen threats |
| **AHES Integration** | Connects with endocrine system for holistic bio-inspired safety |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Input Behavior                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Layer 1: Innate Immunity                    │
│  • Pattern detectors (8 threat types)                       │
│  • Fast response (<1ms)                                      │
│  • Immediate neutralization for severe threats              │
└─────────────────────────────────────────────────────────────┘
                              │ alerts
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Layer 2: Adaptive Immunity                   │
│  • T-cell coordination (Helper, Killer, Regulatory)         │
│  • Antibody selection and generation                         │
│  • Clonal selection (amplify effective responses)           │
└─────────────────────────────────────────────────────────────┘
                              │ forms
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Layer 3: Immune Memory                    │
│  • Store successful antibodies                               │
│  • Rapid recall on re-exposure                               │
│  • Memory consolidation and forgetting                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Installation

```bash
cd agp-core
pip install -r requirements_immunity.txt
```

### Basic Usage

```python
from src.immunity import ArtificialImmuneSystem, ImmuneConfig

# Create immune-protected model
config = ImmuneConfig(behavior_dim=512)
ais = ArtificialImmuneSystem(your_model, config)

# Train self-tolerance (CRITICAL: do this first!)
ais.train_self_tolerance(aligned_examples)

# Vaccinate against known threats
ais.vaccination(known_threats)

# Protected inference
output, diagnostics = ais(input_data, return_diagnostics=True)

if diagnostics['threat_detected']:
    print(f"⚠️ Threat: {diagnostics['threat_type']}")
```

---

## Training Protocols

### Phase 1: Negative Selection (Self-Tolerance)

```python
from src.immunity.training import NegativeSelectionTrainer

trainer = NegativeSelectionTrainer(ais)
trainer.train(aligned_dataset, max_fp_rate=0.01)
```

### Phase 2: Vaccination

```python
from src.immunity.training import VaccinationProtocol

protocol = VaccinationProtocol(ais)
protocol.vaccinate(known_threats, verify=True)
```

### Phase 3: Live Training

```python
from src.immunity.training import LiveTrainingProtocol

live = LiveTrainingProtocol(ais)
metrics = live.train(mixed_dataset, num_epochs=10)
```

---

## Evaluation

### Run Benchmarks

```bash
cd examples/immunity
python run_benchmarks.py
```

### Benchmarks

| Benchmark | Target | Description |
|-----------|--------|-------------|
| Self-Tolerance | FPR < 1% | No autoimmune reactions |
| Threat Detection | TPR > 90% | Detect known threats |
| Memory Speed | 10x speedup | Faster recall response |
| Clonal Selection | +20% fitness | Evolution of antibodies |
| Adversarial | >80% detection | Robustness to perturbations |

### Run Experiments

```bash
python run_experiments.py
```

---

## Module Structure

```
src/immunity/
├── __init__.py           # Module exports
├── antibody.py           # Antibody + AntibodyPool
├── tcell.py              # T-cell types and populations
├── memory.py             # MemoryCell + MemoryBank
├── innate.py             # InnateImmuneSystem
├── adaptive.py           # AdaptiveImmuneSystem
├── immune_system.py      # Complete AIS
├── integration.py        # AHES integration
├── evaluation/           # Metrics and benchmarks
├── experiments/          # Validation experiments
└── training/             # Training protocols
```

---

## Patent Claims

This implementation supports patent claims **7.1-7.5**:

- **7.1**: Multi-layered artificial immune architecture
- **7.2**: Negative selection for self-tolerance
- **7.3**: Clonal selection for adaptive amplification
- **7.4**: Immune memory for rapid recall
- **7.5**: Vaccination protocol for pre-training

---

## Citation

```bibtex
@software{ais_asi_2026,
  title={AIS-ASI: Artificial Immune System for ASI Safety},
  author={NEXUS Research Team},
  year={2026},
  url={https://github.com/nexus/agp-core}
}
```

---

## License

MIT License - See LICENSE file for details.
