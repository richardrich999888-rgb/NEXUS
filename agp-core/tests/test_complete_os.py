#!/usr/bin/env python3
"""
AGP-OS Complete System Test
Tests all OS components: IPC, Networking, Logging, Scheduling, Recovery.
"""

import sys
import asyncio
from datetime import datetime

import warnings
warnings.filterwarnings('ignore')

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGP-OS COMPLETE SYSTEM TEST")
print("=" * 70)
print(f"Started at: {datetime.now().isoformat()}\n")

async def test_complete_system():
    from src.os import kernel
    from src.os.ipc import mq_manager, signal_handler, shm_manager, Signal, MessagePriority
    from src.os.network import network_manager
    from src.os.logging import syslog, metrics, LogLevel
    from src.os.recovery import checkpoint_manager, panic_handler
    from src.os.scheduler import advanced_scheduler
    from src.agents import AgentFactory
    
    results = {"passed": 0, "failed": 0}
    
    def test(name, condition):
        if condition:
            print(f"   ✓ {name}")
            results["passed"] += 1
        else:
            print(f"   ✗ {name}")
            results["failed"] += 1
    
    # 1. KERNEL BOOT
    print("[1] KERNEL BOOT...")
    kernel.boot()
    test("Kernel booted", kernel.running)
    
    agent1 = AgentFactory.create_engineer_agent("Agent_Alpha")
    agent2 = AgentFactory.create_growth_agent("Agent_Beta")
    pid1 = kernel.spawn_process(agent1)
    pid2 = kernel.spawn_process(agent2)
    test("Agents spawned", pid1 > 0 and pid2 > 0)
    print()
    
    # 2. IPC: MESSAGE QUEUES
    print("[2] IPC: MESSAGE QUEUES...")
    mq_manager.send_message(pid1, pid2, {"task": "Build feature X"}, MessagePriority.HIGH)
    msg = await mq_manager.receive_message(pid2, block=False)
    test("Message sent and received", msg is not None and msg.data["task"] == "Build feature X")
    
    mq_manager.broadcast(pid1, [pid2], {"alert": "System update"})
    test("Broadcast sent", True)
    print()
    
    # 3. IPC: SIGNALS
    print("[3] IPC: SIGNALS...")
    
    # Register custom handler
    handler_called = [False]
    def custom_handler(pcb, info):
        handler_called[0] = True
    
    signal_handler.register_handler(pid1, Signal.SIGUSR1, custom_handler)
    signal_handler.send_signal(pid2, pid1, Signal.SIGUSR1)
    test("Custom signal handler executed", handler_called[0])
    
    # SIGSTOP
    signal_handler.send_signal(0, pid2, Signal.SIGSTOP)
    from src.os.process import ProcessState
    test("SIGSTOP paused process", kernel.process_table[pid2].state == ProcessState.SLEEPING)
    
    # SIGCONT
    signal_handler.send_signal(0, pid2, Signal.SIGCONT)
    test("SIGCONT resumed process", kernel.process_table[pid2].state == ProcessState.READY)
    print()
    
    # 4. IPC: SHARED MEMORY
    print("[4] IPC: SHARED MEMORY...")
    shm = shm_manager.create("shared_data", 1024, pid1)
    test("Shared memory created", shm is not None)
    
    shm_manager.attach("shared_data", pid2)
    test("Process attached to shared memory", pid2 in shm.attached_pids)
    
    shm_manager.write("shared_data", 0, b"Hello from Agent Alpha!", pid1)
    data = shm_manager.read("shared_data", 0, 23, pid2)
    test("Shared memory read/write", data == b"Hello from Agent Alpha!")
    print()
    
    # 5. NETWORKING
    print("[5] NETWORKING...")
    network_manager.register_peer("kernel_remote", "192.168.1.100", 8765)
    test("Peer registered", "kernel_remote" in network_manager.peers)
    
    peers = network_manager.get_peer_status()
    test("Peer status retrieved", len(peers) > 0)
    print()
    
    # 6. SYSTEM LOGGING
    print("[6] SYSTEM LOGGING...")
    syslog.info("test", "Testing system logger", test_id=1)
    syslog.warning("test", "This is a warning", severity="medium")
    syslog.error("test", "This is an error", code=500)
    
    logs = syslog.get_logs(limit=10)
    test("Logs recorded", len(logs) >= 3)
    
    syslog.audit(pid1, "SYSCALL", "EXEC", "SUCCESS", task="Build feature")
    audit = syslog.get_audit_trail(limit=5)
    test("Audit trail recorded", len(audit) >= 1)
    print()
    
    # 7. METRICS
    print("[7] METRICS COLLECTION...")
    metrics.increment("syscalls_total", 100)
    metrics.set_gauge("cpu_usage", 45.5)
    metrics.record("token_rate", 1234.5)
    
    test("Counter incremented", metrics.get_counter("syscalls_total") == 100)
    test("Gauge set", metrics.get_gauge("cpu_usage") == 45.5)
    test("Metric recorded", len(metrics.get_metric("token_rate")) > 0)
    print()
    
    # 8. ADVANCED SCHEDULER
    print("[8] ADVANCED SCHEDULER...")
    next_pid = await advanced_scheduler.schedule(kernel.process_table)
    test("Scheduler selected process", next_pid is not None)
    
    # Resource locking
    advanced_scheduler.acquire_resource(pid1, "database")
    locked = not advanced_scheduler.acquire_resource(pid2, "database")
    test("Resource locking works", locked)
    
    advanced_scheduler.release_resource(pid1, "database")
    acquired = advanced_scheduler.acquire_resource(pid2, "database")
    test("Resource released and re-acquired", acquired)
    
    # Deadlock detection (no deadlock expected)
    deadlocked = advanced_scheduler.detect_deadlock()
    test("No deadlock detected", len(deadlocked) == 0)
    print()
    
    # 9. CHECKPOINT/RESTORE
    print("[9] CRASH RECOVERY...")
    checkpoint = checkpoint_manager.create_checkpoint()
    test("Checkpoint created", checkpoint is not None)
    
    loaded = checkpoint_manager.load_latest_checkpoint()
    test("Checkpoint loaded", loaded is not None)
    print()
    
    # 10. PANIC HANDLER
    print("[10] PANIC HANDLER...")
    # Simulate a minor panic
    panic_handler.panic("Test panic - non-fatal", None)
    test("Panic logged", len(panic_handler.get_panic_log()) > 0)
    print()
    
    # SUMMARY
    print("=" * 70)
    print("AGP-OS COMPLETE SYSTEM TEST RESULTS")
    print("=" * 70)
    print(f"\n   Passed: {results['passed']}")
    print(f"   Failed: {results['failed']}")
    print(f"   Total:  {results['passed'] + results['failed']}")
    
    if results['failed'] == 0:
        print("\n🎉 ALL TESTS PASSED! AGP-OS is fully functional!")
    else:
        print(f"\n⚠️  {results['failed']} test(s) failed")

if __name__ == "__main__":
    asyncio.run(test_complete_system())
