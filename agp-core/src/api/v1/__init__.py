"""
API v1 module
"""

from fastapi import APIRouter
from .agents import router as agents_router
from .observe import router as observe_router
from .system import router as system_router
from .blockchain import router as blockchain_router
from .economics import router as economics_router
from .autonomous import router as autonomous_router
from .webhooks import router as webhooks_router
from .governance import router as governance_router

router = APIRouter()
router.include_router(agents_router)
router.include_router(observe_router)
router.include_router(system_router)
router.include_router(blockchain_router)
router.include_router(economics_router)
router.include_router(autonomous_router)
router.include_router(webhooks_router)
router.include_router(governance_router)
