# Network manager for handling peers and offline verification

"""Provides a simple manager to keep track of peer nodes and perform offline verification.
Future enhancements will include peer discovery, consensus, and Merkle proof handling.
"""

from typing import Dict, List

from .peer import PeerNode
from .offline import OfflineVerifier

class NetworkManager:
    """Manages peer nodes and offline verification.

    Attributes:
        peers: Mapping from peer_id to PeerNode instances.
        offline_verifier: Instance of OfflineVerifier for local verification.
    """

    def __init__(self, ria_instance, initial_invariants: Dict[str, Dict]):
        self.peers: Dict[str, PeerNode] = {}
        self.offline_verifier = OfflineVerifier(ria_instance, initial_invariants)

    def add_peer(self, peer_id: str, base_url: str) -> None:
        """Add a new peer to the manager."""
        self.peers[peer_id] = PeerNode(peer_id, base_url)

    def get_peer(self, peer_id: str) -> PeerNode:
        """Retrieve a peer node by its identifier."""
        return self.peers.get(peer_id)

    def list_peers(self) -> List[Dict[str, str]]:
        """Return a list of peer information dictionaries."""
        return [{"peer_id": pid, "base_url": p.base_url} for pid, p in self.peers.items()]

    def verify_offline(self, signature, sender_id: str, network_id: str = "mainnet") -> bool:
        """Delegate verification to the OfflineVerifier."""
        return self.offline_verifier.verify(signature, sender_id, network_id)
