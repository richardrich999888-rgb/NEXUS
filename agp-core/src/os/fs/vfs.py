"""
KAIRON-FS: Virtual File System for AGP-OS
Core abstractions for distributed, CRDT-backed filesystem.
"""

from abc import ABC, abstractmethod
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
import structlog

logger = structlog.get_logger()

class FileType(Enum):
    FILE = "file"
    DIRECTORY = "directory"
    SYMLINK = "symlink"

@dataclass
class FileMetadata:
    """Metadata for a filesystem entry"""
    path: str
    type: FileType
    size: int = 0
    created_at: datetime = field(default_factory=datetime.utcnow)
    modified_at: datetime = field(default_factory=datetime.utcnow)
    owner: Optional[str] = None  # Agent ID
    permissions: str = "rw-r--r--"  # Unix-style permissions
    metadata: Dict[str, Any] = field(default_factory=dict)

class Path:
    """
    Path abstraction (similar to pathlib.Path)
    Handles path parsing and validation.
    """
    
    def __init__(self, path: str):
        self.path = path.rstrip('/') if path != '/' else '/'
        self.parts = [p for p in self.path.split('/') if p]
    
    @property
    def name(self) -> str:
        """Get the final component of the path"""
        return self.parts[-1] if self.parts else ""
    
    @property
    def parent(self) -> 'Path':
        """Get the parent directory"""
        if not self.parts:
            return Path('/')
        parent_path = '/' + '/'.join(self.parts[:-1])
        return Path(parent_path if parent_path != '' else '/')
    
    def join(self, *others: str) -> 'Path':
        """Join path components"""
        combined = self.path
        for other in others:
            other = other.lstrip('/')
            combined = f"{combined}/{other}" if combined != '/' else f"/{other}"
        return Path(combined)
    
    def is_child_of(self, parent: 'Path') -> bool:
        """Check if this path is a child of parent"""
        return self.path.startswith(parent.path + '/')
    
    def __str__(self) -> str:
        return self.path
    
    def __repr__(self) -> str:
        return f"Path('{self.path}')"
    
    def __eq__(self, other) -> bool:
        if isinstance(other, Path):
            return self.path == other.path
        return self.path == str(other)
    
    def __hash__(self) -> int:
        return hash(self.path)

class VirtualFileSystem(ABC):
    """
    Abstract base class for filesystem handlers.
    Each mounted filesystem (proc, home, shared) implements this interface.
    """
    
    @abstractmethod
    def read(self, path: Path) -> bytes:
        """Read file contents"""
        pass
    
    @abstractmethod
    def write(self, path: Path, data: bytes) -> None:
        """Write file contents"""
        pass
    
    @abstractmethod
    def list(self, path: Path) -> List[str]:
        """List directory contents (filenames only)"""
        pass
    
    @abstractmethod
    def stat(self, path: Path) -> FileMetadata:
        """Get file metadata"""
        pass
    
    @abstractmethod
    def exists(self, path: Path) -> bool:
        """Check if path exists"""
        pass
    
    @abstractmethod
    def mkdir(self, path: Path) -> None:
        """Create directory"""
        pass
    
    @abstractmethod
    def remove(self, path: Path) -> None:
        """Remove file or directory"""
        pass

class FileSystemMount:
    """
    Represents a mounted filesystem at a specific path.
    """
    
    def __init__(self, mount_point: Path, fs: VirtualFileSystem):
        self.mount_point = mount_point
        self.fs = fs
    
    def handles(self, path: Path) -> bool:
        """Check if this mount handles the given path"""
        return path == self.mount_point or path.is_child_of(self.mount_point)
    
    def relative_path(self, path: Path) -> Path:
        """Convert absolute path to relative path within this mount"""
        if path == self.mount_point:
            return Path('/')
        
        # Remove mount point prefix
        relative = path.path[len(self.mount_point.path):]
        return Path(relative if relative else '/')

class KAIRONFS:
    """
    Main filesystem coordinator.
    Manages multiple VFS mounts (proc, home, shared).
    """
    
    def __init__(self):
        self.mounts: List[FileSystemMount] = []
        self.watchers: Dict[str, List[Callable]] = {}  # path -> [callbacks]
    
    def mount(self, mount_point: str, fs: VirtualFileSystem):
        """Mount a filesystem at the given path"""
        path = Path(mount_point)
        mount = FileSystemMount(path, fs)
        self.mounts.append(mount)
        logger.info("fs_mount", mount_point=mount_point, fs=type(fs).__name__)
    
    def _find_mount(self, path: Path) -> Optional[FileSystemMount]:
        """Find the mount that handles this path"""
        # Sort by mount point length (longest first) to handle nested mounts
        sorted_mounts = sorted(self.mounts, key=lambda m: len(m.mount_point.path), reverse=True)
        
        for mount in sorted_mounts:
            if mount.handles(path):
                return mount
        
        return None
    
    def read(self, path: str) -> bytes:
        """Read file contents"""
        p = Path(path)
        mount = self._find_mount(p)
        
        if not mount:
            raise FileNotFoundError(f"No filesystem mounted at {path}")
        
        relative = mount.relative_path(p)
        return mount.fs.read(relative)
    
    def write(self, path: str, data: bytes) -> None:
        """Write file contents"""
        p = Path(path)
        mount = self._find_mount(p)
        
        if not mount:
            raise FileNotFoundError(f"No filesystem mounted at {path}")
        
        relative = mount.relative_path(p)
        mount.fs.write(relative, data)
        
        # Trigger watchers
        self._notify_watchers(path, data)
    
    def list(self, path: str) -> List[str]:
        """List directory contents"""
        p = Path(path)
        mount = self._find_mount(p)
        
        if not mount:
            # List mount points if at root
            if p.path == '/':
                return [m.mount_point.name for m in self.mounts if m.mount_point.parent.path == '/']
            raise FileNotFoundError(f"No filesystem mounted at {path}")
        
        relative = mount.relative_path(p)
        return mount.fs.list(relative)
    
    def stat(self, path: str) -> FileMetadata:
        """Get file metadata"""
        p = Path(path)
        mount = self._find_mount(p)
        
        if not mount:
            raise FileNotFoundError(f"No filesystem mounted at {path}")
        
        relative = mount.relative_path(p)
        return mount.fs.stat(relative)
    
    def exists(self, path: str) -> bool:
        """Check if path exists"""
        try:
            self.stat(path)
            return True
        except:
            return False
    
    def watch(self, path: str, callback: Callable[[str, bytes], None]):
        """Watch a path for changes"""
        if path not in self.watchers:
            self.watchers[path] = []
        self.watchers[path].append(callback)
        logger.info("fs_watch", path=path)
    
    def _notify_watchers(self, path: str, data: bytes):
        """Notify watchers of file changes"""
        if path in self.watchers:
            for callback in self.watchers[path]:
                try:
                    callback(path, data)
                except Exception as e:
                    logger.error("watcher_error", path=path, error=str(e))

# Global filesystem instance
filesystem = KAIRONFS()
