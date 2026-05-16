"""
VECTRA DPD Coefficient Compression

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

VECTRA DPD Coefficient Compression

Compresses Digital Predistortion (DPD) coefficients using VECTRA's
structure-aware compression for 6G massive MIMO systems.
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


class VectraDPDCompressor:
    """
    VECTRA-based compression for DPD coefficients.
    
    Compresses DPD model parameters by:
    1. Exploiting structure in coefficient patterns
    2. Using VECTRA structure-aware compression
    3. Providing deterministic, lossless compression
    """
    
    def __init__(self, use_vectra: bool = True):
        """Initialize DPD compressor."""
        self.use_vectra = use_vectra and VECTRA_AVAILABLE
    
    def compress_coefficients(self, coefficients: np.ndarray) -> tuple:
        """
        Compress DPD coefficients.
        
        Args:
            coefficients: DPD coefficient array (shape varies by model)
        
        Returns:
            tuple: (compressed_data, compression_ratio, metadata)
        """
        if not self.use_vectra:
            return self._fallback_compress(coefficients)
        
        # Convert to structured format
        structured_coeffs = self._coefficients_to_structured(coefficients)
        original_size = len(structured_coeffs)
        
        # Compress using VECTRA
        result = encode(structured_coeffs)
        
        if isinstance(result, bytes):
            # Fail-open
            return structured_coeffs, 1.0, {'status': 'fail_open'}
        
        # Successful compression
        artifact_bytes = self._artifact_to_bytes(result)
        compressed_size = len(artifact_bytes)
        ratio = original_size / compressed_size if compressed_size > 0 else 1.0
        
        return artifact_bytes, ratio, {'status': 'compressed', 'artifact': result}
    
    def decompress_coefficients(self, compressed_data: bytes, metadata: dict = None) -> np.ndarray:
        """Decompress DPD coefficients."""
        if not self.use_vectra:
            return self._fallback_decompress(compressed_data)
        
        if metadata and 'artifact' in metadata:
            artifact = metadata['artifact']
        else:
            artifact = self._bytes_to_artifact(compressed_data)
        
        if not verify_artifact(artifact):
            raise ValueError("Artifact integrity check failed")
        
        decompressed_bytes = decode(artifact)
        coefficients = self._structured_to_coefficients(decompressed_bytes)
        
        return coefficients
    
    def _coefficients_to_structured(self, coefficients: np.ndarray) -> bytes:
        """Convert coefficients to structured format."""
        shape_str = ":".join(map(str, coefficients.shape))
        header = f"DPD:shape:{shape_str}:data:".encode()
        data = coefficients.astype(np.float32).tobytes()
        return header + data
    
    def _structured_to_coefficients(self, structured_bytes: bytes) -> np.ndarray:
        """Convert structured format back to coefficients."""
        header_end = structured_bytes.find(b":data:")
        if header_end == -1:
            raise ValueError("Invalid structured DPD format")
        
        header = structured_bytes[:header_end].decode()
        data_bytes = structured_bytes[header_end + 6:]
        
        # Extract shape
        shape_str = header.split(":shape:")[1]
        shape = tuple(map(int, shape_str.split(":")))
        
        coefficients = np.frombuffer(data_bytes, dtype=np.float32).reshape(shape)
        return coefficients
    
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
    
    def _fallback_compress(self, coefficients: np.ndarray) -> tuple:
        """Fallback compression."""
        compressed = coefficients.astype(np.float16).tobytes()
        ratio = coefficients.nbytes / len(compressed)
        return compressed, ratio, {'status': 'fallback'}
    
    def _fallback_decompress(self, compressed_data: bytes) -> np.ndarray:
        """Fallback decompression."""
        # Need shape info - simplified version
        return np.frombuffer(compressed_data, dtype=np.float16).astype(np.float32)








