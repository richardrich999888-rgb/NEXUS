"""
Webhook & Event Streaming Service - Phase 7
Partner integrations and real-time event delivery
"""

import uuid
import hmac
import hashlib
import asyncio
from typing import Dict, List, Optional, Any, Callable
from datetime import datetime
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
import httpx


class EventType(str, Enum):
    AGENT_CREATED = "agent.created"
    AGENT_UPDATED = "agent.updated"
    REPUTATION_CHANGED = "reputation.changed"
    PRIVILEGE_CHANGED = "privilege.changed"
    OBSERVATION_RECORDED = "observation.recorded"
    SWARM_CREATED = "swarm.created"
    SWARM_MEMBER_JOINED = "swarm.member.joined"
    DECISION_PROPOSED = "decision.proposed"
    DECISION_FINALIZED = "decision.finalized"
    TASK_OFFERED = "task.offered"
    TASK_ACCEPTED = "task.accepted"
    TASK_COMPLETED = "task.completed"
    BRIDGE_INITIATED = "bridge.initiated"
    BRIDGE_COMPLETED = "bridge.completed"
    STAKE_CREATED = "stake.created"
    STAKE_RELEASED = "stake.released"


@dataclass
class WebhookEndpoint:
    """Registered webhook endpoint"""
    id: uuid.UUID
    url: str
    secret: str
    events: List[EventType]
    active: bool = True
    created_at: datetime = field(default_factory=datetime.utcnow)
    last_delivery: Optional[datetime] = None
    failure_count: int = 0
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class Event:
    """System event"""
    id: uuid.UUID
    type: EventType
    timestamp: datetime
    data: Dict[str, Any]
    source: str
    version: str = "1.0"


@dataclass
class DeliveryAttempt:
    """Webhook delivery attempt"""
    id: uuid.UUID
    webhook_id: uuid.UUID
    event_id: uuid.UUID
    timestamp: datetime
    status_code: int
    success: bool
    response_time_ms: float
    error: Optional[str] = None


class WebhookService:
    """
    Manages webhook registrations and event delivery
    """
    
    def __init__(self):
        self.webhooks: Dict[uuid.UUID, WebhookEndpoint] = {}
        self.events: List[Event] = []
        self.deliveries: List[DeliveryAttempt] = []
        self.subscribers: Dict[EventType, List[uuid.UUID]] = defaultdict(list)
        self._client = httpx.AsyncClient(timeout=30.0)
    
    def generate_secret(self) -> str:
        """Generate a webhook signing secret"""
        return hashlib.sha256(uuid.uuid4().bytes).hexdigest()
    
    def register_webhook(
        self,
        url: str,
        events: List[EventType],
        metadata: Optional[Dict] = None
    ) -> WebhookEndpoint:
        """Register a new webhook endpoint"""
        webhook = WebhookEndpoint(
            id=uuid.uuid4(),
            url=url,
            secret=self.generate_secret(),
            events=events,
            metadata=metadata or {}
        )
        
        self.webhooks[webhook.id] = webhook
        
        # Index by event type
        for event_type in events:
            self.subscribers[event_type].append(webhook.id)
        
        return webhook
    
    def update_webhook(
        self,
        webhook_id: uuid.UUID,
        events: Optional[List[EventType]] = None,
        active: Optional[bool] = None
    ) -> Optional[WebhookEndpoint]:
        """Update webhook configuration"""
        webhook = self.webhooks.get(webhook_id)
        if not webhook:
            return None
        
        if events is not None:
            # Update subscriptions
            for event_type in webhook.events:
                if webhook_id in self.subscribers[event_type]:
                    self.subscribers[event_type].remove(webhook_id)
            
            webhook.events = events
            for event_type in events:
                self.subscribers[event_type].append(webhook_id)
        
        if active is not None:
            webhook.active = active
        
        return webhook
    
    def delete_webhook(self, webhook_id: uuid.UUID) -> bool:
        """Delete a webhook"""
        webhook = self.webhooks.get(webhook_id)
        if not webhook:
            return False
        
        for event_type in webhook.events:
            if webhook_id in self.subscribers[event_type]:
                self.subscribers[event_type].remove(webhook_id)
        
        del self.webhooks[webhook_id]
        return True
    
    def sign_payload(self, secret: str, payload: bytes) -> str:
        """Sign payload for verification"""
        return hmac.new(secret.encode(), payload, hashlib.sha256).hexdigest()
    
    async def emit(
        self,
        event_type: EventType,
        data: Dict[str, Any],
        source: str = "agp-core"
    ) -> Event:
        """Emit an event and deliver to subscribers"""
        event = Event(
            id=uuid.uuid4(),
            type=event_type,
            timestamp=datetime.utcnow(),
            data=data,
            source=source
        )
        
        self.events.append(event)
        
        # Deliver to subscribers
        webhook_ids = self.subscribers.get(event_type, [])
        
        for webhook_id in webhook_ids:
            webhook = self.webhooks.get(webhook_id)
            if webhook and webhook.active:
                asyncio.create_task(self._deliver(webhook, event))
        
        return event
    
    async def _deliver(self, webhook: WebhookEndpoint, event: Event):
        """Deliver event to webhook endpoint"""
        import json
        
        payload = json.dumps({
            "id": str(event.id),
            "type": event.type.value,
            "timestamp": event.timestamp.isoformat(),
            "data": event.data,
            "source": event.source
        }).encode()
        
        signature = self.sign_payload(webhook.secret, payload)
        
        headers = {
            "Content-Type": "application/json",
            "X-AGP-Signature": signature,
            "X-AGP-Event": event.type.value,
            "X-AGP-Delivery": str(uuid.uuid4())
        }
        
        start_time = datetime.utcnow()
        
        try:
            response = await self._client.post(
                webhook.url,
                content=payload,
                headers=headers
            )
            
            response_time = (datetime.utcnow() - start_time).total_seconds() * 1000
            success = response.is_success
            
            delivery = DeliveryAttempt(
                id=uuid.uuid4(),
                webhook_id=webhook.id,
                event_id=event.id,
                timestamp=datetime.utcnow(),
                status_code=response.status_code,
                success=success,
                response_time_ms=response_time
            )
            
            if success:
                webhook.failure_count = 0
            else:
                webhook.failure_count += 1
            
            webhook.last_delivery = datetime.utcnow()
            
        except Exception as e:
            response_time = (datetime.utcnow() - start_time).total_seconds() * 1000
            
            delivery = DeliveryAttempt(
                id=uuid.uuid4(),
                webhook_id=webhook.id,
                event_id=event.id,
                timestamp=datetime.utcnow(),
                status_code=0,
                success=False,
                response_time_ms=response_time,
                error=str(e)
            )
            
            webhook.failure_count += 1
        
        self.deliveries.append(delivery)
        
        # Disable webhook after too many failures
        if webhook.failure_count >= 10:
            webhook.active = False
    
    def get_webhook_stats(self, webhook_id: uuid.UUID) -> Dict:
        """Get delivery statistics for a webhook"""
        webhook = self.webhooks.get(webhook_id)
        if not webhook:
            return {}
        
        deliveries = [d for d in self.deliveries if d.webhook_id == webhook_id]
        successful = [d for d in deliveries if d.success]
        
        return {
            "webhook_id": str(webhook_id),
            "total_deliveries": len(deliveries),
            "successful": len(successful),
            "success_rate": len(successful) / len(deliveries) if deliveries else 0,
            "avg_response_time_ms": sum(d.response_time_ms for d in deliveries) / len(deliveries) if deliveries else 0,
            "failure_count": webhook.failure_count,
            "active": webhook.active
        }


class PluginSystem:
    """
    Plugin system for extending AGP-CORE functionality
    """
    
    def __init__(self, webhook_service: WebhookService):
        self.webhook_service = webhook_service
        self.plugins: Dict[str, Dict] = {}
        self.hooks: Dict[str, List[Callable]] = defaultdict(list)
    
    def register_plugin(
        self,
        name: str,
        version: str,
        author: str,
        hooks: List[str],
        config: Optional[Dict] = None
    ) -> Dict:
        """Register a plugin"""
        plugin = {
            "id": str(uuid.uuid4()),
            "name": name,
            "version": version,
            "author": author,
            "hooks": hooks,
            "config": config or {},
            "enabled": True,
            "registered_at": datetime.utcnow().isoformat()
        }
        
        self.plugins[name] = plugin
        return plugin
    
    def add_hook(self, hook_name: str, handler: Callable):
        """Add a hook handler"""
        self.hooks[hook_name].append(handler)
    
    async def execute_hook(self, hook_name: str, *args, **kwargs) -> List[Any]:
        """Execute all handlers for a hook"""
        results = []
        
        for handler in self.hooks.get(hook_name, []):
            try:
                if asyncio.iscoroutinefunction(handler):
                    result = await handler(*args, **kwargs)
                else:
                    result = handler(*args, **kwargs)
                results.append(result)
            except Exception as e:
                results.append({"error": str(e)})
        
        return results
    
    def disable_plugin(self, name: str) -> bool:
        """Disable a plugin"""
        if name in self.plugins:
            self.plugins[name]["enabled"] = False
            return True
        return False
    
    def get_plugin(self, name: str) -> Optional[Dict]:
        """Get plugin info"""
        return self.plugins.get(name)
    
    def list_plugins(self, enabled_only: bool = False) -> List[Dict]:
        """List all plugins"""
        plugins = list(self.plugins.values())
        if enabled_only:
            plugins = [p for p in plugins if p["enabled"]]
        return plugins


# Create singleton instances
webhook_service = WebhookService()
plugin_system = PluginSystem(webhook_service)
