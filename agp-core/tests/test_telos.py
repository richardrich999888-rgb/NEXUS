#!/usr/bin/env python3
"""
TELOS Commitment Membrane Test
Verifies entropy metering, authority verification, and trust accumulation.
"""

import sys
from pathlib import Path
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.telos import (
    CommitmentMembrane, Decision, ConsequenceTier
)

print("=" * 70)
print("TELOS COMMITMENT MEMBRANE TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# Create fresh membrane
membrane = CommitmentMembrane()

# 1. Register agents with different authority levels
print("\n[1] REGISTERING AGENTS...")
membrane.register_agent("admin-agent", ["*"])  # Full access
membrane.register_agent("reader-agent", ["read:*"])  # Read only
membrane.register_agent("writer-agent", ["read:*", "write:models"])  # Limited write

status = membrane.get_status()
test("3 agents registered", status["registered_agents"] == 3)

# 2. Trivial crossing (low entropy, any authority)
print("\n[2] TRIVIAL CROSSING (LOW COST)...")
decision = Decision(
    decision_id="read-1",
    action="read_model",
    agent_id="reader-agent",
    tier=ConsequenceTier.TRIVIAL
)
result = membrane.request_crossing(decision, required_scope="read:models")
test("Trivial crossing allowed", result.allowed)
test("Low entropy spent", result.entropy_spent == 1)

# 3. Authority denied
print("\n[3] AUTHORITY DENIED...")
decision = Decision(
    decision_id="write-1",
    action="write_model",
    agent_id="reader-agent",
    tier=ConsequenceTier.MEDIUM
)
result = membrane.request_crossing(decision, required_scope="write:models")
test("Write crossing denied for reader", not result.allowed)
test("Reason is authority", "AUTHORITY_DENIED" in result.reason)

# 4. Medium crossing with proper authority
print("\n[4] MEDIUM CROSSING (PROPER AUTHORITY)...")
decision = Decision(
    decision_id="write-2",
    action="write_model",
    agent_id="writer-agent",
    tier=ConsequenceTier.MEDIUM
)
result = membrane.request_crossing(decision, required_scope="write:models")
test("Write crossing allowed for writer", result.allowed)
test("Medium entropy spent (100)", result.entropy_spent == 100)

# 5. High-consequence crossing (trust check)
print("\n[5] HIGH-CONSEQUENCE CROSSING (TRUST CHECK)...")
# New agent has default trust 0.5, blocked for HIGH actions
decision = Decision(
    decision_id="deploy-1",
    action="deploy_production",
    agent_id="admin-agent",
    tier=ConsequenceTier.HIGH
)
result = membrane.request_crossing(decision, required_scope="execute:deploy")
test("High action denied (low trust)", not result.allowed)
test("Reason is trust", "TRUST_INSUFFICIENT" in result.reason)

# 6. Build trust through successful crossings
print("\n[6] BUILDING TRUST...")
for i in range(15):
    decision = Decision(
        decision_id=f"trust-build-{i}",
        action="safe_action",
        agent_id="admin-agent",
        tier=ConsequenceTier.LOW
    )
    membrane.request_crossing(decision, required_scope="execute:safe")

trust = membrane.trust_accumulator.get_trust("admin-agent")
test(f"Trust increased to {trust:.2f}", trust > 0.6)

# 7. High-consequence now allowed (trust built)
print("\n[7] HIGH-CONSEQUENCE WITH TRUST...")
decision = Decision(
    decision_id="deploy-2",
    action="deploy_production",
    agent_id="admin-agent",
    tier=ConsequenceTier.HIGH
)
result = membrane.request_crossing(decision, required_scope="execute:deploy")
test("High action now allowed (trust built)", result.allowed)

# 8. Entropy exhaustion
print("\n[8] ENTROPY EXHAUSTION...")
membrane2 = CommitmentMembrane()
membrane2.entropy_meter.budget = 50  # Very low budget
membrane2.register_agent("test-agent", ["*"])

decision = Decision(
    decision_id="exhaust-1",
    action="critical_action",
    agent_id="test-agent",
    tier=ConsequenceTier.CRITICAL  # Costs 2000
)
result = membrane2.request_crossing(decision, required_scope="*")
test("Critical action denied (no entropy)", not result.allowed)
test("Reason is entropy", "ENTROPY_EXHAUSTED" in result.reason)

# 9. Check final stats
print("\n[9] FINAL STATUS...")
status = membrane.get_status()
test("Crossings recorded", status["total_crossings"] > 0)
test("Some successful", status["successful_crossings"] > 0)

# Summary
print("\n" + "=" * 70)
print("TELOS COMMITMENT MEMBRANE TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ TELOS MEMBRANE VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
