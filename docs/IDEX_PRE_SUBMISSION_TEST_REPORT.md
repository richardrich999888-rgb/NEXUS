# iDEX Pre-Submission Test Report

Date: 2026-05-16

## Result

Status: PASS for the proposed software validation scope.

The pre-submission suite validates the execution guard, PQC activation path, RTOS/no_std core, TELOS commitment membrane, AGP governance, immune bridge, AGP-OS stability, resource control, ROS2 simulation adapter, and multi-agent governance ranking.

## Fixes Applied During Validation

- Stabilized weighted homeostasis convergence to prevent high-priority metrics from oscillating across setpoints.
- Penalized repeated governance failures as compliance violations so low-performing agents are ranked down before formal blocks.
- Made AGP governance scripts fail with non-zero exit status when internal checks fail.
- Made AIS behavior encoding adapt to base model output dimensions instead of assuming 512 hidden units.
- Added batched adaptive-immune handling by deriving a representative threat signature while preserving batch neutralization.
- Repaired unified immune integration against current AIS and governance bridge APIs.
- Restored AGP-OS syscall compatibility through `handle(...)`, async fork handling, and page-based MALLOC accounting.
- Fixed reputation-engine negative hormone deltas and reduced success-induced cortisol accumulation.
- Converted the multi-agent governance simulation into a deterministic gate that fails if expected high performers are not ranked high or the malicious actor is not ranked low.

## Test Matrix

| Area | Command | Result |
| --- | --- | --- |
| Execution guard red team | `cargo test -p nexus-executor --test red_team_execution -- --nocapture` | PASS, 10/10 |
| Homeostasis engine | `cargo test -p homeostasis-engine -- --nocapture` | PASS, 57/57 |
| Multi-ASI immune | `cargo test -p multi-asi-immune -- --nocapture` | PASS, 68/68, 1 ignored doctest |
| TELOS protocol | `cargo test -p telos-protocol --lib -- --nocapture` | PASS, 50/50 |
| PQC path | `cargo test -p nexus-pcu --features pqc pqc -- --nocapture` | PASS, 7/7 |
| RTOS core | `cargo test -p nexus-rtos-core -- --nocapture` | PASS, 4/4 |
| RTOS no_std target | `cargo check -p nexus-rtos-core --target wasm32-unknown-unknown` | PASS |
| TELOS membrane Python | `agp-core/.venv/bin/python agp-core/tests/test_telos.py` | PASS, 15/15 |
| TELOS gate | `agp-core/.venv/bin/python agp-core/tests/test_telos_gate.py` | PASS |
| AGP RT scheduler | `agp-core/.venv/bin/python agp-core/tests/test_rtos.py` | PASS, 8/8 |
| ROS2/Gazebo bridge | `agp-core/.venv/bin/python agp-core/tests/test_ros2.py` | PASS, 16/16 |
| Resource controller | `agp-core/.venv/bin/python agp-core/tests/test_resources.py` | PASS, 12/12 |
| Mesh coordination | `agp-core/.venv/bin/python agp-core/tests/test_mesh.py` | PASS, 13/13 |
| AGP governance | `agp-core/.venv/bin/python agp-core/tests/test_governance.py` | PASS, 13/13 |
| AHES | `agp-core/.venv/bin/python agp-core/tests/test_ahes.py` | PASS, 20/20 |
| Governance-immune bridge | `agp-core/.venv/bin/python agp-core/tests/test_immune_bridge.py` | PASS, 19/19 |
| Complete AGP-OS | `agp-core/.venv/bin/python agp-core/tests/test_complete_os.py` | PASS, 24/24 |
| Production ROS2 adapter | `agp-core/.venv/bin/python agp-core/tests/test_production.py` | PASS, 22/22 |
| AGP-OS stability/load | `agp-core/.venv/bin/python agp-core/tests/test_agp_os.py` | PASS, no syscall exceptions in rerun log |
| AIS immune system | `PYTHONPATH=. .venv/bin/python tests/immunity/test_immune_system.py` | PASS, 44/44 |
| Unified immune system | `PYTHONPATH=. .venv/bin/python tests/immunity/test_unified_immune.py` | PASS, 10/10 |
| Reputation engine | `PYTHONPATH=. .venv/bin/python tests/test_reputation_engine.py` | PASS |
| Real environment integration | `PYTHONPATH=. .venv/bin/python tests/test_real_environment.py` | PASS, 10/10 |
| Multi-agent governance | `agp-core/.venv/bin/python agp-core/tests/test_multi_agent_governance.py` | PASS; top 3 includes 2 expected high performers and Iota ranks 12/12 |

## Submission Caveats

- ROS2 production testing ran in simulation mode because `rclpy` is not installed in this environment.
- RTOS bare-metal readiness was validated through the `no_std`-safe crate and `wasm32-unknown-unknown` target check, not on a physical MCU board.
- PQC was validated at the hybrid-signature unit level; full network-wide PQC enforcement still needs integration testing.
- Multi-agent governance ranking now gates correctly, but anomaly and health calibration still need tuning before a live defence demo because several simulated agents are flagged anomalous or critical under stress-heavy workloads.

## Readiness Call

The repository is ready for a software-evidence iDEX submission package. For a hardware/live demo claim, add board flashing evidence, ROS2 with real `rclpy`, and a network-level PQC integration run.
