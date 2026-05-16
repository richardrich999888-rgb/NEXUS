"""
AGP-OS System Calls
Secure interface for User Space (Agents) to request Kernel resources.
"""

from enum import Enum, auto
from typing import Dict, Any, Optional
import inspect
import structlog
import time
import uuid # Added for _handle_malloc

from src.os.kernel import kernel, BioKernel
from src.os.process import ProcessState
from src.agents.orchestrator import AgentOrchestrator
from src.os.security.auth import auth_manager, rate_limiter, Permission
from src.os.persistence.database import db
from src.os.observability.prometheus import prom_metrics
from src.os.hal.hal import hal # Added HAL import
from src.os.resilience.circuit_breaker import llm_circuit
from src.governance import protocol_enforcer, Decision

logger = structlog.get_logger()

class SysCallType(Enum):
    EXEC = auto()   # Execute tool/action
    MALLOC = auto() # Request more tokens/context
    FORK = auto()   # Spawn sub-agent
    SIGNAL = auto() # Broadcast intent
    EXIT = auto()   # Terminate self
    SENSOR_READ = auto() # Read from a sensor
    ACTUATOR_MOVE = auto() # Move an actuator

class SysCallHandler:
    """
    Handles system calls from agents.
    Enforces security and resource quotas.
    """

    def __init__(self, kernel_instance: BioKernel):
        self.kernel = kernel_instance
        self.orchestrator = AgentOrchestrator()
        self.enforcer = protocol_enforcer

    async def handle(self, pid: int, syscall_type: SysCallType, args: Dict, token: str = None) -> Dict:
        """Backward-compatible syscall entry point."""
        return await self.handle_syscall(pid, syscall_type, args, token)

    async def handle_syscall(self, pid: int, syscall_type: SysCallType, args: Dict, token: str = None) -> Dict:
        """Handle a system call with security, rate limiting, and governance"""
        start_time = time.perf_counter()

        # Get the process for the syscall
        process = self.kernel.process_table.get(pid)
        if not process:
            prom_metrics.record_syscall(syscall_type.name, 0, False)
            db.log_audit(pid, syscall_type.name, "system", "FAILURE", details={"error": "ESRCH (No such process)"})
            return {"error": "ESRCH (No such process)"}

        # Get agent info from registry
        from src.agents import agent_registry
        agent = agent_registry.get_agent(process.agent_id)
        agent_name = agent.name if agent else f"agent-{pid}"

        # 1. GOVERNANCE CHECK (Pre-execution)
        governance_decision = await self.enforcer.enforce(
            agent_id=str(process.agent_id),
            agent_name=agent_name,
            action_type=syscall_type.name,
            action_details=args
        )

        # If governance denies, return immediately
        if not governance_decision.is_allowed():
            logger.warning("syscall_blocked_by_governance",
                         pid=pid,
                         agent=agent_name,
                         decision=governance_decision.decision.value,
                         reason=governance_decision.reason)
            prom_metrics.record_syscall(syscall_type.name, 0, False)
            return {
                "status": "error",
                "error": f"EPERM: {governance_decision.reason}",
                "governance": {
                    "decision": governance_decision.decision.value,
                    "alignment": governance_decision.alignment,
                    "rules_triggered": governance_decision.rules_triggered
                }
            }

        # Log if warning
        if governance_decision.decision == Decision.WARN:
            logger.warning("syscall_warning",
                         pid=pid,
                         agent=agent_name,
                         alignment=governance_decision.alignment)

        # 2. JWT & Rate Limiting (Traditional Security)
        if not self._verify_request(pid, token, syscall_type):
            prom_metrics.record_syscall(syscall_type.name, 0, False)
            return {"status": "error", "error": "EACCES: Permission denied or rate limited"}

        # 3. Execution
        try:
            # Dispatch based on syscall_type
            syscall_name = syscall_type.name
            if syscall_name == 'EXEC':
                result = await self._handle_exec(process, args)
            elif syscall_name == 'MALLOC':
                result = await self._handle_malloc(process, args)
            elif syscall_name == 'SENSOR_READ':
                # Pass alignment to sensor read for potential governance-aware behavior
                result = await self._handle_sensor_read(process, args, governance_decision.alignment)
            elif syscall_name == 'ACTUATOR_MOVE':
                # Pass alignment to actuator move for potential governance-aware behavior
                result = await self._handle_actuator_move(process, args, governance_decision.alignment)
            else:
                # Fallback to generic handler if specific one not found
                handler = getattr(self, f"_handle_{syscall_type.name.lower()}", None)
                if handler:
                    result = handler(process, args)
                    if inspect.isawaitable(result):
                        result = await result
                else:
                    result = {'error': f'Unknown syscall: {syscall_name}'}

            # 4. Record result in governance (Post-execution)
            execution_time = int((time.perf_counter() - start_time) * 1000)
            self.enforcer.record_action_result(
                agent_id=str(process.agent_id),
                agent_name=agent_name,
                action_type=syscall_type.name,
                args=args,
                result=result,
                latency_ms=execution_time
            )

            # 5. Audit & Metrics
            duration = time.perf_counter() - start_time
            prom_metrics.record_syscall(syscall_type.name, duration, True)

            if syscall_type in [SysCallType.EXEC, SysCallType.MALLOC]:
                db.log_audit(pid, syscall_type.name, "system", "SUCCESS", details=args)

            # Add governance info to result
            if isinstance(result, dict):
                result["governance"] = {
                    "alignment": governance_decision.alignment,
                    "decision": governance_decision.decision.value
                }

            return result
        except Exception as e:
            logger.error("syscall_error", pid=pid, type=syscall_type.name, error=str(e))
            prom_metrics.record_syscall(syscall_type.name, 0, False)
            db.log_audit(pid, syscall_type.name, "system", "FAILURE", details={"error": str(e)})

            # Record failure in governance
            execution_time = int((time.perf_counter() - start_time) * 1000)
            self.enforcer.record_action_result(
                agent_id=str(process.agent_id),
                agent_name=agent_name,
                action_type=syscall_type.name,
                args=args,
                result={"error": str(e), "success": False},
                latency_ms=execution_time
            )

            return {"status": "error", "error": f"ESRCH: Internal Error ({str(e)})"}

    def _verify_request(self, pid: int, token_str: str, syscall_type: SysCallType) -> bool:
        """Verify JWT token and rate limits"""
        # Rate limit check (Production)
        if not rate_limiter.acquire(pid):
            logger.warning("rate_limit_exceeded", pid=pid, syscall=syscall_type.name)
            return False

        # Token check (if provided or mandatory in prod)
        if token_str:
            token = auth_manager.verify_token(token_str)
            if not token:
                logger.warning("invalid_token", pid=pid)
                return False
            if token.process_id and token.process_id != pid:
                logger.warning("token_pid_mismatch", pid=pid, expected=token.process_id)
                return False

            # Permission mapping
            perm_map = {
                SysCallType.EXEC: Permission.SYSCALL_EXEC,
                SysCallType.MALLOC: Permission.SYSCALL_MALLOC,
                SysCallType.FORK: Permission.SYSCALL_FORK,
                SysCallType.EXIT: Permission.SYSCALL_KILL
            }
            if syscall_type in perm_map and not auth_manager.has_permission(token, perm_map[syscall_type]):
                logger.warning("insufficient_permission", pid=pid, syscall=syscall_type.name)
                return False

        return True

    async def _handle_exec(self, process: ProcessState, args: Dict) -> Dict:
        """
        EXEC: Run a task/tool with REAL LLM execution.
        Includes token budget enforcement and streaming accounting.
        """
        task = args.get("task")
        complexity = args.get("complexity", 0.5)
        use_real_llm = args.get("use_real_llm", False)  # Flag to enable real LLM

        # Get actual agent object
        from src.agents import agent_registry
        try:
            agent_uuid = uuid.UUID(process.agent_id)
            agent = agent_registry.get_agent(agent_uuid)
        except:
            agent = None

        if not agent:
             return {"error": "EFAULT (Bad address - Agent not found)"}

        # Check Circuit Breaker for LLM
        if use_real_llm and not llm_circuit.can_execute():
            logger.warning("llm_circuit_open", pid=process.pid)
            return {"status": "error", "error": "EAGAIN: AI System overloaded (Circuit Open)"}

        if not use_real_llm:
            # If real LLM is disabled, fall back to orchestrator simulation
            cost = 100  # estimated tokens
            if process.usage.tokens_used + cost > process.quota_tokens:
                 return {"error": "EDQUOT (Quota exceeded)"}

            result = await self.orchestrator.run_task(agent, task, complexity=complexity)

            if result["success"]:
                 process.update_usage(tokens=cost, runtime=result["governance_metrics"]["duration"])

            return result

        # === REAL LLM EXECUTION PATH ===
        from src.os.llm_provider import get_provider
        from src.os.budget import budget_enforcer

        # Get LLM provider (default to OpenAI if available)
        provider = get_provider("openai") or get_provider("anthropic")
        if not provider:
            return {"error": "ENODEV (No LLM provider available)"}

        # Build messages
        messages = [
            {"role": "system", "content": f"You are {agent.name}. Execute the following task efficiently."},
            {"role": "user", "content": task}
        ]

        # Estimate tokens
        estimated_tokens = provider.count_tokens(task) + 500  # Task + response estimate

        # Check quota
        if not budget_enforcer.check_quota(process, estimated_tokens):
            budget_enforcer.handle_quota_violation(process, agent)
            return {"error": "EDQUOT (Token quota exceeded)", "success": False}

        # Reserve tokens
        budget_enforcer.reserve_tokens(process, estimated_tokens)

        try:
            # Execute LLM completion
            start_time = time.time()
            result = await provider.complete(
                messages=messages,
                model=getattr(agent, 'model', 'gpt-4'),
                max_tokens=min(1000, process.quota_tokens - process.usage.tokens_used),
                temperature=0.7
            )

            # Release reservation with actual usage
            budget_enforcer.release_tokens(process, result.tokens_used)

            # Update process usage
            process.update_usage(tokens=result.tokens_used, runtime=result.duration)

            llm_circuit.record_success() # Record success for circuit breaker
            return {
                "success": True,
                "result": result.content,
                "tokens_used": result.tokens_used,
                "duration": result.duration,
                "model": result.model,
                "governance_metrics": {
                    "duration": result.duration,
                    "tokens": result.tokens_used
                }
            }

        except Exception as e:
            # Release reservation on error
            budget_enforcer.release_tokens(process, 0)
            logger.error("llm_execution_failed", pid=process.pid, error=str(e))
            llm_circuit.record_failure() # Record failure for circuit breaker
            return {"error": f"EIO (LLM execution failed: {str(e)})", "success": False}

    async def _handle_malloc(self, process: ProcessState, args: Dict) -> Dict:
        """MALLOC: Allocate virtual memory for an agent"""
        size = int(args.get("size", args.get("amount", 1024)))
        page_size = 4096
        pages = max(1, (size + page_size - 1) // page_size)
        quota_pages = getattr(process, "quota_memory_pages", 4096)

        if process.usage.memory_pages + pages > quota_pages:
             return {"error": "ENOMEM (Quota exceeded)"}

        process.usage.memory_pages += pages
        return {
            "status": "success",
            "addr": f"0x{uuid.uuid4().hex[:8]}",
            "size": size,
            "pages": pages,
        }

    async def _handle_sensor_read(self, process: ProcessState, args: Dict, alignment: float) -> Dict:
        """SENSOR_READ: Fetch telemetry from robotic sensors"""
        device_id = args.get("device_id")
        if not device_id:
            return {"error": "EINVAL (Missing device_id)"}

        # Optional: Some sensors might only be visible to highly-aligned agents
        # For now, all agents can read sensors if they are online
        result = hal.read_sensor(device_id)
        return result

    async def _handle_actuator_move(self, process: ProcessState, args: Dict, alignment: float) -> Dict:
        """ACTUATOR_MOVE: Trigger physical hardware movement"""
        device_id = args.get("device_id")
        command = args.get("command")

        if not device_id or command is None:
            return {"error": "EINVAL (Missing device_id or command)"}

        # The HAL itself handles the safety interlock using the alignment
        result = hal.move_actuator(device_id, command, agent_alignment=alignment)
        return result

    async def _handle_fork(self, process: ProcessState, args: Dict) -> Dict:
        """
        FORK: Spawn sub-agent.
        """
        name = args.get("name", f"{process.name}_child")

        # In real impl, we'd clone the parent's endocrine state
        from src.agents import AgentFactory
        child_agent = AgentFactory.create_engineer_agent(name) # Default to Eng for now

        child_pid = self.kernel.spawn_process(child_agent)
        return {"child_pid": child_pid}

# Global Handler
syscall_handler = SysCallHandler(kernel)
