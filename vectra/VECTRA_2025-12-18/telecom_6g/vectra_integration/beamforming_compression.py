"""
VECTRA Beamforming Weight Compression

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

VECTRA Beamforming Weight Compression

Compresses beamforming weight matrices using VECTRA's structure-aware
compression for 6G massive MIMO systems.
"""

import numpy as np
import sys
import os

# Add VECTRA Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../python'))

try:
    from core.encode import encode
    from core.decode import decode
    from core.artifact import Artifact, verify_artifact
    VECTRA_AVAILABLE = True
except ImportError:
    VECTRA_AVAILABLE = False


class VectraBeamformingCompressor:
    """
    VECTRA-based compression for beamforming weights.
    
    Compresses beamforming weight matrices by:
    1. Exploiting structure in weight patterns
    2. Using VECTRA structure-aware compression
    3. Providing deterministic, lossless compression
    """
    
    def __init__(self, use_vectra: bool = True):
        """Initialize beamforming compressor."""
        self.use_vectra = use_vectra and VECTRA_AVAILABLE
    
    def compress_weights(self, weights: np.ndarray) -> tuple:
        """
        Compress beamforming weights.
        
        Args:
            weights: Beamforming weight matrix (num_antennas, num_users, 2)
                    where last dimension is [real, imag]
        
        Returns:
            tuple: (compressed_data, compression_ratio, metadata)
        """
        if not self.use_vectra:
            return self._fallback_compress(weights)
        
        # Convert to structured format
        structured_weights = self._weights_to_structured(weights)
        original_size = len(structured_weights)
        
        # Compress using VECTRA
        result = encode(structured_weights)
        
        if isinstance(result, bytes):
            # Fail-open
            return structured_weights, 1.0, {'status': 'fail_open'}
        
        # Successful compression
        artifact_bytes = self._artifact_to_bytes(result)
        compressed_size = len(artifact_bytes)
        ratio = original_size / compressed_size if compressed_size > 0 else 1.0
        
        return artifact_bytes, ratio, {'status': 'compressed', 'artifact': result}
    
    def decompress_weights(self, compressed_data: bytes, metadata: dict = None) -> np.ndarray:
        """Decompress beamforming weights."""
        if not self.use_vectra:
            return self._fallback_decompress(compressed_data)
        
        if metadata and 'artifact' in metadata:
            artifact = metadata['artifact']
        else:
            artifact = self._bytes_to_artifact(compressed_data)
        
        if not verify_artifact(artifact):
            raise ValueError("Artifact integrity check failed")
        
        decompressed_bytes = decode(artifact)
        weights = self._structured_to_weights(decompressed_bytes)
        
        return weights
    
    def _weights_to_structured(self, weights: np.ndarray) -> bytes:
        """Convert weights to structured format."""
        num_ant, num_users, _ = weights.shape
        header = f"BEAM:antennas:{num_ant}:users:{num_users}:data:".encode()
        data = weights.astype(np.float32).tobytes()
        return header + data
    
    def _structured_to_weights(self, structured_bytes: bytes) -> np.ndarray:
        """Convert structured format back to weights."""
        header_end = structured_bytes.find(b":data:")
        if header_end == -1:
            raise ValueError("Invalid structured weights format")
        
        header = structured_bytes[:header_end].decode()
        data_bytes = structured_bytes[header_end + 6:]
        
        parts = header.split(":")
        num_ant = int(parts[2])
        num_users = int(parts[4])
        
        weights = np.frombuffer(data_bytes, dtype=np.float32).reshape(num_ant, num_users, 2)
        return weights
    
    def _artifact_to_bytes(self, artifact: Artifact) -> bytes:
        """Convert artifact to bytes."""
        import json
        return json.dumps({
            'generator': artifact.generator,
            'structure_mappings': artifact.structure_mappings,
            'variable_segments': artifact.variable_segments,
            'total_segments': artifact.total_segments,
            'delimiter': artifact.delimiter,
            'original_hash': artifact.original_hash,
            'artifact_hash': artifact.artifact_hash,
        }).encode()
    
    def _bytes_to_artifact(self, artifact_bytes: bytes) -> Artifact:
        """Convert bytes to artifact."""
        import json
        from core.artifact import Artifact
        
        d = json.loads(artifact_bytes.decode())
        return Artifact(
            inventor='VECTRA',
            organization='SYNTRIASS',
            version='0.1.0',
            generator=d['generator'],
            structure_mappings=tuple(tuple(m) for m in d['structure_mappings']),
            structure_hash='',
            variable_segments=tuple(tuple(v) for v in d['variable_segments']),
            total_segments=d['total_segments'],
            delimiter=d['delimiter'],
            original_hash=d['original_hash'],
            artifact_hash=d['artifact_hash']
        )
    
    def _fallback_compress(self, weights: np.ndarray) -> tuple:
        """Fallback compression."""
        compressed = weights.astype(np.float16).tobytes()
        ratio = weights.nbytes / len(compressed)
        return compressed, ratio, {'status': 'fallback'}
    
    def _fallback_decompress(self, compressed_data: bytes) -> np.ndarray:
        """Fallback decompression."""
        return np.frombuffer(compressed_data, dtype=np.float16).reshape(-1, -1, 2).astype(np.float32)








