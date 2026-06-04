# Evidence - AGP-OS Robotics Safety Layer

## Relevant Code Modules

- `agp-core`
- RTOS scheduling and HAL safety modules
- ROS2 bridge and simulation tests
- Resource controller and production-mode tests

## Existing Test Evidence

Shared report: [IDEX_PRE_SUBMISSION_TEST_REPORT.md](../../IDEX_PRE_SUBMISSION_TEST_REPORT.md)

Relevant commands and test files:

```bash
pytest agp-core/tests/test_rtos.py
pytest agp-core/tests/test_ros2.py
pytest agp-core/tests/test_resources.py
pytest agp-core/tests/test_production.py
cargo test -p nexus-rtos-core -- --nocapture
cargo check -p nexus-rtos-core --target wasm32-unknown-unknown
```

Recorded pre-submission evidence includes RTOS core tests, WASM target check, AGP production-mode checks, and ROS2 simulation-oriented tests.

## Demo Script

1. Start ROS2/Gazebo-style simulation mode.
2. Submit a governed robot command through the ROS2 bridge.
3. Show AGP policy admission for an allowed command.
4. Submit a command exceeding resource limits and show denial.
5. Trigger RTOS priority scheduling behavior for safety-critical control.
6. Trigger simulated HAL safety interlock refusal.
7. Export audit trace for command decisions and resource state.

## TRL Caveats

- ROS2 evidence is simulation oriented.
- RTOS tests do not prove physical board timing guarantees.
- HAL safety interlocks require board-specific integration under the iDEX effort.
- Hardware-in-loop validation is planned, not claimed as completed.
