"""
AGP-OS Shell
Command-line interface for the Agent Operating System.
"""

import asyncio
import shlex
import structlog
from typing import List, Dict, Optional
import sys
import time

from src.os.kernel import kernel
from src.os.syscalls import SysCallHandler, SysCallType
from src.os.security.auth import auth_manager, Permission
from src.os.persistence.database import db
from src.os.process import ProcessState
from src.agents import AgentFactory

logger = structlog.get_logger()

class Shell:
    """
    The User Interface to AGP-OS.
    Parses commands and invokes Kernel/Syscalls.
    """
    
    def __init__(self, kernel_instance: kernel):
        self.kernel = kernel_instance
        self.handler = SysCallHandler(kernel_instance)
        self.session_token: Optional[str] = None
        self.current_user: str = "root"
        
        logger.info("shell_started")
        
    async def run_command_loop(self):
        """Main REPL loop (Simulation)"""
        print("AGP-OS v1.0 (Bio-Kernel)")
        print("Type 'help' for commands.")
        
        while True:
            try:
                cmd_str = input(self._get_prompt())
                if not cmd_str: continue
                
                result = await self.execute(cmd_str)
                if result:
                    print(result)
                
            except KeyboardInterrupt:
                print("\nShutdown...")
                break
            except Exception as e:
                print(f"Error: {e}")

    def _get_prompt(self) -> str:
        token_indicator = "🔑" if self.session_token else ""
        return f"agp-os@{self.current_user}:{token_indicator}> "

    async def execute(self, cmd_str: str) -> str:
        """Execute a single shell command line"""
        parts = shlex.split(cmd_str)
        if not parts: return ""
        
        cmd = parts[0].lower()
        args = parts[1:]
        
        if cmd == "help":
            return self._help()
        elif cmd == "ps":
            return self._ps()
        elif cmd == "spawn":
            return self._spawn(args)
        elif cmd == "kill":
            return self._kill(args)
        elif cmd == "exec":
            return await self._exec(args)
        elif cmd == "top":
            return self._top()
        elif cmd == "login":
            return self._login()
        elif cmd == "logout":
            return self._logout()
        elif cmd == "stats":
            return self._stats()
        elif cmd == "exit":
            sys.exit(0)
        else:
            return f"Unknown command: {cmd}"

    def _help(self) -> str:
        return """
Commands:
  ps            List processes
  spawn <type>  Spawn new agent (growth|eng|product) [name]
  kill <pid>    Terminate process
  exec <pid> <task> Execute task on agent
  top           Show system status
  login         Login to the shell (generates a session token)
  logout        Logout from the shell (clears session token)
  stats         Show system statistics
  exit          Exit shell
"""

    def _ps(self) -> str:
        procs = self.kernel.ps()
        output = [f"{'PID':<5} {'Name':<20} {'State':<10} {'Pri':<5} {'Time':<8} {'Tokens'}"]
        output.append("-" * 60)
        for p in procs:
            output.append(f"{p['PID']:<5} {p['Name']:<20} {p['State']:<10} {p['Pri']:<5} {p['Time']:<8} {p['Tokens']}")
        return "\n".join(output)

    def _spawn(self, args: List[str]) -> str:
        if not args: return "Usage: spawn <type> [name]"
        agent_type = args[0].lower()
        name = args[1] if len(args) > 1 else f"{agent_type}_{self.kernel.pid_counter}"
        
        try:
            if agent_type == "growth":
                agent = AgentFactory.create_growth_agent(name)
            elif agent_type == "eng":
                agent = AgentFactory.create_engineer_agent(name)
            elif agent_type == "product":
                agent = AgentFactory.create_product_agent(name)
            else:
                return f"Unknown agent type: {agent_type}"
                
            pid = self.kernel.spawn_process(agent)
            return f"Spawned process {pid} ({name})"
        except Exception as e:
            return f"Spawn failed: {e}"

    def _kill(self, args: List[str]) -> str:
        if not args: return "Usage: kill <pid>"
        try:
            pid = int(args[0])
            self.kernel.kill_process(pid, reason="shell_kill")
            return f"Killed process {pid}"
        except ValueError:
            return "Invalid PID"

    async def _exec(self, args: List[str]) -> str:
        if len(args) < 2: return "Usage: exec <pid> <task>"
        try:
            pid = int(args[0])
            task = " ".join(args[1:])
            
            result = await syscall_handler.handle(
                pid, 
                SysCallType.EXEC, 
                {"task": task, "complexity": 0.5}
            )
            return f"Result: {result}"
        except ValueError:
            return "Invalid PID"

    def _top(self) -> str:
        # Simple OS stats
        procs = self.kernel.ps()
        running = sum(1 for p in procs if p['State'] == 'running')
        total_tokens = sum(p['Tokens'] for p in procs)
        
        return f"""
System Status: RUNNING
Processes: {len(procs)} total, {running} running
Total Tokens Used: {total_tokens}
Kernel Tick: {self.kernel.scheduler_interval}s
"""

# Global Shell
shell = Shell(kernel)
