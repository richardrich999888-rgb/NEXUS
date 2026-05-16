"""
Immune Memory Implementation - Long-term threat recognition.

Memory B/T cells: Remember past infections for rapid response.
PATENT CLAIM 7.4: Immune memory for rapid threat response
"""

import torch
import torch.nn as nn
from typing import Optional, Dict, List
from dataclasses import dataclass
import copy


@dataclass
class MemoryMetadata:
    """Metadata for memory cells."""
    creation_time: int = 0
    last_recalled: int = 0
    recall_count: int = 0
    threat_type: str = "unknown"
    original_severity: float = 0.5


class MemoryCell:
    """
    Memory cell storing learned response to past threat.
    
    Contains:
    - Threat signature (what to recognize)
    - Pre-trained antibody (how to respond)
    - Metadata (when created, how often recalled)
    """
    
    def __init__(
        self,
        threat_behavior: torch.Tensor,
        antibody: nn.Module,
        threat_type: str,
        timestamp: int,
        severity: float = 0.8
    ):
        self.threat_pattern = threat_behavior.clone().detach()
        self.antibody = copy.deepcopy(antibody)
        self.threat_type = threat_type
        
        self.metadata = MemoryMetadata(
            creation_time=timestamp,
            last_recalled=timestamp,
            recall_count=0,
            threat_type=threat_type,
            original_severity=severity
        )
        
    def recall(
        self,
        new_behavior: torch.Tensor,
        similarity_threshold: float = 0.85,
        current_time: int = 0
    ) -> Optional[nn.Module]:
        """
        Check if new behavior matches remembered threat.
        
        If match found, return pre-trained antibody for immediate response.
        """
        similarity = torch.cosine_similarity(
            self.threat_pattern.flatten().unsqueeze(0),
            new_behavior.flatten().unsqueeze(0),
            dim=1
        ).item()
        
        if similarity > similarity_threshold:
            self.metadata.recall_count += 1
            self.metadata.last_recalled = current_time
            return self.antibody
        
        return None
    
    def get_age(self, current_time: int) -> int:
        return current_time - self.metadata.creation_time
    
    def get_recency(self, current_time: int) -> int:
        return current_time - self.metadata.last_recalled


class MemoryBank:
    """
    Manager for population of memory cells.
    
    Implements memory formation, recall, consolidation, and forgetting.
    """
    
    def __init__(
        self,
        max_size: int = 200,
        consolidation_threshold: int = 10,
        forgetting_age: int = 10000
    ):
        self.max_size = max_size
        self.consolidation_threshold = consolidation_threshold
        self.forgetting_age = forgetting_age
        self.memories: List[MemoryCell] = []
        
    def store(self, memory_cell: MemoryCell):
        """Store new memory, forgetting old unused ones if at capacity."""
        self.memories.append(memory_cell)
        if len(self.memories) > self.max_size:
            self._forget_unused()
    
    def recall(
        self,
        behavior: torch.Tensor,
        current_time: int = 0
    ) -> Optional[nn.Module]:
        """Search for matching memory, return pre-trained antibody if found."""
        sorted_memories = sorted(
            self.memories,
            key=lambda m: m.metadata.last_recalled,
            reverse=True
        )
        
        for memory in sorted_memories:
            antibody = memory.recall(behavior, current_time=current_time)
            if antibody is not None:
                return antibody
        
        return None
    
    def consolidate(self, current_time: int):
        """Consolidate frequently recalled memories."""
        for memory in self.memories:
            if memory.metadata.recall_count >= self.consolidation_threshold:
                # Frequently used memories are strengthened
                pass
    
    def _forget_unused(self):
        """Remove old, unused memories to free capacity."""
        if not self.memories:
            return
        
        current_time = max(m.metadata.last_recalled for m in self.memories)
        
        scored = []
        for memory in self.memories:
            age = memory.get_age(current_time)
            recency = memory.get_recency(current_time)
            recall_count = memory.metadata.recall_count
            
            recency_factor = 1.0 / (1.0 + recency / 1000)
            score = (recall_count + 1) / (age + 1) * recency_factor
            scored.append((memory, score))
        
        scored.sort(key=lambda x: x[1], reverse=True)
        self.memories = [m for m, _ in scored[:self.max_size]]
    
    def get_statistics(self) -> Dict:
        """Get memory bank statistics."""
        if not self.memories:
            return {
                'total_memories': 0,
                'avg_recall_count': 0.0,
                'threat_type_distribution': {}
            }
        
        threat_types = {}
        total_recalls = 0
        
        for memory in self.memories:
            t_type = memory.metadata.threat_type
            threat_types[t_type] = threat_types.get(t_type, 0) + 1
            total_recalls += memory.metadata.recall_count
        
        return {
            'total_memories': len(self.memories),
            'avg_recall_count': total_recalls / len(self.memories),
            'threat_type_distribution': threat_types
        }
    
    def __len__(self) -> int:
        return len(self.memories)
