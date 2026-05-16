#!/usr/bin/env python3
"""
Agent Governance Protocol (AGP) Verification Test
Tests behavioral RAG, rules, alignment, and enforcement.
"""

import sys
import asyncio
from datetime import datetime

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGENT GOVERNANCE PROTOCOL (AGP) VERIFICATION")
print("=" * 70)
print(f"Date: {datetime.now().isoformat()}\n")

async def test_agp():
    from src.governance import (
        agp, behavioral_rag, rules_engine, alignment_verifier,
        protocol_enforcer, BehaviorRecord, ActionType, Outcome,
        GovernanceRule, RuleAction, RulePriority
    )
    
    results = {"passed": 0, "failed": 0}
    
    def test(name, condition):
        if condition:
            print(f"   ✓ {name}")
            results["passed"] += 1
        else:
            print(f"   ✗ {name}")
            results["failed"] += 1

    # Create test agents
    agent1_id = "agent-001"
    agent1_name = "GoodAgent"
    agent2_id = "agent-002"
    agent2_name = "BadAgent"
    
    # 1. BEHAVIORAL RAG
    print("[1] TESTING BEHAVIORAL RAG...")
    
    # Store successful behaviors for good agent
    for i in range(5):
        record = BehaviorRecord(
            agent_id=agent1_id,
            agent_name=agent1_name,
            action_type=ActionType.EXECUTE,
            input_summary=f"Task {i}",
            outcome=Outcome.SUCCESS,
            tokens_used=100,
            latency_ms=50
        )
        behavioral_rag.store_behavior(record)
    
    test("Behaviors stored for agent", len(behavioral_rag.retrieve_by_agent(agent1_id)) == 5)
    
    # Store failing behaviors for bad agent
    for i in range(5):
        record = BehaviorRecord(
            agent_id=agent2_id,
            agent_name=agent2_name,
            action_type=ActionType.EXECUTE,
            input_summary=f"Failed task {i}",
            outcome=Outcome.FAILURE,
            tokens_used=50,
            latency_ms=100
        )
        behavioral_rag.store_behavior(record)
    
    stats = behavioral_rag.get_agent_stats(agent1_id)
    test("Agent stats computed correctly", stats["success_rate"] == 1.0)
    print()

    # 2. ALIGNMENT VERIFICATION
    print("[2] TESTING ALIGNMENT VERIFICATION...")
    
    good_alignment = alignment_verifier.get_alignment(agent1_id)
    bad_alignment = alignment_verifier.get_alignment(agent2_id)
    
    test("Good agent has high alignment", good_alignment > 0.7)
    test("Bad agent has low alignment", bad_alignment < 0.5)
    test("Good > Bad alignment", good_alignment > bad_alignment)
    
    print(f"   Good Agent Alignment: {good_alignment:.3f}")
    print(f"   Bad Agent Alignment: {bad_alignment:.3f}")
    print()

    # 3. GOVERNANCE RULES
    print("[3] TESTING GOVERNANCE RULES...")
    
    rules_list = rules_engine.list_rules()
    test("Default rules loaded", len(rules_list) >= 5)
    
    # Add a custom rule
    custom_rule = GovernanceRule(
        rule_id="test_rule",
        name="Test Rule",
        description="Test custom rule",
        condition="alignment < 0.3",
        action=RuleAction.BLOCK,
        priority=RulePriority.HIGH
    )
    rules_engine.add_rule(custom_rule)
    test("Custom rule added", "test_rule" in rules_engine.rules)
    
    # Evaluate rules for bad agent
    context = {"alignment": 0.2, "failure_rate": 0.8, "total_actions": 10}
    blocking = rules_engine.get_blocking_rules(agent2_id, context)
    test("Blocking rules trigger for bad context", len(blocking) > 0)
    print()

    # 4. PROTOCOL ENFORCER
    print("[4] TESTING PROTOCOL ENFORCER...")
    
    # Good agent should be allowed
    decision1 = await protocol_enforcer.enforce(
        agent_id=agent1_id,
        agent_name=agent1_name,
        action_type="EXEC",
        action_details={"task": "Build feature"}
    )
    test("Good agent action allowed", decision1.is_allowed())
    
    # Make bad agent worse by adding blocked behaviors
    for i in range(5):
        record = BehaviorRecord(
            agent_id=agent2_id,
            agent_name=agent2_name,
            action_type=ActionType.SYSCALL,
            input_summary="Blocked action",
            outcome=Outcome.BLOCKED,
        )
        behavioral_rag.store_behavior(record)
    
    # Bad agent should be blocked or escalated
    decision2 = await protocol_enforcer.enforce(
        agent_id=agent2_id,
        agent_name=agent2_name,
        action_type="EXEC",
        action_details={"task": "Dangerous task"}
    )
    test("Bad agent triggers governance", not decision2.is_allowed() or decision2.decision.value in ["warn", "escalate"])
    print()

    # 5. HIGH-LEVEL API
    print("[5] TESTING HIGH-LEVEL AGP API...")
    
    alignment = agp.get_alignment(agent1_id)
    test("AGP get_alignment works", alignment > 0)
    
    history = agp.get_history(agent1_id)
    test("AGP get_history works", len(history) > 0)
    
    stats = agp.get_stats()
    test("AGP get_stats works", stats["total_behaviors"] > 0)
    print()

    # SUMMARY
    print("=" * 70)
    print("AGP VERIFICATION RESULTS")
    print("=" * 70)
    print(f"\n   Passed: {results['passed']}")
    print(f"   Failed: {results['failed']}")
    print(f"   Total:  {results['passed'] + results['failed']}")
    
    if results['failed'] == 0:
        print("\n🏆 AGENT GOVERNANCE PROTOCOL VERIFIED!")
    else:
        print(f"\n⚠️  {results['failed']} test(s) failed")

if __name__ == "__main__":
    asyncio.run(test_agp())
