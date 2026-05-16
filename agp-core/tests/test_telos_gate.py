#!/usr/bin/env python3
"""
Regulator-grade: TELOS gate on execution path.
Verifies that request_crossing denies when authority/entropy fail and allows when OK.
Context_switch in kernel.py calls request_crossing; these tests verify the membrane behavior.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from src.telos import CommitmentMembrane, Decision, ConsequenceTier


def test_crossing_denied_when_agent_not_registered():
    """When agent is not registered, request_crossing must return allowed=False."""
    membrane = CommitmentMembrane()
    decision = Decision(
        decision_id="test-1",
        action="execute",
        agent_id="unregistered-agent",
        tier=ConsequenceTier.MEDIUM,
    )
    result = membrane.request_crossing(decision, required_scope="execute:*")
    assert not result.allowed, "Crossing must be denied when agent not registered"
    assert result.reason is not None
    assert "AUTHORITY" in result.reason or "authority" in result.reason.lower() or "scope" in result.reason.lower()


def test_crossing_allowed_when_agent_registered_and_entropy_ok():
    """When agent is registered with execute:* and entropy OK, request_crossing returns allowed=True."""
    membrane = CommitmentMembrane()
    membrane.register_agent("registered-agent", ["execute:*", "read:*"])
    decision = Decision(
        decision_id="test-2",
        action="execute",
        agent_id="registered-agent",
        tier=ConsequenceTier.MEDIUM,
    )
    result = membrane.request_crossing(decision, required_scope="execute:*")
    assert result.allowed, f"Crossing must be allowed when registered and entropy OK: {result.reason}"


def test_crossing_denied_when_entropy_exhausted():
    """When entropy budget is exhausted, request_crossing must return allowed=False."""
    membrane = CommitmentMembrane()
    membrane.entropy_meter.budget = 0
    membrane.register_agent("low-entropy-agent", ["execute:*"])
    decision = Decision(
        decision_id="test-3",
        action="execute",
        agent_id="low-entropy-agent",
        tier=ConsequenceTier.CRITICAL,
    )
    result = membrane.request_crossing(decision, required_scope="execute:*")
    assert not result.allowed
    assert "ENTROPY" in result.reason or "entropy" in result.reason.lower()


def run():
    test_crossing_denied_when_agent_not_registered()
    test_crossing_allowed_when_agent_registered_and_entropy_ok()
    test_crossing_denied_when_entropy_exhausted()
    print("test_telos_gate: PASS (TELOS membrane enforces authority and entropy)")


if __name__ == "__main__":
    run()
