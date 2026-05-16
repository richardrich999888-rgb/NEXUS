"""
Cross-Chain Bridge Service - Phase 4 Week 16
"""

import uuid
import hashlib
from typing import Dict, List, Optional, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum


class BridgeStatus(str, Enum):
    INITIATED = "initiated"
    PENDING_CONFIRMATION = "pending_confirmation"
    RELAYING = "relaying"
    COMPLETED = "completed"
    FAILED = "failed"
    REFUNDED = "refunded"


class ChainType(str, Enum):
    ETHEREUM = "ethereum"
    POLYGON = "polygon"
    ARBITRUM = "arbitrum"
    OPTIMISM = "optimism"
    BASE = "base"


@dataclass
class ChainConfig:
    """Configuration for a supported chain"""
    chain_id: int
    chain_type: ChainType
    name: str
    bridge_address: str
    rpc_endpoint: str
    confirmation_blocks: int = 12
    avg_block_time_seconds: int = 12


@dataclass
class BridgeTransaction:
    """Cross-chain bridge transaction"""
    id: uuid.UUID
    source_chain: ChainType
    target_chain: ChainType
    sender: str
    recipient: str
    amount: Decimal
    fee: Decimal
    status: BridgeStatus
    source_tx_hash: Optional[str] = None
    target_tx_hash: Optional[str] = None
    message_hash: str = ""
    initiated_at: datetime = field(default_factory=datetime.utcnow)
    completed_at: Optional[datetime] = None
    estimated_completion: Optional[datetime] = None


@dataclass
class BridgeRoute:
    """Available bridge route between chains"""
    source_chain: ChainType
    target_chain: ChainType
    min_amount: Decimal
    max_amount: Decimal
    fee_percent: Decimal
    estimated_time_minutes: int
    is_active: bool = True


class CrossChainBridgeService:
    """
    Manages cross-chain reputation token transfers
    
    Features:
    - Multi-chain support (EVM compatible)
    - Relayer-based message passing
    - Fee calculation and limits
    - Transaction tracking
    """
    
    def __init__(self):
        self.chains: Dict[ChainType, ChainConfig] = {}
        self.routes: Dict[Tuple[ChainType, ChainType], BridgeRoute] = {}
        self.transactions: Dict[uuid.UUID, BridgeTransaction] = {}
        self.relayers: List[str] = []
        
        # Initialize default chains
        self._init_default_chains()
        self._init_default_routes()
    
    def _init_default_chains(self):
        """Initialize supported chains"""
        default_chains = [
            ChainConfig(1, ChainType.ETHEREUM, "Ethereum Mainnet", 
                       "0x0000000000000000000000000000000000000001",
                       "https://eth.llamarpc.com", 12, 12),
            ChainConfig(137, ChainType.POLYGON, "Polygon",
                       "0x0000000000000000000000000000000000000002",
                       "https://polygon-rpc.com", 128, 2),
            ChainConfig(42161, ChainType.ARBITRUM, "Arbitrum One",
                       "0x0000000000000000000000000000000000000003",
                       "https://arb1.arbitrum.io/rpc", 12, 1),
            ChainConfig(10, ChainType.OPTIMISM, "Optimism",
                       "0x0000000000000000000000000000000000000004",
                       "https://mainnet.optimism.io", 12, 2),
            ChainConfig(8453, ChainType.BASE, "Base",
                       "0x0000000000000000000000000000000000000005",
                       "https://mainnet.base.org", 12, 2),
        ]
        
        for chain in default_chains:
            self.chains[chain.chain_type] = chain
    
    def _init_default_routes(self):
        """Initialize bridge routes"""
        chain_types = list(ChainType)
        
        for source in chain_types:
            for target in chain_types:
                if source != target:
                    # Base fee varies by chain
                    base_fee = Decimal("0.1")
                    if source == ChainType.ETHEREUM or target == ChainType.ETHEREUM:
                        base_fee = Decimal("0.3")  # Higher for ETH mainnet
                    
                    route = BridgeRoute(
                        source_chain=source,
                        target_chain=target,
                        min_amount=Decimal("10"),
                        max_amount=Decimal("1000000"),
                        fee_percent=base_fee,
                        estimated_time_minutes=self._estimate_bridge_time(source, target)
                    )
                    self.routes[(source, target)] = route
    
    def _estimate_bridge_time(self, source: ChainType, target: ChainType) -> int:
        """Estimate bridge time in minutes"""
        source_config = self.chains.get(source)
        target_config = self.chains.get(target)
        
        if not source_config or not target_config:
            return 30
        
        # Time for source confirmations + relaying + target confirmation
        source_time = source_config.confirmation_blocks * source_config.avg_block_time_seconds
        target_time = target_config.confirmation_blocks * target_config.avg_block_time_seconds
        relay_time = 60  # 1 minute for relay
        
        return (source_time + relay_time + target_time) // 60
    
    def get_available_routes(self, source_chain: Optional[ChainType] = None) -> List[BridgeRoute]:
        """Get available bridge routes"""
        routes = list(self.routes.values())
        
        if source_chain:
            routes = [r for r in routes if r.source_chain == source_chain and r.is_active]
        else:
            routes = [r for r in routes if r.is_active]
        
        return routes
    
    def calculate_fee(
        self,
        source_chain: ChainType,
        target_chain: ChainType,
        amount: Decimal
    ) -> Tuple[Decimal, Decimal]:
        """Calculate bridge fee and amount after fee"""
        route = self.routes.get((source_chain, target_chain))
        if not route:
            raise ValueError("Route not available")
        
        fee = amount * (route.fee_percent / 100)
        amount_after_fee = amount - fee
        
        return (fee, amount_after_fee)
    
    def initiate_bridge(
        self,
        source_chain: ChainType,
        target_chain: ChainType,
        sender: str,
        recipient: str,
        amount: Decimal
    ) -> BridgeTransaction:
        """Initiate a cross-chain bridge transaction"""
        route = self.routes.get((source_chain, target_chain))
        if not route or not route.is_active:
            raise ValueError("Bridge route not available")
        
        if amount < route.min_amount:
            raise ValueError(f"Amount below minimum: {route.min_amount}")
        
        if amount > route.max_amount:
            raise ValueError(f"Amount exceeds maximum: {route.max_amount}")
        
        fee, amount_after_fee = self.calculate_fee(source_chain, target_chain, amount)
        
        # Generate message hash
        tx_id = uuid.uuid4()
        message_hash = hashlib.sha256(
            f"{tx_id}{source_chain}{target_chain}{sender}{recipient}{amount}".encode()
        ).hexdigest()
        
        estimated_completion = datetime.utcnow() + timedelta(minutes=route.estimated_time_minutes)
        
        tx = BridgeTransaction(
            id=tx_id,
            source_chain=source_chain,
            target_chain=target_chain,
            sender=sender,
            recipient=recipient,
            amount=amount,
            fee=fee,
            status=BridgeStatus.INITIATED,
            message_hash=message_hash,
            estimated_completion=estimated_completion
        )
        
        self.transactions[tx.id] = tx
        return tx
    
    def confirm_source(self, tx_id: uuid.UUID, source_tx_hash: str) -> BridgeTransaction:
        """Confirm source chain transaction"""
        tx = self.transactions.get(tx_id)
        if not tx:
            raise ValueError("Transaction not found")
        
        if tx.status != BridgeStatus.INITIATED:
            raise ValueError("Invalid transaction status")
        
        tx.source_tx_hash = source_tx_hash
        tx.status = BridgeStatus.PENDING_CONFIRMATION
        
        return tx
    
    def start_relay(self, tx_id: uuid.UUID, relayer: str) -> BridgeTransaction:
        """Start relaying to target chain"""
        tx = self.transactions.get(tx_id)
        if not tx:
            raise ValueError("Transaction not found")
        
        if tx.status != BridgeStatus.PENDING_CONFIRMATION:
            raise ValueError("Transaction not confirmed")
        
        if relayer not in self.relayers:
            raise ValueError("Not an authorized relayer")
        
        tx.status = BridgeStatus.RELAYING
        return tx
    
    def complete_bridge(
        self,
        tx_id: uuid.UUID,
        target_tx_hash: str
    ) -> BridgeTransaction:
        """Complete bridge transaction on target chain"""
        tx = self.transactions.get(tx_id)
        if not tx:
            raise ValueError("Transaction not found")
        
        if tx.status != BridgeStatus.RELAYING:
            raise ValueError("Transaction not being relayed")
        
        tx.target_tx_hash = target_tx_hash
        tx.status = BridgeStatus.COMPLETED
        tx.completed_at = datetime.utcnow()
        
        return tx
    
    def refund(self, tx_id: uuid.UUID, reason: str) -> BridgeTransaction:
        """Refund a failed bridge transaction"""
        tx = self.transactions.get(tx_id)
        if not tx:
            raise ValueError("Transaction not found")
        
        if tx.status == BridgeStatus.COMPLETED:
            raise ValueError("Cannot refund completed transaction")
        
        tx.status = BridgeStatus.REFUNDED
        return tx
    
    def get_transaction(self, tx_id: uuid.UUID) -> Optional[BridgeTransaction]:
        """Get transaction by ID"""
        return self.transactions.get(tx_id)
    
    def get_transactions_by_sender(self, sender: str) -> List[BridgeTransaction]:
        """Get all transactions for a sender"""
        return [tx for tx in self.transactions.values() if tx.sender.lower() == sender.lower()]
    
    def add_relayer(self, relayer_address: str):
        """Add an authorized relayer"""
        if relayer_address not in self.relayers:
            self.relayers.append(relayer_address)
    
    def get_bridge_stats(self) -> Dict:
        """Get bridge statistics"""
        total_volume = sum(tx.amount for tx in self.transactions.values() if tx.status == BridgeStatus.COMPLETED)
        total_fees = sum(tx.fee for tx in self.transactions.values() if tx.status == BridgeStatus.COMPLETED)
        
        by_route = {}
        for (source, target), route in self.routes.items():
            route_txs = [
                tx for tx in self.transactions.values()
                if tx.source_chain == source and tx.target_chain == target
            ]
            by_route[f"{source.value}->{target.value}"] = {
                "count": len(route_txs),
                "volume": float(sum(tx.amount for tx in route_txs)),
                "active": route.is_active
            }
        
        return {
            "total_transactions": len(self.transactions),
            "completed": len([tx for tx in self.transactions.values() if tx.status == BridgeStatus.COMPLETED]),
            "pending": len([tx for tx in self.transactions.values() if tx.status in [BridgeStatus.INITIATED, BridgeStatus.PENDING_CONFIRMATION, BridgeStatus.RELAYING]]),
            "total_volume": float(total_volume),
            "total_fees": float(total_fees),
            "supported_chains": [c.value for c in self.chains.keys()],
            "by_route": by_route
        }


# Create singleton
bridge_service = CrossChainBridgeService()
