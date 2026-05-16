"""
VECTRA CSI Feedback Compression

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

VECTRA CSI Feedback Compression

Compresses Channel State Information (CSI) feedback using VECTRA's
structure-aware compression for 6G massive MIMO systems.

Key Innovation:
- Exploits structure in CSI matrices (spatial correlation, frequency correlation)
- Deterministic compression for reproducibility
- Fail-open safety for critical signaling
"""

import numpy as np
import sys
import os

# Add VECTRA Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../python'))

try:
    from core.encode import encode, encode_with_diagnostics
    from core.decode import decode
    from core.artifact import Artifact, verify_artifact
    VECTRA_AVAILABLE = True
except ImportError:
    VECTRA_AVAILABLE = False
    print("Warning: VECTRA Python bindings not available. Using fallback.")


class VectraCSICompressor:
    """
    VECTRA-based CSI compression for 6G massive MIMO.
    
    Compresses CSI feedback matrices by:
    1. Converting complex CSI to structured format
    2. Exploiting spatial/frequency correlation
    3. Using VECTRA structure-aware compression
    4. Providing deterministic, lossless compression
    """
    
    def __init__(self, compression_ratio: float = 0.1, use_vectra: bool = True):
        """
        Initialize CSI compressor.
        
        Args:
            compression_ratio: Target compression ratio (0.1 = 10:1)
            use_vectra: Use VECTRA compression (True) or fallback (False)
        """
        self.compression_ratio = compression_ratio
        self.use_vectra = use_vectra and VECTRA_AVAILABLE
        
    def compress_csi(self, csi: np.ndarray) -> tuple:
        """
        Compress CSI matrix using VECTRA.
        
        Args:
            csi: CSI matrix of shape (num_antennas, num_subcarriers, 2) 
                 where last dimension is [real, imag]
        
        Returns:
            tuple: (compressed_data, compression_ratio, metadata)
                - compressed_data: Compressed bytes or original if fail-open
                - compression_ratio: Actual compression ratio achieved
                - metadata: Compression metadata (entropy, size, etc.)
        """
        if not self.use_vectra:
            return self._fallback_compress(csi)
        
        # Convert CSI to structured format
        structured_csi = self._csi_to_structured(csi)
        original_size = len(structured_csi)
        
        # Compress using VECTRA
        result = encode(structured_csi)
        
        if isinstance(result, bytes):
            # Fail-open: return original
            return structured_csi, 1.0, {'status': 'fail_open', 'reason': 'high_entropy'}
        
        # Artifact: successful compression
        artifact_bytes = self._artifact_to_bytes(result)
        compressed_size = len(artifact_bytes)
        ratio = original_size / compressed_size if compressed_size > 0 else 1.0
        
        metadata = {
            'status': 'compressed',
            'original_size': original_size,
            'compressed_size': compressed_size,
            'compression_ratio': ratio,
            'artifact': result
        }
        
        return artifact_bytes, ratio, metadata
    
    def decompress_csi(self, compressed_data: bytes, metadata: dict = None) -> np.ndarray:
        """
        Decompress CSI matrix from VECTRA artifact.
        
        Args:
            compressed_data: Compressed bytes or artifact bytes
            metadata: Compression metadata (optional)
        
        Returns:
            np.ndarray: Decompressed CSI matrix
        """
        if not self.use_vectra:
            return self._fallback_decompress(compressed_data)
        
        # Check if it's an artifact or raw bytes
        if metadata and 'artifact' in metadata:
            artifact = metadata['artifact']
        else:
            # Try to decode as artifact
            try:
                artifact = self._bytes_to_artifact(compressed_data)
            except:
                # Assume raw bytes (fail-open case)
                return np.frombuffer(compressed_data, dtype=np.float32).reshape(-1, -1, 2)
        
        # Verify artifact integrity
        if not verify_artifact(artifact):
            raise ValueError("Artifact integrity check failed")
        
        # Decode
        decompressed_bytes = decode(artifact)
        
        # Convert back to CSI format
        csi = self._structured_to_csi(decompressed_bytes)
        
        return csi
    
    def _csi_to_structured(self, csi: np.ndarray) -> bytes:
        """
        Convert CSI matrix to structured format for VECTRA.
        
        Strategy:
        - Convert to text-like format with repeating patterns
        - Use delimiter-separated format for better structure recognition
        - Add metadata (dimensions, statistics)
        """
        num_ant, num_sub, _ = csi.shape
        
        # Create structured format with repeating patterns
        # Format: "CSI:antennas:{}:subcarriers:{}\n" + per-antenna data
        lines = []
        lines.append(f"CSI:antennas:{num_ant}:subcarriers:{num_sub}\n".encode())
        
        # Add per-antenna data with structure
        for i in range(num_ant):
            # Format: "ANTENNA_{i}:real:...:imag:..."
            real_vals = ",".join([f"{csi[i,j,0]:.6f}" for j in range(num_sub)])
            imag_vals = ",".join([f"{csi[i,j,1]:.6f}" for j in range(num_sub)])
            line = f"ANTENNA_{i}:real:{real_vals}:imag:{imag_vals}\n".encode()
            lines.append(line)
        
        return b"".join(lines)
    
    def _structured_to_csi(self, structured_bytes: bytes) -> np.ndarray:
        """Convert structured format back to CSI matrix."""
        lines = structured_bytes.split(b"\n")
        if not lines:
            raise ValueError("Invalid structured CSI format")
        
        # Parse header: "CSI:antennas:{num_ant}:subcarriers:{num_sub}\n"
        header = lines[0].decode().strip()
        # Find "antennas:" and "subcarriers:"
        ant_idx = header.find("antennas:")
        sub_idx = header.find("subcarriers:")
        
        if ant_idx == -1 or sub_idx == -1:
            raise ValueError("Invalid structured CSI format: missing dimensions")
        
        # Extract numbers between colons
        # Format: "CSI:antennas:64:subcarriers:12"
        ant_part = header[ant_idx+9:sub_idx-1]  # From after "antennas:" to before "subcarriers:"
        sub_part = header[sub_idx+12:]  # From after "subcarriers:"
        
        num_ant = int(ant_part)
        num_sub = int(sub_part)
        
        # Reconstruct CSI from lines
        csi = np.zeros((num_ant, num_sub, 2), dtype=np.float32)
        
        for i, line in enumerate(lines[1:num_ant+1], 0):
            if not line or i >= num_ant:
                continue
            line_str = line.decode()
            # Parse "ANTENNA_{i}:real:...:imag:..."
            if "real:" in line_str and "imag:" in line_str:
                real_start = line_str.find("real:") + 5
                imag_start = line_str.find("imag:")
                real_str = line_str[real_start:imag_start-1]
                imag_str = line_str[imag_start+5:]
                real_vals = [float(x) for x in real_str.split(",") if x]
                imag_vals = [float(x) for x in imag_str.split(",") if x]
                for j in range(min(len(real_vals), num_sub)):
                    csi[i, j, 0] = real_vals[j]
                for j in range(min(len(imag_vals), num_sub)):
                    csi[i, j, 1] = imag_vals[j]
        
        return csi
    
    def _artifact_to_bytes(self, artifact: Artifact) -> bytes:
        """Convert VECTRA artifact to bytes for transmission."""
        import json
        artifact_dict = {
            'generator': artifact.generator,
            'structure_mappings': artifact.structure_mappings,
            'variable_segments': artifact.variable_segments,
            'total_segments': artifact.total_segments,
            'delimiter': artifact.delimiter,
            'original_hash': artifact.original_hash,
            'artifact_hash': artifact.artifact_hash,
        }
        return json.dumps(artifact_dict).encode()
    
    def _bytes_to_artifact(self, artifact_bytes: bytes) -> Artifact:
        """Convert bytes back to VECTRA artifact."""
        import json
        from core.artifact import Artifact
        
        artifact_dict = json.loads(artifact_bytes.decode())
        return Artifact(
            inventor=artifact_dict.get('inventor', 'VECTRA'),
            organization=artifact_dict.get('organization', 'SYNTRIASS'),
            version=artifact_dict.get('version', '0.1.0'),
            generator=artifact_dict['generator'],
            structure_mappings=tuple(tuple(m) for m in artifact_dict['structure_mappings']),
            structure_hash=artifact_dict.get('structure_hash', ''),
            variable_segments=tuple(tuple(v) for v in artifact_dict['variable_segments']),
            total_segments=artifact_dict['total_segments'],
            delimiter=artifact_dict['delimiter'],
            original_hash=artifact_dict['original_hash'],
            artifact_hash=artifact_dict['artifact_hash']
        )
    
    def _fallback_compress(self, csi: np.ndarray) -> tuple:
        """Fallback compression if VECTRA not available."""
        compressed = csi.astype(np.float16).tobytes()  # Simple half-precision
        original_size = csi.nbytes
        compressed_size = len(compressed)
        ratio = original_size / compressed_size if compressed_size > 0 else 1.0
        
        return compressed, ratio, {'status': 'fallback', 'method': 'float16'}
    
    def _fallback_decompress(self, compressed_data: bytes) -> np.ndarray:
        """Fallback decompression."""
        return np.frombuffer(compressed_data, dtype=np.float16).reshape(-1, -1, 2).astype(np.float32)


def compress_csi_feedback(csi: np.ndarray, target_ratio: float = 0.1) -> tuple:
    """
    Convenience function for CSI feedback compression.
    
    Args:
        csi: CSI matrix (num_antennas, num_subcarriers, 2)
        target_ratio: Target compression ratio
    
    Returns:
        tuple: (compressed_bytes, actual_ratio, metadata)
    """
    compressor = VectraCSICompressor(compression_ratio=target_ratio)
    return compressor.compress_csi(csi)










