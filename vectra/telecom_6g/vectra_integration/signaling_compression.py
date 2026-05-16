"""
VECTRA Signaling Message Compression

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

VECTRA Signaling Message Compression

Compresses 5G/6G signaling messages (NAS, RRC, NGAP) using VECTRA's
structure-aware compression.

Key Benefits:
- 2x-5x compression for structured protocol messages
- Deterministic compression for testing/reproducibility
- Transparent to protocol stack (fail-open safety)
"""

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


class VectraSignalingCompressor:
    """
    VECTRA-based compression for 5G/6G signaling messages.
    
    Supports:
    - NAS (Non-Access Stratum) messages
    - RRC (Radio Resource Control) messages
    - NGAP (NG Application Protocol) messages
    - Custom protocol messages
    """
    
    def __init__(self, use_vectra: bool = True):
        """
        Initialize signaling compressor.
        
        Args:
            use_vectra: Use VECTRA compression (True) or pass-through (False)
        """
        self.use_vectra = use_vectra and VECTRA_AVAILABLE
        
    def compress_message(self, message: bytes, message_type: str = "NAS") -> tuple:
        """
        Compress signaling message using VECTRA.
        
        Args:
            message: Raw message bytes
            message_type: Message type ("NAS", "RRC", "NGAP", etc.)
        
        Returns:
            tuple: (compressed_data, compression_ratio, metadata)
        """
        if not self.use_vectra:
            return message, 1.0, {'status': 'pass_through', 'reason': 'vectra_unavailable'}
        
        # Add message type header for structure
        structured_message = self._add_message_header(message, message_type)
        
        # Compress using VECTRA
        result = encode(structured_message)
        
        if isinstance(result, bytes):
            # Fail-open: return original
            return message, 1.0, {'status': 'fail_open', 'reason': 'high_entropy'}
        
        # Artifact: successful compression
        artifact_bytes = self._artifact_to_bytes(result)
        original_size = len(message)
        compressed_size = len(artifact_bytes)
        ratio = original_size / compressed_size if compressed_size > 0 else 1.0
        
        metadata = {
            'status': 'compressed',
            'message_type': message_type,
            'original_size': original_size,
            'compressed_size': compressed_size,
            'compression_ratio': ratio,
            'artifact': result
        }
        
        return artifact_bytes, ratio, metadata
    
    def decompress_message(self, compressed_data: bytes, metadata: dict = None) -> bytes:
        """
        Decompress signaling message from VECTRA artifact.
        
        Args:
            compressed_data: Compressed bytes or artifact bytes
            metadata: Compression metadata (optional)
        
        Returns:
            bytes: Decompressed message
        """
        if not self.use_vectra:
            return compressed_data
        
        # Check if it's an artifact or raw bytes
        if metadata and 'artifact' in metadata:
            artifact = metadata['artifact']
        else:
            # Try to decode as artifact
            try:
                artifact = self._bytes_to_artifact(compressed_data)
            except:
                # Assume raw bytes (fail-open case)
                return compressed_data
        
        # Verify artifact integrity
        if not verify_artifact(artifact):
            raise ValueError("Artifact integrity check failed")
        
        # Decode
        decompressed_bytes = decode(artifact)
        
        # Remove message header
        message = self._remove_message_header(decompressed_bytes)
        
        return message
    
    def _add_message_header(self, message: bytes, message_type: str) -> bytes:
        """Add message type header for structure recognition."""
        header = f"MSG_TYPE:{message_type}:".encode()
        return header + message
    
    def _remove_message_header(self, structured_bytes: bytes) -> bytes:
        """Remove message type header."""
        header_end = structured_bytes.find(b":", structured_bytes.find(b":") + 1) + 1
        if header_end > 0:
            return structured_bytes[header_end:]
        return structured_bytes
    
    def _artifact_to_bytes(self, artifact: Artifact) -> bytes:
        """Convert VECTRA artifact to bytes."""
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


def compress_nas_message(message: bytes) -> tuple:
    """Compress NAS message."""
    compressor = VectraSignalingCompressor()
    return compressor.compress_message(message, "NAS")


def compress_rrc_message(message: bytes) -> tuple:
    """Compress RRC message."""
    compressor = VectraSignalingCompressor()
    return compressor.compress_message(message, "RRC")


def compress_ngap_message(message: bytes) -> tuple:
    """Compress NGAP message."""
    compressor = VectraSignalingCompressor()
    return compressor.compress_message(message, "NGAP")










