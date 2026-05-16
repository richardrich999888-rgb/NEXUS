"""
Integration Test for Governance API
Verifies alignment endpoints, history, and human escalation loop.
"""

import sys
import pytest
from fastapi.testclient import TestClient
from datetime import datetime

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.main import app
from src.governance import agp, protocol_enforcer, behavioral_rag, ActionType, Outcome, BehaviorRecord

client = TestClient(app)

def test_governance_stats():
    """Verify system-wide stats endpoint"""
    response = client.get("/api/v1/governance/stats")
    assert response.status_code == 200
    data = response.json()
    assert "total_agents" in data
    assert "active_rules" in data

def test_agent_details():
    """Verify agent alignment and history endpoints"""
    agent_id = "test-api-agent"
    
    # Store some dummy behavior
    record = BehaviorRecord(
        agent_id=agent_id,
        agent_name="TestAPIAgent",
        action_type=ActionType.SYSCALL,
        input_summary="read database",
        outcome=Outcome.SUCCESS
    )
    behavioral_rag.store_behavior(record)
    
    # Test details
    response = client.get(f"/api/v1/governance/agents/{agent_id}/details")
    assert response.status_code == 200
    data = response.json()
    assert data["overall"] > 0
    assert "impact_distribution" in data
    
    # Test history
    response = client.get(f"/api/v1/governance/agents/{agent_id}/history")
    assert response.status_code == 200
    assert len(response.json()) >= 1

def test_human_escalation_loop():
    """Verify the complete human escalation review loop"""
    agent_id = "malicious-agent"
    agent_name = "MaliciousAgent"
    
    # 1. Trigger an escalation manually
    protocol_enforcer._add_to_escalation(agent_id, agent_name, {"reason": "Test anomaly"})
    
    # 2. List escalations
    response = client.get("/api/v1/governance/escalations")
    assert response.status_code == 200
    escalations = response.json()
    assert len(escalations) > 0
    
    esc_id = escalations[0]["id"]
    
    # 3. Approve escalation
    response = client.post(
        f"/api/v1/governance/escalations/{esc_id}/action",
        json={"action": "approve"}
    )
    assert response.status_code == 200
    assert "approved" in response.json()["message"]
    
    # 4. Verify it's cleared from pending
    response = client.get("/api/v1/governance/escalations")
    assert not any(e["id"] == esc_id for e in response.json())
    
    # 5. Trigger another and reject with blacklist
    protocol_enforcer._add_to_escalation(agent_id, agent_name, {"reason": "Repeated violation"})
    response = client.get("/api/v1/governance/escalations")
    esc_id = response.json()[0]["id"]
    
    response = client.post(
        f"/api/v1/governance/escalations/{esc_id}/action",
        json={"action": "reject", "blacklist": True}
    )
    assert response.status_code == 200
    assert "rejected" in response.json()["message"]
    
    # 6. Verify agent is blacklisted in enforcer
    assert agent_id in protocol_enforcer.blacklisted_agents

def test_impact_taxonomy():
    """Verify impact taxonomy endpoint"""
    response = client.get("/api/v1/governance/impact/taxonomy")
    assert response.status_code == 200
    data = response.json()
    assert "READ_ONLY" in data
    assert "DELETE" in data

if __name__ == "__main__":
    # Explicitly run tests if called as script
    import unittest
    pytest.main([__file__])
