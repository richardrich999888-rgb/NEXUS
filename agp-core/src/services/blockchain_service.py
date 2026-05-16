"""
Blockchain Service - Phase 4
Web3 integration for AGP-CORE
"""

import asyncio
import json
import hashlib
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime, timedelta
from decimal import Decimal
import logging

from web3 import AsyncWeb3
from eth_account import Account
from eth_account.messages import encode_defunct

# Handle web3.py v6 vs v7 API differences
try:
    from web3.middleware import ExtraDataToPOAMiddleware as poa_middleware
except ImportError:
    try:
        from web3.middleware import geth_poa_middleware as poa_middleware
    except ImportError:
        poa_middleware = None

from src.core.database import db
from src.config import settings

logger = logging.getLogger(__name__)

class BlockchainService:
    """
    Service for blockchain integration:
    - Wallet connection & verification
    - Contract interactions (read/write)
    - Token indexing & syncing
    - Event monitoring
    """
    
    def __init__(self):
        self.w3_instances: Dict[str, AsyncWeb3] = {}
        self.contract_cache: Dict[str, Any] = {}
    
    async def get_web3(self, network_id: str) -> Optional[AsyncWeb3]:
        """Get or initialize AsyncWeb3 for a network"""
        if network_id in self.w3_instances:
            return self.w3_instances[network_id]
        
        async with db.pool.acquire() as conn:
            network = await conn.fetchrow(
                "SELECT * FROM blockchain_networks WHERE network_id = $1 AND is_active = TRUE",
                network_id
            )
            
            if not network:
                return None
            
            w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider(network['rpc_endpoint']))
            
            # POA middleware for Polygon/L2s
            if network['network_type'] in ['sidechain', 'layer2'] and poa_middleware:
                w3.middleware_onion.inject(poa_middleware, layer=0)
            
            try:
                if await w3.is_connected():
                    self.w3_instances[network_id] = w3
                    return w3
            except Exception as e:
                logger.error(f"Failed to connect to network {network['network_name']}: {e}")
            
            return None

    async def verify_signature(self, message: str, signature: str, address: str) -> bool:
        """Verify an Ethereum signature"""
        try:
            encoded_message = encode_defunct(text=message)
            recovered_addr = Account.recover_message(encoded_message, signature=signature)
            return recovered_addr.lower() == address.lower()
        except Exception:
            return False

    async def call_contract(self, 
                            contract_id: str, 
                            function_name: str, 
                            *args, 
                            network_id: Optional[str] = None) -> Any:
        """Call a read-only contract function"""
        async with db.pool.acquire() as conn:
            contract = await conn.fetchrow(
                "SELECT * FROM smart_contracts WHERE contract_id = $1", contract_id
            )
            if not contract:
                raise ValueError("Contract not found")
            
            net_id = network_id or str(contract['network_id'])
            w3 = await self.get_web3(net_id)
            if not w3:
                raise ValueError("Network not available")
            
            # Use cached contract instance if possible
            cache_key = f"{contract['contract_address']}_{net_id}"
            if cache_key in self.contract_cache:
                contract_inst = self.contract_cache[cache_key]
            else:
                contract_inst = w3.eth.contract(
                    address=w3.to_checksum_address(contract['contract_address']),
                    abi=contract['abi']
                )
                self.contract_cache[cache_key] = contract_inst
            
            func = getattr(contract_inst.functions, function_name)
            return await func(*args).call()

    async def execute_transaction(self,
                                  contract_id: str,
                                  function_name: str,
                                  private_key: str,
                                  *args,
                                  value: int = 0) -> str:
        """Execute a write transaction on-chain"""
        async with db.pool.acquire() as conn:
            contract = await conn.fetchrow(
                "SELECT * FROM smart_contracts WHERE contract_id = $1", contract_id
            )
            if not contract:
                raise ValueError("Contract not found")
            
            net_id = str(contract['network_id'])
            w3 = await self.get_web3(net_id)
            if not w3:
                raise ValueError("Network not available")
            
            account = Account.from_key(private_key)
            
            contract_inst = w3.eth.contract(
                address=w3.to_checksum_address(contract['contract_address']),
                abi=contract['abi']
            )
            
            func = getattr(contract_inst.functions, function_name)
            
            nonce = await w3.eth.get_transaction_count(account.address)
            gas_estimate = await func(*args).estimate_gas({'from': account.address, 'value': value})
            
            tx = await func(*args).build_transaction({
                'from': account.address,
                'nonce': nonce,
                'gas': int(gas_estimate * 1.2),
                'gasPrice': await w3.eth.gas_price,
                'value': value
            })
            
            signed_tx = account.sign_transaction(tx)
            tx_hash = await w3.eth.send_raw_transaction(signed_tx.raw_transaction)
            
            # Record transaction in DB
            await conn.execute(
                """INSERT INTO blockchain_transactions 
                (network_id, contract_id, tx_hash, from_address, to_address, status)
                VALUES ($1, $2, $3, $4, $5, 'pending')""",
                contract['network_id'], contract['contract_id'], tx_hash.hex(), 
                account.address, contract['contract_address']
            )
            
            return tx_hash.hex()

    async def connect_wallet(self, agent_id: str, address: str, chain_id: int, signature: str, message: str) -> bool:
        """Connect and verify a wallet to an agent"""
        if await self.verify_signature(message, signature, address):
            async with db.pool.acquire() as conn:
                await conn.execute(
                    """INSERT INTO wallet_connections 
                    (agent_id, wallet_address, chain_id, connection_method, signature_data, signed_message, is_verified)
                    VALUES ($1, $2, $3, 'signature', $4, $5, TRUE)
                    ON CONFLICT (agent_id, wallet_address, chain_id) DO UPDATE SET is_active = TRUE, verified_at = NOW()""",
                    agent_id, address.lower(), chain_id, json.dumps({'signature': signature}), message
                )
            return True
        return False

# Initialize singleton
blockchain_service = BlockchainService()
