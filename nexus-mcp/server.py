"""
NEXUS MCP Server
Exposes Sovereign Intelligence capabilities (TIH, Immunity) to AI Agents.
Protocol: Model Context Protocol (JSON-RPC 2.0 over Stdio)
"""
import sys
import json
import logging
import asyncio
from typing import Any, Dict, List, Optional
from pydantic import BaseModel

# Add project root to path to import NEXUS modules
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

# Import NEXUS Engines
try:
    from src.core.ria import ResonantInvariantAlgebra, create_ria_for_device
    from src.asi.tih import ThermodynamicHardening
    # Mocking Immunity for MCP demo if heavy deps (torch) are missing in this env
    # In production, we'd import the actual `ArtificialImmuneSystem`
    HAS_IMMUNITY = False
    try:
        from agp_core.src.immunity.immune_system import ArtificialImmuneSystem, ImmuneConfig
        HAS_IMMUNITY = True
    except ImportError:
        pass
except ImportError as e:
    # Fallback for dev environment without full paths
    logging.warning(f"Could not import NEXUS core: {e}. Running in Mock Mode.")
    ResonantInvariantAlgebra = None
    ThermodynamicHardening = None


# --- MCP Protocol Types ---
class JsonRpcRequest(BaseModel):
    jsonrpc: str = "2.0"
    method: str
    params: Optional[Dict[str, Any]] = None
    id: Optional[Any] = None

class JsonRpcResponse(BaseModel):
    jsonrpc: str = "2.0"
    result: Optional[Any] = None
    error: Optional[Dict[str, Any]] = None
    id: Optional[Any] = None

# --- Server Implementation ---
class NexusMcpServer:
    def __init__(self):
        self.tih = self._init_tih()
        self.patents_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../docs/INVENTION_DISCLOSURES.md"))

    def _init_tih(self):
        if ThermodynamicHardening and ResonantInvariantAlgebra:
            ria = create_ria_for_device("standard")
            return ThermodynamicHardening(ria)
        return None

    async def handle_request(self, request: JsonRpcRequest) -> JsonRpcResponse:
        try:
            if request.method == "initialize":
                return self.handle_initialize(request)
            elif request.method == "tools/list":
                return self.handle_tools_list(request)
            elif request.method == "tools/call":
                return await self.handle_tools_call(request)
            elif request.method == "resources/list":
                return self.handle_resources_list(request)
            elif request.method == "resources/read":
                return self.handle_resources_read(request)
            else:
                return JsonRpcResponse(id=request.id, error={"code": -32601, "message": "Method not found"})
        except Exception as e:
            logging.error(f"Error handling request: {e}", exc_info=True)
            return JsonRpcResponse(id=request.id, error={"code": -32000, "message": str(e)})

    def handle_initialize(self, request: JsonRpcRequest) -> JsonRpcResponse:
        return JsonRpcResponse(id=request.id, result={
            "protocolVersion": "0.1.0",
            "server": {"name": "nexus-mcp", "version": "1.0.0"},
            "capabilities": {
                "tools": {},
                "resources": {}
            }
        })

    def handle_tools_list(self, request: JsonRpcRequest) -> JsonRpcResponse:
        tools = [
            {
                "name": "measure_entropy",
                "description": "Calculates Thermodynamic Entropy of a logic trace (TIH). Detects chaotic intent.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "trace": {"type": "string", "description": "The logic chain execution trace"},
                        "intent": {"type": "string", "description": "The stated intent"}
                    },
                    "required": ["trace"]
                }
            },
            {
                "name": "scan_threat_pattern",
                "description": "Scans a behavioral vector against the Swarm Immune System.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "vector": {"type": "array", "items": {"type": "number"}, "description": "512-dim behavior vector"},
                        "source_id": {"type": "string", "description": "Origin agent ID"}
                    },
                    "required": ["vector"]
                }
            }
        ]
        return JsonRpcResponse(id=request.id, result={"tools": tools})

    async def handle_tools_call(self, request: JsonRpcRequest) -> JsonRpcResponse:
        name = request.params.get("name")
        args = request.params.get("arguments", {})
        
        if name == "measure_entropy":
            trace = args.get("trace", "")
            intent = args.get("intent", "execute")
            
            if self.tih:
                result = self.tih.monitor_intent(intent, trace)
                return JsonRpcResponse(id=request.id, result={"content": [{"type": "text", "text": json.dumps(result, indent=2)}]})
            else:
                # Mock Mode
                import math
                prob = [float(trace.count(c)) / len(trace) for c in set(trace)]
                entropy = -sum([p * math.log(p) / math.log(2.0) for p in prob])
                return JsonRpcResponse(id=request.id, result={"content": [{"type": "text", "text": json.dumps({"status": "MOCK", "entropy": entropy})}]})

        elif name == "scan_threat_pattern":
            # Simulate Immunity Scan
            vector = args.get("vector", [])
            is_malicious = sum(vector) > 10.0 # Dummy logic for MVP
            
            result = {
                "threat_detected": is_malicious,
                "confidence": 0.9 if is_malicious else 0.2,
                "immune_response": "ISOLATE" if is_malicious else "ALLOW"
            }
            return JsonRpcResponse(id=request.id, result={"content": [{"type": "text", "text": json.dumps(result, indent=2)}]})
        
        return JsonRpcResponse(id=request.id, error={"code": -32601, "message": "Tool not found"})

    def handle_resources_list(self, request: JsonRpcRequest) -> JsonRpcResponse:
        resources = [{
            "uri": "nexus://patents",
            "name": "NEXUS Invention Disclosures",
            "mimeType": "text/markdown"
        }]
        return JsonRpcResponse(id=request.id, result={"resources": resources})

    def handle_resources_read(self, request: JsonRpcRequest) -> JsonRpcResponse:
        uri = request.params.get("uri")
        if uri == "nexus://patents":
            try:
                with open(self.patents_path, "r") as f:
                    content = f.read()
                return JsonRpcResponse(id=request.id, result={"contents": [{"uri": uri, "mimeType": "text/markdown", "text": content}]})
            except FileNotFoundError:
                return JsonRpcResponse(id=request.id, error={"code": -32002, "message": "Patent file not found"})
        
        return JsonRpcResponse(id=request.id, error={"code": -32002, "message": "Resource not found"})

    async def run_stdio(self):
        """Run JSON-RPC loop over Stdin/Stdout."""
        # Use simple synchronous read for MVP robustness in script
        for line in sys.stdin:
            try:
                line = line.strip()
                if not line: continue
                
                request_dict = json.loads(line)
                request = JsonRpcRequest(**request_dict)
                response = await self.handle_request(request)
                
                # Write output
                print(response.model_dump_json(), flush=True)
            except Exception as e:
                logging.error(f"Stream error: {e}")

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, stream=sys.stderr)
    server = NexusMcpServer()
    asyncio.run(server.run_stdio())
