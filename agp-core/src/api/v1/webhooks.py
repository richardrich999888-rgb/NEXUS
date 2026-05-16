"""
Webhook API - Phase 7
"""

import uuid
from typing import List, Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services.webhook_service import (
    webhook_service, plugin_system, EventType
)

router = APIRouter(prefix="/webhooks", tags=["webhooks"])


class RegisterWebhookRequest(BaseModel):
    url: str
    events: List[str]
    metadata: Optional[dict] = None

class UpdateWebhookRequest(BaseModel):
    events: Optional[List[str]] = None
    active: Optional[bool] = None

class RegisterPluginRequest(BaseModel):
    name: str
    version: str
    author: str
    hooks: List[str]
    config: Optional[dict] = None


@router.post("/register")
async def register_webhook(request: RegisterWebhookRequest):
    """Register a new webhook endpoint"""
    try:
        events = [EventType(e) for e in request.events]
        webhook = webhook_service.register_webhook(
            url=request.url,
            events=events,
            metadata=request.metadata
        )
        return {
            "id": str(webhook.id),
            "url": webhook.url,
            "secret": webhook.secret,
            "events": [e.value for e in webhook.events]
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/{webhook_id}")
async def get_webhook(webhook_id: uuid.UUID):
    """Get webhook details"""
    webhook = webhook_service.webhooks.get(webhook_id)
    if not webhook:
        raise HTTPException(status_code=404, detail="Webhook not found")
    return {
        "id": str(webhook.id),
        "url": webhook.url,
        "events": [e.value for e in webhook.events],
        "active": webhook.active,
        "failure_count": webhook.failure_count
    }

@router.patch("/{webhook_id}")
async def update_webhook(webhook_id: uuid.UUID, request: UpdateWebhookRequest):
    """Update webhook configuration"""
    events = [EventType(e) for e in request.events] if request.events else None
    webhook = webhook_service.update_webhook(webhook_id, events, request.active)
    if not webhook:
        raise HTTPException(status_code=404, detail="Webhook not found")
    return {"updated": True}

@router.delete("/{webhook_id}")
async def delete_webhook(webhook_id: uuid.UUID):
    """Delete a webhook"""
    if not webhook_service.delete_webhook(webhook_id):
        raise HTTPException(status_code=404, detail="Webhook not found")
    return {"deleted": True}

@router.get("/{webhook_id}/stats")
async def get_webhook_stats(webhook_id: uuid.UUID):
    """Get webhook delivery statistics"""
    return webhook_service.get_webhook_stats(webhook_id)

@router.get("/events/types")
async def list_event_types():
    """List available event types"""
    return [{"type": e.value, "name": e.name} for e in EventType]


# Plugin endpoints
@router.post("/plugins/register")
async def register_plugin(request: RegisterPluginRequest):
    """Register a plugin"""
    plugin = plugin_system.register_plugin(
        name=request.name,
        version=request.version,
        author=request.author,
        hooks=request.hooks,
        config=request.config
    )
    return plugin

@router.get("/plugins")
async def list_plugins(enabled_only: bool = False):
    """List registered plugins"""
    return plugin_system.list_plugins(enabled_only)

@router.get("/plugins/{name}")
async def get_plugin(name: str):
    """Get plugin info"""
    plugin = plugin_system.get_plugin(name)
    if not plugin:
        raise HTTPException(status_code=404, detail="Plugin not found")
    return plugin

@router.post("/plugins/{name}/disable")
async def disable_plugin(name: str):
    """Disable a plugin"""
    if not plugin_system.disable_plugin(name):
        raise HTTPException(status_code=404, detail="Plugin not found")
    return {"disabled": True}
