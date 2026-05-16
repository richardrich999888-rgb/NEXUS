"""
FYNTRAX TFEC Adapter

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Adapter for TFEC (Telecom Forward Error Correction).
Integrates VECTRA compression with TFEC encoding.
"""

from typing import Optional, Tuple
from dataclasses import dataclass


@dataclass
class TFECResult:
    """TFEC compression result."""
    original_size: int
    compressed_size: int
    ratio: float
    entropy_before: float
    entropy_after: float


class TFECAdapter:
    """
    Adapter for TFEC entropy compression.
    
    Integrates FYNTRAX with TFEC for protocol entropy reduction.
    """
    
    def __init__(self, compression_level: int = 5):
        """
        Initialize TFEC adapter.
        
        Args:
            compression_level: Compression level (1-9)
        """
        self.compression_level = compression_level
        self.enabled = True
        self.total_bytes_in = 0
        self.total_bytes_out = 0

    def compress(self, payload: bytes) -> bytes:
        """
        Compress payload using TFEC.
        
        Args:
            payload: Input bytes
            
        Returns:
            Compressed bytes
        """
        if not self.enabled or not payload:
            return payload
        
        # Placeholder: integrate actual TFEC engine here
        # For now, simulate compression
        self.total_bytes_in += len(payload)
        
        # Simulated compression ratio based on level
        ratio = 0.5 + 0.05 * (9 - self.compression_level)
        compressed = payload[:int(len(payload) * ratio)]
        
        self.total_bytes_out += len(compressed)
        return compressed

    def decompress(self, compressed: bytes) -> bytes:
        """
        Decompress TFEC payload.
        
        Args:
            compressed: Compressed bytes
            
        Returns:
            Original bytes
        """
        # Placeholder: integrate actual TFEC engine
        return compressed

    def analyze(self, payload: bytes) -> TFECResult:
        """
        Analyze payload compressibility.
        
        Args:
            payload: Input bytes
            
        Returns:
            Analysis result
        """
        from ..models.entropy import byte_entropy
        
        original_size = len(payload)
        compressed = self.compress(payload)
        compressed_size = len(compressed)
        
        entropy_before = byte_entropy(payload)
        entropy_after = byte_entropy(compressed) if compressed else 0.0
        
        return TFECResult(
            original_size=original_size,
            compressed_size=compressed_size,
            ratio=compressed_size / original_size if original_size > 0 else 1.0,
            entropy_before=entropy_before,
            entropy_after=entropy_after,
        )

    def statistics(self) -> dict:
        """Get adapter statistics."""
        return {
            "enabled": self.enabled,
            "compression_level": self.compression_level,
            "total_bytes_in": self.total_bytes_in,
            "total_bytes_out": self.total_bytes_out,
            "overall_ratio": self.total_bytes_out / self.total_bytes_in if self.total_bytes_in > 0 else 1.0,
        }
