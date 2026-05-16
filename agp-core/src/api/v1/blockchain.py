"""
Blockchain API Router - Phase 4
"""

import uuid
from typing import List, Optional, Dict, Any
from fastapi import APIRouter, Depends, HTTPException, Header, Body
from pydantic import BaseModel

from src.services import blockchain_service
from src.core.database import db

router = APIRouter(prefix="/blockchain", tags=["blockchain"])

# =============================================================================
# MODELS
# =============================================================================

class WalletConnectRequest(BaseModel):
    agent_id: uuid.UUID
    address: str
    chain_id: int
    signature: str
    message: str

class ContractCallRequest(BaseModel):
    contract_id: uuid.UUID
    function_name: str
    args: List[Any] = []
    network_id: Optional[uuid.UUID] = None

class ContractExecuteRequest(BaseModel):
    contract_id: uuid.UUID
    function_name: str
    args: List[Any] = []
    value: int = 0
    # In production, private keys MUST NOT be sent over API
    # This is for MVP demonstration/testing only
    private_key: str 

# =============================================================================
# ENDPOINTS
# =============================================================================

@router.post("/connect-wallet")
async def connect_wallet(request: WalletConnectRequest):
    """Verify and connect a wallet to an agent"""
    success = await blockchain_service.connect_wallet(
        str(request.agent_id),
        request.address,
        request.chain_id,
        request.signature,
        request.message
    )
    if not success:
        raise HTTPException(status_code=400, detail="Signature verification failed")
    
    return {"status": "success", "address": request.address}

@router.get("/networks")
async def list_networks():
    """List supported blockchain networks"""
    async with db.pool.acquire() as conn:
        networks = await conn.fetch("SELECT * FROM blockchain_networks WHERE is_active = TRUE")
        return [dict(n) for n in networks]

@router.get("/contracts/{protocol_id}")
async def get_protocol_contracts(protocol_id: uuid.UUID):
    """Get smart contracts deployed for a protocol"""
    async with db.pool.acquire() as conn:
        contracts = await conn.fetch(
            "SELECT * FROM smart_contracts WHERE protocol_id = $1", protocol_id
        )
        return [dict(c) for c in contracts]

@router.post("/call")
async def call_contract(request: ContractCallRequest):
    """Call a read-only function on a smart contract"""
    try:
        result = await blockchain_service.call_contract(
            str(request.contract_id),
            request.function_name,
            *request.args,
            network_id=str(request.network_id) if request.network_id else None
        )
        return {"result": result}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/execute")
async def execute_contract(request: ContractExecuteRequest):
    """Execute a write transaction on a smart contract"""
    try:
        tx_hash = await blockchain_service.execute_transaction(
            str(request.contract_id),
            request.function_name,
            request.private_key,
            *request.args,
            value=request.value
        )
        return {"tx_hash": tx_hash}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.get("/agent/{agent_id}/summary")
async def get_agent_blockchain_summary(agent_id: uuid.UUID):
    """Get comprehensive blockchain summary for an agent"""
    # Simplified summary for now
    async with db.pool.acquire() as conn:
        wallets = await conn.fetch(
            "SELECT * FROM wallet_connections WHERE agent_id = $1 AND is_active = TRUE",
            agent_id
        )
        balances = await conn.fetch(
            "SELECT * FROM token_balances WHERE agent_id = $1",
            agent_id
        )
        transactions = await conn.fetch(
            """SELECT bt.* FROM blockchain_transactions bt 
            JOIN wallet_connections wc ON bt.from_address = wc.wallet_address 
            WHERE wc.agent_id = $1 ORDER BY bt.created_at DESC LIMIT 10""",
            agent_id
        )
        
        return {
            "wallets": [dict(w) for w in wallets],
            "balances": [dict(b) for b in balances],
            "recent_transactions": [dict(t) for t in transactions]
        }
