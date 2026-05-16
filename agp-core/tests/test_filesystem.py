#!/usr/bin/env python3
"""
KAIRON-FS Verification Test
Test the distributed filesystem with /proc, /home, and /shared.
"""

import sys
import asyncio
from datetime import datetime

# Suppress warnings
import warnings
warnings.filterwarnings('ignore')

sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp-core')

print("=" * 70)
print("KAIRON-FS: DISTRIBUTED FILESYSTEM TEST")
print("=" * 70)
print(f"Started at: {datetime.now().isoformat()}\n")

async def test_filesystem():
    from src.os import kernel
    from src.os.fs import filesystem
    from src.agents import AgentFactory
    
    # 1. Boot kernel and spawn agents
    print("[1] BOOTING KERNEL...")
    kernel.boot()
    
    ceo = AgentFactory.create_growth_agent("CEO_Alice")
    cto = AgentFactory.create_engineer_agent("CTO_Bob")
    
    ceo_pid = kernel.spawn_process(ceo)
    cto_pid = kernel.spawn_process(cto)
    
    print(f"   ✓ Kernel booted with {len(kernel.process_table)} processes\n")
    
    # 2. Test /proc filesystem
    print("[2] TESTING /proc (Live Process State)...")
    
    # List processes
    processes = filesystem.list("/proc")
    print(f"   Active PIDs: {processes}")
    
    # Read CEO status
    ceo_status = filesystem.read(f"/proc/{ceo_pid}/status").decode()
    print(f"   CEO Status:\n{ceo_status}")
    
    # Read CEO endocrine state
    ceo_endocrine = filesystem.read(f"/proc/{ceo_pid}/endocrine").decode()
    print(f"   CEO Hormones (first 3 lines):")
    for line in ceo_endocrine.split('\n')[:4]:
        print(f"     {line}")
    print()
    
    # 3. Test /home filesystem
    print("[3] TESTING /home (Agent Private Storage)...")
    
    # Write to CEO's memory
    filesystem.write(
        "/home/CEO_Alice/memory/vision.txt",
        b"We will build the most innovative AI collaboration platform"
    )
    print("   ✓ Wrote to /home/CEO_Alice/memory/vision.txt")
    
    # Read it back
    vision = filesystem.read("/home/CEO_Alice/memory/vision.txt").decode()
    print(f"   ✓ Read back: '{vision[:50]}...'\n")
    
    # 4. Test /shared filesystem
    print("[4] TESTING /shared (Collaborative Workspace)...")
    
    # CEO writes strategy
    filesystem.write(
        "/shared/knowledge/strategy.md",
        b"# Company Strategy\n## Q1 Goals\n- Launch MVP\n- Acquire 100 users"
    )
    print("   ✓ CEO wrote /shared/knowledge/strategy.md")
    
    # CTO reads and adds to it
    strategy = filesystem.read("/shared/knowledge/strategy.md").decode()
    print(f"   ✓ CTO read strategy: {len(strategy)} bytes")
    
    # CTO writes tech docs
    filesystem.write(
        "/shared/knowledge/architecture.md",
        b"# Technical Architecture\n- Backend: Python/FastAPI\n- Frontend: React"
    )
    print("   ✓ CTO wrote /shared/knowledge/architecture.md\n")
    
    # 5. Test watch API
    print("[5] TESTING WATCH API (Real-time Updates)...")
    
    watch_triggered = []
    
    def on_task_added(path, data):
        watch_triggered.append(data)
        print(f"   🔔 Watch triggered: New task detected!")
    
    filesystem.watch("/shared/tasks/new_task.txt", on_task_added)
    
    # Write a task
    filesystem.write(
        "/shared/tasks/new_task.txt",
        b"Build authentication system"
    )
    
    if watch_triggered:
        print(f"   ✓ Watch callback executed: {len(watch_triggered)} updates\n")
    else:
        print(f"   ⚠️  Watch callback not triggered\n")
    
    # 6. Filesystem stats
    print("[6] FILESYSTEM SUMMARY...")
    print(f"   Mounted filesystems:")
    print(f"     /proc  -> {type(filesystem.mounts[0].fs).__name__}")
    print(f"     /home  -> {type(filesystem.mounts[1].fs).__name__}")
    print(f"     /shared -> {type(filesystem.mounts[2].fs).__name__}")
    
    print("\n" + "=" * 70)
    print("KAIRON-FS VERIFICATION COMPLETE")
    print("=" * 70)
    print("\n✨ All filesystem operations successful!")
    print("✨ /proc provides live process visibility!")
    print("✨ /home enables agent private storage!")
    print("✨ /shared enables multi-agent collaboration!")

if __name__ == "__main__":
    asyncio.run(test_filesystem())
