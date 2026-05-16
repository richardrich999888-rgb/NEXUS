# FYNTRAX

FYNTRAX is a **physics-first, entropy-optimized telecom control platform** that reduces absolute network energy, stabilizes AI-driven Open RAN control loops, and prepares networks for real-time economic optimization.

Unlike conventional RAN optimization tools, FYNTRAX treats the mobile network as:
- a **thermodynamic system** (energy ↔ entropy),
- a **control system** (stability under delay), and
- an **information engine** (useful bits per joule).

**Author / Inventor:** Katta Naga Sri Ganesh

## Core Innovations

1. **Receiver-Initiated Wake-Up RAN (RI-WuR)** - Base stations sleep by default
2. **Entropy-aware Idle-Mode Orchestration** - Decisions based on information demand
3. **Lyapunov-Stabilized AI Control for O-RAN** - Provable stability guarantees
4. **TFEC-compatible Entropy Compression Layer** - Minimize protocol overhead

## Installation

```bash
pip install -e .
```

## Quick Start

```bash
python scripts/run_simulation.py
```

## Repository Structure

```
fyntrax/
├── docs/                  # Technical documentation
│   ├── architecture.md
│   ├── physics_and_math.md
│   ├── control_theory.md
│   ├── energy_model.md
│   ├── patent_draft.md
│   └── regulatory_and_TEC.md
├── src/fyntrax/           # Source code
│   ├── models/            # Physics models
│   ├── ran/               # RAN control modules
│   ├── control/           # Control theory
│   ├── tfec/              # TFEC integration
│   ├── billing/           # SRv6 pricing stubs
│   └── simulator/         # Site simulation
├── tests/                 # Unit tests
└── scripts/               # Runnable scripts
```

## Status

- ✅ Fully digital / software-defined
- ✅ Buildable in Cursor / Python
- ✅ Designed for lab simulation, pilot deployment, and patent filing

## Disclaimer

This repository is a **reference architecture and simulation stack**, not a drop-in replacement for 3GPP gNB software.

## License

Proprietary - SYNTRIASS Labs
