# Peer node implementation for AURA network

"""PeerNode handles communication between AURA nodes.
It provides methods to connect, send, and receive messages.
For now, it uses a simple HTTP-based placeholder implementation.
"""

import requests
from typing import Any, Dict

class PeerNode:
    """Represents a remote AURA node.

    Attributes:
        node_id: Unique identifier for the peer.
        base_url: Base URL for the peer's API.
    """

    def __init__(self, node_id: str, base_url: str) -> None:
        self.node_id = node_id
        self.base_url = base_url.rstrip('/')

    def send(self, endpoint: str, payload: Dict[str, Any]) -> Dict[str, Any]:
        """Send a JSON payload to the peer.

        Args:
            endpoint: API endpoint relative to the base URL.
            payload: JSON-serializable dictionary.

        Returns:
            Parsed JSON response from the peer.
        """
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        response = requests.post(url, json=payload, timeout=5)
        response.raise_for_status()
        return response.json()

    def health_check(self) -> bool:
        """Check if the peer is reachable.

        Returns:
            True if the peer responds with HTTP 200 to /health, else False.
        """
        try:
            resp = requests.get(f"{self.base_url}/health", timeout=3)
            return resp.status_code == 200
        except Exception:
            return False
