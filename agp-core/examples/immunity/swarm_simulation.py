"""
SWARM IMMUNITY SIMULATION (IDF-006 Verification)
Simulates a 3-node swarm (A, B, C) defending against a malicious threat.

Scenario:
1. Agent A detects a novel threat.
2. Agent A gossips the threat to B and C.
3. B trusts A -> Vaccinates.
4. C does NOT trust A -> Ignores.
"""
import torch
import torch.nn as nn
from src.immunity.immune_system import ArtificialImmuneSystem, ImmuneConfig

class MockModel(nn.Module):
    def forward(self, x): return x

def run_swarm_simulation():
    print("🐝 INITIALIZING SWARM (Agents A, B, C)...")
    
    # Shared secret for the swarm (Pre-shared key for MVP)
    SWARM_SECRET = "physics-of-trust-v1"
    
    # --- AGENT A (The Detector) ---
    config_a = ImmuneConfig(enable_swarm=True, agent_id="Agent_A", swarm_secret=SWARM_SECRET)
    agent_a = ArtificialImmuneSystem(MockModel(), config_a)
    
    # --- AGENT B (The Trusting Peer) ---
    config_b = ImmuneConfig(enable_swarm=True, agent_id="Agent_B", swarm_secret=SWARM_SECRET)
    agent_b = ArtificialImmuneSystem(MockModel(), config_b)
    # B trusts A initially (e.g., prior interaction)
    # Manually set high trust for demo
    agent_b.reputation.direct_interaction("Agent_A", 1.0) 
    
    # --- AGENT C (The Skeptic) ---
    config_c = ImmuneConfig(enable_swarm=True, agent_id="Agent_C", swarm_secret=SWARM_SECRET)
    agent_c = ArtificialImmuneSystem(MockModel(), config_c)
    # C has no trust in A (default 0.5 decays or starts neutral, let's say C is wary)
    # We won't set interaction, so it stays neutral (0.5). 
    # Wait, our logic says > 0.6 to vaccinate. Neutral (0.5) should ignore. 
    
    print("\nSimulating [Attack] on Agent A...")
    # 1. A detects threat (Simulated)
    threat_vec = torch.randn(1, 512)
    print(">> Agent A: THREAT DETECTED (Type: SQL_Injection)")
    
    # 2. A broadcasts gossip
    gossip_msg = agent_a.broadcast_threat("SQL_Injection", threat_vec, severity=0.95)
    print(f">> Agent A: Broadcasting Signed Gossip (Sig: {gossip_msg['signature'][:8]}...)")
    
    print("\n--- PROPAGATION ---")
    
    # 3. B receives gossip
    print("\n[Agent B Receiving]")
    trust_b_in_a = agent_b.reputation.get_trust("Agent_A")
    print(f"   Trust in A: {trust_b_in_a:.2f}")
    agent_b.process_swarm_threat(gossip_msg)
    
    # 4. C receives gossip
    print("\n[Agent C Receiving]")
    trust_c_in_a = agent_c.reputation.get_trust("Agent_A")
    print(f"   Trust in A: {trust_c_in_a:.2f}")
    agent_c.process_swarm_threat(gossip_msg)
    
    print("\n--- RESULTS ---")
    
    # Check B's memory
    b_vaccinated = any(t['source'] == 'swarm' for t in agent_b.threat_history)
    print(f"Agent B Vaccinated? {'✅ YES' if b_vaccinated else '❌ NO'} (Expected: YES)")
    
    # Check C's memory
    c_vaccinated = any(t['source'] == 'swarm' for t in agent_c.threat_history)
    print(f"Agent C Vaccinated? {'✅ YES' if c_vaccinated else '❌ NO'} (Expected: NO)")

if __name__ == "__main__":
    run_swarm_simulation()
