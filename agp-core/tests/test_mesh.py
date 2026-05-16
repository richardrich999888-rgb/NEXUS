#!/usr/bin/env python3
"""
Mesh Coordination Test
Verifies inter-agent messaging and consensus voting.
"""

import sys
import time
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.os.mesh.mesh import mesh

print("=" * 70)
print("MESH COORDINATION TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# Register 3 agents
print("\n[1] REGISTERING AGENTS...")
mesh.register_agent("agent-alpha")
mesh.register_agent("agent-beta")
mesh.register_agent("agent-gamma")
test("3 agents registered", len(mesh.known_agents) == 3)

# ========== Message Bus Tests ==========
print("\n[2] TESTING PEER-TO-PEER MESSAGING...")
result = mesh.send_message("agent-alpha", "agent-beta", {"cmd": "ping"})
test("Message sent successfully", result.get("status") == "sent")

messages = mesh.receive_messages("agent-beta")
test("Beta received message from Alpha", len(messages) == 1)
test("Message content is correct", messages[0].get("content", {}).get("cmd") == "ping")

# Read again (should be empty since marked read)
messages = mesh.receive_messages("agent-beta")
test("Message marked as read (no repeat)", len(messages) == 0)

print("\n[3] TESTING BROADCAST...")
result = mesh.broadcast("agent-alpha", {"alert": "All hands on deck"})
test("Broadcast sent to 2 agents", result.get("recipients") == 2)

beta_msgs = mesh.receive_messages("agent-beta")
gamma_msgs = mesh.receive_messages("agent-gamma")
test("Beta received broadcast", len(beta_msgs) == 1)
test("Gamma received broadcast", len(gamma_msgs) == 1)

# ========== Consensus Engine Tests ==========
print("\n[4] TESTING PROPOSAL CREATION...")
result = mesh.propose("agent-alpha", "HALT_ALL", "Emergency stop for all robots")
test("Proposal created", result.get("status") == "proposed")
proposal_id = result.get("proposal_id")

print("\n[5] TESTING VOTING...")
# Alpha implicitly approves (proposer)
vote1 = mesh.vote("agent-beta", proposal_id, True)
test("Beta voted FOR", vote1.get("status") == "voted")

vote2 = mesh.vote("agent-gamma", proposal_id, False)
test("Gamma voted AGAINST", vote2.get("status") == "voted")

# Check status (2 for, 1 against, 50% required = APPROVED)
status = mesh.get_proposal_status(proposal_id)
print(f"   Proposal Status: {status}")
# Note: With 3 agents, 2 FOR = 66% > 50% required
# But proposer didn't vote explicitly, so let's check alpha's implicit vote
# Our current implementation doesn't auto-count proposer vote, so only Beta voted FOR
# 1 FOR out of 3 = 33% < 50%, so should still be OPEN or needs more votes

# Actually, the proposer needs to vote too. Let's have alpha vote.
vote3 = mesh.vote("agent-alpha", proposal_id, True)
status = mesh.get_proposal_status(proposal_id)
print(f"   Updated Status: {status}")
test("Proposal resolved (APPROVED)", status.get("state") == "approved")

# ========== Double Vote Prevention ==========
print("\n[6] TESTING DOUBLE-VOTE PREVENTION...")
vote_again = mesh.vote("agent-alpha", proposal_id, False)
# Either "Already voted" or "Proposal already approved" is acceptable
rejection_reasons = ["Already voted", "Proposal already approved"]
test("Double vote rejected", vote_again.get("reason") in rejection_reasons)

# Summary
print("\n" + "=" * 70)
print("MESH COORDINATION TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ MESH COORDINATION VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
