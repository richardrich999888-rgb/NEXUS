#!/usr/bin/env python3
"""
Real-Time Scheduler Test
Verifies priority scheduling, deadline handling, and motor control separation.
"""

import sys
import asyncio
import time
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.os.rtos.scheduler import rt_scheduler, TaskPriority

print("=" * 70)
print("REAL-TIME SCHEDULER TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}
execution_order = []

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# Mock tasks
def motor_stop():
    execution_order.append("MOTOR_STOP")
    return "Emergency stop executed"

def sensor_read():
    execution_order.append("SENSOR_READ")
    return {"distance": 1.5}

def governance_check():
    execution_order.append("GOVERNANCE")
    time.sleep(0.01)  # Simulate slow governance
    return "Alignment verified"

def background_log():
    execution_order.append("BACKGROUND")
    return "Logged"

async def run_tests():
    global execution_order
    
    # 1. Submit tasks in reverse priority order
    print("\n[1] SUBMITTING TASKS (REVERSE ORDER)...")
    rt_scheduler.submit_background("bg-1", background_log)
    rt_scheduler.submit("gov-1", governance_check, priority=TaskPriority.NORMAL)
    rt_scheduler.submit("sensor-1", sensor_read, priority=TaskPriority.HIGH)
    rt_scheduler.submit_critical("motor-1", motor_stop)
    
    stats = rt_scheduler.get_stats()
    test("4 tasks queued", stats["queued"] == 4)
    
    # 2. Execute all tasks
    print("\n[2] EXECUTING TASKS...")
    while rt_scheduler.task_queue:
        result = await rt_scheduler.run_once()
        if result:
            print(f"      Executed: {result['task_id']} ({result['priority']})")
    
    # 3. Verify execution order (critical first)
    print("\n[3] VERIFYING PRIORITY ORDER...")
    print(f"   Execution order: {execution_order}")
    test("MOTOR_STOP executed first (CRITICAL)", execution_order[0] == "MOTOR_STOP")
    test("SENSOR_READ executed second (HIGH)", execution_order[1] == "SENSOR_READ")
    test("GOVERNANCE executed third (NORMAL)", execution_order[2] == "GOVERNANCE")
    test("BACKGROUND executed last (LOW)", execution_order[3] == "BACKGROUND")
    
    # 4. Test deadline miss tracking
    print("\n[4] TESTING DEADLINE MISS DETECTION...")
    execution_order = []
    
    # Submit task with already-passed deadline
    rt_scheduler.submit("overdue-1", lambda: "late", 
                       priority=TaskPriority.NORMAL,
                       deadline_ms=-100)  # Negative = already overdue
    
    await rt_scheduler.run_once()
    test("Deadline miss detected", rt_scheduler.missed_deadlines >= 1)
    
    # 5. Test statistics
    print("\n[5] TESTING STATISTICS...")
    stats = rt_scheduler.get_stats()
    test("Completed count tracked", stats["completed"] >= 5)
    test("Queue breakdown available", "CRITICAL" in stats["queue_breakdown"])
    
    # Summary
    print("\n" + "=" * 70)
    print("REAL-TIME SCHEDULER TEST RESULTS")
    print("=" * 70)
    print(f"\n   Passed: {results['passed']}")
    print(f"   Failed: {results['failed']}")
    print(f"   Total:  {results['passed'] + results['failed']}")

    if results['failed'] == 0:
        print("\n✅ REAL-TIME SCHEDULER VERIFIED!")
    else:
        print(f"\n⚠️  {results['failed']} test(s) failed")

if __name__ == "__main__":
    asyncio.run(run_tests())
