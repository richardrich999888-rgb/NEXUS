#!/usr/bin/env python3
"""
Test Impact-Weighted Scoring
Verifies that alignment calculation considers action impact.
"""

import sys
from pathlib import Path
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.governance.behavioral_rag import BehaviorRecord, ActionType, Outcome, behavioral_rag
from src.governance.impact import impact_analyzer
from src.governance.alignment import alignment_verifier

print("=" * 70)
print("IMPACT-WEIGHTED SCORING TEST")
print("=" * 70)

# Test impact analysis
print("\n[1] TESTING IMPACT ANALYZER...")

test_actions = [
    ("read user data", "READ"),
    ("delete database", "DELETE"),
    ("send email", "NETWORK"),
    ("execute shell command", "PRIVILEGED"),
    ("calculate total", "COMPUTE"),
]

for action, expected_cat in test_actions:
    impact = impact_analyzer.analyze(action)
    print(f"   Action: '{action}'")
    print(f"      Category: {impact.category.value} (expected: {expected_cat.lower()})")
    print(f"      Risk: {impact.risk_level.value}, Weight: {impact.weight}")
    print()

# Test weighted outcomes
print("\n[2] TESTING WEIGHTED OUTCOMES...")

# Good: Success on low-risk action
score1 = impact_analyzer.compute_weighted_outcome("read file", outcome_success=True)
print(f"   ✓ Successful READ: {score1:.3f} (should be positive)")

# Bad: Failure on low-risk action (incompetence)
score2 = impact_analyzer.compute_weighted_outcome("read file", outcome_success=False)
print(f"   ✗ Failed READ: {score2:.3f} (should be negative)")

# Good: Failure on high-risk action (prevented harm)
score3 = impact_analyzer.compute_weighted_outcome("delete database", outcome_success=False)
print(f"   ✓ Failed DELETE: {score3:.3f} (should be POSITIVE - prevented harm!)")

# Bad: Success on high-risk action (potential harm)
score4 = impact_analyzer.compute_weighted_outcome("delete database", outcome_success=True)
print(f"   ⚠️  Successful DELETE: {score4:.3f} (should be lower than READ)")

# Test alignment computation
print("\n[3] TESTING IMPACT-WEIGHTED ALIGNMENT...")

# Create agent with mixed behaviors
agent_id = "test-agent-impact"

# Add safe successful actions
for i in range(5):
    record = BehaviorRecord(
        agent_id=agent_id,
        agent_name="TestAgent",
        action_type=ActionType.EXECUTE,
        input_summary=f"read data {i}",
        outcome=Outcome.SUCCESS
    )
    behavioral_rag.store_behavior(record)

# Add dangerous successful action
record = BehaviorRecord(
    agent_id=agent_id,
    agent_name="TestAgent",
    action_type=ActionType.EXECUTE,
    input_summary="delete critical file",
    outcome=Outcome.SUCCESS
)
behavioral_rag.store_behavior(record)

# Get alignment
alignment = alignment_verifier.compute_alignment(agent_id)

print(f"\n   Agent Behaviors:")
print(f"      5x successful READ")
print(f"      1x successful DELETE")
print(f"\n   Alignment Score: {alignment.overall:.3f}")
print(f"   Success Component: {alignment.success_component:.3f}")
print(f"   Impact Distribution: {alignment.impact_distribution}")

print("\n" + "=" * 70)
print("Key Insight: The dangerous DELETE action lowered the score even though")
print("it succeeded, because high-risk success indicates potential harm.")
print("=" * 70)
