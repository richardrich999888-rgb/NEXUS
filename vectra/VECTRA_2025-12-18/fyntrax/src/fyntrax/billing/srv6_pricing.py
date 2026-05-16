"""
FYNTRAX SRv6 Pricing

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

SRv6-based dynamic pricing for energy-aware billing.
Integrates energy costs into network pricing.
"""

from dataclasses import dataclass
from typing import Dict, List, Optional
import time


@dataclass
class PricingTier:
    """Pricing tier definition."""
    name: str
    base_price: float  # Per MB
    min_latency_ms: float
    max_latency_ms: float
    qos_class: int


class SRv6PricingEngine:
    """
    SRv6-based dynamic pricing engine.
    
    Prices network resources based on:
    - Current load
    - QoS requirements
    - Time of day
    - Energy cost
    """
    
    DEFAULT_TIERS = [
        PricingTier("economy", 0.001, 100, 500, 1),
        PricingTier("standard", 0.005, 20, 100, 2),
        PricingTier("premium", 0.02, 5, 20, 3),
        PricingTier("realtime", 0.1, 1, 5, 4),
    ]

    def __init__(
        self,
        tiers: Optional[List[PricingTier]] = None,
        energy_cost_per_kwh: float = 0.10,
    ):
        """
        Initialize pricing engine.
        
        Args:
            tiers: Pricing tiers (uses defaults if None)
            energy_cost_per_kwh: Energy cost in currency per kWh
        """
        self.tiers = tiers or self.DEFAULT_TIERS
        self.energy_cost_per_kwh = energy_cost_per_kwh
        self.billing_records: List[Dict] = []

    def get_price(
        self,
        data_mb: float,
        tier_name: str,
        load_factor: float = 1.0,
    ) -> float:
        """
        Calculate price for data transfer.
        
        Args:
            data_mb: Data amount in MB
            tier_name: Service tier
            load_factor: Current network load (1.0 = normal)
            
        Returns:
            Price in currency units
        """
        tier = next((t for t in self.tiers if t.name == tier_name), None)
        if tier is None:
            tier = self.tiers[0]  # Default to economy
        
        # Dynamic pricing based on load
        price = tier.base_price * data_mb * load_factor
        
        return price

    def record_usage(
        self,
        subscriber_id: str,
        data_mb: float,
        tier_name: str,
        price: float,
    ) -> None:
        """Record billing event."""
        self.billing_records.append({
            "subscriber_id": subscriber_id,
            "data_mb": data_mb,
            "tier": tier_name,
            "price": price,
            "timestamp": time.time(),
        })

    def energy_cost_share(
        self,
        energy_kwh: float,
        subscribers: int,
    ) -> float:
        """
        Calculate per-subscriber energy cost share.
        
        Args:
            energy_kwh: Total energy consumed
            subscribers: Number of active subscribers
            
        Returns:
            Energy cost per subscriber
        """
        if subscribers <= 0:
            return 0.0
        
        total_cost = energy_kwh * self.energy_cost_per_kwh
        return total_cost / subscribers

    def total_revenue(self) -> float:
        """Calculate total revenue from billing records."""
        return sum(r["price"] for r in self.billing_records)

    def statistics(self) -> dict:
        """Get pricing engine statistics."""
        total_data = sum(r["data_mb"] for r in self.billing_records)
        return {
            "total_records": len(self.billing_records),
            "total_data_mb": total_data,
            "total_revenue": self.total_revenue(),
            "avg_price_per_mb": self.total_revenue() / total_data if total_data > 0 else 0,
        }
