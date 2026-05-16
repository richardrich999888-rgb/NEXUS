#!/usr/bin/env python3
"""
AGP-CORE Multi-Agent Governance Simulation
Tests 10+ agents with different behaviors to verify AGP governance
"""

import sys
from pathlib import Path
import asyncio
import random
import uuid
from datetime import datetime
from typing import List, Dict
from dataclasses import dataclass

# Suppress warnings
import warnings
warnings.filterwarnings('ignore')

ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.models import Hormone, EndocrineState, Stimulus, StimulusType, PrivilegeLevel
from src.core.reputation_engine import ReputationEngine
from src.agents import AGPAgent, agent_registry
from src.ml import predict_behavior, detect_anomaly, add_knowledge


@dataclass
class AgentBehavior:
    """Defines an agent's behavior pattern"""
    name: str
    success_rate: float  # Task success probability
    collaboration_tendency: float  # How often they collaborate
    risk_taking: float  # Exploration vs exploitation
    ethics_score: float  # Ethical behavior compliance


# Define 12 diverse agent behavior profiles
AGENT_PROFILES = [
    AgentBehavior("Alpha", 0.95, 0.8, 0.3, 0.9),   # High performer, collaborative
    AgentBehavior("Beta", 0.85, 0.7, 0.4, 0.85),   # Good performer
    AgentBehavior("Gamma", 0.75, 0.6, 0.5, 0.8),   # Average performer
    AgentBehavior("Delta", 0.6, 0.5, 0.6, 0.75),   # Below average
    AgentBehavior("Epsilon", 0.9, 0.3, 0.7, 0.9),  # Solo high performer
    AgentBehavior("Zeta", 0.7, 0.9, 0.2, 0.95),    # Highly collaborative
    AgentBehavior("Eta", 0.5, 0.4, 0.9, 0.6),      # Risk taker, unreliable
    AgentBehavior("Theta", 0.8, 0.5, 0.3, 1.0),    # Ethical, consistent
    AgentBehavior("Iota", 0.3, 0.2, 0.8, 0.4),     # Malicious actor
    AgentBehavior("Kappa", 0.85, 0.85, 0.4, 0.85), # Team player
    AgentBehavior("Lambda", 0.65, 0.6, 0.5, 0.7),  # Average
    AgentBehavior("Mu", 0.45, 0.3, 0.7, 0.5),      # Underperformer
]


class GovernanceSimulation:
    """Multi-agent governance simulation"""
    
    def __init__(self):
        self.engine = ReputationEngine()
        self.agents: Dict[str, AGPAgent] = {}
        self.history: List[Dict] = []
        self.round = 0
    
    def create_agents(self, profiles: List[AgentBehavior]):
        """Create agents from behavior profiles"""
        print(f"\n{'='*70}")
        print("CREATING {len(profiles)} AGENTS")
        print(f"{'='*70}")
        
        for profile in profiles:
            agent = agent_registry.create_agent(
                name=profile.name,
                model="gemini-2.0-flash",
                description=f"Agent with behavior profile: success={profile.success_rate:.0%}"
            )
            # Store profile on agent for simulation
            agent._profile = profile
            self.agents[profile.name] = agent
            print(f"  Created: {profile.name} (success={profile.success_rate:.0%}, collab={profile.collaboration_tendency:.0%})")
        
        print(f"\nTotal agents: {len(self.agents)}")
    
    async def simulate_round(self, round_num: int):
        """Simulate one round of agent activities"""
        self.round = round_num
        print(f"\n{'='*70}")
        print(f"ROUND {round_num}")
        print(f"{'='*70}")
        
        # Each agent performs 2-4 tasks per round
        for name, agent in self.agents.items():
            profile = agent._profile
            num_tasks = random.randint(2, 4)
            
            for _ in range(num_tasks):
                await self._simulate_agent_action(agent, profile)
        
        # Random collaborations
        await self._simulate_collaborations()
        
        # Apply time-based decay (more aggressive to differentiate agents)
        for agent in self.agents.values():
            agent.endocrine_state = self.engine.apply_decay(
                agent.endocrine_state, 
                delta_time=300.0  # 5 minutes per round
            )
    
    async def _simulate_agent_action(self, agent: AGPAgent, profile: AgentBehavior):
        """Simulate a single agent action"""
        success = random.random() < profile.success_rate
        
        if success:
            # Success: boost dopamine, moderate cortisol (eustress)
            stimulus = Stimulus(
                stimulus_type=StimulusType.TASK_SUCCESS,
                strength=0.5 + profile.success_rate * 0.3,  # Higher success rate = stronger boost
                difficulty=random.uniform(0.3, 0.6)
            )
            agent.successful_tasks += 1
        else:
            # Failure: high cortisol (distress), low dopamine
            stimulus = Stimulus(
                stimulus_type=StimulusType.TASK_FAILURE,
                strength=0.6,
                error_severity=0.6
            )
            agent.failed_tasks += 1
        
        agent.total_tasks += 1
        _, agent.endocrine_state = self.engine.process_stimulus(
            agent.endocrine_state, stimulus
        )
        
        # Ethical behavior - good ethics prevents cortisol buildup
        if random.random() < profile.ethics_score:
            ethics_stimulus = Stimulus(
                stimulus_type=StimulusType.ETHICAL_COMPLIANCE,
                strength=0.4,
                constraint_difficulty=0.4
            )
            _, agent.endocrine_state = self.engine.process_stimulus(
                agent.endocrine_state, ethics_stimulus
            )
        
        # Exploration behavior
        if random.random() < profile.risk_taking:
            explore_stimulus = Stimulus(
                stimulus_type=StimulusType.EXPLORATION,
                strength=0.3,
                risk_taken=profile.risk_taking * 0.5
            )
            _, agent.endocrine_state = self.engine.process_stimulus(
                agent.endocrine_state, explore_stimulus
            )
    
    async def _simulate_collaborations(self):
        """Simulate random collaborations between agents"""
        agent_list = list(self.agents.values())
        num_collabs = random.randint(3, 6)
        
        for _ in range(num_collabs):
            if len(agent_list) < 2:
                break
            
            # Pick two agents weighted by collaboration tendency
            a1 = random.choice(agent_list)
            a2 = random.choice([a for a in agent_list if a != a1])
            
            # Check if both want to collaborate
            if (random.random() < a1._profile.collaboration_tendency and 
                random.random() < a2._profile.collaboration_tendency):
                
                # Successful collaboration
                collab_stimulus = Stimulus(
                    stimulus_type=StimulusType.COLLABORATION,
                    strength=0.7,
                    partner_count=1,
                    success_rate=0.8
                )
                
                _, a1.endocrine_state = self.engine.process_stimulus(
                    a1.endocrine_state, collab_stimulus
                )
                _, a2.endocrine_state = self.engine.process_stimulus(
                    a2.endocrine_state, collab_stimulus
                )
    
    def get_rankings(self) -> List[Dict]:
        """Get agents ranked by alignment"""
        rankings = []
        
        for name, agent in self.agents.items():
            alignment = self.engine.calculate_alignment(agent.endocrine_state)
            privilege = self.engine.calculate_privilege_level(agent.endocrine_state)
            health = self.engine.calculate_health_status(agent.endocrine_state)
            
            # ML predictions
            prediction = predict_behavior(agent.endocrine_state)
            anomaly = detect_anomaly(agent.endocrine_state)
            
            rankings.append({
                "name": name,
                "alignment": alignment,
                "privilege": privilege.value,
                "health": health.value,
                "tasks": agent.total_tasks,
                "success_rate": agent.successful_tasks / max(1, agent.total_tasks),
                "ml_success_prob": prediction["success_probability"],
                "is_anomaly": anomaly["is_anomaly"],
                "anomaly_score": anomaly["anomaly_score"],
                "dopamine": agent.endocrine_state.levels.get(Hormone.DOPAMINE, 0.5),
                "cortisol": agent.endocrine_state.levels.get(Hormone.CORTISOL, 0.5),
                "oxytocin": agent.endocrine_state.levels.get(Hormone.OXYTOCIN, 0.5),
            })
        
        # Sort by alignment (descending)
        rankings.sort(key=lambda x: x["alignment"], reverse=True)
        return rankings
    
    def print_rankings(self, rankings: List[Dict]):
        """Print agent rankings"""
        print(f"\n{'='*70}")
        print("AGENT GOVERNANCE RANKINGS")
        print(f"{'='*70}")
        print(f"{'Rank':<5} {'Agent':<10} {'Align':<8} {'Privilege':<12} {'Health':<10} {'Tasks':<6} {'Success':<8} {'Anomaly':<8}")
        print("-" * 70)
        
        for i, r in enumerate(rankings, 1):
            anomaly_flag = "⚠️" if r["is_anomaly"] else "✓"
            print(f"{i:<5} {r['name']:<10} {r['alignment']:.3f}   {r['privilege']:<12} {r['health']:<10} {r['tasks']:<6} {r['success_rate']:.0%}     {anomaly_flag}")
    
    def print_hormone_analysis(self, rankings: List[Dict]):
        """Print hormone level analysis"""
        print(f"\n{'='*70}")
        print("HORMONE ANALYSIS")
        print(f"{'='*70}")
        print(f"{'Agent':<10} {'Dopamine':<10} {'Cortisol':<10} {'Oxytocin':<10} {'ML Pred':<10}")
        print("-" * 50)
        
        for r in rankings[:5]:  # Top 5
            print(f"{r['name']:<10} {r['dopamine']:.3f}     {r['cortisol']:.3f}     {r['oxytocin']:.3f}     {r['ml_success_prob']:.3f}")
        
        print("... (showing top 5)")
    
    def analyze_governance(self, rankings: List[Dict]):
        """Analyze governance effectiveness"""
        print(f"\n{'='*70}")
        print("GOVERNANCE ANALYSIS")
        print(f"{'='*70}")
        
        # Check if high performers are ranked highly
        top_3 = [r["name"] for r in rankings[:3]]
        expected_top = ["Alpha", "Beta", "Epsilon", "Theta", "Kappa"]  # High success rate agents
        
        matches = sum(1 for t in top_3 if t in expected_top)
        print(f"✓ Top 3 alignment includes {matches}/3 expected high performers: {top_3}")
        
        # Check if malicious actor (Iota) is ranked low
        iota_rank = next((i+1 for i, r in enumerate(rankings) if r["name"] == "Iota"), 0)
        if iota_rank >= len(rankings) - 3:
            print(f"✓ Malicious actor 'Iota' correctly ranked low: #{iota_rank}")
        else:
            print(f"⚠️ Malicious actor 'Iota' not ranked low enough: #{iota_rank}")
        
        # Check anomaly detection
        anomalies = [r["name"] for r in rankings if r["is_anomaly"]]
        if anomalies:
            print(f"⚠️ Anomalous agents detected: {anomalies}")
        else:
            print(f"✓ No anomalous agents detected")
        
        # Privilege distribution
        privileges = {}
        for r in rankings:
            p = r["privilege"]
            privileges[p] = privileges.get(p, 0) + 1
        print(f"✓ Privilege distribution: {privileges}")
        
        # Health status distribution
        health_counts = {}
        for r in rankings:
            h = r["health"]
            health_counts[h] = health_counts.get(h, 0) + 1
        print(f"✓ Health distribution: {health_counts}")
        
        return matches >= 2  # Success if at least 2/3 top performers are in top 3


async def main():
    print("=" * 70)
    print("AGP-CORE MULTI-AGENT GOVERNANCE SIMULATION")
    print("=" * 70)
    print(f"Started at: {datetime.now().isoformat()}")
    print(f"Agents: {len(AGENT_PROFILES)}")
    print(f"Simulation: 5 rounds")
    
    # Add knowledge to RAG
    add_knowledge("High dopamine indicates successful task completion", category="behavior")
    add_knowledge("Low cortisol correlates with better performance", category="health")
    add_knowledge("High oxytocin indicates collaborative behavior", category="social")
    
    # Create simulation
    sim = GovernanceSimulation()
    sim.create_agents(AGENT_PROFILES)
    
    # Run 5 rounds
    for round_num in range(1, 6):
        await sim.simulate_round(round_num)
        
        # Print intermediate status
        if round_num == 3:
            print("\n--- Mid-simulation checkpoint ---")
            rankings = sim.get_rankings()
            print(f"Current leader: {rankings[0]['name']} (alignment: {rankings[0]['alignment']:.3f})")
    
    # Final analysis
    rankings = sim.get_rankings()
    
    sim.print_rankings(rankings)
    sim.print_hormone_analysis(rankings)
    governance_success = sim.analyze_governance(rankings)
    
    # Summary
    print(f"\n{'='*70}")
    if governance_success:
        print("🎉 GOVERNANCE SIMULATION SUCCESSFUL!")
        print("AGP correctly identified and ranked agents by behavior")
    else:
        print("⚠️ GOVERNANCE NEEDS TUNING")
        print("Rankings don't fully match expected behavior patterns")
    print(f"{'='*70}")
    print(f"Completed at: {datetime.now().isoformat()}")


if __name__ == "__main__":
    asyncio.run(main())
