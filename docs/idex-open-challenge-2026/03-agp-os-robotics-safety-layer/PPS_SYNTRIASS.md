# Annexure - 1

Preferably on Company's letterhead (if available)

# Proposed Solution Template (Open Challenge)

## 1. Applicant Name

Katta Naga Sri Ganesh

## 2. Startup/ MSME Name

SYNTRIASS Labs Private Limited

## 3. Challenge Title

AGP-OS Robotics Safety Layer: Governed ROS2 And RTOS Control For Defence Robots

## 4. Proposed duration (in months)

12 months

## 5. Contact & Email Id

To be inserted before portal upload

## 1. Brief Summary of the proposed Solution (upto 250 words)

Defence robots increasingly use ROS2, edge AI, and distributed autonomy. These stacks do not by themselves provide defence-grade governance over who may issue commands, which commands may consume constrained compute resources, how real-time priorities are enforced, or when a hardware safety interlock must deny execution.

AGP-OS Robotics Safety Layer proposes a governed operating layer for ROS2-based robotic systems. It combines AGP policy control, RTOS scheduling concepts, HAL safety interlocks, and resource-denial logic. Robot commands pass through explicit governance and resource checks before reaching motion, actuator, sensor, or mission-critical control paths.

The first prototype will be demonstrated in ROS2/Gazebo-style simulation. A command flow will show governed robot command admission, RTOS priority scheduling behavior, resource-limit denial, and HAL safety interlock refusal. The package does not claim physical platform validation in phase one. It proposes a 12-month prototype that converts existing AGP, RTOS, ROS2, HAL, and resource-control work into a robotics-focused evaluation package.

## 2. Key Technology(s) Used (5-6 keywords)

AGP, ROS2, RTOS scheduling, HAL interlocks, resource control, Python

## 3. Deliverable(s)

| S. No | Deliverable Name | Brief Description |
| --- | --- | --- |
| 1 | ROS2 governance bridge | Converts robot commands into governed requests |
| 2 | RTOS priority demo | Demonstrates scheduling behavior for safety-critical tasks |
| 3 | Resource controller | Denies commands exceeding configured budgets |
| 4 | HAL interlock simulation | Shows actuator/safety denial under unsafe conditions |

## 4. Proposed Timeline(s) (in months)

| Phase | Months | Output |
| --- | --- | --- |
| 1 | 1-2 | Robotics threat model and ROS2 command schema |
| 2 | 3-5 | ROS2 bridge and AGP command admission prototype |
| 3 | 6-8 | RTOS scheduling and resource-denial demo |
| 4 | 9-10 | HAL safety interlock simulation and audit trace |
| 5 | 11-12 | Final tests, documentation, and hardware-in-loop plan |
