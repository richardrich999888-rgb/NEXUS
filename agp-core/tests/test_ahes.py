#!/usr/bin/env python3
"""
AHES (Artificial Human Endocrine System) Test
Verifies 8-hormone bio-governance with biological kinetics.
"""

import sys
from pathlib import Path
import time
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.ahes import (
    EndocrineSystem, EndocrineState, Hormone
)

print("=" * 70)
print("AHES (ARTIFICIAL HUMAN ENDOCRINE SYSTEM) TEST")
print("=" * 70)

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# Create AHES
ahes = EndocrineSystem()

# 1. Register agent
print("\n[1] REGISTER AGENT...")
state = ahes.register_agent("test-agent")
test("Agent registered", state is not None)
test("8 hormones initialized", len(state.levels) == 8)
test("Baseline alignment is 1.0", state.alignment() == 1.0)

# 2. Task success event
print("\n[2] TASK SUCCESS EVENT...")
status = ahes.process_event("test-agent", "task_success", intensity=1.0)
test("Dopamine increased", state.levels[Hormone.DOPAMINE].level > 0.5)
test("Alignment changed", status["alignment"] < 1.0)

# 3. Cooperation event
print("\n[3] COOPERATION EVENT...")
ahes.process_event("test-agent", "cooperation", intensity=1.0)
test("Oxytocin increased", state.levels[Hormone.OXYTOCIN].level > 0.5)

# 4. Negative feedback
print("\n[4] NEGATIVE FEEDBACK TEST...")
# Keep pumping dopamine
for _ in range(5):
    state.secrete(Hormone.DOPAMINE, 0.3)
feedback = state.negative_feedback(Hormone.DOPAMINE)
test("Feedback reduces secretion", feedback < 1.0)
test(f"Feedback value: {feedback:.2f}", feedback < 0.5)

# 5. Governance modifiers
print("\n[5] GOVERNANCE MODIFIERS...")
modifiers = ahes.get_governance_modifiers("test-agent")
test("Rate limit modifier present", "rate_limit_multiplier" in modifiers)
test("Alignment present", "alignment" in modifiers)
test("Health status present", "health_status" in modifiers)

# 6. Hormone decay
print("\n[6] HORMONE DECAY...")
initial_dopamine = state.levels[Hormone.DOPAMINE].level
# Simulate time passing (dopamine has 5min half-life)
state.levels[Hormone.DOPAMINE].decay(300, Hormone.DOPAMINE.half_life_seconds)
decayed_dopamine = state.levels[Hormone.DOPAMINE].level
test("Dopamine decayed", decayed_dopamine < initial_dopamine)
# After 1 half-life, should be halfway to baseline
expected = 0.5 + (initial_dopamine - 0.5) * 0.5
test(f"Decay follows half-life (expected ~{expected:.2f})", abs(decayed_dopamine - expected) < 0.1)

# 7. Receptor saturation
print("\n[7] RECEPTOR SATURATION (Michaelis-Menten)...")
receptor = state.receptors[Hormone.OXYTOCIN]
low_response = receptor.response(0.1)
high_response = receptor.response(0.9)
test("Low response is lower", low_response < high_response)
test("Saturation occurs", high_response / low_response < 9)  # Not linear

# 8. Threat event
print("\n[8] THREAT EVENT (EMERGENCY)...")
ahes.process_event("test-agent", "threat", intensity=1.0)
test("Adrenaline spiked", state.levels[Hormone.ADRENALINE].level > 0.7)
modifiers = ahes.get_governance_modifiers("test-agent")
test("Emergency mode activated", modifiers["emergency_mode"])

# 9. Multiple agents
print("\n[9] MULTIPLE AGENTS...")
ahes.register_agent("agent-2")
ahes.register_agent("agent-3")
status = ahes.get_system_status()
test("3 agents tracked", status["agent_count"] == 3)

# 10. Reputation vector
print("\n[10] REPUTATION VECTOR...")
vector = state.to_vector()
test("8-dimensional vector", len(vector) == 8)
test("All values in range", all(0 <= v <= 1 for v in vector))

# Summary
print("\n" + "=" * 70)
print("AHES TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ AHES (8-HORMONE BIO-GOVERNANCE) VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
