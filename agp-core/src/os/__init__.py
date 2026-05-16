"""
AGP-OS Process Management
Defines the Process Control Block (PCB) for AI Agents.
"""

from .process import ProcessControlBlock, ProcessState, ResourceUsage
from .kernel import BioKernel, kernel
from .syscalls import SysCallHandler, SysCallType, syscall_handler
from .context_manager import ContextManager, context_manager
from .shell import Shell, shell

__all__ = [
    "ProcessControlBlock",
    "ProcessState",
    "ResourceUsage",
    "BioKernel",
    "kernel",
    "SysCallHandler",
    "SysCallType",
    "syscall_handler",
    "ContextManager",
    "context_manager",
    "Shell",
    "shell"
]
