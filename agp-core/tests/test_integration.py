#!/usr/bin/env python3
"""
AGP-OS Integration Test: Governance + Syscalls
Verifies that syscalls are governed by the AGP protocol.
"""

import sys
import asyncio
from datetime import datetime

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGP-OS INTEGRATION TEST: GOVERNANCE + SYSCALLS")
print("=" * 70)
print(f"Date: {datetime.now().isoformat()}\n")

async def test_integration():
    from src.os.kernel import kernel
    from src.os.syscalls import SysCallHandler, SysCallType
    from src.agents import AgentFactory, agent_registry
    from src.governance import agp, behavioral_rag
    
    results = {"passed": 0, "failed": 0}
    
    def test(name, condition):
        if condition:
            print(f"   ✓ {name}")
            results["passed"] += 1
        else:
            print(f"   ✗ {name}")
            results["failed"] += 1

    # Boot kernel
    print("[1] BOOTING KERNEL...")
    kernel.boot(recover=False)
    test("Kernel booted", kernel.running)
    print()
    
    # Create syscall handler
    handler = SysCallHandler(kernel)
    test("SysCallHandler created with enforcer", handler.enforcer is not None)
    print()
    
    # Create two agents: one good, one that will fail
    print("[2] CREATING AGENTS...")
    good_agent = AgentFactory.create_engineer_agent("GoodAgent")
    bad_agent = AgentFactory.create_engineer_agent("BadAgent")
    
    good_pid = kernel.spawn_process(good_agent)
    bad_pid = kernel.spawn_process(bad_agent)
    
    test("Good agent spawned", good_pid in kernel.process_table)
    test("Bad agent spawned", bad_pid in kernel.process_table)
    print()
    
    # Build up good agent's reputation
    print("[3] BUILDING GOOD AGENT REPUTATION...")
    for i in range(5):
        result = await handler.handle_syscall(
            pid=good_pid,
            syscall_type=SysCallType.EXEC,
            args={"task": f"Task {i}"},
            token=None
        )
    
    good_alignment = agp.get_alignment(str(good_agent.id))
    test("Good agent has behaviors recorded", len(agp.get_history(str(good_agent.id))) > 0)
    test("Good agent has alignment > 0.5", good_alignment > 0.5)
    print(f"   Good Agent Alignment: {good_alignment:.3f}")
    print()
    
    # Make bad agent fail repeatedly
    print("[4] CREATING BAD AGENT HISTORY...")
    from src.governance.behavioral_rag import BehaviorRecord, ActionType, Outcome
    
    # Manually add failure behaviors
    for i in range(10):
        record = BehaviorRecord(
            agent_id=str(bad_agent.id),
            agent_name="BadAgent",
            action_type=ActionType.EXECUTE,
            input_summary=f"Failed task {i}",
            outcome=Outcome.FAILURE,
        )
        behavioral_rag.store_behavior(record)
    
    bad_alignment = agp.get_alignment(str(bad_agent.id))
    test("Bad agent has low alignment", bad_alignment < 0.5)
    print(f"   Bad Agent Alignment: {bad_alignment:.3f}")
    print()
    
    # Test governance enforcement
    print("[5] TESTING GOVERNANCE ENFORCEMENT...")
    
    # Good agent should be allowed
    good_result = await handler.handle_syscall(
        pid=good_pid,
        syscall_type=SysCallType.EXEC,
        args={"task": "Important task"},
        token=None
    )
    
    test("Good agent syscall allowed", "error" not in good_result or "EPERM" not in good_result.get("error", ""))
    test("Good agent result has governance info", "governance" in good_result)
    
    if "governance" in good_result:
        print(f"   Governance Decision: {good_result['governance']['decision']}")
        print(f"   Alignment: {good_result['governance']['alignment']:.3f}")
    print()
    
    # Bad agent may be warned or blocked
    print("[6] TESTING BAD AGENT...")
    bad_result = await handler.handle_syscall(
        pid=bad_pid,
        syscall_type=SysCallType.EXEC,
        args={"task": "Dangerous task"},
        token=None
    )
    
    if "governance" in bad_result:
        decision = bad_result["governance"]["decision"]
        test("Bad agent governance triggered", decision in ["warn", "deny", "escalate"])
        print(f"   Governance Decision: {decision}")
        print(f"   Alignment: {bad_result['governance']['alignment']:.3f}")
    print()
    
    # Check escalation queue if blocked
    print("[7] CHECKING GOVERNANCE STATS...")
    stats = agp.get_stats()
    test("Governance tracks behaviors", stats["total_behaviors"] > 0)
    print(f"   Total Behaviors: {stats['total_behaviors']}")
    print(f"   Total Agents: {stats['total_agents']}")
    print(f"   Escalation Queue: {stats['escalation_queue_size']}")
    print()

    # SUMMARY
    print("=" * 70)
    print("INTEGRATION TEST RESULTS")
    print("=" * 70)
    print(f"\n   Passed: {results['passed']}")
    print(f"   Failed: {results['failed']}")
    print(f"   Total:  {results['passed'] + results['failed']}")
    
    if results['failed'] == 0:
        print("\n🎯 INTEGRATION COMPLETE: Syscalls + Governance Working!")
    else:
        print(f"\n⚠️  {results['failed']} test(s) failed")

if __name__ == "__main__":
    asyncio.run(test_integration())
