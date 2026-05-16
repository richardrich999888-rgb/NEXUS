"""
KAIRON Cache Adapter for AGP-CORE
Native CRDT-based distributed cache replacing Redis
Based on KAIRON's BoundedLWWRegister and StateWeave patterns
"""

import asyncio
import hashlib
import json
import time
from typing import Dict, Optional, Any, List, Set, Generic, TypeVar
from datetime import datetime
from dataclasses import dataclass, field
from collections import defaultdict
import threading
from abc import ABC, abstractmethod

T = TypeVar('T')


# =============================================================================
# HLC (Hybrid Logical Clock) - from KAIRON
# =============================================================================

@dataclass
class HLCTimestamp:
    """Hybrid Logical Clock timestamp"""
    physical_time: int  # Wall clock ms
    logical_counter: int  # Logical extension
    node_id: str
    
    def __lt__(self, other: "HLCTimestamp") -> bool:
        if self.physical_time != other.physical_time:
            return self.physical_time < other.physical_time
        if self.logical_counter != other.logical_counter:
            return self.logical_counter < other.logical_counter
        return self.node_id < other.node_id
    
    def __le__(self, other: "HLCTimestamp") -> bool:
        return self == other or self < other
    
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, HLCTimestamp):
            return False
        return (self.physical_time == other.physical_time and 
                self.logical_counter == other.logical_counter and
                self.node_id == other.node_id)
    
    def to_string(self) -> str:
        return f"{self.physical_time}:{self.logical_counter}:{self.node_id}"


class HLC:
    """Hybrid Logical Clock generator"""
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.physical_time = 0
        self.logical_counter = 0
        self._lock = threading.Lock()
    
    def now(self) -> HLCTimestamp:
        with self._lock:
            wall = int(time.time() * 1000)
            
            if wall > self.physical_time:
                self.physical_time = wall
                self.logical_counter = 0
            else:
                self.logical_counter += 1
            
            return HLCTimestamp(
                physical_time=self.physical_time,
                logical_counter=self.logical_counter,
                node_id=self.node_id
            )
    
    def receive(self, remote: HLCTimestamp) -> HLCTimestamp:
        with self._lock:
            wall = int(time.time() * 1000)
            
            if wall > self.physical_time and wall > remote.physical_time:
                self.physical_time = wall
                self.logical_counter = 0
            elif remote.physical_time > self.physical_time:
                self.physical_time = remote.physical_time
                self.logical_counter = remote.logical_counter + 1
            elif self.physical_time > remote.physical_time:
                self.logical_counter += 1
            else:
                self.logical_counter = max(self.logical_counter, remote.logical_counter) + 1
            
            return HLCTimestamp(
                physical_time=self.physical_time,
                logical_counter=self.logical_counter,
                node_id=self.node_id
            )


# =============================================================================
# CRDT Base Types - from KAIRON
# =============================================================================

@dataclass
class BoundsConfig:
    """Bounds configuration for BoundedLWWRegister"""
    min_value: Optional[float] = None
    max_value: Optional[float] = None
    max_delta_per_ms: Optional[float] = None
    mode: str = "clamp"  # "reject" or "clamp"


class BoundedLWWRegister(Generic[T]):
    """
    KAIRON Bounded-LWW-Register CRDT
    Last-Writer-Wins with physical plausibility constraints
    """
    
    def __init__(self, value: T, timestamp: HLCTimestamp, bounds: Optional[BoundsConfig] = None):
        self.value = value
        self.timestamp = timestamp
        self.bounds = bounds or BoundsConfig()
        self.rejected_count = 0
    
    def set(self, new_value: T, timestamp: HLCTimestamp) -> bool:
        """Set value if timestamp is newer and bounds are valid"""
        if timestamp < self.timestamp:
            return False
        
        # Validate bounds for numeric types
        if isinstance(new_value, (int, float)) and self.bounds:
            if not self._validate_bounds(new_value, timestamp):
                self.rejected_count += 1
                return False
        
        self.value = new_value
        self.timestamp = timestamp
        return True
    
    def _validate_bounds(self, new_value: float, timestamp: HLCTimestamp) -> bool:
        """Validate value against bounds (KAIRON novelty)"""
        # Static bounds
        if self.bounds.min_value is not None and new_value < self.bounds.min_value:
            if self.bounds.mode == "reject":
                return False
            new_value = self.bounds.min_value
        
        if self.bounds.max_value is not None and new_value > self.bounds.max_value:
            if self.bounds.mode == "reject":
                return False
            new_value = self.bounds.max_value
        
        # Rate-of-change bounds
        if self.bounds.max_delta_per_ms is not None and isinstance(self.value, (int, float)):
            delta_time = timestamp.physical_time - self.timestamp.physical_time
            if delta_time > 0:
                delta_value = abs(new_value - float(self.value))
                max_allowed = self.bounds.max_delta_per_ms * delta_time
                if delta_value > max_allowed:
                    return False
        
        return True
    
    def merge(self, other: "BoundedLWWRegister[T]") -> "BoundedLWWRegister[T]":
        """Merge with another register (LWW semantics with bounds)"""
        if other.timestamp > self.timestamp:
            if isinstance(other.value, (int, float)) and self.bounds:
                if self._validate_bounds(other.value, other.timestamp):
                    return BoundedLWWRegister(other.value, other.timestamp, self.bounds)
                else:
                    self.rejected_count += 1
                    return self
            return BoundedLWWRegister(other.value, other.timestamp, self.bounds)
        return self
    
    def get(self) -> T:
        return self.value


class GCounter:
    """KAIRON Grow-only Counter CRDT"""
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.counts: Dict[str, int] = defaultdict(int)
    
    def increment(self, amount: int = 1):
        self.counts[self.node_id] += amount
    
    def value(self) -> int:
        return sum(self.counts.values())
    
    def merge(self, other: "GCounter") -> "GCounter":
        result = GCounter(self.node_id)
        all_nodes = set(self.counts.keys()) | set(other.counts.keys())
        for node in all_nodes:
            result.counts[node] = max(self.counts.get(node, 0), other.counts.get(node, 0))
        return result


class ORSet(Generic[T]):
    """KAIRON Observed-Remove Set CRDT"""
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.elements: Dict[T, Set[str]] = defaultdict(set)
        self.tombstones: Set[str] = set()
        self._counter = 0
    
    def add(self, element: T):
        self._counter += 1
        unique_id = f"{self.node_id}:{self._counter}"
        self.elements[element].add(unique_id)
    
    def remove(self, element: T):
        if element in self.elements:
            self.tombstones.update(self.elements[element])
    
    def contains(self, element: T) -> bool:
        if element not in self.elements:
            return False
        live = self.elements[element] - self.tombstones
        return len(live) > 0
    
    def values(self) -> List[T]:
        return [e for e in self.elements if self.contains(e)]
    
    def merge(self, other: "ORSet[T]") -> "ORSet[T]":
        result = ORSet(self.node_id)
        all_elements = set(self.elements.keys()) | set(other.elements.keys())
        for elem in all_elements:
            result.elements[elem] = self.elements.get(elem, set()) | other.elements.get(elem, set())
        result.tombstones = self.tombstones | other.tombstones
        return result


# =============================================================================
# KAIRON Cache Service
# =============================================================================

class KaironCache:
    """
    KAIRON-native cache service for AGP-CORE
    
    Replaces Redis with CRDT-based distributed state:
    - BoundedLWWRegister for key-value with physical plausibility
    - GCounter for metrics
    - ORSet for collections
    - HLC for ordering
    """
    
    def __init__(self, node_id: str = "agp-core-1", sync_interval_ms: int = 5000):
        self.node_id = node_id
        self.hlc = HLC(node_id)
        self.sync_interval_ms = sync_interval_ms
        
        # CRDT state
        self.registers: Dict[str, BoundedLWWRegister] = {}
        self.counters: Dict[str, GCounter] = {}
        self.sets: Dict[str, ORSet] = {}
        
        self._lock = threading.RLock()
        self._running = False
    
    async def start(self):
        """Start the cache service"""
        self._running = True
    
    async def stop(self):
        """Stop the cache service"""
        self._running = False
    
    # =========================================================================
    # KEY-VALUE (BoundedLWWRegister)
    # =========================================================================
    
    async def get(self, key: str) -> Optional[Any]:
        """Get value for key"""
        with self._lock:
            if key not in self.registers:
                return None
            
            reg = self.registers[key]
            val = reg.get()
            
            # Check expiration
            if isinstance(val, dict) and "__expires_at" in val:
                if time.time() * 1000 > val["__expires_at"]:
                    del self.registers[key]
                    return None
                return val.get("__value")
            
            return val
    
    async def set(self, key: str, value: Any, ttl_seconds: Optional[int] = None,
                  bounds: Optional[BoundsConfig] = None):
        """Set key to value with optional TTL and bounds"""
        with self._lock:
            ts = self.hlc.now()
            
            if ttl_seconds:
                stored = {
                    "__value": value,
                    "__expires_at": ts.physical_time + (ttl_seconds * 1000)
                }
            else:
                stored = value
            
            if key in self.registers:
                self.registers[key].set(stored, ts)
            else:
                self.registers[key] = BoundedLWWRegister(stored, ts, bounds)
    
    async def delete(self, key: str) -> bool:
        """Delete a key"""
        with self._lock:
            if key in self.registers:
                ts = self.hlc.now()
                self.registers[key].set(None, ts)
                return True
            return False
    
    async def exists(self, key: str) -> bool:
        """Check if key exists"""
        val = await self.get(key)
        return val is not None
    
    async def setex(self, key: str, seconds: int, value: Any):
        """Set with expiration"""
        await self.set(key, value, ttl_seconds=seconds)
    
    # =========================================================================
    # COUNTERS (GCounter)
    # =========================================================================
    
    async def incr(self, key: str, amount: int = 1) -> int:
        """Increment counter"""
        with self._lock:
            if key not in self.counters:
                self.counters[key] = GCounter(self.node_id)
            self.counters[key].increment(amount)
            return self.counters[key].value()
    
    async def get_counter(self, key: str) -> int:
        """Get counter value"""
        with self._lock:
            if key not in self.counters:
                return 0
            return self.counters[key].value()
    
    # =========================================================================
    # SETS (ORSet)
    # =========================================================================
    
    async def sadd(self, key: str, *members: str) -> int:
        """Add members to set"""
        with self._lock:
            if key not in self.sets:
                self.sets[key] = ORSet(self.node_id)
            
            added = 0
            for member in members:
                if not self.sets[key].contains(member):
                    self.sets[key].add(member)
                    added += 1
            return added
    
    async def srem(self, key: str, *members: str) -> int:
        """Remove members from set"""
        with self._lock:
            if key not in self.sets:
                return 0
            
            removed = 0
            for member in members:
                if self.sets[key].contains(member):
                    self.sets[key].remove(member)
                    removed += 1
            return removed
    
    async def smembers(self, key: str) -> List[str]:
        """Get all set members"""
        with self._lock:
            if key not in self.sets:
                return []
            return self.sets[key].values()
    
    async def sismember(self, key: str, member: str) -> bool:
        """Check if member is in set"""
        with self._lock:
            if key not in self.sets:
                return False
            return self.sets[key].contains(member)
    
    # =========================================================================
    # CLUSTER SYNC
    # =========================================================================
    
    def merge_from(self, other: "KaironCache"):
        """Merge state from another node (CRDT merge)"""
        with self._lock:
            # Merge registers
            for key, reg in other.registers.items():
                if key in self.registers:
                    self.registers[key] = self.registers[key].merge(reg)
                else:
                    self.registers[key] = reg
            
            # Merge counters
            for key, counter in other.counters.items():
                if key in self.counters:
                    self.counters[key] = self.counters[key].merge(counter)
                else:
                    self.counters[key] = counter
            
            # Merge sets
            for key, orset in other.sets.items():
                if key in self.sets:
                    self.sets[key] = self.sets[key].merge(orset)
                else:
                    self.sets[key] = orset
    
    def state_vector(self) -> Dict[str, int]:
        """Get state vector for sync detection"""
        with self._lock:
            return {
                "registers": len(self.registers),
                "counters": sum(c.value() for c in self.counters.values()),
                "sets": sum(len(s.values()) for s in self.sets.values()),
                "hlc": self.hlc.physical_time
            }
    
    # =========================================================================
    # STATS
    # =========================================================================
    
    async def info(self) -> Dict:
        """Get cache statistics"""
        with self._lock:
            rejected = sum(r.rejected_count for r in self.registers.values() 
                          if hasattr(r, 'rejected_count'))
            return {
                "node_id": self.node_id,
                "registers": len(self.registers),
                "counters": len(self.counters),
                "sets": len(self.sets),
                "rejected_updates": rejected,
                "hlc_time": self.hlc.physical_time
            }
    
    async def ping(self) -> str:
        """Health check"""
        return "PONG"


# =============================================================================
# Global Instance
# =============================================================================

from src.config import settings

kairon_cache = KaironCache(
    node_id=settings.kairon_node_id,
    sync_interval_ms=settings.kairon_sync_interval_ms
)


def get_cache() -> KaironCache:
    """Get KAIRON cache instance"""
    return kairon_cache


async def init_cache() -> KaironCache:
    """Initialize KAIRON cache"""
    await kairon_cache.start()
    return kairon_cache
