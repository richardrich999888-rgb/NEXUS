"""
ASIM - Statistical Field Alignment (SFA)
Treats collective intelligence as a statistical field to be minimized for divergence.
"""
import math
import numpy as np
from typing import List, Dict, Any, Optional
from src.core.ria import ResonantInvariantAlgebra

class StatisticalFieldAlignment:
    """
    Implements alignment using the concept of Mutual Information Divergence.
    Treats multiple agent 'logic vectors' as a field.
    """
    
    def __init__(self, ria: ResonantInvariantAlgebra):
        self.ria = ria
        self.field_history: List[np.ndarray] = []
        
    def calculate_alignment_coherence(self, agent_opinions: List[str]) -> float:
        """
        Calculate the 'Coherence' of the agentic field.
        A high coherence (1.0) means agents are perfectly aligned.
        """
        if not agent_opinions:
            return 1.0
            
        # 1. Map agent opinions (logic strings) to high-dim vectors
        # Using a simple hash-based vector map for physics simulation
        vectors = []
        for opinion in agent_opinions:
            h = hashlib.sha3_256(opinion.encode()).digest()
            # Map uint8 [0, 255] to centered float [-1, 1] for better cosine sensitivity
            v = (np.frombuffer(h, dtype=np.uint8).astype(float) - 127.5) / 127.5
            vectors.append(v)
            
        # 2. Calculate the 'Field Centroid' (Mean Opinion)
        centroid = np.mean(vectors, axis=0)
        
        # Avoid division by zero for zero vectors
        centroid_norm = np.linalg.norm(centroid)
        if centroid_norm < 1e-9:
            return 0.0
            
        # 3. Calculate 'Variance' (Information Divergence)
        similarities = []
        for v in vectors:
            v_norm = np.linalg.norm(v)
            if v_norm < 1e-9:
                similarities.append(0.0)
                continue
            cos_sim = np.dot(v, centroid) / (v_norm * centroid_norm)
            similarities.append(cos_sim)
            
        coherence = np.mean(similarities)
        self.field_history.append(centroid)
        
        return float(coherence)

    def verify_collective_decision(self, agent_opinions: List[str], threshold: float = 0.9) -> bool:
        """
        Verify if a collective decision is valid based on field coherence.
        """
        coherence = self.calculate_alignment_coherence(agent_opinions)
        aligned = coherence >= threshold
        
        if not aligned:
            print(f"!!! SFA DECOHERENCE DETECTED: Coherence {coherence:.4f} < {threshold} !!!")
            
        return aligned

    def get_field_stats(self) -> Dict[str, Any]:
        """Return the current coherence state."""
        return {
            "node_id": self.ria.seed.hex()[:8],
            "field_dimensions": 32, # SHA256 bytes
            "history_depth": len(self.field_history),
            "E_anchored": int(self.ria.E)
        }

import hashlib
