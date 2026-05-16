"""
AGP-OS: WebSocket Networking
Real WebSocket server/client for inter-kernel communication.
"""

import asyncio
import json
import structlog
from typing import Dict, Optional, Callable, Any
from dataclasses import dataclass
from datetime import datetime

logger = structlog.get_logger()

# Check for websockets library
try:
    import websockets
    from websockets.server import serve
    from websockets.client import connect
    HAS_WEBSOCKETS = True
except ImportError:
    HAS_WEBSOCKETS = False
    logger.warning("websockets_not_installed", message="pip install websockets for networking")

@dataclass
class NetworkConfig:
    """Network configuration"""
    host: str = "0.0.0.0"
    port: int = 8765
    heartbeat_interval: float = 30.0
    reconnect_delay: float = 5.0
    max_message_size: int = 10 * 1024 * 1024  # 10MB

class WebSocketServer:
    """
    WebSocket server for accepting connections from remote kernels.
    """
    
    def __init__(self, kernel_id: str, config: NetworkConfig = None):
        self.kernel_id = kernel_id
        self.config = config or NetworkConfig()
        self.connections: Dict[str, Any] = {}  # peer_id -> websocket
        self.handlers: Dict[str, Callable] = {}
        self.running = False
        self.server = None
    
    def register_handler(self, message_type: str, handler: Callable):
        """Register a handler for a message type"""
        self.handlers[message_type] = handler
    
    async def _handle_connection(self, websocket):
        """Handle a new WebSocket connection"""
        peer_id = None
        
        try:
            # Wait for handshake
            handshake = await asyncio.wait_for(websocket.recv(), timeout=10)
            data = json.loads(handshake)
            
            if data.get("type") != "handshake":
                await websocket.close(1002, "Expected handshake")
                return
            
            peer_id = data.get("kernel_id")
            if not peer_id:
                await websocket.close(1002, "Missing kernel_id")
                return
            
            # Send handshake response
            await websocket.send(json.dumps({
                "type": "handshake_ack",
                "kernel_id": self.kernel_id,
                "timestamp": datetime.utcnow().timestamp()
            }))
            
            self.connections[peer_id] = websocket
            logger.info("peer_connected", peer_id=peer_id)
            
            # Main message loop
            async for message in websocket:
                try:
                    data = json.loads(message)
                    msg_type = data.get("type")
                    
                    if msg_type in self.handlers:
                        response = await self.handlers[msg_type](data)
                        if response:
                            await websocket.send(json.dumps(response))
                    else:
                        logger.warning("unknown_message_type", type=msg_type)
                        
                except json.JSONDecodeError:
                    logger.error("invalid_json")
                except Exception as e:
                    logger.error("message_handler_error", error=str(e))
        
        except asyncio.TimeoutError:
            logger.warning("handshake_timeout")
        except Exception as e:
            logger.error("connection_error", error=str(e))
        finally:
            if peer_id and peer_id in self.connections:
                del self.connections[peer_id]
                logger.info("peer_disconnected", peer_id=peer_id)
    
    async def start(self):
        """Start the WebSocket server"""
        if not HAS_WEBSOCKETS:
            logger.error("websockets_required")
            return
        
        self.running = True
        
        self.server = await serve(
            self._handle_connection,
            self.config.host,
            self.config.port,
            max_size=self.config.max_message_size
        )
        
        logger.info("server_started", host=self.config.host, port=self.config.port)
    
    async def stop(self):
        """Stop the WebSocket server"""
        self.running = False
        
        # Close all connections
        for peer_id, ws in list(self.connections.items()):
            try:
                await ws.close()
            except:
                pass
        
        if self.server:
            self.server.close()
            await self.server.wait_closed()
        
        logger.info("server_stopped")
    
    async def broadcast(self, message: dict):
        """Broadcast a message to all connected peers"""
        msg_str = json.dumps(message)
        
        for peer_id, ws in list(self.connections.items()):
            try:
                await ws.send(msg_str)
            except Exception as e:
                logger.error("broadcast_error", peer_id=peer_id, error=str(e))
    
    async def send_to_peer(self, peer_id: str, message: dict) -> bool:
        """Send a message to a specific peer"""
        ws = self.connections.get(peer_id)
        if not ws:
            return False
        
        try:
            await ws.send(json.dumps(message))
            return True
        except Exception as e:
            logger.error("send_error", peer_id=peer_id, error=str(e))
            return False

class WebSocketClient:
    """
    WebSocket client for connecting to remote kernels.
    """
    
    def __init__(self, kernel_id: str, config: NetworkConfig = None):
        self.kernel_id = kernel_id
        self.config = config or NetworkConfig()
        self.connections: Dict[str, Any] = {}
        self.handlers: Dict[str, Callable] = {}
    
    def register_handler(self, message_type: str, handler: Callable):
        """Register a handler for a message type"""
        self.handlers[message_type] = handler
    
    async def connect(self, peer_id: str, uri: str) -> bool:
        """Connect to a remote kernel"""
        if not HAS_WEBSOCKETS:
            logger.error("websockets_required")
            return False
        
        try:
            ws = await connect(uri, max_size=self.config.max_message_size)
            
            # Send handshake
            await ws.send(json.dumps({
                "type": "handshake",
                "kernel_id": self.kernel_id,
                "timestamp": datetime.utcnow().timestamp()
            }))
            
            # Wait for ack
            response = await asyncio.wait_for(ws.recv(), timeout=10)
            data = json.loads(response)
            
            if data.get("type") != "handshake_ack":
                await ws.close()
                return False
            
            self.connections[peer_id] = ws
            logger.info("connected_to_peer", peer_id=peer_id, uri=uri)
            
            # Start message receiver task
            asyncio.create_task(self._receive_loop(peer_id, ws))
            
            return True
            
        except Exception as e:
            logger.error("connect_error", peer_id=peer_id, uri=uri, error=str(e))
            return False
    
    async def _receive_loop(self, peer_id: str, ws):
        """Receive messages from a peer"""
        try:
            async for message in ws:
                try:
                    data = json.loads(message)
                    msg_type = data.get("type")
                    
                    if msg_type in self.handlers:
                        await self.handlers[msg_type](data)
                        
                except json.JSONDecodeError:
                    pass
                except Exception as e:
                    logger.error("receive_error", error=str(e))
        
        except Exception as e:
            logger.warning("peer_disconnected", peer_id=peer_id, error=str(e))
        finally:
            if peer_id in self.connections:
                del self.connections[peer_id]
    
    async def send(self, peer_id: str, message: dict) -> bool:
        """Send a message to a peer"""
        ws = self.connections.get(peer_id)
        if not ws:
            return False
        
        try:
            await ws.send(json.dumps(message))
            return True
        except Exception as e:
            logger.error("send_error", peer_id=peer_id, error=str(e))
            return False
    
    async def disconnect(self, peer_id: str):
        """Disconnect from a peer"""
        ws = self.connections.get(peer_id)
        if ws:
            try:
                await ws.close()
            except:
                pass
            del self.connections[peer_id]
    
    async def disconnect_all(self):
        """Disconnect from all peers"""
        for peer_id in list(self.connections.keys()):
            await self.disconnect(peer_id)
