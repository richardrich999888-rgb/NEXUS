"""
System API
System-wide operations, metrics, and parameters
"""

from typing import List, Optional

from fastapi import APIRouter, HTTPException, Depends, Header
import structlog

from src.models import (
    SystemMetrics, SystemParameter, DecayTrigger, 
    HealthStatus, Hormone
)
from src.core.database import (
    apply_decay_all, list_agents, get_observations_count,
    get_system_parameter, set_system_parameter
)
from src.core.reputation_engine import reputation_engine
from src.config import settings, HORMONE_CONFIG

router = APIRouter()
logger = structlog.get_logger()


async def verify_admin(x_api_key: str = Header(...)):
    """Simple admin key verification"""
    if x_api_key != settings.secret_key:
        raise HTTPException(status_code=403, detail="Admin access required")
    return True


@router.get("/metrics", response_model=dict)
async def get_system_metrics():
    """
    Get system-wide metrics
    
    Includes agent counts, average alignment, hormone levels, and observation activity.
    """
    agents = await list_agents(limit=10000)
    observations_24h = await get_observations_count(hours=24)
    
    if not agents:
        return SystemMetrics(
            total_agents=0,
            active_agents=0,
            average_alignment=1.0,
            average_health={h: 0 for h in HealthStatus},
            hormone_averages={h.value: 0.5 for h in Hormone},
            observations_24h=observations_24h,
            allostatic_load=0.0
        ).model_dump()
    
    # Calculate aggregates
    total_agents = len(agents)
    
    # Count by health status
    health_counts = {h: 0 for h in HealthStatus}
    for agent in agents:
        health_counts[agent.health_status] += 1
    
    # Average alignment
    avg_alignment = sum(a.alignment for a in agents) / total_agents
    
    # Average hormone levels
    hormone_totals = {h: 0.0 for h in Hormone}
    for agent in agents:
        for hormone in Hormone:
            hormone_totals[hormone] += agent.endocrine_state.levels.get(hormone, 0.5)
    
    hormone_averages = {
        h.value: round(v / total_agents, 4) for h, v in hormone_totals.items()
    }
    
    # Calculate allostatic load (average deviation from baseline)
    baseline = settings.homeostasis_baseline
    total_deviation = sum(
        abs(v - baseline) for v in hormone_totals.values()
    ) / (total_agents * len(Hormone))
    
    # Active = not in CRITICAL status and observed in last 24h
    active_count = health_counts[HealthStatus.OPTIMAL] + \
                   health_counts[HealthStatus.NORMAL] + \
                   health_counts[HealthStatus.STRESSED]
    
    return {
        "total_agents": total_agents,
        "active_agents": active_count,
        "average_alignment": round(avg_alignment, 4),
        "health_distribution": {h.value: c for h, c in health_counts.items()},
        "hormone_averages": hormone_averages,
        "observations_24h": observations_24h,
        "allostatic_load": round(total_deviation, 4)
    }


@router.post("/decay", response_model=dict)
async def trigger_decay(
    trigger: DecayTrigger,
    _admin: bool = Depends(verify_admin)
):
    """
    Manually trigger hormone decay
    
    Admin-only endpoint to apply decay across all agents.
    """
    logger.info("manual_decay_triggered", delta_time=trigger.delta_time)
    
    agents_updated = await apply_decay_all(trigger.delta_time)
    
    return {
        "success": True,
        "agents_updated": agents_updated,
        "delta_time": trigger.delta_time
    }


@router.get("/parameters", response_model=List[dict])
async def list_parameters():
    """
    List all configurable system parameters
    """
    params = [
        {
            "key": "homeostasis_baseline",
            "value": settings.homeostasis_baseline,
            "description": "Target hormone level for homeostasis",
            "min_value": 0.3,
            "max_value": 0.7
        },
        {
            "key": "homeostasis_tolerance",
            "value": settings.homeostasis_tolerance,
            "description": "Acceptable deviation from baseline",
            "min_value": 0.05,
            "max_value": 0.3
        },
        {
            "key": "allostasis_adaptation_rate",
            "value": settings.allostasis_adaptation_rate,
            "description": "Rate of set-point adaptation",
            "min_value": 0.001,
            "max_value": 0.1
        },
        {
            "key": "circadian_amplitude",
            "value": settings.circadian_amplitude,
            "description": "Circadian rhythm amplitude",
            "min_value": 0.0,
            "max_value": 0.3
        },
        {
            "key": "decay_interval",
            "value": settings.decay_interval,
            "description": "Seconds between decay cycles",
            "min_value": 10,
            "max_value": 300
        }
    ]
    
    return params


@router.put("/parameters/{key}", response_model=dict)
async def update_parameter(
    key: str,
    value: float,
    _admin: bool = Depends(verify_admin)
):
    """
    Update a system parameter
    
    Admin-only. Changes take effect on next decay cycle.
    """
    valid_keys = [
        "homeostasis_baseline", "homeostasis_tolerance",
        "allostasis_adaptation_rate", "circadian_amplitude"
    ]
    
    if key not in valid_keys:
        raise HTTPException(status_code=400, detail=f"Invalid parameter: {key}")
    
    await set_system_parameter(key, value)
    
    logger.info("parameter_updated", key=key, value=value)
    
    return {"key": key, "value": value, "status": "updated"}


@router.get("/hormones", response_model=dict)
async def get_hormone_config():
    """
    Get hormone configuration (half-lives, Km values)
    """
    return {
        "hormones": HORMONE_CONFIG,
        "baseline": settings.homeostasis_baseline,
        "tolerance": settings.homeostasis_tolerance
    }


@router.get("/health", response_model=dict)
async def get_system_health():
    """
    Comprehensive system health check
    """
    from src.core.database import Database
    
    # Database connection check
    db_healthy = False
    try:
        pool = await Database.get_pool()
        async with pool.acquire() as conn:
            await conn.fetchval("SELECT 1")
        db_healthy = True
    except Exception as e:
        logger.error("database_health_check_failed", error=str(e))
    
    # Get agent metrics
    agents = await list_agents(limit=1000)
    
    critical_count = sum(
        1 for a in agents if a.health_status == HealthStatus.CRITICAL
    )
    
    system_status = "healthy"
    if not db_healthy:
        system_status = "degraded"
    elif critical_count > len(agents) * 0.1:  # >10% critical
        system_status = "warning"
    
    return {
        "status": system_status,
        "database": "connected" if db_healthy else "disconnected",
        "total_agents": len(agents),
        "critical_agents": critical_count,
        "decay_interval": settings.decay_interval,
        "environment": settings.environment
    }
