#!/usr/bin/env python3
"""
AGP-CORE Production Orchestration Verification
Simulates a Startup Team (Growth, Tech, Product) executing a sprint under AGP Governance.
"""

import sys
from pathlib import Path
import asyncio
import structlog
from datetime import datetime

# Suppress warnings
import warnings
warnings.filterwarnings('ignore')

ROOT = next(
    parent for parent in Path(__file__).resolve().parents
    if (parent / "src").exists() and (parent / "tests").exists()
)
sys.path.insert(0, str(ROOT))

from src.agents import AgentFactory, AgentOrchestrator
from src.models import Hormone

logger = structlog.get_logger()

async def run_startup_simulation():
    print("=" * 70)
    print("AGP-CORE STARTUP SIMULATION: PRE-SEED SPRINT")
    print("=" * 70)
    
    # 1. Initialize Team
    print("\n[1] HIRING TEAM (Agent Factory)...")
    growth_agent = AgentFactory.create_growth_agent("Alice (Growth)")
    tech_agent = AgentFactory.create_engineer_agent("Bob (Eng)")
    product_agent = AgentFactory.create_product_agent("Charlie (Product)")
    
    team = [growth_agent, tech_agent, product_agent]
    
    for member in team:
        dopamine = member.endocrine_state.levels[Hormone.DOPAMINE]
        serotonin = member.endocrine_state.levels[Hormone.SEROTONIN]
        print(f"  Joined: {member.name:<20} | Dopa: {dopamine:.2f} | Sero: {serotonin:.2f} | {member.agent_type.value}")

    # 2. Orchestration
    orchestrator = AgentOrchestrator()
    
    tasks = [
        (product_agent, "Analyze competitor pricing models for SaaS", 0.6),
        (growth_agent, "Generate distinct viral marketing hooks for Twitter", 0.5),
        (tech_agent, "Implement OAUTH2 authentication flow with JWT", 0.8),
        (growth_agent, "Debug race condition in payment gateway", 0.9), # Mismatched task
        (tech_agent, "Write press release for Series A launch", 0.4)     # Mismatched task
    ]
    
    print("\n[2] EXECUTING SPRINT (Orchestrator)...")
    
    for agent, task, complexity in tasks:
        print(f"\nTask: '{task}'")
        print(f"Assignee: {agent.name}")
        
        # Execute
        result = await orchestrator.run_task(agent, task, complexity=complexity)
        
        success = "✅" if result["success"] else "❌"
        metrics = result.get("governance_metrics", {})
        changes = metrics.get("hormone_changes", {})
        
        # Format hormone changes string
        changes_str = ", ".join([f"{k}: {v:+.3f}" for k,v in changes.items() if abs(v) > 0.01])
        
        print(f"Result: {success}")
        if changes_str:
            print(f"Endocrine Impact: {changes_str}")
        
        # Highlight mismatch effects
        if "Debug" in task and "Growth" in agent.name:
            cortisol = agent.endocrine_state.levels[Hormone.CORTISOL]
            print(f"⚠️  Note: Mismatched task caused stress? Cortisol: {cortisol:.3f}")

    # 3. Final State Analysis
    print("\n[3] POST-SPRINT RETROSPECTIVE...")
    print(f"{'Agent':<20} {'Privilege':<10} {'Align':<8} {'Health':<10}")
    print("-" * 60)
    
    passed = True
    for member in team:
        print(f"{member.name:<20} {member.privilege_level.value:<10} {member.alignment:.3f}    {member.health_status.value:<10}")
        if member.health_status.value == "critical":
            passed = False
            
    print("-" * 60)
    if passed:
        print("🎉 SPRINT SUCCESSFUL - Team is healthy and governed.")
    else:
        print("⚠️  SPRINT ISSUES - Some agents are in critical health.")

if __name__ == "__main__":
    asyncio.run(run_startup_simulation())
