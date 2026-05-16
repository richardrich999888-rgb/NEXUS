"""
Production HTTP API Server for AURA Protocol
FastAPI-based with full production features
"""
import asyncio
import time
import json
import uuid
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional
from decimal import Decimal

from fastapi import FastAPI, HTTPException, Depends, status, Request, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field, validator
import uvicorn
import prometheus_client
from prometheus_client import Counter, Histogram, Gauge
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded
import redis.asyncio as redis

from config.production import get_config, ProductionConfig
from models.database import get_session, get_db_manager, init_database
from core.quantum_ria import QuantumResistantRIA, create_quantum_ria, QuantumSignature
from src.network.manager import NetworkManager
from monetization.billing import BillingEngine, create_billing_engine
from src.asi.core import AsiOrchestrator

# Initialize configuration
config = get_config()

# Global NetworkManager instance (initialized in startup)
network_manager: NetworkManager = None  # type: ignore
# Global ASIM Orchestrator instance
asi_orchestrator: AsiOrchestrator = None # type: ignore

# Initialize metrics
VERIFICATION_COUNTER = Counter(
    'aura_verifications_total',
    'Total number of verifications',
    ['customer_id', 'network_id']
)

VERIFICATION_DURATION = Histogram(
    'aura_verification_duration_seconds',
    'Verification duration in seconds',
    ['customer_id']
)

ACTIVE_REQUESTS = Gauge(
    'aura_active_requests',
    'Number of active requests'
)

REVENUE_COUNTER = Counter(
    'aura_revenue_total',
    'Total revenue generated',
    ['currency']
)

# Rate limiter
limiter = Limiter(key_func=get_remote_address)
app = FastAPI(
    title="AURA Protocol API",
    description="Quantum-resistant, infrastructure-less verification protocol",
    version="1.0.0",
    docs_url="/docs" if config.ENVIRONMENT != "production" else None,
    redoc_url="/redoc" if config.ENVIRONMENT != "production" else None
)

# Add rate limiting
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# Add middlewares
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"] if config.ENVIRONMENT != "production" else [
        "https://aura-protocol.com",
        "https://dashboard.aura-protocol.com"
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.add_middleware(
    TrustedHostMiddleware,
    allowed_hosts=["*"] if config.ENVIRONMENT != "production" else [
        "aura-protocol.com",
        "api.aura-protocol.com"
    ]
)

app.add_middleware(GZipMiddleware, minimum_size=1000)

# Security
security = HTTPBearer(auto_error=False)

# Pydantic models
class TransactionRequest(BaseModel):
    sender: str = Field(..., min_length=1, max_length=128)
    receiver: str = Field(..., min_length=1, max_length=128)
    amount: Decimal = Field(..., gt=0)
    network_id: str = Field(default="mainnet", max_length=32)
    metadata: Optional[Dict[str, Any]] = None
    
    @validator('sender', 'receiver')
    def validate_address(cls, v):
        # Allow any format for MVP/Demo
        return v

class VerificationResponse(BaseModel):
    valid: bool
    proof: str
    new_E: int
    verifier_id: str
    verification_time_ms: float
    billing: Dict[str, Any]

class BatchVerificationRequest(BaseModel):
    transactions: List[TransactionRequest]
    customer_id: Optional[str] = None
    async_mode: bool = False

class BatchVerificationResponse(BaseModel):
    results: List[VerificationResponse]
    summary: Dict[str, Any]
    batch_id: str

class CustomerCreateRequest(BaseModel):
    email: str = Field(..., regex=r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$')
    company: Optional[str] = None
    plan: str = Field(default="free", regex="^(free|pro|enterprise)$")

class CustomerResponse(BaseModel):
    customer_id: str
    email: str
    company: Optional[str]
    plan: str
    api_key: str
    monthly_limit: int
    created_at: int

class InvoiceRequest(BaseModel):
    verifications: int = Field(..., gt=0, le=1000000000)
    description: Optional[str] = None

class InvoiceResponse(BaseModel):
    invoice_id: str
    customer_id: str
    amount_usd: Decimal
    verifications: int
    status: str
    payment_url: str
    created_at: int

class WebhookRequest(BaseModel):
    url: str
    events: List[str]

# ASIM Models
class AsiDecisionRequest(BaseModel):
    intent: str
    logic_chain: str
    agent_opinions: List[str]
    provenance_signal_hex: str

class AsiDecisionResponse(BaseModel):
    status: str
    safe: bool
    current_invariant: Optional[int]
    diagnostics: Dict[str, Any]

class AsiStatusResponse(BaseModel):
    stats: Dict[str, Any]
    thermodynamic: Dict[str, Any]
    potential_wells: Dict[str, Any]
    field_alignment: Dict[str, Any]

# Dependency injection
async def get_ria() -> QuantumResistantRIA:
    """Get RIA instance"""
    ria = create_quantum_ria(config)
    return ria

async def get_billing() -> BillingEngine:
    """Get billing engine"""
    return create_billing_engine(config)

async def get_redis() -> redis.Redis:
    """Get Redis client"""
    db_manager = get_db_manager()
    return await db_manager.get_aioredis()

async def verify_api_key(
    credentials: Optional[HTTPAuthorizationCredentials] = Depends(security)
) -> Optional[str]:
    """Verify API key and return customer ID"""
    if not credentials:
        return None
    
    api_key = credentials.credentials
    
    # Check Redis cache first
    try:
        redis_client = await get_redis()
        cache_key = f"api_key:{api_key}"
        customer_id = await redis_client.get(cache_key)
        
        if customer_id:
            return customer_id
    except:
        pass # Fallback to DB if Redis fails
    
    # Check database
    from models.database import Customer
    try:
        # Use a new session
        db_manager = get_db_manager()
        with db_manager.get_session() as session:
            customer = session.query(Customer).filter(
                Customer.api_key == api_key,
                Customer.is_active == True
            ).first()
            
            if customer:
                # Cache for 1 hour
                try:
                    await redis_client.setex(cache_key, 3600, str(customer.id))
                except:
                    pass
                return str(customer.id)
    except Exception as e:
        print(f"Auth error: {e}")
    
    raise HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Invalid API key"
    )

# Rate limiting per customer
def customer_rate_limit(request: Request):
    """Rate limit based on customer"""
    # This is a simplification. Real implementation would inspect
    # request state set by a middleware or dependency
    return "100/second"

# API Endpoints
@app.get("/health")
async def health_check():
    """Health check endpoint"""
    db_manager = get_db_manager()
    db_health = db_manager.health_check()
    
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat(),
        "version": config.VERSION,
        "environment": config.ENVIRONMENT,
        "database": db_health
    }

@app.get("/metrics")
async def metrics():
    """Prometheus metrics endpoint"""
    return prometheus_client.generate_latest()

@app.post("/v1/verify", response_model=VerificationResponse)
@limiter.limit(customer_rate_limit)
async def verify_transaction(
    request: Request,
    transaction: TransactionRequest,
    background_tasks: BackgroundTasks,
    customer_id: Optional[str] = Depends(verify_api_key),
    manager: NetworkManager = Depends(lambda: network_manager)
):
    """
    Verify a single transaction
    """
    ACTIVE_REQUESTS.inc()
    start_time = time.time()
    
    try:
        # Get RIA instance
        ria = await get_ria()
        # Optionally use offline verification via NetworkManager (currently just a placeholder)
        # offline_valid = manager.verify_offline(signature, transaction.sender)
        # if not offline_valid:
        #     raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Offline verification failed")
        
        # Create signature
        signature = ria.create_transaction_signature(
            sender_id=transaction.sender,
            receiver_id=transaction.receiver,
            amount=transaction.amount
        )
        
        # Verify transaction
        is_valid, new_E = ria.verify_transaction(signature, transaction.sender)
        
        if not is_valid:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Invalid transaction signature"
            )
        
        # Get billing engine
        billing = await get_billing()
        
        # Calculate cost
        cost = await billing.calculate_cost(
            customer_id=customer_id,
            verifications=1,
            is_successful=is_valid
        )
        
        # Record verification
        verification_id = str(uuid.uuid4())
        
        # Store in database (async)
        background_tasks.add_task(
            _record_verification,
            verification_id=verification_id,
            transaction=transaction,
            signature=signature,
            customer_id=customer_id,
            is_valid=is_valid,
            cost=cost,
            new_E=new_E
        )
        
        # Update metrics
        VERIFICATION_COUNTER.labels(
            customer_id=customer_id or "anonymous",
            network_id=transaction.network_id
        ).inc()
        
        if cost > 0:
            REVENUE_COUNTER.labels(currency="USD").inc(float(cost))
        
        verification_time = (time.time() - start_time) * 1000
        VERIFICATION_DURATION.labels(
            customer_id=customer_id or "anonymous"
        ).observe(verification_time / 1000)
        
        return VerificationResponse(
            valid=is_valid,
            proof=signature.proof.hex() if signature.proof else "",
            new_E=int(new_E),
            verifier_id=config.NODE_ID,
            verification_time_ms=verification_time,
            billing={
                "cost_usd": float(cost),
                "verifications_used": 1,
                "customer_id": customer_id,
                "remaining_free": await billing.get_remaining_free(customer_id)
            }
        )
        
    except Exception as e:
        import traceback
        traceback.print_exc()
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Verification failed: {str(e)}"
        )
    finally:
        ACTIVE_REQUESTS.dec()

@app.post("/v1/verify/batch", response_model=BatchVerificationResponse)
@limiter.limit(customer_rate_limit)
async def batch_verify(
    request: Request,
    batch_request: BatchVerificationRequest,
    customer_id: Optional[str] = Depends(verify_api_key)
):
    """
    Batch verify multiple transactions
    """
    ACTIVE_REQUESTS.inc()
    start_time = time.time()
    
    try:
        ria = await get_ria()
        billing = await get_billing()
        
        results = []
        batch_id = str(uuid.uuid4())
        
        # Process batch
        for i, tx in enumerate(batch_request.transactions):
            try:
                # Create and verify signature
                signature = ria.create_transaction_signature(
                    sender_id=tx.sender,
                    receiver_id=tx.receiver,
                    amount=tx.amount
                )
                
                is_valid, new_E = ria.verify_transaction(signature, tx.sender)
                
                # Calculate cost
                cost = await billing.calculate_cost(
                    customer_id=customer_id,
                    verifications=1,
                    is_successful=is_valid,
                    is_batch=True,
                    batch_size=len(batch_request.transactions)
                )
                
                results.append(
                    VerificationResponse(
                        valid=is_valid,
                        proof=signature.proof.hex() if signature.proof else "",
                        new_E=int(new_E),
                        verifier_id=config.NODE_ID,
                        verification_time_ms=0,  # Will calculate average if needed
                        billing={
                            "cost_usd": float(cost),
                            "verifications_used": 1,
                            "customer_id": customer_id
                        }
                    )
                )
                
            except Exception as e:
                results.append(
                    VerificationResponse(
                        valid=False,
                        proof="",
                        new_E=0,
                        verifier_id=config.NODE_ID,
                        verification_time_ms=0,
                        billing={
                            "cost_usd": 0.0,
                            "verifications_used": 1,
                            "customer_id": customer_id,
                            "error": str(e)
                        }
                    )
                )
        
        # Calculate summary
        total = len(results)
        successful = sum(1 for r in results if r.valid)
        total_cost = sum(r.billing["cost_usd"] for r in results)
        
        total_time = (time.time() - start_time) * 1000
        
        # Update metrics
        VERIFICATION_COUNTER.labels(
            customer_id=customer_id or "anonymous",
            network_id="batch"
        ).inc(total)
        
        if total_cost > 0:
            REVENUE_COUNTER.labels(currency="USD").inc(total_cost)
        
        return BatchVerificationResponse(
            results=results,
            summary={
                "total": total,
                "successful": successful,
                "failed": total - successful,
                "total_cost_usd": total_cost,
                "avg_cost_per_tx": total_cost / total if total > 0 else 0,
                "total_time_ms": total_time,
                "avg_time_per_tx": total_time / total if total > 0 else 0
            },
            batch_id=batch_id
        )
        
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Batch verification failed: {str(e)}"
        )
    finally:
        ACTIVE_REQUESTS.dec()

@app.post("/v1/customers", response_model=CustomerResponse)
@limiter.limit("5/minute")
async def create_customer(
    request: Request,
    customer_data: CustomerCreateRequest
):
    """Create new customer account"""
    try:
        billing = await get_billing()
        
        customer = await billing.create_customer(
            email=customer_data.email,
            company=customer_data.company,
            plan=customer_data.plan
        )
        
        return CustomerResponse(
            customer_id=customer["customer_id"],
            email=customer["email"],
            company=customer.get("company"),
            plan=customer["plan"],
            api_key=customer["api_key"],
            monthly_limit=customer["monthly_limit"],
            created_at=customer["created_at"]
        )
        
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Customer creation failed: {str(e)}"
        )

@app.get("/v1/peers", response_model=List[Dict[str, str]])
async def list_peers(manager: NetworkManager = Depends(lambda: network_manager)):
    """List all registered peers."""
    if not manager:
        return []
    return manager.list_peers()

@app.post("/v1/peers", status_code=201)
async def add_peer(peer_id: str, base_url: str, manager: NetworkManager = Depends(lambda: network_manager)):
    """Add a new peer to the network manager."""
    if not manager:
        raise HTTPException(status_code=503, detail="Network manager not initialized")
    manager.add_peer(peer_id, base_url)
    return {"status": "peer added", "peer_id": peer_id}

@app.get("/v1/usage")
async def get_usage(customer_id: str = Depends(verify_api_key)):
    """Get customer usage statistics."""
    billing = await get_billing()
    return await billing.get_customer_usage(customer_id)

@app.post("/v1/webhooks")
async def register_webhook_endpoint(
    webhook: WebhookRequest,
    customer_id: str = Depends(verify_api_key)
):
    """Register a new webhook."""
    billing = await get_billing()
    return await billing.register_webhook(
        customer_id=customer_id,
        url=webhook.url,
        events=webhook.events
    )

# ASIM Endpoints
@app.post("/v1/asi/decision", response_model=AsiDecisionResponse)
@limiter.limit("10/minute")
async def process_asi_decision(
    request: Request,
    decision_request: AsiDecisionRequest,
    customer_id: Optional[str] = Depends(verify_api_key),
    orch: AsiOrchestrator = Depends(lambda: asi_orchestrator)
):
    """Process an ASI decision through the physical safety mesh."""
    if not orch:
        raise HTTPException(status_code=503, detail="ASI Mesh not initialized")
        
    try:
        provenance_bytes = bytes.fromhex(decision_request.provenance_signal_hex)
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid provenance_signal_hex")
        
    result = orch.process_decision(
        intent=decision_request.intent,
        logic_chain=decision_request.logic_chain,
        agent_opinions=decision_request.agent_opinions,
        provenance_signal=provenance_bytes
    )
    
    return AsiDecisionResponse(
        status=result["status"],
        safe=result.get("safe", False),
        current_invariant=result.get("current_invariant"),
        diagnostics=result.get("diagnostics", {})
    )

@app.get("/v1/asi/status", response_model=AsiStatusResponse)
async def get_asi_status(orch: AsiOrchestrator = Depends(lambda: asi_orchestrator)):
    """Get the current health and physics status of the ASI mesh."""
    if not orch:
        raise HTTPException(status_code=503, detail="ASI Mesh not initialized")
    return orch.get_mesh_status()

# Background tasks
async def _record_verification(
    verification_id: str,
    transaction: TransactionRequest,
    signature: QuantumSignature,
    customer_id: Optional[str],
    is_valid: bool,
    cost: Decimal,
    new_E: int
):
    """Record verification in database (async)"""
    try:
        db_manager = get_db_manager()
        
        with db_manager.get_session() as session:
            from models.database import VerificationTransaction, NetworkInvariant
            
            # Store transaction
            tx = VerificationTransaction(
                tx_hash=verification_id,
                sender_id=transaction.sender,
                receiver_id=transaction.receiver,
                amount=transaction.amount,
                psi_signature=int.from_bytes(signature.signature[:8], 'big'),
                timestamp=signature.timestamp,
                network_id=transaction.network_id,
                verified_by=config.NODE_ID,
                confidence=1.0 if is_valid else 0.0,
                proof=signature.proof.hex() if signature.proof else None,
                metadata_json=transaction.metadata
            )
            session.add(tx)
            
            # Update network invariant
            invariant = session.query(NetworkInvariant).filter_by(
                network_id=transaction.network_id
            ).first()
            
            if invariant:
                invariant.E_value = new_E
                invariant.last_updated = int(time.time())
                invariant.total_transactions += 1
                invariant.total_value += transaction.amount
            else:
                invariant = NetworkInvariant(
                    network_id=transaction.network_id,
                    E_value=new_E,
                    last_updated=int(time.time()),
                    confidence=1.0,
                    total_transactions=1,
                    total_value=transaction.amount
                )
                session.add(invariant)
            
            session.commit()
            
    except Exception as e:
        import traceback
        traceback.print_exc()

# Startup event
@app.on_event("startup")
async def startup_event():
    """Initialize application on startup"""
    
    # Initialize database
    db_manager = init_database(config)
    
    # Initialize Redis
    await db_manager.init_async_db()
    
    # Init billing
    billing = await get_billing()
    await billing.init_async()

    # Initialize NetworkManager with RIA instance and empty invariants
    ria_instance = create_quantum_ria(config)
    global network_manager
    network_manager = NetworkManager(ria_instance, initial_invariants={})

    # Initialize ASIM Orchestrator
    global asi_orchestrator
    asi_orchestrator = AsiOrchestrator(ria_instance)


# Main entry point
def run_server():
    """Run the production server"""
    # Validate configuration
    if not config.validate():
        print("Invalid configuration")
        return
    
    # Setup logging
    config.setup_logging()
    
    # Start server
    uvicorn.run(
        "server.api:app",
        host="0.0.0.0",
        port=config.HTTP_PORT,
        workers=config.WORKER_COUNT,
        log_level="info",
        access_log=True
    )

if __name__ == "__main__":
    run_server()
