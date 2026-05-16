# FYNTRAX Architecture

## Overview

FYNTRAX is a physics-first, entropy-optimized telecom control platform.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    External Interfaces                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   O-RAN RIC  │  │  TFEC Engine │  │   Billing    │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼─────────────────┼─────────────────┼──────────┘
          │                 │                 │
┌─────────▼─────────────────▼─────────────────▼──────────┐
│                    Control Layer                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │          Lyapunov Supervisor                     │   │
│  │  • Safety constraint: V(x_{t+1}) < V(x_t)       │   │
│  │  • AI action filtering                          │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────┬──────────────────────────────┘
                          │
┌─────────────────────────▼──────────────────────────────┐
│                      RAN Layer                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────────┐   │
│  │  Wake-Up   │  │   Idle     │  │  SSB Scheduler │   │
│  │  Receiver  │  │ Orchestr.  │  │  (Targeted)    │   │
│  └────────────┘  └────────────┘  └────────────────┘   │
│  ┌────────────────────────────────────────────────┐   │
│  │           Handover Controller                   │   │
│  │  • Zero-RACH predictive handover               │   │
│  │  • Context teleportation                        │   │
│  └────────────────────────────────────────────────┘   │
└─────────────────────────┬──────────────────────────────┘
                          │
┌─────────────────────────▼──────────────────────────────┐
│                   Models Layer                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  Energy  │  │ Entropy  │  │ Channel  │  │Traffic │ │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
└────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Wake-Up Receiver (WuR)
- Ultra-low-power (~1 μW) always-on listener
- Triggers main radio activation on demand
- Enables zero-watt idle state

### 2. Idle Mode Orchestrator
- Entropy-based state decisions
- States: DEEP_SLEEP, LIGHT_SLEEP, ACTIVE
- Hysteresis to prevent oscillation

### 3. Lyapunov Supervisor
- Guarantees BIBO stability
- Filters AI/ML control actions
- Provable safety constraints

### 4. TFEC Integration
- Protocol entropy compression
- Reduces signaling overhead

## Data Flow

1. **Reception**: WuR detects wake-up signal
2. **Decision**: Orchestrator selects power state
3. **Validation**: Lyapunov supervisor checks safety
4. **Execution**: RAN state transition
5. **Optimization**: Continuous feedback loop

## Deployment Model

- **Simulation**: Python-based site simulator
- **Pilot**: xApp on O-RAN RIC
- **Production**: Native integration with gNB
