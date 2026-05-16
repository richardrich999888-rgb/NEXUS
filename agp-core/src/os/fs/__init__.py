"""
KAIRON-FS: Distributed Filesystem for AGP-OS
"""

from .vfs import (
    VirtualFileSystem,
    Path,
    FileMetadata,
    FileType,
    KAIRONFS,
    filesystem
)

from .proc_fs import proc_fs
from .home_fs import home_fs
from .shared_fs import shared_fs

# Auto-mount filesystems
filesystem.mount("/proc", proc_fs)
filesystem.mount("/home", home_fs)
filesystem.mount("/shared", shared_fs)

__all__ = [
    "VirtualFileSystem",
    "Path",
    "FileMetadata",
    "FileType",
    "KAIRONFS",
    "filesystem",
    "proc_fs",
    "home_fs",
    "shared_fs"
]
