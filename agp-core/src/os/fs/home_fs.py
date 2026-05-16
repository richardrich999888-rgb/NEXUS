"""
KAIRON-FS: /home filesystem
Per-agent private storage backed by KAIRON CRDTs.
"""

from typing import List, Dict
from datetime import datetime

from src.os.fs.vfs import VirtualFileSystem, Path, FileMetadata, FileType
from src.core.kairon_cache import KaironCache, BoundedLWWRegister

class HomeFS(VirtualFileSystem):
    """
    /home filesystem implementation.
    Each agent gets a private directory: /home/<agent_name>/
    """
    
    def __init__(self):
        # Map agent_name -> KaironCache instance
        self.agent_stores: Dict[str, KaironCache] = {}
    
    async def _get_or_create_store(self, agent_name: str) -> KaironCache:
        """Get or create KaironCache for an agent"""
        if agent_name not in self.agent_stores:
            store = KaironCache(f"home_{agent_name}")
            await store.start()
            self.agent_stores[agent_name] = store
        return self.agent_stores[agent_name]
    
    def read(self, path: Path) -> bytes:
        """Read file contents"""
        parts = path.parts
        
        if len(parts) < 2:
            raise IsADirectoryError(f"{path} is a directory")
        
        agent_name = parts[0]
        file_path = '/'.join(parts[1:])
        
        # Simplified: Use in-memory dict for now instead of async CRDT
        # In production, this would be properly async
        key = f"{agent_name}:{file_path}"
        if not hasattr(self, '_storage'):
            self._storage = {}
        
        if key not in self._storage:
            raise FileNotFoundError(f"File not found: {path}")
        
        data = self._storage[key]
        return data if isinstance(data, bytes) else data.encode()
    
    def write(self, path: Path, data: bytes) -> None:
        """Write file contents"""
        parts = path.parts
        
        if len(parts) < 2:
            raise PermissionError("Cannot write to /home root")
        
        agent_name = parts[0]
        file_path = '/'.join(parts[1:])
        
        # Simplified in-memory storage
        key = f"{agent_name}:{file_path}"
        if not hasattr(self, '_storage'):
            self._storage = {}
        
        self._storage[key] = data
    
    def list(self, path: Path) -> List[str]:
        """List directory contents"""
        parts = path.parts
        
        if not parts:
            # List all agent directories
            return list(self.agent_stores.keys())
        
        agent_name = parts[0]
        
        if len(parts) == 1:
            # List agent's root directories
            return ["memory", "logs", "config"]
        
        # TODO: Implement directory listing from CRDT
        return []
    
    def stat(self, path: Path) -> FileMetadata:
        """Get file metadata"""
        parts = path.parts
        
        if not parts:
            return FileMetadata(
                path="/home",
                type=FileType.DIRECTORY,
                size=0,
                permissions="rwxr-xr-x"
            )
        
        agent_name = parts[0]
        
        if len(parts) == 1:
            return FileMetadata(
                path=f"/home/{agent_name}",
                type=FileType.DIRECTORY,
                size=0,
                owner=agent_name,
                permissions="rwx------"  # Private
            )
        
        # File stat
        try:
            content = self.read(path)
            return FileMetadata(
                path=str(path),
                type=FileType.FILE,
                size=len(content),
                owner=agent_name,
                permissions="rw-------"
            )
        except:
            raise FileNotFoundError(f"File not found: {path}")
    
    def exists(self, path: Path) -> bool:
        """Check if path exists"""
        try:
            self.stat(path)
            return True
        except:
            return False
    
    def mkdir(self, path: Path) -> None:
        """Create directory"""
        # Directories are implicit in CRDT storage
        pass
    
    def remove(self, path: Path) -> None:
        """Remove file"""
        parts = path.parts
        
        if len(parts) < 2:
            raise PermissionError("Cannot delete agent home directory")
        
        agent_name = parts[0]
        file_path = '/'.join(parts[1:])
        
        import asyncio
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
        
        async def _async_remove():
            store = await self._get_or_create_store(agent_name)
            await store.delete(f"file:{file_path}")
        
        loop.run_until_complete(_async_remove())

# Global instance
home_fs = HomeFS()
