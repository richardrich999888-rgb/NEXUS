"""
KAIRON-FS: /proc filesystem
Dynamic filesystem exposing live process state from the kernel.
"""

import json
from typing import List
from datetime import datetime

from src.os.fs.vfs import VirtualFileSystem, Path, FileMetadata, FileType
from src.os.kernel import kernel
from src.models import Hormone

class ProcFS(VirtualFileSystem):
    """
    /proc filesystem implementation.
    Dynamically generates files from kernel process table.
    All files are read-only.
    """
    
    def __init__(self, kernel_instance):
        self.kernel = kernel_instance
    
    def _get_process(self, pid: int):
        """Get process by PID"""
        pcb = self.kernel.process_table.get(pid)
        if not pcb:
            raise FileNotFoundError(f"Process {pid} not found")
        return pcb
    
    def read(self, path: Path) -> bytes:
        """Read file contents"""
        parts = path.parts
        
        if not parts:
            raise IsADirectoryError("/proc is a directory")
        
        # /proc/<pid>/...
        try:
            pid = int(parts[0])
        except ValueError:
            raise FileNotFoundError(f"Invalid PID: {parts[0]}")
        
        pcb = self._get_process(pid)
        
        if len(parts) == 1:
            raise IsADirectoryError(f"/proc/{pid} is a directory")
        
        filename = parts[1]
        
        # Generate file contents
        if filename == "status":
            return self._generate_status(pcb).encode()
        
        elif filename == "endocrine":
            return self._generate_endocrine(pcb).encode()
        
        elif filename == "usage":
            return self._generate_usage(pcb).encode()
        
        elif filename == "context":
            return self._generate_context(pcb).encode()
        
        else:
            raise FileNotFoundError(f"Unknown file: {filename}")
    
    def write(self, path: Path, data: bytes) -> None:
        """Write is not supported (read-only filesystem)"""
        raise PermissionError("/proc is read-only")
    
    def list(self, path: Path) -> List[str]:
        """List directory contents"""
        parts = path.parts
        
        if not parts:
            # List all PIDs
            return [str(pid) for pid in self.kernel.process_table.keys()]
        
        # /proc/<pid>/ contents
        try:
            pid = int(parts[0])
        except ValueError:
            raise FileNotFoundError(f"Invalid PID: {parts[0]}")
        
        # Verify process exists
        pcb = self._get_process(pid)
        
        # Return standard proc files
        return ["status", "endocrine", "usage", "context"]
    
    def stat(self, path: Path) -> FileMetadata:
        """Get file metadata"""
        parts = path.parts
        
        if not parts:
            # /proc directory
            return FileMetadata(
                path="/proc",
                type=FileType.DIRECTORY,
                size=0,
                permissions="r-xr-xr-x"
            )
        
        try:
            pid = int(parts[0])
        except ValueError:
            raise FileNotFoundError(f"Invalid PID: {parts[0]}")
        
        pcb = self._get_process(pid)
        
        if len(parts) == 1:
            # /proc/<pid> directory
            return FileMetadata(
                path=f"/proc/{pid}",
                type=FileType.DIRECTORY,
                size=0,
                permissions="r-xr-xr-x",
                owner=pcb.name
            )
        
        # /proc/<pid>/<file>
        filename = parts[1]
        content = self.read(path)
        
        return FileMetadata(
            path=str(path),
            type=FileType.FILE,
            size=len(content),
            permissions="r--r--r--",
            owner=pcb.name
        )
    
    def exists(self, path: Path) -> bool:
        """Check if path exists"""
        try:
            self.stat(path)
            return True
        except:
            return False
    
    def mkdir(self, path: Path) -> None:
        """Create directory (not supported)"""
        raise PermissionError("/proc is read-only")
    
    def remove(self, path: Path) -> None:
        """Remove file (not supported)"""
        raise PermissionError("/proc is read-only")
    
    # File generators
    
    def _generate_status(self, pcb) -> str:
        """Generate /proc/<pid>/status"""
        return f"""Name:    {pcb.name}
PID:     {pcb.pid}
State:   {pcb.state.value}
Priority: {pcb.priority:.2f}
Nice:    {pcb.nice}
Created: {datetime.fromtimestamp(pcb.created_at).isoformat()}
Runtime: {pcb.total_runtime:.2f}s
"""
    
    def _generate_endocrine(self, pcb) -> str:
        """Generate /proc/<pid>/endocrine"""
        # Get agent to access endocrine state
        from src.agents import agent_registry
        import uuid
        
        try:
            agent_uuid = uuid.UUID(pcb.agent_id)
            agent = agent_registry.get_agent(agent_uuid)
        except:
            return "Endocrine state unavailable\n"
        
        if not agent:
            return "Endocrine state unavailable\n"
        
        lines = ["Hormone Levels:\n"]
        for hormone in Hormone:
            level = agent.endocrine_state.levels.get(hormone, 0.5)
            bar = "█" * int(level * 20)
            lines.append(f"  {hormone.value:15} {level:.2f} {bar}\n")
        
        return "".join(lines)
    
    def _generate_usage(self, pcb) -> str:
        """Generate /proc/<pid>/usage"""
        return f"""CPU Cycles:    {pcb.usage.cpu_cycles}
Tokens Used:   {pcb.usage.tokens_used}
Token Quota:   {pcb.quota_tokens}
Memory Pages:  {pcb.usage.memory_pages}
Disk Bytes:    {pcb.usage.disk_bytes}
"""
    
    def _generate_context(self, pcb) -> str:
        """Generate /proc/<pid>/context"""
        from src.os.context_manager import context_manager
        
        if pcb.pid not in context_manager.active_pages:
            return "No active context\n"
        
        pages = context_manager.active_pages[pcb.pid]
        
        lines = [f"Active Pages: {len(pages)}\n\n"]
        for i, page in enumerate(pages[-5:]):  # Show last 5 pages
            lines.append(f"--- Page {i+1} ---\n")
            lines.append(f"{page.content[:200]}...\n\n")
        
        return "".join(lines)

# Create global instance (will be mounted by kernel)
proc_fs = ProcFS(kernel)
