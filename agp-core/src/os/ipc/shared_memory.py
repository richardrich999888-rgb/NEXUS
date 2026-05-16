"""
AGP-OS: Shared Memory
Shared memory segments for efficient data exchange between processes.
"""

import structlog
from typing import Dict, Optional, Set
from dataclasses import dataclass, field
from datetime import datetime
import threading

logger = structlog.get_logger()

@dataclass
class SharedMemorySegment:
    """A shared memory segment accessible by multiple processes"""
    segment_id: str
    size: int
    data: bytearray
    owner_pid: int
    attached_pids: Set[int] = field(default_factory=set)
    created_at: datetime = field(default_factory=datetime.utcnow)
    lock: threading.Lock = field(default_factory=threading.Lock)
    
    def __post_init__(self):
        self.attached_pids.add(self.owner_pid)

class SharedMemoryManager:
    """
    Manages shared memory segments.
    Processes can create, attach to, and detach from shared memory.
    """
    
    def __init__(self):
        self.segments: Dict[str, SharedMemorySegment] = {}
        self.process_segments: Dict[int, Set[str]] = {}  # PID -> segment IDs
    
    def create(self, segment_id: str, size: int, owner_pid: int) -> SharedMemorySegment:
        """
        Create a new shared memory segment.
        """
        if segment_id in self.segments:
            raise ValueError(f"Segment {segment_id} already exists")
        
        segment = SharedMemorySegment(
            segment_id=segment_id,
            size=size,
            data=bytearray(size),
            owner_pid=owner_pid
        )
        
        self.segments[segment_id] = segment
        
        if owner_pid not in self.process_segments:
            self.process_segments[owner_pid] = set()
        self.process_segments[owner_pid].add(segment_id)
        
        logger.info("shm_created", segment_id=segment_id, size=size, owner=owner_pid)
        return segment
    
    def attach(self, segment_id: str, pid: int) -> Optional[SharedMemorySegment]:
        """
        Attach a process to an existing shared memory segment.
        """
        segment = self.segments.get(segment_id)
        if not segment:
            return None
        
        with segment.lock:
            segment.attached_pids.add(pid)
        
        if pid not in self.process_segments:
            self.process_segments[pid] = set()
        self.process_segments[pid].add(segment_id)
        
        logger.info("shm_attached", segment_id=segment_id, pid=pid)
        return segment
    
    def detach(self, segment_id: str, pid: int) -> bool:
        """
        Detach a process from a shared memory segment.
        """
        segment = self.segments.get(segment_id)
        if not segment:
            return False
        
        with segment.lock:
            segment.attached_pids.discard(pid)
        
        if pid in self.process_segments:
            self.process_segments[pid].discard(segment_id)
        
        logger.info("shm_detached", segment_id=segment_id, pid=pid)
        
        # Delete segment if no processes attached
        if not segment.attached_pids:
            self._delete_segment(segment_id)
        
        return True
    
    def _delete_segment(self, segment_id: str):
        """Delete a segment"""
        if segment_id in self.segments:
            del self.segments[segment_id]
            logger.info("shm_deleted", segment_id=segment_id)
    
    def write(self, segment_id: str, offset: int, data: bytes, pid: int) -> bool:
        """
        Write data to a shared memory segment.
        """
        segment = self.segments.get(segment_id)
        if not segment:
            return False
        
        if pid not in segment.attached_pids:
            logger.warning("shm_not_attached", segment_id=segment_id, pid=pid)
            return False
        
        if offset + len(data) > segment.size:
            logger.warning("shm_overflow", segment_id=segment_id, 
                          offset=offset, data_len=len(data), size=segment.size)
            return False
        
        with segment.lock:
            segment.data[offset:offset + len(data)] = data
        
        return True
    
    def read(self, segment_id: str, offset: int, length: int, pid: int) -> Optional[bytes]:
        """
        Read data from a shared memory segment.
        """
        segment = self.segments.get(segment_id)
        if not segment:
            return None
        
        if pid not in segment.attached_pids:
            logger.warning("shm_not_attached", segment_id=segment_id, pid=pid)
            return None
        
        if offset + length > segment.size:
            length = segment.size - offset
        
        with segment.lock:
            return bytes(segment.data[offset:offset + length])
    
    def cleanup_process(self, pid: int):
        """Clean up all shared memory for a terminated process"""
        if pid not in self.process_segments:
            return
        
        segments_to_detach = list(self.process_segments[pid])
        for segment_id in segments_to_detach:
            self.detach(segment_id, pid)
        
        if pid in self.process_segments:
            del self.process_segments[pid]
    
    def get_segment_info(self, segment_id: str) -> Optional[Dict]:
        """Get information about a segment"""
        segment = self.segments.get(segment_id)
        if not segment:
            return None
        
        return {
            "segment_id": segment.segment_id,
            "size": segment.size,
            "owner_pid": segment.owner_pid,
            "attached_pids": list(segment.attached_pids),
            "created_at": segment.created_at.isoformat()
        }

# Global shared memory manager
shm_manager = SharedMemoryManager()
