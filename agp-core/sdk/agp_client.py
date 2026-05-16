"""
AGP-CORE Python SDK
Enterprise client library for AGP-CORE integration
"""

import os
import hmac
import hashlib
import time
from typing import Dict, List, Optional, Any, Union
from datetime import datetime
from dataclasses import dataclass
import httpx


@dataclass
class AgentConfig:
    """Configuration for an AGP agent"""
    agent_id: str
    api_key: str
    capabilities: List[str]
    metadata: Dict[str, Any] = None


@dataclass
class APIResponse:
    """Standard API response wrapper"""
    success: bool
    data: Optional[Any]
    error: Optional[str]
    status_code: int
    request_id: str


class AGPClient:
    """
    AGP-CORE SDK Client
    
    Enterprise-grade Python client for interacting with AGP-CORE APIs.
    
    Usage:
        client = AGPClient(base_url="https://api.agp-core.io", api_key="...")
        
        # Register agent
        agent = client.agents.register(capabilities=["inference", "verification"])
        
        # Submit observation
        client.observe.submit(agent.agent_id, stimulus_type="task_success", magnitude=0.8)
        
        # Check reputation
        rep = client.agents.get_reputation(agent.agent_id)
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:8000",
        api_key: Optional[str] = None,
        timeout: float = 30.0,
        retry_attempts: int = 3
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key or os.getenv("AGP_API_KEY")
        self.timeout = timeout
        self.retry_attempts = retry_attempts
        
        self._client = httpx.Client(timeout=timeout)
        
        # Initialize sub-clients
        self.agents = AgentsAPI(self)
        self.observe = ObserveAPI(self)
        self.blockchain = BlockchainAPI(self)
        self.swarms = SwarmsAPI(self)
        self.economics = EconomicsAPI(self)
    
    def _headers(self) -> Dict[str, str]:
        """Generate request headers"""
        headers = {
            "Content-Type": "application/json",
            "X-Request-ID": f"sdk-{int(time.time() * 1000)}",
            "User-Agent": "AGP-SDK/1.0.0"
        }
        if self.api_key:
            headers["X-API-Key"] = self.api_key
        return headers
    
    def _request(
        self,
        method: str,
        path: str,
        data: Optional[Dict] = None,
        params: Optional[Dict] = None
    ) -> APIResponse:
        """Make an API request with retry logic"""
        url = f"{self.base_url}/api/v1{path}"
        
        last_error = None
        for attempt in range(self.retry_attempts):
            try:
                response = self._client.request(
                    method=method,
                    url=url,
                    json=data,
                    params=params,
                    headers=self._headers()
                )
                
                return APIResponse(
                    success=response.is_success,
                    data=response.json() if response.is_success else None,
                    error=response.text if not response.is_success else None,
                    status_code=response.status_code,
                    request_id=response.headers.get("X-Request-ID", "")
                )
            except httpx.RequestError as e:
                last_error = str(e)
                if attempt < self.retry_attempts - 1:
                    time.sleep(2 ** attempt)  # Exponential backoff
        
        return APIResponse(
            success=False,
            data=None,
            error=f"Request failed after {self.retry_attempts} attempts: {last_error}",
            status_code=0,
            request_id=""
        )
    
    def get(self, path: str, params: Optional[Dict] = None) -> APIResponse:
        return self._request("GET", path, params=params)
    
    def post(self, path: str, data: Optional[Dict] = None) -> APIResponse:
        return self._request("POST", path, data=data)
    
    def put(self, path: str, data: Optional[Dict] = None) -> APIResponse:
        return self._request("PUT", path, data=data)
    
    def delete(self, path: str) -> APIResponse:
        return self._request("DELETE", path)
    
    def health_check(self) -> bool:
        """Check API health"""
        try:
            resp = self._client.get(f"{self.base_url}/health")
            return resp.status_code == 200
        except:
            return False
    
    def close(self):
        """Close the client"""
        self._client.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()


class AgentsAPI:
    """Agents API operations"""
    
    def __init__(self, client: AGPClient):
        self._client = client
    
    def register(
        self,
        fingerprint: str,
        agent_type: str = "inference",
        capabilities: List[str] = None
    ) -> APIResponse:
        """Register a new agent"""
        return self._client.post("/agents/", data={
            "fingerprint": fingerprint,
            "agent_type": agent_type,
            "capabilities": capabilities or []
        })
    
    def get(self, agent_id: str) -> APIResponse:
        """Get agent by ID"""
        return self._client.get(f"/agents/{agent_id}")
    
    def get_by_fingerprint(self, fingerprint: str) -> APIResponse:
        """Get agent by fingerprint"""
        return self._client.get(f"/agents/fingerprint/{fingerprint}")
    
    def list(self, skip: int = 0, limit: int = 100) -> APIResponse:
        """List agents"""
        return self._client.get("/agents/", params={"skip": skip, "limit": limit})
    
    def get_hormones(self, agent_id: str) -> APIResponse:
        """Get agent hormone levels"""
        return self._client.get(f"/agents/{agent_id}/hormones")
    
    def get_privilege(self, agent_id: str) -> APIResponse:
        """Get agent privilege level"""
        return self._client.get(f"/agents/{agent_id}/privilege")
    
    def get_reputation(self, agent_id: str) -> APIResponse:
        """Get agent reputation summary"""
        return self._client.get(f"/agents/{agent_id}/reputation")


class ObserveAPI:
    """Observations API operations"""
    
    def __init__(self, client: AGPClient):
        self._client = client
    
    def submit(
        self,
        agent_id: str,
        stimulus_type: str,
        magnitude: float,
        context: Optional[Dict] = None
    ) -> APIResponse:
        """Submit an observation"""
        return self._client.post("/observe/", data={
            "agent_id": agent_id,
            "stimulus_type": stimulus_type,
            "magnitude": magnitude,
            "context": context or {}
        })
    
    def task_success(self, agent_id: str, magnitude: float = 0.7) -> APIResponse:
        """Record task success"""
        return self._client.post(f"/observe/task_success/{agent_id}", params={"magnitude": magnitude})
    
    def task_failure(self, agent_id: str, magnitude: float = 0.7) -> APIResponse:
        """Record task failure"""
        return self._client.post(f"/observe/task_failure/{agent_id}", params={"magnitude": magnitude})
    
    def collaboration(self, agent_id: str, magnitude: float = 0.5) -> APIResponse:
        """Record collaboration"""
        return self._client.post(f"/observe/collaboration/{agent_id}", params={"magnitude": magnitude})
    
    def calculate_cost(self, agent_id: str, action_type: str, base_cost: float) -> APIResponse:
        """Calculate action cost"""
        return self._client.post("/observe/cost", data={
            "agent_id": agent_id,
            "action_type": action_type,
            "base_cost": base_cost
        })


class BlockchainAPI:
    """Blockchain API operations"""
    
    def __init__(self, client: AGPClient):
        self._client = client
    
    def list_networks(self) -> APIResponse:
        """List supported blockchain networks"""
        return self._client.get("/blockchain/networks")
    
    def connect_wallet(
        self,
        agent_id: str,
        address: str,
        chain_id: int,
        signature: str,
        message: str
    ) -> APIResponse:
        """Connect wallet to agent"""
        return self._client.post("/blockchain/connect-wallet", data={
            "agent_id": agent_id,
            "address": address,
            "chain_id": chain_id,
            "signature": signature,
            "message": message
        })
    
    def get_agent_summary(self, agent_id: str) -> APIResponse:
        """Get blockchain summary for agent"""
        return self._client.get(f"/blockchain/agent/{agent_id}/summary")


class SwarmsAPI:
    """Swarms API operations"""
    
    def __init__(self, client: AGPClient):
        self._client = client
    
    def create(
        self,
        name: str,
        objective: str,
        founder_id: str,
        founder_reputation: float = 0.5
    ) -> APIResponse:
        """Create a new swarm"""
        return self._client.post("/agents/swarms/create", data={
            "name": name,
            "objective": objective,
            "founder_id": founder_id,
            "founder_reputation": founder_reputation
        })
    
    def join(
        self,
        swarm_id: str,
        agent_id: str,
        reputation: float,
        capabilities: List[str]
    ) -> APIResponse:
        """Join a swarm"""
        return self._client.post(f"/agents/swarms/{swarm_id}/join", data={
            "agent_id": agent_id,
            "reputation": reputation,
            "capabilities": capabilities
        })
    
    def get_stats(self, swarm_id: str) -> APIResponse:
        """Get swarm statistics"""
        return self._client.get(f"/agents/swarms/{swarm_id}/stats")
    
    def propose_decision(
        self,
        swarm_id: str,
        question: str,
        options: List[str],
        duration_hours: int = 24
    ) -> APIResponse:
        """Propose a decision for swarm voting"""
        return self._client.post(f"/agents/swarms/{swarm_id}/propose", data={
            "question": question,
            "options": options,
            "duration_hours": duration_hours
        })
    
    def vote(self, decision_id: str, agent_id: str, option_index: int) -> APIResponse:
        """Cast a vote"""
        return self._client.post(
            f"/agents/swarms/decisions/{decision_id}/vote",
            params={"agent_id": agent_id, "option_index": option_index}
        )


class EconomicsAPI:
    """Economics API operations"""
    
    def __init__(self, client: AGPClient):
        self._client = client
    
    def get_distribution_summary(self) -> APIResponse:
        """Get token distribution summary"""
        return self._client.get("/economics/distribution/summary")
    
    def simulate_supply(self, months: int = 48) -> APIResponse:
        """Simulate circulating supply"""
        return self._client.get("/economics/simulation/supply", params={"months": months})
    
    def get_bridge_routes(self, source_chain: Optional[str] = None) -> APIResponse:
        """Get available bridge routes"""
        params = {"source_chain": source_chain} if source_chain else None
        return self._client.get("/economics/bridge/routes", params=params)
    
    def initiate_bridge(
        self,
        source_chain: str,
        target_chain: str,
        sender: str,
        recipient: str,
        amount: float
    ) -> APIResponse:
        """Initiate cross-chain bridge"""
        return self._client.post("/economics/bridge/initiate", data={
            "source_chain": source_chain,
            "target_chain": target_chain,
            "sender": sender,
            "recipient": recipient,
            "amount": amount
        })


# Convenience function
def create_client(
    base_url: str = "http://localhost:8000",
    api_key: Optional[str] = None
) -> AGPClient:
    """Create an AGP client"""
    return AGPClient(base_url=base_url, api_key=api_key)
