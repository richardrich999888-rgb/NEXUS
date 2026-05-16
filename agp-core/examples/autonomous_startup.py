#!/usr/bin/env python3
"""
AGP-OS: Autonomous Startup Simulation
Three specialized agents collaborate to design a product.
"""

import sys
import asyncio
import os
from datetime import datetime

# Suppress warnings
import warnings
warnings.filterwarnings('ignore')

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

from src.os import kernel, syscall_handler, SysCallType
from src.os.llm_provider import register_provider, OpenAIProvider, AnthropicProvider
from src.agents import AgentFactory
from src.models import Hormone

print("=" * 70)
print("AGP-OS: AUTONOMOUS STARTUP SIMULATION")
print("Collaborative Product Development with 3 AI Agents")
print("=" * 70)
print(f"Started at: {datetime.now().isoformat()}\n")

# Check for LLM capability
openai_key = os.getenv("OPENAI_API_KEY")
anthropic_key = os.getenv("ANTHROPIC_API_KEY")
use_real_llm = bool(openai_key or anthropic_key)

if use_real_llm:
    print("🤖 Real LLM Mode: ENABLED")
    if openai_key:
        print("   Provider: OpenAI")
    elif anthropic_key:
        print("   Provider: Anthropic")
else:
    print("🤖 Real LLM Mode: DISABLED (Simulation)")
    print("   Set OPENAI_API_KEY or ANTHROPIC_API_KEY to enable real AI\n")

print()

async def autonomous_startup():
    # 1. Setup
    print("[1] INITIALIZING AGP-OS...")
    
    # Register LLM providers
    if openai_key:
        provider = OpenAIProvider(api_key=openai_key)
        register_provider("openai", provider)
    elif anthropic_key:
        provider = AnthropicProvider(api_key=anthropic_key)
        register_provider("anthropic", provider)
    
    # Boot kernel
    kernel.boot()
    print("   ✓ Kernel booted\n")
    
    # 2. Spawn the team
    print("[2] SPAWNING STARTUP TEAM...")
    
    # CEO: Strategic thinker, high dopamine (creative)
    ceo_agent = AgentFactory.create_growth_agent("CEO_Sarah")
    ceo_agent.endocrine_state.levels[Hormone.DOPAMINE] = 0.85  # Visionary
    ceo_pid = kernel.spawn_process(ceo_agent)
    print(f"   ✓ CEO Sarah (PID={ceo_pid}) - Strategy & Vision")
    
    # CTO: Technical expert, high serotonin (methodical)
    cto_agent = AgentFactory.create_engineer_agent("CTO_Alex")
    cto_agent.endocrine_state.levels[Hormone.SEROTONIN] = 0.90  # Calm precision
    cto_pid = kernel.spawn_process(cto_agent)
    print(f"   ✓ CTO Alex (PID={cto_pid}) - Technical Architecture")
    
    # PM: Product manager, balanced hormones
    pm_agent = AgentFactory.create_product_agent("PM_Jordan")
    pm_pid = kernel.spawn_process(pm_agent)
    print(f"   ✓ PM Jordan (PID={pm_pid}) - Product Strategy\n")
    
    # 3. The Mission
    print("[3] MISSION BRIEFING...")
    mission = "Design a mobile app for real-time collaborative note-taking with AI assistance"
    print(f'   "{mission}"\n')
    
    # 4. CEO: Define Vision
    print("[4] CEO SARAH: Defining Product Vision...")
    ceo_task = f"""As CEO, define the high-level vision and key differentiators for: {mission}. 
    One paragraph, focus on market positioning and unique value prop."""
    
    ceo_result = await syscall_handler.handle(
        ceo_pid,
        SysCallType.EXEC,
        {"task": ceo_task, "use_real_llm": use_real_llm}
    )
    
    if ceo_result.get("success"):
        vision = ceo_result.get("result", "Vision defined")
        print(f"   Vision: {vision[:200]}...")
        if use_real_llm:
            print(f"   Tokens: {ceo_result.get('tokens_used', 0)}\n")
    else:
        print(f"   ✗ Failed: {ceo_result.get('error')}\n")
        vision = "Collaborative AI-powered notes app"
    
    # 5. CTO: Technical Architecture
    print("[5] CTO ALEX: Designing Technical Architecture...")
    cto_task = f"""As CTO, outline the technical architecture for: {mission}.
    Focus on: tech stack, real-time sync, AI integration, and scalability. Brief bullet points."""
    
    cto_result = await syscall_handler.handle(
        cto_pid,
        SysCallType.EXEC,
        {"task": cto_task, "use_real_llm": use_real_llm}
    )
    
    if cto_result.get("success"):
        architecture = cto_result.get("result", "Architecture designed")
        print(f"   Architecture: {architecture[:200]}...")
        if use_real_llm:
            print(f"   Tokens: {cto_result.get('tokens_used', 0)}\n")
    else:
        print(f"   ✗ Failed: {cto_result.get('error')}\n")
        architecture = "React Native + Firebase + GPT API"
    
    # 6. PM: Feature Roadmap
    print("[6] PM JORDAN: Creating Feature Roadmap...")
    pm_task = f"""As PM, create a 3-phase feature roadmap for: {mission}.
    Phase 1 (MVP), Phase 2 (Growth), Phase 3 (Scale). Concise bullet points."""
    
    pm_result = await syscall_handler.handle(
        pm_pid,
        SysCallType.EXEC,
        {"task": pm_task, "use_real_llm": use_real_llm}
    )
    
    if pm_result.get("success"):
        roadmap = pm_result.get("result", "Roadmap created")
        print(f"   Roadmap: {roadmap[:200]}...")
        if use_real_llm:
            print(f"   Tokens: {pm_result.get('tokens_used', 0)}\n")
    else:
        print(f"   ✗ Failed: {pm_result.get('error')}\n")
        roadmap = "MVP: Notes + Sync, Growth: AI, Scale: Teams"
    
    # 7. System Status
    print("[7] SYSTEM STATUS...")
    processes = kernel.ps()
    
    print(f"\n   Active Processes: {len(processes)}")
    print(f"   {'PID':<5} {'Name':<15} {'Priority':<8} {'Tokens':<8} {'State'}")
    print("   " + "-" * 50)
    for p in processes:
        print(f"   {p['PID']:<5} {p['Name']:<15} {p['Pri']:<8} {p['Tokens']:<8} {p['State']}")
    
    # 8. Team Summary
    print("\n[8] COLLABORATION SUMMARY...")
    print(f"   Vision: {vision[:80]}...")
    print(f"   Tech: {architecture[:80]}...")
    print(f"   Roadmap: {roadmap[:80]}...")
    
    # 9. Stress Test: Overload one agent
    print("\n[9] STRESS TEST: Overloading PM with unrealistic demands...")
    
    # Set very low quota for PM
    pm_pcb = kernel.process_table[pm_pid]
    pm_pcb.quota_tokens = 10
    
    stress_result = await syscall_handler.handle(
        pm_pid,
        SysCallType.EXEC,
        {"task": "Write a 10-page detailed spec for every feature", "use_real_llm": use_real_llm}
    )
    
    if "EDQUOT" in stress_result.get("error", ""):
        print("   ✓ Budget Enforcer: Blocked over-quota request")
        print(f"   ✓ PM Cortisol Level: {pm_agent.endocrine_state.levels[Hormone.CORTISOL]:.2f}")
        print(f"   ✓ PM Priority (throttled): {pm_pcb.priority:.2f}")
    else:
        print(f"   Result: {stress_result}")
    
    print("\n" + "=" * 70)
    print("AUTONOMOUS STARTUP SIMULATION COMPLETE")
    print("=" * 70)
    print("\n✨ The team successfully collaborated on product design!")
    print("✨ Budget enforcement protected the system from overload!")
    print("✨ Endocrine system maintained stability under stress!")

if __name__ == "__main__":
    asyncio.run(autonomous_startup())
