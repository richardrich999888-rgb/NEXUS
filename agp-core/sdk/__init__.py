"""
AGP-CORE SDK
"""

from .agp_client import (
    AGPClient,
    AgentConfig,
    APIResponse,
    create_client,
    AgentsAPI,
    ObserveAPI,
    BlockchainAPI,
    SwarmsAPI,
    EconomicsAPI
)

__version__ = "1.0.0"
__all__ = [
    "AGPClient",
    "AgentConfig",
    "APIResponse",
    "create_client",
    "AgentsAPI",
    "ObserveAPI",
    "BlockchainAPI",
    "SwarmsAPI",
    "EconomicsAPI"
]
