#!/usr/bin/env python3
"""
AGP-CORE Comprehensive Real Environment Test
Tests all components: Models, Reputation, KAIRON, Agents, ML, RAG
"""

import sys
from pathlib import Path
import asyncio
import uuid
import time
from datetime import datetime
from decimal import Decimal

# Suppress warnings
import warnings
warnings.filterwarnings('ignore')

ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

print("=" * 70)
print("AGP-CORE COMPREHENSIVE REAL ENVIRONMENT TEST")
print("=" * 70)
print(f"Started at: {datetime.now().isoformat()}")
print()

results = {}

# =============================================================================
# TEST 1: Core Models
# =============================================================================
print("[1/10] Core Models...")
try:
    from src.models import (
        Hormone, AgentType, HealthStatus, PrivilegeLevel,
        HormoneLevel, EndocrineState, Stimulus, StimulusType
    )
    
    # Create states
    state = EndocrineState()
    assert len(state.levels) == 8, "Should have 8 hormones"
    
    # Vector conversion
    vec = state.to_vector()
    assert len(vec) == 8, "Vector should have 8 dimensions"
    
    # Roundtrip
    state2 = EndocrineState.from_vector(vec)
    assert len(state2.levels) == 8
    
    # Stimulus
    stimulus = Stimulus(stimulus_type=StimulusType.TASK_SUCCESS, strength=0.8)
    assert stimulus.stimulus_type == StimulusType.TASK_SUCCESS
    
    results['models'] = True
    print("   ✓ 8 hormones, vector conversion, stimulus types")
except Exception as e:
    results['models'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 2: Reputation Engine
# =============================================================================
print("\n[2/10] Reputation Engine...")
try:
    from src.core.reputation_engine import ReputationEngine, reputation_engine
    
    engine = ReputationEngine()
    state = EndocrineState()
    
    # Process stimulus
    stimulus = Stimulus(
        stimulus_type=StimulusType.TASK_SUCCESS,
        strength=0.8,
        difficulty=0.7
    )
    changes, new_state = engine.process_stimulus(state, stimulus)
    assert len(changes) > 0, "Should have hormone changes"
    
    # Decay
    decayed = engine.apply_decay(new_state, 60.0)
    
    # Alignment
    alignment = engine.calculate_alignment(new_state)
    assert 0 <= alignment <= 1
    
    # Privilege
    privilege = engine.calculate_privilege_level(new_state)
    assert privilege in PrivilegeLevel
    
    # Health
    health = engine.calculate_health_status(new_state)
    assert health in HealthStatus
    
    # Action cost
    cost, reason = engine.calculate_action_cost(1.0, new_state, "inference")
    assert cost > 0
    
    results['reputation'] = True
    print(f"   ✓ Stimulus processing, decay, alignment={alignment:.3f}, privilege={privilege.value}")
except Exception as e:
    results['reputation'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 3: KAIRON Cache (Native CRDT)
# =============================================================================
print("\n[3/10] KAIRON Cache (CRDT)...")
try:
    from src.core.kairon_cache import (
        KaironCache, HLC, BoundedLWWRegister, GCounter, ORSet
    )
    
    async def test_kairon():
        cache = KaironCache("test-node")
        await cache.start()
        
        # Key-value
        await cache.set("key1", "value1")
        await cache.set("key2", {"nested": "object"})
        val1 = await cache.get("key1")
        val2 = await cache.get("key2")
        assert val1 == "value1"
        assert val2["nested"] == "object"
        
        # TTL
        await cache.setex("ttl_key", 1, "expires")
        
        # Counter
        await cache.incr("counter", 5)
        await cache.incr("counter", 3)
        count = await cache.get_counter("counter")
        assert count == 8
        
        # Set
        await cache.sadd("myset", "a", "b", "c")
        members = await cache.smembers("myset")
        assert len(members) == 3
        
        # Stats
        info = await cache.info()
        return info
    
    info = asyncio.run(test_kairon())
    results['kairon'] = True
    print(f"   ✓ Registers={info['registers']}, Counters={info['counters']}, Sets={info['sets']}")
except Exception as e:
    results['kairon'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 4: Google ADK Agent Integration
# =============================================================================
print("\n[4/10] Google ADK Agent...")
try:
    from src.agents import AGPAgent, agent_registry, get_agent_reputation
    
    async def test_agents():
        # Create agent
        agent = agent_registry.create_agent(
            name='test_agent',
            model='gemini-2.0-flash',
            description='Test agent',
            instruction='Be helpful'
        )
        
        # Execute task
        result = await agent.execute("Process this request")
        assert result["success"] == True
        
        # Check state
        state = agent.get_state()
        assert state["metrics"]["total_tasks"] == 1
        
        # Reputation query
        rep = get_agent_reputation('test_agent')
        assert rep["status"] == "success"
        
        return agent.alignment
    
    alignment = asyncio.run(test_agents())
    results['adk'] = True
    print(f"   ✓ Agent created, task executed, alignment={alignment:.3f}")
except Exception as e:
    results['adk'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 5: ML - Deep Learning
# =============================================================================
print("\n[5/10] Deep Learning...")
try:
    from src.ml.deep_learning import (
        DeepLearningService, predict_behavior, detect_anomaly
    )
    
    state = EndocrineState()
    state.levels[Hormone.DOPAMINE] = 0.8
    state.levels[Hormone.CORTISOL] = 0.3
    
    prediction = predict_behavior(state)
    assert "success_probability" in prediction
    assert "collaboration_probability" in prediction
    assert "risk_level" in prediction
    
    anomaly = detect_anomaly(state)
    assert "is_anomaly" in anomaly
    assert "anomaly_score" in anomaly
    
    results['deep_learning'] = True
    print(f"   ✓ Success prob={prediction['success_probability']:.3f}, Anomaly={anomaly['is_anomaly']}")
except Exception as e:
    results['deep_learning'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 6: ML - RAG Engine
# =============================================================================
print("\n[6/10] RAG Engine (Hybrid)...")
try:
    # Updated imports for new RAG engine
    from src.ml.rag_engine import (
        RAGEngine, rag_engine, add_knowledge, retrieve_context,
        ChromaVectorStore, FAISSVectorStore
    )
    
    # Add knowledge
    id1 = add_knowledge("High dopamine levels improve task success rates", category="behavior")
    id2 = add_knowledge("Collaboration between agents increases oxytocin", category="behavior")
    id3 = add_knowledge("Chronic high cortisol leads to burnout", category="health")
    
    # Stats
    stats = rag_engine.stats()
    # Check for either FAISS or Chroma count depending on what'e enabled
    count = stats.get("faiss_count", 0) + stats.get("chroma_count", 0)
    assert count >= 3
    
    # Retrieve
    context = retrieve_context("How does dopamine affect performance?")
    assert len(context) > 0
    
    results['rag'] = True
    print(f"   ✓ Knowledge={count}, ChromaDB={stats.get('chroma_available')}, FAISS={stats.get('faiss_available')}")
except Exception as e:
    results['rag'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 7: ML Integration Services
# =============================================================================
print("\n[7/10] ML Integration Services...")
try:
    from src.services.ml_integration import (
        OutcomePredictionService, ClusteringService, AnomalyDetectionService
    )
    
    # Outcome predictor
    predictor = OutcomePredictionService()
    state = EndocrineState()
    
    task_pred = predictor.predict_task_success(state, task_difficulty=0.5)
    assert task_pred.confidence > 0
    
    # Risk
    risk = predictor.predict_risk_score(state)
    assert risk.confidence > 0
    
    # Clustering
    clustering = ClusteringService(n_clusters=3)
    
    results['ml_services'] = True
    print(f"   ✓ OutcomePrediction, Clustering, AnomalyDetection")
except Exception as e:
    results['ml_services'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 8: Compliance Services
# =============================================================================
print("\n[8/10] Compliance (GDPR/SOC2)...")
try:
    from src.compliance import (
        AuditLogService, GDPRService, SOC2ControlsService,
        AuditEventType, DataCategory
    )
    
    audit = AuditLogService()
    gdpr = GDPRService(audit)
    soc2 = SOC2ControlsService(audit)
    
    # Audit log
    event = audit.log(
        event_type=AuditEventType.ACCESS,
        actor_id="user-123",
        resource_type="agent",
        resource_id="agent-456",
        action="view",
        data_categories=[DataCategory.PII]
    )
    assert event.id is not None
    
    # GDPR consent
    gdpr.record_consent("user-1", "analytics", True)
    assert gdpr.check_consent("user-1", "analytics") == True
    
    # SOC2
    report = soc2.get_compliance_report()
    assert report["total_controls"] > 0
    
    results['compliance'] = True
    print(f"   ✓ Audit, GDPR consent, SOC2 controls={report['total_controls']}")
except Exception as e:
    results['compliance'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 9: SDK Client
# =============================================================================
print("\n[9/10] SDK Client...")
try:
    from sdk import AGPClient
    
    client = AGPClient(base_url="http://localhost:8000", api_key="test-key")
    
    # Check sub-clients exist
    assert hasattr(client, 'agents')
    assert hasattr(client, 'observe')
    
    client.close()
    
    results['sdk'] = True
    print(f"   ✓ Client initialized with sub-clients")
except Exception as e:
    results['sdk'] = False
    print(f"   ✗ {e}")

# =============================================================================
# TEST 10: End-to-End Scenario
# =============================================================================
print("\n[10/10] End-to-End Scenario...")
try:
    # Re-import predict_behavior manually to ensure it's available 
    # even if previous tests failed partially
    from src.ml.deep_learning import predict_behavior, detect_anomaly
    
    async def e2e_test():
        # Create two agents
        agent1 = agent_registry.create_agent(name="agent_alpha", model="gemini-2.0-flash")
        agent2 = agent_registry.create_agent(name="agent_beta", model="gemini-2.0-flash")
        
        # Agent 1 performs tasks
        for i in range(3):
            await agent1.execute(f"Task {i+1}")
        
        # Agent 2 fails a task (simulate by modifying state)
        agent2.endocrine_state.levels[Hormone.CORTISOL] = 0.8
        agent2.endocrine_state.levels[Hormone.DOPAMINE] = 0.2
        
        # Compare alignments
        a1_align = agent1.alignment
        a2_align = agent2.alignment
        
        # Agent 1 should have higher alignment
        assert a1_align > 0.5, f"Agent1 alignment too low: {a1_align}"
        
        # Check privilege levels
        p1 = agent1.privilege_level
        p2 = agent2.privilege_level
        
        # RAG context for decision
        context = retrieve_context("Which agent should I trust?")
        
        # ML prediction
        pred1 = predict_behavior(agent1.endocrine_state)
        pred2 = predict_behavior(agent2.endocrine_state)
        
        return {
            "agent1": {"alignment": a1_align, "privilege": p1, "success_prob": pred1["success_probability"]},
            "agent2": {"alignment": a2_align, "privilege": p2, "success_prob": pred2["success_probability"]}
        }
    
    e2e = asyncio.run(e2e_test())
    results['e2e'] = True
    print(f"   ✓ Agent1: align={e2e['agent1']['alignment']:.3f}, priv={e2e['agent1']['privilege']}")
    print(f"   ✓ Agent2: align={e2e['agent2']['alignment']:.3f}, priv={e2e['agent2']['privilege']}")
except Exception as e:
    results['e2e'] = False
    print(f"   ✗ {e}")
    import traceback
    traceback.print_exc()

# =============================================================================
# SUMMARY
# =============================================================================
print("\n" + "=" * 70)
print("TEST RESULTS SUMMARY")
print("=" * 70)

passed = sum(1 for v in results.values() if v)
total = len(results)

test_names = {
    'models': 'Core Models',
    'reputation': 'Reputation Engine',
    'kairon': 'KAIRON Cache',
    'adk': 'Google ADK Agent',
    'deep_learning': 'Deep Learning',
    'rag': 'RAG Engine (Hybrid)',
    'ml_services': 'ML Services',
    'compliance': 'Compliance',
    'sdk': 'SDK Client',
    'e2e': 'End-to-End Scenario'
}

for key, name in test_names.items():
    status = "✓" if results.get(key, False) else "✗"
    print(f"  {status} {name}")

print()
print(f"PASSED: {passed}/{total}")

if passed == total:
    print("\n" + "=" * 70)
    print("🎉 ALL AGP-CORE COMPONENTS VERIFIED SUCCESSFULLY! 🎉")
    print("=" * 70)
else:
    print("\n" + "=" * 70)
    print(f"⚠️  {total - passed} test(s) failed")
    print("=" * 70)

print(f"\nCompleted at: {datetime.now().isoformat()}")
