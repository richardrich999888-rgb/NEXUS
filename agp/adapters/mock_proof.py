"""Mock proof system for testing."""
from dataclasses import dataclass

@dataclass
class MockProof:
    proof_bytes: bytes
    verified: bool = True

class MockProofSystem:
    def generate_zkml_proof(self, task_id: bytes, output: bytes) -> MockProof:
        return MockProof(proof_bytes=b"mock_zkml_proof", verified=True)
    
    def generate_tee_attestation(self, task_id: bytes, output: bytes) -> MockProof:
        return MockProof(proof_bytes=b"mock_tee_attestation", verified=True)
    
    def verify_proof(self, proof: MockProof) -> bool:
        return proof.verified
