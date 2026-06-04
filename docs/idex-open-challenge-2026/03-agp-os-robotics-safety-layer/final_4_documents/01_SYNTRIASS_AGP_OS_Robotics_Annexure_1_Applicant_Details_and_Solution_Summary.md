# IDEX OPEN CHALLENGE SUBMISSION

# Annexure Outline

Company identification and section outline

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

## Company Identification

| Field | Details |
| --- | --- |
| Legal Entity Name | Syntriass Labs Private Limited |
| CIN | U62011AP2025PTC120239 |
| PAN | ABQCS7152R |
| TAN | VPNS31351F |
| Registered Office | 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India |
| Contact Email | kattanaga5555@gmail.com |
| Contact Phone | +91 88864 68060 |
| Submission Date | 17 May 2026 |

## Annexure-1 Outline

Purpose: applicant details and proposed solution summary for the iDEX Open Challenge.

Contents:

- Company and applicant details.
- Challenge title.
- Intended defence end-user profile.
- Brief solution summary under 250 words.
- Key technologies used.
- Deliverables table.
- Phase-wise 12-month timeline.

## Annexure-2 Outline

Purpose: technical architecture and implementation approach for AGP-OS Robotics Safety Layer.

Contents:

- ROS2 command bridge.
- AGP governance admission path.
- RTOS priority scheduling.
- Resource-denial controller.
- HAL safety interlock.
- Production adapter and deployment artifacts.

## Annexure-3 Outline

Purpose: advantages, product value, commercial value, and competencies.

## Annexure-4 Outline

Purpose: supporting evidence, screenshots, test output, repository locations, artifact locations, and readiness caveats.

```{=typst}
#pagebreak()
```

# Annexure-1

Application and proposed solution summary

| CIN | PAN | TAN |
| --- | --- | --- |
| U62011AP2025PTC120239 | ABQCS7152R | VPNS31351F |

| Applicant Entity | Contact |
| --- | --- |
| Syntriass Labs Private Limited | kattanaga5555@gmail.com |
| 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India | +91 88864 68060 |

# Applicant Details and Proposed Solution Summary

## Company Identification Details

| Field | Details |
| --- | --- |
| Legal Entity Name | Syntriass Labs Private Limited |
| CIN | U62011AP2025PTC120239 |
| PAN | ABQCS7152R |
| TAN | VPNS31351F |
| Registered Office | 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India |
| Contact Email | kattanaga5555@gmail.com |
| Contact Phone | +91 88864 68060 |
| Submission Date | 17 May 2026 |

## 1. Applicant Details

| Field | Details |
| --- | --- |
| Applicant Startup Name | Syntriass Labs Private Limited |
| Technology / Platform Name | AGP-OS Robotics Safety Layer |
| Intended Defence End User | Indian Armed Forces, DRDO robotics and autonomy laboratories, unmanned ground and aerial systems teams, ROS2-based robotics integrators, defence PSUs, and system assurance evaluators. |
| Applicant Name | K. Naga Sri Ganesh |
| Contact Email | kattanaga5555@gmail.com |
| Contact Number | +91 88864 68060 |
| Registered Office Address | 12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India |
| CIN / Incorporation Number | U62011AP2025PTC120239 |
| PAN | ABQCS7152R |
| TAN | VPNS31351F |
| DPIIT, Certificate No. | DIPP215355 |
| Proposed Project Duration | 12 months |
| Submission Date | 17 May 2026 |

## 2. Final Challenge Title

AGP-OS Robotics Safety Layer: Governed ROS2 and RTOS Control for Defence Robots

## 3. Intended Defence End Users

| End-User Group | Operational Need Addressed |
| --- | --- |
| Unmanned ground, aerial, surface, and inspection robot units | Govern command admission before movement, actuator, sensor, or mission-control actions. |
| DRDO robotics and autonomy laboratories | Evaluate ROS2 command governance, RTOS priority behavior, and HAL interlock logic in simulation. |
| Defence system integrators | Add a safety-governance layer without replacing the full robotics stack. |
| Command-and-control assurance teams | Review denial reasons, resource state, and command audit traces. |
| Robotics cyber and AI safety evaluators | Test resource abuse, unsafe velocity requests, watchdog timeout, and simulated actuator denial. |
| Defence PSUs and production teams | Use deployment artifacts as a starting point for hardware-in-loop validation. |

```{=typst}
#pagebreak()
```

## 4. A. Brief Summary of Proposed Solution

Defence robots increasingly combine ROS2 middleware, edge AI agents, and real-time control loops. Standard robot stacks can move commands from software to actuators, but they do not automatically prove whether a command was governed, prioritized correctly, kept inside resource limits, or blocked by a safety interlock when the action became unsafe.

AGP-OS Robotics Safety Layer proposes a governed operating layer for ROS2-based defence robots. Robot commands are converted into governed requests, prioritized through an RTOS-style scheduler, checked against resource budgets, and then passed through a HAL safety interlock before simulated actuator execution. The layer records whether a command was allowed, capped, blocked, or denied, and why.

The 12-month iDEX prototype will demonstrate a ROS2/Gazebo-style software simulation with governed robot command flow, RTOS priority scheduling, resource-limit denial, watchdog timeout, velocity capping, and HAL safety refusal. Current evidence supports software subsystem TRL 3-4. Physical robot, military-grade hardware, environmental, and hard real-time certification are proposed as validation work, not claimed as complete.

## 5. Critical Defence Problems Addressed

| Critical Problem | Operational Relevance For Defence Users | Proposed Control |
| --- | --- | --- |
| Unsafe robotic command admission | AI or operator software may issue motion or actuator commands without the right safety checks. | ROS2 bridge plus AGP command admission boundary. |
| Emergency-stop delay risk | Normal autonomy logic must not delay motor safety paths. | RTOS-style priority model where critical tasks run ahead of lower-priority work. |
| Resource exhaustion | AI agents can consume memory, tokens, CPU cycles, or I/O during a mission. | Resource controller grants or denies requests against configured quotas. |
| Unsafe actuator access | A command may be misaligned or exceed safe velocity. | HAL safety interlock and command capping path. |
| Robot heartbeat loss | Hardware or link failure can require immediate stop behavior. | Safety watchdog with timeout and emergency-stop state. |
| Weak prototype portability | Robotics code often remains lab-only. | ROS2 deployment files, simulation fallback, and Rust `no_std` RTOS core check. |

```{=typst}
#pagebreak()
```

## 6. B. Key Technologies Used

- AGP governed command admission
- ROS2 bridge and simulation mode
- RTOS-style priority scheduler
- HAL safety interlocks
- Resource quota controller
- Rust `no_std` RTOS core

## 7. C. Deliverables

| Deliverable | Defence-Oriented Description |
| --- | --- |
| ROS2 Governance Bridge | Converts robot topics, services, and command messages into governed requests. |
| RTOS Priority Demo | Demonstrates critical safety tasks executing ahead of normal governance and background work. |
| Resource Controller | Grants or denies memory, token, CPU, and I/O requests against configured mission budgets. |
| HAL Safety Interlock | Blocks unsafe simulated actuator commands and caps velocity requests. |
| Production ROS2 Adapter | Demonstrates watchdog, heartbeat timeout, emergency stop, and simulation fallback. |
| Rust RTOS Core | Provides `no_std`, `unsafe`-denied fixed-capacity scheduling primitives for future board ports. |
| Evidence Dashboard Prototype | Shows command state, denial reason, priority order, resource state, and safety event trace. |
| Validation Report | Provides test output, scenario results, limitations, and hardware-in-loop plan. |

## 8. D. Proposed Timeline

| Phase | Duration | Work Package | Expected Output |
| --- | --- | --- | --- |
| Phase 1 | Month 1 to Month 2 | Robotics safety profile and command schema | ROS2 command taxonomy, safety policy, and integration boundary. |
| Phase 2 | Month 3 to Month 4 | ROS2 governance bridge | Governed command admission for motion, sensor, and actuator flows. |
| Phase 3 | Month 5 to Month 6 | RTOS scheduling model | Critical, high, normal, low, and idle priority demo with deadline tracking. |
| Phase 4 | Month 7 to Month 8 | Resource denial and degradation | Memory, token, CPU, and I/O quota policies with denial trace. |
| Phase 5 | Month 9 | HAL interlock and watchdog | Velocity cap, low-alignment block, heartbeat timeout, and emergency-stop event. |
| Phase 6 | Month 10 | Simulation and adversarial scenarios | Unsafe command, resource abuse, timeout, and priority inversion tests. |
| Phase 7 | Month 11 | Deployment packaging | Docker, systemd, runbook, and board-porting interface notes. |
| Phase 8 | Month 12 | Final demonstration | iDEX demo package, test report, and hardware-in-loop plan. |

## 9. E. Readiness Position

Fresh evidence includes RTOS scheduler 8/8, ROS2 bridge 16/16, resource controller 12/12, production adapter 22/22, Rust RTOS core 4/4, and `wasm32-unknown-unknown` check passed.

Submission boundary: software plus simulation evidence only. Physical robot validation, target-board timing, environmental testing, service-specific policies, and operational certification remain proposed iDEX work packages.
