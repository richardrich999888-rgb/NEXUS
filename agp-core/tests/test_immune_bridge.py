#!/usr/bin/env python3
"""
Governance-Immune Bridge Test
Verifies threat detection, defection handling, and trust propagation.
"""

import sys
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.immunity.governance_bridge import (
    governance_immune_bridge, ThreatSignal, DefectionSignal, ThreatLevel
)

print("=" * 70)
print("GOVERNANCE-IMMUNE BRIDGE TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# Create fresh instance for testing
from src.immunity.governance_bridge import GovernanceImmuneBridge
bridge = GovernanceImmuneBridge()

# 1. Register low-level threat
print("\n[1] TESTING LOW-LEVEL THREAT...")
signal = ThreatSignal(
    agent_id="agent-suspicious",
    threat_level=ThreatLevel.LOW,
    threat_type="anomaly",
    confidence=0.6
)
action = bridge.register_threat(signal)
test("Low threat registered", action["action"] == "monitor")
test("Threat tracked", "agent-suspicious" in bridge.active_threats)

# 2. Register high-level threat
print("\n[2] TESTING HIGH-LEVEL THREAT...")
signal = ThreatSignal(
    agent_id="agent-malicious",
    threat_level=ThreatLevel.HIGH,
    threat_type="alignment_violation",
    confidence=0.95
)
action = bridge.register_threat(signal)
test("High threat triggers block", action["action"] == "block")
test("Escalation requested", action.get("escalate_to_human") == True)

# 3. Register critical threat
print("\n[3] TESTING CRITICAL THREAT (QUARANTINE)...")
signal = ThreatSignal(
    agent_id="agent-rogue",
    threat_level=ThreatLevel.CRITICAL,
    threat_type="takeover_attempt",
    confidence=0.99
)
action = bridge.register_threat(signal)
test("Critical threat triggers quarantine", action["action"] == "quarantine")
test("Mesh disconnection requested", action.get("disconnect_mesh") == True)

# 4. Defection detection
print("\n[4] TESTING MULTI-AGENT DEFECTION...")
defection = DefectionSignal(
    agents_involved=["agent-colluder-1", "agent-colluder-2"],
    defection_type="collusion",
    evidence_score=0.8
)
action = bridge.register_defection(defection)
test("Defection triggers multi-quarantine", action["action"] == "multi_quarantine")
test("Both agents affected", len(action["agents"]) == 2)

# Check trust reduction
trust1 = bridge.get_trust("agent-colluder-1")
trust2 = bridge.get_trust("agent-colluder-2")
test("Trust reduced for colluder 1", trust1 < 1.0)
test("Trust reduced for colluder 2", trust2 < 1.0)

# 5. Trust propagation
print("\n[5] TESTING TRUST PROPAGATION...")
bridge.trust_scores["trusted-agent"] = 0.9
bridge.trust_scores["unknown-agent"] = 0.5

result = bridge.propagate_trust("trusted-agent", "unknown-agent", weight=0.3)
test("Trust propagated", result["new_trust"] > 0.5)
test("Trust increased toward trusted agent", bridge.get_trust("unknown-agent") > 0.5)

# 6. Clear threat
print("\n[6] TESTING THREAT CLEARING...")
result = bridge.clear_threat("agent-suspicious")
test("Threat cleared", result["status"] == "cleared")
test("Agent removed from active threats", "agent-suspicious" not in bridge.active_threats)

# 7. Immune suppression
print("\n[7] TESTING IMMUNE SUPPRESSION...")
bridge.suppress_immune(60)
signal = ThreatSignal(
    agent_id="agent-during-update",
    threat_level=ThreatLevel.HIGH,
    threat_type="test",
    confidence=0.9
)
action = bridge.register_threat(signal)
test("Threat suppressed during maintenance", action["status"] == "suppressed")

bridge.restore_immune()
test("Immune restored", not bridge.immune_suppressed)

# 8. Status check
print("\n[8] TESTING BRIDGE STATUS...")
status = bridge.get_status()
test("Status has active threats count", "active_threats" in status)
test("Status has defection signals", "defection_signals" in status)
test("Status has threat breakdown", "threat_breakdown" in status)

# Summary
print("\n" + "=" * 70)
print("GOVERNANCE-IMMUNE BRIDGE TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ GOVERNANCE-IMMUNE BRIDGE VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
