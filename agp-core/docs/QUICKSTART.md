# AGP-OS Quick-Start Deployment Guide

## Overview

AGP-OS is a governed operating system for AI agents and robots. This guide covers deployment for development, simulation, and production.

## Prerequisites

- Python 3.10+
- Docker (for containerized deployment)
- ROS2 Humble (for real robot deployment)

## Quick Start

### 1. Development Setup

```bash
# Clone and setup
cd /path/to/NEXUS/agp-core
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Run tests
pytest tests/ -v

# Start API server
uvicorn src.main:app --reload
```

### 2. Run Benchmarks

```bash
# Full benchmark suite
python benchmarks/full_benchmark.py

# Individual test suites
python tests/sim_robot.py      # HAL tests
python tests/test_mesh.py      # Mesh coordination
python tests/test_rtos.py      # Real-time scheduler
python tests/test_ros2.py      # ROS2 bridge
```

### 3. Docker Deployment

```bash
# Build standard image
docker build -t agp-os .

# Build ROS2 robot image
docker build -f deploy/Dockerfile.ros2 -t agp-os-robot .

# Run API server
docker run -p 8000:8000 agp-os

# Run robot controller (with host networking for ROS2)
docker run --network host --privileged agp-os-robot
```

### 4. Production Robot Deployment

```bash
# Copy to robot
scp -r agp-core robot@192.168.1.100:/opt/agp-os

# Install systemd service
ssh robot@192.168.1.100
sudo cp /opt/agp-os/deploy/agp-os-robot.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable agp-os-robot
sudo systemctl start agp-os-robot

# Check status
sudo systemctl status agp-os-robot
sudo journalctl -u agp-os-robot -f
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      AGP-OS Stack                           │
├─────────────────────────────────────────────────────────────┤
│  API Layer     │ FastAPI REST endpoints                     │
├────────────────┼────────────────────────────────────────────┤
│  Governance    │ Behavioral RAG, Rules, Alignment, Enforcer │
├────────────────┼────────────────────────────────────────────┤
│  Coordination  │ Mesh (Mailbox, Consensus), RTOS Scheduler  │
├────────────────┼────────────────────────────────────────────┤
│  Hardware      │ HAL (Sensors, Actuators, Safety Interlocks)│
├────────────────┼────────────────────────────────────────────┤
│  ROS2 Bridge   │ Topics, Robots, Production Adapter         │
├────────────────┼────────────────────────────────────────────┤
│  Resources     │ CPU, Memory, Token Quotas                  │
└─────────────────────────────────────────────────────────────┘
```

## Key Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /health` | System health check |
| `GET /api/v1/governance/stats` | Governance statistics |
| `GET /api/v1/governance/agents` | Agent alignment leaderboard |
| `POST /api/v1/governance/escalations/{id}/action` | Human review |

## Safety Features

- **Safety Interlocks**: Agents with alignment < 0.4 blocked from actuators
- **Watchdog**: 500ms heartbeat timeout triggers emergency stop
- **Velocity Capping**: Max 1.0 m/s linear, 1.0 rad/s angular
- **Human Escalation**: Critical actions require human approval

## Support

- Tests: 120+ verified across 8 layers
- Benchmarks: Sub-millisecond latency for core operations
