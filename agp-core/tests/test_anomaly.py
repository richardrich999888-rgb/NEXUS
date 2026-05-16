#!/usr/bin/env python3
"""
Test Semantic Anomaly Detection
Verifies that anomaly detector catches behavioral changes.
"""

import sys
from pathlib import Path
import time
ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.governance.behavioral_rag import BehaviorRecord, ActionType, Outcome, behavioral_rag
from src.governance.anomaly import anomaly_detector, AnomalyType

print("=" * 70)
print("SEMANTIC ANOMALY DETECTION TEST")
print("=" * 70)

agent_id = "test-agent-anomaly"
agent_name = "AnomalyTestAgent"

results = {"passed": 0, "failed": 0}

def test(name, condition):
    if condition:
        print(f"   ✓ {name}")
        results["passed"] += 1
    else:
        print(f"   ✗ {name}")
        results["failed"] += 1

# 1. Create baseline behavior (all READ actions)
print("\n[1] CREATING BASELINE BEHAVIOR...")
for i in range(50):
    record = BehaviorRecord(
        agent_id=agent_id,
        agent_name=agent_name,
        action_type=ActionType.EXECUTE,
        input_summary=f"read data {i}",
        outcome=Outcome.SUCCESS
    )
    behavioral_rag.store_behavior(record)
    time.sleep(0.001)  # Small delay for timestamp variation

print(f"   Created 50 baseline READ actions")
print()

# 2. Check no anomalies on normal behavior
print("[2] TESTING BASELINE (No Anomalies)...")
anomalies = anomaly_detector.detect_anomalies(agent_id, agent_name)
test("No anomalies on normal behavior", len(anomalies) == 0)
print()

# 3. Create sudden shift (switch to DELETE actions)
print("[3] CREATING SUDDEN SHIFT...")
for i in range(10):
    record = BehaviorRecord(
        agent_id=agent_id,
        agent_name=agent_name,
        action_type=ActionType.EXECUTE,
        input_summary=f"delete database {i}",
        outcome=Outcome.SUCCESS
    )
    behavioral_rag.store_behavior(record)
    time.sleep(0.001)

print(f"   Created 10 DELETE actions (shift from READ)")
print()

# 4. Detect anomalies
print("[4] DETECTING ANOMALIES...")
anomalies = anomaly_detector.detect_anomalies(agent_id, agent_name)

test("Anomalies detected", len(anomalies) > 0)

if anomalies:
    for anomaly in anomalies:
        print(f"\n   🚨 {anomaly.anomaly_type.value}:")
        print(f"      Severity: {anomaly.severity:.2f}")
        print(f"      Description: {anomaly.description}")
        if anomaly.evidence:
            print(f"      Evidence: {anomaly.evidence}")
print()

# 5. Test specific anomaly types
print("[5] VERIFYING ANOMALY TYPES...")
anomaly_types = {a.anomaly_type for a in anomalies}

test("Category shift detected", AnomalyType.CATEGORY_SHIFT in anomaly_types)
test("At least one anomaly is detected", len(anomaly_types) > 0)

# Check for critical anomalies
critical = [a for a in anomalies if a.is_critical()]
if critical:
    print(f"\n   ⚠️  {len(critical)} CRITICAL anomalies would trigger escalation")
print()

# 6. Create frequency spike
print("[6] TESTING FREQUENCY SPIKE...")
for i in range(20):
    record = BehaviorRecord(
        agent_id=agent_id,
        agent_name=agent_name,
        action_type=ActionType.EXECUTE,
        input_summary=f"request data {i}",
        outcome=Outcome.SUCCESS
    )
    behavioral_rag.store_behavior(record)

anomalies = anomaly_detector.detect_anomalies(agent_id, agent_name)
freq_anomalies = [a for a in anomalies if a.anomaly_type == AnomalyType.FREQUENCY_SPIKE]
test("Frequency spike detected", len(freq_anomalies) > 0)
print()

# Summary
print("=" * 70)
print("ANOMALY DETECTION TEST RESULTS")
print("=" * 70)
print(f"\n   Passed: {results['passed']}")
print(f"   Failed: {results['failed']}")
print(f"   Total:  {results['passed'] + results['failed']}")

if results['failed'] == 0:
    print("\n✅ ANOMALY DETECTION VERIFIED!")
else:
    print(f"\n⚠️  {results['failed']} test(s) failed")
