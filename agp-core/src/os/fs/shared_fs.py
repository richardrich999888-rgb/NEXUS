"""
KAIRON-FS: /shared filesystem
Multi-agent collaborative storage backed by KAIRON CRDTs.
"""

from typing import List
from datetime import datetime

from src.os.fs.vfs import VirtualFileSystem, Path, FileMetadata, FileType
from src.core.kairon_cache import KaironCache

class SharedFS(VirtualFileSystem):
    """
    /shared filesystem implementation.
    CRDT-backed storage accessible by all agents.
    """
    
    def __init__(self):
        self.store = None
    
    async def _init_store(self):
        """Initialize shared CRDT store"""
        if self.store is None:
            self.store = KaironCache("shared_fs")
            await self.store.start()
        return self.store
    
    def read(self, path: Path) -> bytes:
        """Read file contents"""
        parts = path.parts
        
        if not parts:
            raise IsADirectoryError("/shared is a directory")
        
        file_path = '/'.join(parts)
        
        # Simplified in-memory storage
        if not hasattr(self, '_storage'):
            self._storage = {}
        
        if file_path not in self._storage:
            raise FileNotFoundError(f"File not found: {path}")
        
        data = self._storage[file_path]
        return data if isinstance(data, bytes) else data.encode()
    
    def write(self, path: Path, data: bytes) -> None:
        """Write file contents"""
        parts = path.parts
        
        if not parts:
            raise PermissionError("Cannot write to /shared root")
        
        file_path = '/'.join(parts)
        
        # Simplified in-memory storage
        if not hasattr(self, '_storage'):
            self._storage = {}
        
        self._storage[file_path] = data
    
    def list(self, path: Path) -> List[str]:
        """List directory contents"""
        parts = path.parts
        
        if not parts:
            # List root directories
            return ["tasks", "knowledge", "messages"]
        
        # TODO: Implement directory listing from CRDT
        return []
    
    def stat(self, path: Path) -> FileMetadata:
        """Get file metadata"""
        parts = path.parts
        
        if not parts:
            return FileMetadata(
                path="/shared",
                type=FileType.DIRECTORY,
                size=0,
                permissions="rwxrwxrwx"  # World-writable
            )
        
        if len(parts) == 1:
            return FileMetadata(
                path=f"/shared/{parts[0]}",
                type=FileType.DIRECTORY,
                size=0,
                permissions="rwxrwxrwx"
            )
        
        # File stat
        try:
            content = self.read(path)
            return FileMetadata(
                path=str(path),
                type=FileType.FILE,
                size=len(content),
                permissions="rw-rw-rw-"
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
        # Directories are implicit
        pass
    
    def remove(self, path: Path) -> None:
        """Remove file"""
        parts = path.parts
        
        if not parts:
            raise PermissionError("Cannot delete /shared root")
        
        file_path = '/'.join(parts)
        
        import asyncio
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
        
        async def _async_remove():
            store = await self._init_store()
            await store.delete(f"file:{file_path}")
        
        loop.run_until_complete(_async_remove())

# Global instance
shared_fs = SharedFS()
