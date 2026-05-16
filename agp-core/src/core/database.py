"""
AGP-CORE Database Layer
Async PostgreSQL operations with asyncpg
"""

import asyncpg
import uuid
import json
from typing import Optional, List, Dict, Any
from datetime import datetime, timedelta
import hashlib

from src.config import settings
from src.models import (
    Agent, AgentCreate, Hormone, EndocrineState, 
    HealthStatus, PrivilegeLevel, AgentType
)


class Database:
    """Async PostgreSQL connection pool manager"""
    
    _pool: Optional[asyncpg.Pool] = None
    
    @classmethod
    async def connect(cls):
        """Initialize connection pool"""
        if cls._pool is None:
            # Parse URL for asyncpg (remove +asyncpg suffix)
            db_url = settings.database_url.replace("postgresql+asyncpg://", "postgresql://")
            cls._pool = await asyncpg.create_pool(
                db_url,
                min_size=5,
                max_size=settings.db_pool_size,
                command_timeout=60
            )
        return cls._pool
    
    @classmethod
    async def disconnect(cls):
        """Close connection pool"""
        if cls._pool:
            await cls._pool.close()
            cls._pool = None
    
    @classmethod
    async def get_pool(cls) -> asyncpg.Pool:
        """Get or create pool"""
        if cls._pool is None:
            await cls.connect()
        return cls._pool


class DatabaseProxy:
    """
    Lazy proxy for database operations.
    Allows importing 'db' without requiring an active connection.
    """
    
    @property
    def pool(self):
        """Get pool - requires async context"""
        return Database._pool
    
    async def acquire(self):
        """Acquire a connection from the pool"""
        pool = await Database.get_pool()
        return pool.acquire()
    
    async def execute(self, query: str, *args):
        """Execute a query"""
        pool = await Database.get_pool()
        async with pool.acquire() as conn:
            return await conn.execute(query, *args)
    
    async def fetch(self, query: str, *args):
        """Fetch all rows"""
        pool = await Database.get_pool()
        async with pool.acquire() as conn:
            return await conn.fetch(query, *args)
    
    async def fetchrow(self, query: str, *args):
        """Fetch single row"""
        pool = await Database.get_pool()
        async with pool.acquire() as conn:
            return await conn.fetchrow(query, *args)
    
    async def fetchval(self, query: str, *args):
        """Fetch single value"""
        pool = await Database.get_pool()
        async with pool.acquire() as conn:
            return await conn.fetchval(query, *args)


# Global lazy database proxy - can be imported without active connection
db = DatabaseProxy()


# =============================================================================
# AGENT OPERATIONS
# =============================================================================

async def create_agent(agent: AgentCreate) -> Agent:
    """Create a new agent with initial endocrine state"""
    pool = await Database.get_pool()
    
    agent_id = uuid.uuid4()
    now = datetime.utcnow()
    
    # Generate fingerprint from agent data
    fingerprint_data = f"{agent_id}{agent.name}{agent.model_hash or ''}{now.isoformat()}"
    fingerprint = hashlib.sha256(fingerprint_data.encode()).hexdigest()[:32]
    
    # Initialize endocrine state
    if agent.initial_levels:
        levels = {h.value: agent.initial_levels.get(h, 0.5) for h in Hormone}
    else:
        levels = {h.value: 0.5 for h in Hormone}
    
    endocrine_state = {"levels": levels, "system_time": 0.0}
    
    async with pool.acquire() as conn:
        await conn.execute("""
            INSERT INTO agents (
                id, name, fingerprint, agent_type, model_hash, operator_id,
                endocrine_state, alignment, privilege_level, health_status,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        """,
            agent_id, agent.name, fingerprint, agent.agent_type.value,
            agent.model_hash, agent.operator_id, json.dumps(endocrine_state),
            1.0, PrivilegeLevel.STANDARD.value, HealthStatus.NORMAL.value,
            now, now
        )
    
    return Agent(
        id=agent_id,
        name=agent.name,
        fingerprint=fingerprint,
        agent_type=agent.agent_type,
        model_hash=agent.model_hash,
        operator_id=agent.operator_id,
        endocrine_state=EndocrineState.from_vector([0.5] * 8),
        alignment=1.0,
        privilege_level=PrivilegeLevel.STANDARD,
        health_status=HealthStatus.NORMAL,
        created_at=now,
        updated_at=now
    )


async def get_agent(agent_id: uuid.UUID) -> Optional[Agent]:
    """Get agent by ID"""
    pool = await Database.get_pool()
    
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT * FROM agents WHERE id = $1", agent_id
        )
    
    if not row:
        return None
    
    return _row_to_agent(row)


async def get_agent_by_fingerprint(fingerprint: str) -> Optional[Agent]:
    """Get agent by fingerprint"""
    pool = await Database.get_pool()
    
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT * FROM agents WHERE fingerprint = $1", fingerprint
        )
    
    if not row:
        return None
    
    return _row_to_agent(row)


async def list_agents(
    limit: int = 100,
    offset: int = 0,
    agent_type: Optional[AgentType] = None
) -> List[Agent]:
    """List agents with pagination"""
    pool = await Database.get_pool()
    
    async with pool.acquire() as conn:
        if agent_type:
            rows = await conn.fetch(
                """SELECT * FROM agents 
                   WHERE agent_type = $1 
                   ORDER BY created_at DESC 
                   LIMIT $2 OFFSET $3""",
                agent_type.value, limit, offset
            )
        else:
            rows = await conn.fetch(
                """SELECT * FROM agents 
                   ORDER BY created_at DESC 
                   LIMIT $1 OFFSET $2""",
                limit, offset
            )
    
    return [_row_to_agent(row) for row in rows]


async def update_agent_state(
    agent_id: uuid.UUID,
    endocrine_state: Dict,
    alignment: float,
    health_status: HealthStatus
) -> bool:
    """Update agent endocrine state"""
    pool = await Database.get_pool()
    
    # Calculate privilege level from endocrine state
    avg_level = sum(endocrine_state.get("levels", {}).values()) / 8
    if avg_level >= settings.privilege_high_threshold:
        privilege = PrivilegeLevel.ELEVATED
    elif avg_level >= 0.5:
        privilege = PrivilegeLevel.STANDARD
    elif avg_level >= settings.privilege_low_threshold:
        privilege = PrivilegeLevel.BASIC
    else:
        privilege = PrivilegeLevel.MINIMAL
    
    async with pool.acquire() as conn:
        result = await conn.execute("""
            UPDATE agents SET
                endocrine_state = $1,
                alignment = $2,
                health_status = $3,
                privilege_level = $4,
                updated_at = $5
            WHERE id = $6
        """,
            json.dumps(endocrine_state), alignment, health_status.value,
            privilege.value, datetime.utcnow(), agent_id
        )
    
    return "UPDATE 1" in result


async def apply_decay_all(delta_time: float) -> int:
    """Apply hormone decay to all agents"""
    pool = await Database.get_pool()
    
    async with pool.acquire() as conn:
        rows = await conn.fetch("SELECT id, endocrine_state FROM agents")
        
        updated = 0
        for row in rows:
            state = json.loads(row['endocrine_state'])
            levels = state.get("levels", {})
            
            baseline = settings.homeostasis_baseline
            
            # Apply decay for each hormone
            from src.config import HORMONE_CONFIG
            for hormone, config in HORMONE_CONFIG.items():
                if hormone in levels:
                    current = levels[hormone]
                    half_life = config["half_life"]
                    decay_factor = 0.5 ** (delta_time / half_life)
                    # Decay towards baseline
                    levels[hormone] = baseline + (current - baseline) * decay_factor
            
            state["levels"] = levels
            state["system_time"] = state.get("system_time", 0) + delta_time
            
            await conn.execute(
                "UPDATE agents SET endocrine_state = $1, updated_at = $2 WHERE id = $3",
                json.dumps(state), datetime.utcnow(), row['id']
            )
            updated += 1
    
    return updated


# =============================================================================
# OBSERVATION OPERATIONS
# =============================================================================

async def record_observation(
    agent_id: uuid.UUID,
    stimulus_type: str,
    strength: float,
    hormones_affected: Dict[str, float],
    observer_id: Optional[uuid.UUID] = None,
    protocol_id: Optional[uuid.UUID] = None
) -> uuid.UUID:
    """Record an observation"""
    pool = await Database.get_pool()
    
    obs_id = uuid.uuid4()
    
    async with pool.acquire() as conn:
        await conn.execute("""
            INSERT INTO observations (
                id, agent_id, stimulus_type, strength, hormones_affected,
                observer_id, protocol_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        """,
            obs_id, agent_id, stimulus_type, strength, json.dumps(hormones_affected),
            observer_id, protocol_id, datetime.utcnow()
        )
    
    return obs_id


async def get_observations_count(hours: int = 24) -> int:
    """Get observation count in last N hours"""
    pool = await Database.get_pool()
    
    since = datetime.utcnow() - timedelta(hours=hours)
    
    async with pool.acquire() as conn:
        result = await conn.fetchval(
            "SELECT COUNT(*) FROM observations WHERE created_at > $1",
            since
        )
    
    return result or 0


# =============================================================================
# SYSTEM PARAMETERS
# =============================================================================

async def get_system_parameter(key: str) -> Optional[float]:
    """Get a system parameter value"""
    pool = await Database.get_pool()
    
    async with pool.acquire() as conn:
        result = await conn.fetchval(
            "SELECT value FROM system_parameters WHERE key = $1", key
        )
    
    return result


async def set_system_parameter(key: str, value: float) -> bool:
    """Set a system parameter value"""
    pool = await Database.get_pool()
    
    async with pool.acquire() as conn:
        await conn.execute("""
            INSERT INTO system_parameters (key, value, updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = $3
        """, key, value, datetime.utcnow())
    
    return True


# =============================================================================
# HELPERS
# =============================================================================

def _row_to_agent(row) -> Agent:
    """Convert database row to Agent model"""
    state_data = json.loads(row['endocrine_state'])
    levels = state_data.get("levels", {})
    
    # Convert string keys to Hormone enum
    enum_levels = {}
    for h in Hormone:
        enum_levels[h] = levels.get(h.value, 0.5)
    
    endocrine_state = EndocrineState(levels=enum_levels, system_time=state_data.get("system_time", 0))
    
    return Agent(
        id=row['id'],
        name=row['name'],
        fingerprint=row['fingerprint'],
        agent_type=AgentType(row['agent_type']),
        model_hash=row['model_hash'],
        operator_id=row['operator_id'],
        endocrine_state=endocrine_state,
        alignment=row['alignment'],
        privilege_level=PrivilegeLevel(row['privilege_level']),
        health_status=HealthStatus(row['health_status']),
        created_at=row['created_at'],
        updated_at=row['updated_at']
    )
