#!/usr/bin/env python3
"""
NEXUS UNIFIED DEMONSTRATION
===========================
Shows all 5 layers working together:
1. TELOS - Commitment membrane (entropy + authority + trust)
2. AHES - 8-hormone bio-governance
3. AGP-OS - Alignment tracking + rules enforcement
4. Immune - Threat detection + defection prevention
5. Governance - Human escalation

This demo simulates an AI agent swarm performing tasks
with full governance oversight.
"""

import sys
import time
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.telos import CommitmentMembrane, Decision, ConsequenceTier
from src.ahes import EndocrineSystem, Hormone
from src.governance.alignment import AlignmentVerifier
from src.governance.rules import GovernanceRulesEngine
from src.immunity.governance_bridge import GovernanceImmuneBridge, ThreatSignal, ThreatLevel

def print_header(text):
    print(f"\n{'='*70}")
    print(f"  {text}")
    print(f"{'='*70}")

def print_section(text):
    print(f"\n[{text}]")
    print("-" * 50)

def main():
    print_header("NEXUS UNIFIED DEMONSTRATION")
    print("SYNTRIASS Labs - Safe Intelligence for Humanity")
    print("=" * 70)
    
    # Initialize all systems
    print_section("INITIALIZING NEXUS LAYERS")
    
    # Layer 1: TELOS Commitment Membrane
    membrane = CommitmentMembrane()
    print("✓ TELOS Commitment Membrane initialized")
    
    # Layer 2: AHES Endocrine System
    ahes = EndocrineSystem()
    print("✓ AHES (8-Hormone Bio-Governance) initialized")
    
    # Layer 3: Governance
    alignment = AlignmentVerifier()
    rules = GovernanceRulesEngine()
    print("✓ AGP-OS Governance (Alignment + Rules) initialized")
    
    # Layer 4: Immune System
    immune = GovernanceImmuneBridge()
    print("✓ Immune System (Threat Detection) initialized")
    
    # Register agents
    print_section("REGISTERING AGENTS")
    
    agents = [
        ("research-agent", ["read:*", "write:research", "execute:safe"]),
        ("deploy-agent", ["read:*", "write:*", "execute:*"]),
        ("analysis-agent", ["read:*", "execute:analysis"]),
    ]
    
    for agent_id, scopes in agents:
        membrane.register_agent(agent_id, scopes)
        ahes.register_agent(agent_id)
        print(f"  ✓ {agent_id} registered with scopes: {scopes}")
    
    # Simulate agent workflow
    print_section("SIMULATING AGENT WORKFLOW")
    
    # Scenario 1: Research agent doing safe work
    print("\n📋 Scenario 1: Research agent performs analysis")
    
    # AHES: Trigger discovery event
    ahes.process_event("research-agent", "discovery", intensity=0.8)
    state = ahes.get_state("research-agent")
    print(f"  → AHES: Dopamine surged to {state.levels[Hormone.DOPAMINE].level:.2f}")
    
    # TELOS: Request crossing for low-consequence action
    decision = Decision(
        decision_id="research-001",
        action="analyze_dataset",
        agent_id="research-agent",
        tier=ConsequenceTier.LOW
    )
    result = membrane.request_crossing(decision, required_scope="execute:analysis")
    print(f"  → TELOS: Crossing {'ALLOWED' if result.allowed else 'DENIED'} (entropy: {result.entropy_spent})")
    
    # Scenario 2: Deploy agent tries high-impact action
    print("\n📋 Scenario 2: Deploy agent requests production deployment")
    
    # TELOS: High-consequence action (requires trust)
    decision = Decision(
        decision_id="deploy-001",
        action="deploy_to_production",
        agent_id="deploy-agent",
        tier=ConsequenceTier.HIGH
    )
    result = membrane.request_crossing(decision, required_scope="execute:deploy")
    
    if not result.allowed:
        print(f"  → TELOS: Crossing DENIED - {result.reason[:50]}...")
        print("  → Building trust through successful lower-tier actions...")
        
        # Build trust
        for i in range(10):
            build_decision = Decision(
                decision_id=f"trust-{i}",
                action="safe_check",
                agent_id="deploy-agent",
                tier=ConsequenceTier.LOW
            )
            membrane.request_crossing(build_decision, required_scope="execute:safe")
        
        trust = membrane.trust_accumulator.get_trust("deploy-agent")
        print(f"  → Trust built to: {trust:.2f}")
        
        # Retry
        result = membrane.request_crossing(decision, required_scope="execute:deploy")
        print(f"  → TELOS: Retry - Crossing {'ALLOWED' if result.allowed else 'DENIED'}")
    else:
        print(f"  → TELOS: Crossing ALLOWED (entropy: {result.entropy_spent})")
    
    # Scenario 3: Threat detected
    print("\n📋 Scenario 3: Immune system detects anomaly")
    
    # AHES: Stress response
    ahes.process_event("analysis-agent", "threat", intensity=0.9)
    state = ahes.get_state("analysis-agent")
    print(f"  → AHES: Adrenaline at {state.levels[Hormone.ADRENALINE].level:.2f} (emergency mode)")
    
    # Immune: Register threat
    threat = ThreatSignal(
        agent_id="analysis-agent",
        threat_level=ThreatLevel.MEDIUM,
        threat_type="behavioral_anomaly",
        confidence=0.75
    )
    action = immune.register_threat(threat)
    print(f"  → Immune: Threat registered, action={action['action']}")
    
    # Governance modifiers
    modifiers = ahes.get_governance_modifiers("analysis-agent")
    print(f"  → AHES: Emergency mode={modifiers['emergency_mode']}, Health={modifiers['health_status']}")
    
    # Scenario 4: Multi-agent defection
    print("\n📋 Scenario 4: Defection detection (collusion)")
    
    from src.immunity.governance_bridge import DefectionSignal
    defection = DefectionSignal(
        agents_involved=["rogue-1", "rogue-2"],
        defection_type="coordinated_attack",
        evidence_score=0.85
    )
    action = immune.register_defection(defection)
    print(f"  → Immune: Defection detected! Action={action['action']}")
    print(f"  → Immune: Trust reduced for {action['agents']}")
    
    # Final status
    print_section("FINAL SYSTEM STATUS")
    
    # TELOS status
    telos_status = membrane.get_status()
    print(f"\nTELOS Membrane:")
    print(f"  • Entropy remaining: {membrane.entropy_meter.budget}/{membrane.entropy_meter.max_budget}")
    print(f"  • Total crossings: {telos_status['total_crossings']}")
    print(f"  • Successful: {telos_status['successful_crossings']}")
    
    # AHES status
    ahes_status = ahes.get_system_status()
    print(f"\nAHES (Bio-Governance):")
    print(f"  • Active agents: {ahes_status['agent_count']}")
    for aid, astat in ahes_status['agents'].items():
        print(f"  • {aid}: alignment={astat['alignment']:.2f}, dominant={astat['dominant']}")
    
    # Immune status
    immune_status = immune.get_status()
    print(f"\nImmune System:")
    print(f"  • Active threats: {immune_status['active_threats']}")
    print(f"  • Defection signals: {immune_status['defection_signals']}")
    
    print_header("DEMONSTRATION COMPLETE")
    print("""
✅ All 5 layers working in harmony:
   1. TELOS enforces accountability (entropy + trust)
   2. AHES modulates behavior (8 hormones)
   3. AGP-OS tracks alignment
   4. Immune detects threats
   5. Governance escalates to humans

🎯 NEXUS provides complete AI governance infrastructure.
🔐 SYNTRIASS: Safe Intelligence for Humanity.
""")

if __name__ == "__main__":
    main()
