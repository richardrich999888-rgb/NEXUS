"""
AGP-CORE FastAPI Application
Main entry point with endocrine-based agent governance
"""

import asyncio
from contextlib import asynccontextmanager
from typing import Optional

from fastapi import FastAPI, HTTPException, Request, Response
from fastapi.responses import JSONResponse, HTMLResponse
from fastapi.middleware.cors import CORSMiddleware
from prometheus_client import CONTENT_TYPE_LATEST, generate_latest

from src.os.kernel import kernel
from src.os.observability.prometheus import prom_metrics
from src.os.persistence.database import db
from src.os.security.auth import auth_manager

import structlog

from prometheus_client import make_asgi_app, Counter, Histogram
import time

from src.config import settings
from src.core.database import Database, apply_decay_all
from src.api.v1 import router as api_v1_router

# Structured logging
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    wrapper_class=structlog.stdlib.BoundLogger,
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger()

# Prometheus metrics
REQUEST_COUNT = Counter(
    'agp_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status']
)
REQUEST_LATENCY = Histogram(
    'agp_request_latency_seconds',
    'HTTP request latency',
    ['method', 'endpoint']
)
HORMONE_SECRETION = Counter(
    'agp_hormone_secretion_total',
    'Total hormone secretions',
    ['hormone', 'stimulus_type']
)
DECAY_CYCLES = Counter(
    'agp_decay_cycles_total',
    'Total decay cycles executed'
)


# Background decay task
async def decay_scheduler():
    """Background task to apply hormone decay at regular intervals"""
    while True:
        try:
            await asyncio.sleep(settings.decay_interval)
            agents_updated = await apply_decay_all(settings.decay_interval)
            DECAY_CYCLES.inc()
            logger.info("decay_cycle_complete", agents_updated=agents_updated)
        except asyncio.CancelledError:
            break
        except Exception as e:
            logger.error("decay_cycle_error", error=str(e))


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan management"""
    # Startup
    logger.info("starting_agp_core", environment=settings.environment)
    
    # Connect to database
    await Database.connect()
    logger.info("database_connected")
    
    # Start background decay scheduler
    decay_task = asyncio.create_task(decay_scheduler())
    logger.info("decay_scheduler_started", interval=settings.decay_interval)
    
    yield
    
    # Shutdown
    logger.info("shutting_down_agp_core")
    
    # Cancel decay scheduler
    decay_task.cancel()
    try:
        await decay_task
    except asyncio.CancelledError:
        pass
    
    # Disconnect from database
    await Database.disconnect()
    logger.info("database_disconnected")


# Create FastAPI app
app = FastAPI(
    title="AGP-CORE",
    description="""
    Artificial Governance Protocol - Core Service
    
    Implements biologically-inspired agent governance through an Artificial Human Endocrine System (AHES).
    
    ## Hormones
    
    | Hormone | Dimension | Function |
    |---------|-----------|----------|
    | Cortisol | Accuracy | Performance under stress |
    | Oxytocin | Cooperation | Multi-agent collaboration |
    | Serotonin | Stability | Consistent behavior |
    | Dopamine | Uniqueness | Novel solutions |
    | Adrenaline | Latency | Response speed |
    | Endorphins | Ethics | Constraint adherence |
    | Norepinephrine | Novelty | Exploration |
    | GrowthHormone | Longevity | Long-term consistency |
    
    ## Key Principles
    
    - **Continuous signals**: Hormone levels [0.0, 1.0], never binary
    - **Half-life decay**: Levels decay naturally over time
    - **Receptor saturation**: Michaelis-Menten kinetics
    - **Negative feedback**: High levels inhibit further secretion
    """,
    version=settings.app_version,
    lifespan=lifespan
)

# Middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

if settings.environment == "production":
    app.add_middleware(
        TrustedHostMiddleware,
        allowed_hosts=["*.agp-core.io", "localhost"]
    )


@app.middleware("http")
async def metrics_middleware(request: Request, call_next):
    """Track request metrics"""
    start_time = time.time()
    
    response = await call_next(request)
    
    duration = time.time() - start_time
    endpoint = request.url.path
    
    REQUEST_COUNT.labels(
        method=request.method,
        endpoint=endpoint,
        status=response.status_code
    ).inc()
    
    REQUEST_LATENCY.labels(
        method=request.method,
        endpoint=endpoint
    ).observe(duration)
    
    return response


# Include API routers
app.include_router(api_v1_router, prefix="/api/v1")

# Mount Prometheus metrics
metrics_app = make_asgi_app()
app.mount("/metrics", metrics_app)


# Root endpoints
@app.on_event("startup")
async def startup_event():
    """Boot the kernel on application startup"""
    kernel.boot(recover=True)
    logger.info("application_startup", status="kernel_booted")

@app.get("/metrics")
async def metrics():
    """Prometheus metrics endpoint"""
    return Response(
        content=prom_metrics.get_metrics(),
        media_type=prom_metrics.get_content_type()
    )

@app.get("/system/stats")
async def system_stats():
    """Production system statistics"""
    return {
        "kernel": {
            "uptime": time.time() - kernel.start_time,
            "running": kernel.running,
            "processes": len(kernel.process_table)
        },
        "persistence": db.get_stats(),
        "health": "ok"
    }
    return {
        "service": "AGP-CORE",
        "version": settings.app_version,
        "status": "operational",
        "description": "Artificial Governance Protocol with Endocrine-based Reputation"
    }


@app.get("/health", tags=["Health"])
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "database": "connected",
        "decay_scheduler": "running",
        "environment": settings.environment
    }


@app.get("/ready", tags=["Health"])
async def readiness_check():
    """Readiness check for k8s"""
    try:
        pool = await Database.get_pool()
        async with pool.acquire() as conn:
            await conn.fetchval("SELECT 1")
        return {"ready": True}
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Not ready: {str(e)}")


# Error handlers
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    """Global exception handler"""
    logger.error(
        "unhandled_exception",
        path=request.url.path,
        method=request.method,
        error=str(exc)
    )
    return JSONResponse(
        status_code=500,
        content={"detail": "Internal server error"}
    )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "src.main:app",
        host=settings.host,
        port=settings.port,
        reload=settings.debug
    )
