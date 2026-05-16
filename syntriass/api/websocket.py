"""
SYNTRIASS Path 6 — WebSocket API

Bi-directional WebSocket for:
- Streaming preview frames to frontend
- Receiving user control inputs (prompt edits, sliders)
"""

import asyncio
import json
import websockets
from websockets.server import WebSocketServerProtocol
from typing import Dict, Set, Optional, Callable
from syntriass.preview.stream import PreviewBus, FrameEncoder, stream_preview


class PreviewWebSocketServer:
    """
    WebSocket server for preview streaming and control.
    
    Handles:
    - Preview frame streaming
    - User control input (prompt, sliders)
    - Connection management
    """
    
    def __init__(
        self,
        preview_bus: PreviewBus,
        host: str = "localhost",
        port: int = 8765,
    ):
        """
        Args:
            preview_bus: PreviewBus instance
            host: Server host
            port: Server port
        """
        self.preview_bus = preview_bus
        self.host = host
        self.port = port
        self.connections: Set[WebSocketServerProtocol] = set()
        self.control_callbacks: Dict[str, Callable] = {}
        self.encoder = FrameEncoder()
        
    def register_control_handler(
        self,
        control_type: str,
        handler: Callable,
    ):
        """
        Register handler for control messages.
        
        Args:
            control_type: Control type (e.g., "prompt", "style")
            handler: Async handler function
        """
        self.control_callbacks[control_type] = handler
    
    async def handle_client(self, websocket: WebSocketServerProtocol, path: str):
        """
        Handle WebSocket client connection.
        
        Args:
            websocket: WebSocket connection
            path: Connection path
        """
        self.connections.add(websocket)
        print(f"Client connected: {websocket.remote_address}")
        
        try:
            # Start preview stream
            stream_task = asyncio.create_task(
                stream_preview(self.preview_bus, websocket, self.encoder)
            )
            
            # Handle control messages
            async for message in websocket:
                try:
                    data = json.loads(message)
                    await self._handle_control_message(websocket, data)
                except json.JSONDecodeError:
                    await websocket.send(json.dumps({
                        "type": "error",
                        "message": "Invalid JSON"
                    }))
                except Exception as e:
                    await websocket.send(json.dumps({
                        "type": "error",
                        "message": str(e)
                    }))
            
            stream_task.cancel()
            
        except websockets.exceptions.ConnectionClosed:
            pass
        finally:
            self.connections.remove(websocket)
            print(f"Client disconnected: {websocket.remote_address}")
    
    async def _handle_control_message(
        self,
        websocket: WebSocketServerProtocol,
        data: Dict,
    ):
        """
        Handle control message from client.
        
        Args:
            websocket: WebSocket connection
            data: Message data
        """
        msg_type = data.get("type")
        
        if msg_type == "control":
            control_type = data.get("control_type")
            
            if control_type in self.control_callbacks:
                handler = self.control_callbacks[control_type]
                result = await handler(data.get("payload", {}))
                
                await websocket.send(json.dumps({
                    "type": "control_ack",
                    "control_type": control_type,
                    "result": result,
                }))
            else:
                await websocket.send(json.dumps({
                    "type": "error",
                    "message": f"Unknown control type: {control_type}"
                }))
        
        elif msg_type == "ping":
            await websocket.send(json.dumps({"type": "pong"}))
    
    async def broadcast_preview(self, frame_data: str):
        """
        Broadcast preview frame to all connected clients.
        
        Args:
            frame_data: Encoded frame data
        """
        disconnected = set()
        
        for conn in self.connections:
            try:
                await conn.send(frame_data)
            except:
                disconnected.add(conn)
        
        # Clean up disconnected clients
        self.connections -= disconnected
    
    async def start(self):
        """Start WebSocket server"""
        print(f"Starting WebSocket server on ws://{self.host}:{self.port}")
        
        async with websockets.serve(self.handle_client, self.host, self.port):
            await asyncio.Future()  # Run forever
    
    def run(self):
        """Run server (blocking)"""
        asyncio.run(self.start())


async def create_websocket_server(
    preview_bus: PreviewBus,
    host: str = "localhost",
    port: int = 8765,
) -> PreviewWebSocketServer:
    """
    Create and configure WebSocket server.
    
    Args:
        preview_bus: PreviewBus instance
        host: Server host
        port: Server port
        
    Returns:
        Configured PreviewWebSocketServer
    """
    server = PreviewWebSocketServer(preview_bus, host, port)
    return server

