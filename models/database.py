"""
Production Database Models for AURA Protocol
"""
import uuid
import time
from datetime import datetime
from decimal import Decimal
from typing import Optional, List, Dict, Any
from sqlalchemy import create_engine, Column, Integer, String, BigInteger, \
    Text, Float, Boolean, DateTime, ForeignKey, Numeric, Index, CheckConstraint
from sqlalchemy.orm import declarative_base
from sqlalchemy.orm import sessionmaker, relationship
from sqlalchemy.dialects.postgresql import JSONB, UUID
import redis
import redis.asyncio as aioredis
from contextlib import contextmanager
import logging

Base = declarative_base()

class VerificationTransaction(Base):
    """Verified transaction storage"""
    __tablename__ = 'verification_transactions'
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tx_hash = Column(String(64), unique=True, nullable=False, index=True)
    sender_id = Column(String(64), nullable=False, index=True)
    receiver_id = Column(String(64), nullable=False, index=True)
    amount = Column(Numeric(36, 18), nullable=False)  # Supports up to 1B with 18 decimals
    psi_signature = Column(Numeric(78, 0), nullable=False)  # 256-bit integer
    timestamp = Column(BigInteger, nullable=False, index=True)
    network_id = Column(String(32), nullable=False, index=True)
    verified_by = Column(String(64), nullable=False)
    confidence = Column(Float, default=1.0)
    proof = Column(Text)
    metadata_json = Column(JSONB, name='metadata')
    
    # Timestamps
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_tx_sender_network', 'sender_id', 'network_id'),
        Index('idx_tx_receiver_network', 'receiver_id', 'network_id'),
        Index('idx_tx_timestamp_network', 'timestamp', 'network_id'),
        CheckConstraint('confidence >= 0 AND confidence <= 1', name='check_confidence_range'),
        CheckConstraint('amount >= 0', name='check_amount_positive'),
    )

class NetworkInvariant(Base):
    """Network invariant storage"""
    __tablename__ = 'network_invariants'
    
    id = Column(Integer, primary_key=True)
    network_id = Column(String(32), unique=True, nullable=False, index=True)
    E_value = Column(Numeric(78, 0), nullable=False)  # Current invariant value
    last_updated = Column(BigInteger, nullable=False)
    confidence = Column(Float, default=1.0)
    proof_hash = Column(String(64))
    verifier_count = Column(Integer, default=1)
    
    # Statistics
    total_transactions = Column(BigInteger, default=0)
    total_value = Column(Numeric(36, 18), default=0)
    avg_confidence = Column(Float, default=1.0)
    
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

class Customer(Base):
    """Customer/User management"""
    __tablename__ = 'customers'
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    email = Column(String(255), unique=True, nullable=False, index=True)
    company = Column(String(255))
    api_key = Column(String(128), unique=True, nullable=False, index=True)
    plan = Column(String(32), default='free')  # free, pro, enterprise
    monthly_limit = Column(BigInteger, default=10000000)
    
    # Billing
    stripe_customer_id = Column(String(255))
    current_balance = Column(Numeric(12, 2), default=0)
    total_spent = Column(Numeric(12, 2), default=0)
    billing_address = Column(JSONB)
    
    # Status
    is_active = Column(Boolean, default=True)
    is_verified = Column(Boolean, default=False)
    
    # Usage tracking
    verifications_this_month = Column(BigInteger, default=0)
    last_verification = Column(BigInteger)
    
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    
    # Relationships
    invoices = relationship("Invoice", back_populates="customer")
    webhooks = relationship("Webhook", back_populates="customer")
    
    __table_args__ = (
        Index('idx_customer_plan_active', 'plan', 'is_active'),
        CheckConstraint('monthly_limit >= 0', name='check_monthly_limit'),
    )

class Invoice(Base):
    """Billing invoices"""
    __tablename__ = 'invoices'
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    customer_id = Column(UUID(as_uuid=True), ForeignKey('customers.id'), nullable=False)
    invoice_number = Column(String(32), unique=True, nullable=False)
    amount = Column(Numeric(12, 2), nullable=False)
    currency = Column(String(3), default='USD')
    verifications = Column(BigInteger, nullable=False)
    
    # Status
    status = Column(String(20), default='pending')  # pending, paid, failed, refunded
    paid_at = Column(DateTime)
    payment_method = Column(String(32))
    transaction_hash = Column(String(255))
    
    # Line items
    line_items = Column(JSONB)
    
    # Dates
    invoice_date = Column(DateTime, default=datetime.utcnow)
    due_date = Column(DateTime)
    
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    
    # Relationships
    customer = relationship("Customer", back_populates="invoices")
    
    __table_args__ = (
        Index('idx_invoice_customer_status', 'customer_id', 'status'),
        Index('idx_invoice_due_date', 'due_date'),
        CheckConstraint('amount >= 0', name='check_invoice_amount'),
    )

class Webhook(Base):
    """Webhook configuration"""
    __tablename__ = 'webhooks'
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    customer_id = Column(UUID(as_uuid=True), ForeignKey('customers.id'), nullable=False)
    url = Column(Text, nullable=False)
    secret = Column(String(64), nullable=False)
    events = Column(JSONB)  # List of events to trigger
    
    # Status
    is_active = Column(Boolean, default=True)
    last_triggered = Column(DateTime)
    success_count = Column(Integer, default=0)
    failure_count = Column(Integer, default=0)
    
    # Configuration
    retry_policy = Column(JSONB)
    timeout = Column(Integer, default=5)
    
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    
    # Relationships
    customer = relationship("Customer", back_populates="webhooks")

class PeerNode(Base):
    """Peer node information"""
    __tablename__ = 'peer_nodes'
    
    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    node_id = Column(String(64), unique=True, nullable=False, index=True)
    address = Column(String(255))
    port = Column(Integer)
    capabilities = Column(JSONB)  # List of capabilities
    
    # Connection info
    protocol_version = Column(String(16))
    public_key = Column(Text)
    is_ssl = Column(Boolean, default=False)
    
    # Status
    last_seen = Column(BigInteger)
    is_online = Column(Boolean, default=False)
    trust_score = Column(Float, default=0.5)
    sync_count = Column(Integer, default=0)
    
    # Statistics
    total_transactions = Column(BigInteger, default=0)
    total_value = Column(Numeric(36, 18), default=0)
    
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    
    __table_args__ = (
        Index('idx_peer_online_trust', 'is_online', 'trust_score'),
        CheckConstraint('trust_score >= 0 AND trust_score <= 1', name='check_trust_score'),
    )

class PerformanceMetrics(Base):
    """Performance metrics collection"""
    __tablename__ = 'performance_metrics'
    
    id = Column(BigInteger, primary_key=True, autoincrement=True)
    timestamp = Column(BigInteger, nullable=False, index=True)
    metric_type = Column(String(32), nullable=False, index=True)
    metric_value = Column(Float, nullable=False)
    labels = Column(JSONB)  # Additional labels
    
    __table_args__ = (
        Index('idx_metrics_timestamp_type', 'timestamp', 'metric_type'),
    )

# Database connection and session management
class DatabaseManager:
    """Production database manager"""
    
    def __init__(self, database_url: str, redis_url: str = None):
        self.database_url = database_url
        self.redis_url = redis_url
        self.engine = None
        self.SessionLocal = None
        self.redis_client = None
        self.aioredis_pool = None
    
    def init_db(self):
        """Initialize database connection"""
        # Create SQLAlchemy engine
        self.engine = create_engine(
            self.database_url,
            pool_size=20,
            max_overflow=10,
            pool_pre_ping=True,
            pool_recycle=3600,
            echo=False
        )
        
        # Create session factory
        self.SessionLocal = sessionmaker(
            autocommit=False,
            autoflush=False,
            bind=self.engine
        )
        
        # Create tables
        Base.metadata.create_all(bind=self.engine)
        
        # Initialize Redis if URL provided
        if self.redis_url:
            self.redis_client = redis.Redis.from_url(
                self.redis_url,
                decode_responses=True
            )
    
    async def init_async_db(self):
        """Initialize async database connections"""
        # For async Redis
        if self.redis_url:
            self.aioredis_pool = await aioredis.from_url(
                self.redis_url,
                decode_responses=True
            )
    
    @contextmanager
    def get_session(self):
        """Get database session with context manager"""
        session = self.SessionLocal()
        try:
            yield session
            session.commit()
        except Exception:
            session.rollback()
            raise
        finally:
            session.close()
    
    def get_redis(self) -> redis.Redis:
        """Get Redis client"""
        if not self.redis_client:
            raise RuntimeError("Redis not initialized")
        return self.redis_client
    
    async def get_aioredis(self):
        """Get async Redis client"""
        if not self.aioredis_pool:
            raise RuntimeError("Async Redis not initialized")
        return self.aioredis_pool
    
    def health_check(self) -> Dict[str, Any]:
        """Check database health"""
        try:
            # Check PostgreSQL
            with self.get_session() as session:
                session.execute("SELECT 1")
            
            # Check Redis if available
            redis_ok = False
            if self.redis_client:
                redis_ok = self.redis_client.ping()
            
            return {
                "postgresql": "healthy",
                "redis": "healthy" if redis_ok else "not_configured",
                "timestamp": time.time()
            }
        except Exception as e:
            return {
                "postgresql": f"error: {str(e)}",
                "redis": "unknown",
                "timestamp": time.time()
            }
    
    def close(self):
        """Close database connections"""
        if self.engine:
            self.engine.dispose()
        
        if self.redis_client:
            self.redis_client.close()
        
        if self.aioredis_pool:
            import asyncio
            asyncio.create_task(self.aioredis_pool.close())

# Singleton database manager
_db_manager: Optional[DatabaseManager] = None

def init_database(config) -> DatabaseManager:
    """Initialize database manager"""
    global _db_manager
    
    if _db_manager is None:
        _db_manager = DatabaseManager(
            database_url=config.DATABASE_URL,
            redis_url=config.REDIS_URL
        )
        _db_manager.init_db()
    
    return _db_manager

def get_db_manager() -> DatabaseManager:
    """Get database manager instance"""
    if _db_manager is None:
        raise RuntimeError("Database not initialized")
    return _db_manager

def get_session():
    """Get database session for FastAPI dependency injection"""
    manager = get_db_manager()
    with manager.get_session() as session:
        yield session
