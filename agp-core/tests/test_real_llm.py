#!/usr/bin/env python3
"""
AGP-OS Real LLM Integration Test
Verifies that real LLM execution works with budget enforcement.
"""

import sys
import asyncio
import os
from datetime import datetime

# Suppress warnings
import warnings
warnings.filterwarnings('ignore')

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("AGP-OS REAL LLM INTEGRATION TEST")
print("=" * 70)
print(f"Started at: {datetime.now().isoformat()}\n")

# Check for API keys
openai_key = os.getenv("OPENAI_API_KEY")
anthropic_key = os.getenv("ANTHROPIC_API_KEY")

if not openai_key and not anthropic_key:
    print("⚠️  No API keys found. Set OPENAI_API_KEY or ANTHROPIC_API_KEY")
    print("Skipping real LLM tests. Run with simulation mode only.\n")
    has_llm = False
else:
    has_llm = True
    if openai_key:
        print(f"✓ OpenAI API key found")
    if anthropic_key:
        print(f"✓ Anthropic API key found")
    print()

async def test_llm_integration():
    from src.os import kernel, syscall_handler, SysCallType
    from src.os.llm_provider import register_provider, OpenAIProvider, AnthropicProvider
    from src.os.budget import budget_enforcer
    from src.agents import AgentFactory
    
    # 1. Register providers
    print("[1] REGISTERING LLM PROVIDERS...")
    if openai_key:
        openai_provider = OpenAIProvider(api_key=openai_key)
        register_provider("openai", openai_provider)
        print("   ✓ OpenAI provider registered")
    
    if anthropic_key:
        anthropic_provider = AnthropicProvider(api_key=anthropic_key)
        register_provider("anthropic", anthropic_provider)
        print("   ✓ Anthropic provider registered")
    
    # 2. Boot kernel
    print("\n[2] BOOTING KERNEL...")
    kernel.boot()
    print("   ✓ Kernel initialized")
    
    # 3. Spawn test agent
    print("\n[3] SPAWNING TEST AGENT...")
    test_agent = AgentFactory.create_engineer_agent("TestAgent_LLM")
    pid = kernel.spawn_process(test_agent)
    print(f"   ✓ Agent spawned (PID={pid})")
    
    # 4. Test simulated execution (baseline)
    print("\n[4] TESTING SIMULATED EXECUTION...")
    result = await syscall_handler.handle(
        pid,
        SysCallType.EXEC,
        {
            "task": "Calculate 2+2",
            "complexity": 0.1,
            "use_real_llm": False  # Simulation mode
        }
    )
    print(f"   ✓ Simulated execution: {result.get('success', False)}")
    
    # 5. Test real LLM execution (if available)
    if has_llm:
        print("\n[5] TESTING REAL LLM EXECUTION...")
        print("   Task: 'Explain AGP-OS in one sentence'")
        
        result = await syscall_handler.handle(
            pid,
            SysCallType.EXEC,
            {
                "task": "Explain what an operating system for AI agents would do, in one sentence.",
                "complexity": 0.5,
                "use_real_llm": True  # Real LLM mode
            }
        )
        
        if result.get("success"):
            print(f"   ✓ LLM Response: {result['result'][:100]}...")
            print(f"   ✓ Tokens Used: {result['tokens_used']}")
            print(f"   ✓ Duration: {result['duration']:.2f}s")
            print(f"   ✓ Model: {result['model']}")
        else:
            print(f"   ✗ LLM execution failed: {result.get('error')}")
    else:
        print("\n[5] SKIPPING REAL LLM TEST (No API keys)")
    
    # 6. Test budget enforcement
    print("\n[6] TESTING BUDGET ENFORCEMENT...")
    pcb = kernel.process_table[pid]
    original_quota = pcb.quota_tokens
    pcb.quota_tokens = 50  # Set very low quota
    
    result = await syscall_handler.handle(
        pid,
        SysCallType.EXEC,
        {
            "task": "Write a long essay about operating systems",
            "use_real_llm": has_llm
        }
    )
    
    if "EDQUOT" in result.get("error", ""):
        print("   ✓ Budget enforcer correctly blocked over-quota request")
    else:
        print(f"   ⚠️  Expected quota error, got: {result}")
    
    pcb.quota_tokens = original_quota  # Restore
    
    # 7. Check budget stats
    print("\n[7] BUDGET STATISTICS...")
    stats = budget_enforcer.get_stats()
    print(f"   Global Quota: {stats['global_quota']:,} tokens")
    print(f"   Global Used: {stats['global_used']:,} tokens")
    print(f"   Utilization: {stats['utilization_pct']:.2f}%")
    
    print("\n" + "=" * 70)
    print("REAL LLM INTEGRATION TEST COMPLETE")
    print("=" * 70)

if __name__ == "__main__":
    asyncio.run(test_llm_integration())
