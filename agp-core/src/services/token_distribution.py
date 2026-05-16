"""
Token Distribution & Vesting Service - Phase 4 Week 14
"""

import uuid
import math
from typing import Dict, List, Optional, Tuple
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum

from src.core.database import db


class AllocationCategory(str, Enum):
    TEAM = "team"
    INVESTORS = "investors"
    ECOSYSTEM = "ecosystem"
    COMMUNITY = "community"
    TREASURY = "treasury"
    LIQUIDITY = "liquidity"


@dataclass
class VestingSchedule:
    """Vesting schedule definition"""
    id: uuid.UUID
    beneficiary_id: uuid.UUID
    category: AllocationCategory
    total_amount: Decimal
    released_amount: Decimal
    start_time: datetime
    cliff_duration_days: int
    vesting_duration_days: int
    revocable: bool
    revoked: bool = False


@dataclass
class TokenAllocation:
    """Token allocation for a category"""
    category: AllocationCategory
    total_supply_percent: Decimal
    cliff_months: int
    vesting_months: int
    tge_unlock_percent: Decimal  # Token Generation Event unlock


class TokenDistributionService:
    """
    Manages token distribution and vesting schedules
    
    Standard allocation model:
    - Team: 15% (12mo cliff, 36mo vest)
    - Investors: 20% (6mo cliff, 24mo vest)
    - Ecosystem: 25% (no cliff, 48mo vest)
    - Community: 30% (10% TGE, 36mo vest)
    - Treasury: 10% (no cliff, controlled by DAO)
    """
    
    DEFAULT_ALLOCATIONS = [
        TokenAllocation(AllocationCategory.TEAM, Decimal("15"), 12, 36, Decimal("0")),
        TokenAllocation(AllocationCategory.INVESTORS, Decimal("20"), 6, 24, Decimal("0")),
        TokenAllocation(AllocationCategory.ECOSYSTEM, Decimal("25"), 0, 48, Decimal("5")),
        TokenAllocation(AllocationCategory.COMMUNITY, Decimal("30"), 0, 36, Decimal("10")),
        TokenAllocation(AllocationCategory.TREASURY, Decimal("10"), 0, 0, Decimal("100")),
    ]
    
    def __init__(self):
        self.schedules: Dict[uuid.UUID, VestingSchedule] = {}
        self.allocations = {a.category: a for a in self.DEFAULT_ALLOCATIONS}
    
    def create_vesting_schedule(
        self,
        beneficiary_id: uuid.UUID,
        category: AllocationCategory,
        amount: Decimal,
        start_time: Optional[datetime] = None,
        revocable: bool = True
    ) -> VestingSchedule:
        """Create a new vesting schedule based on category defaults"""
        allocation = self.allocations.get(category)
        if not allocation:
            raise ValueError(f"Unknown category: {category}")
        
        start = start_time or datetime.utcnow()
        
        # Calculate TGE unlock
        tge_amount = amount * (allocation.tge_unlock_percent / 100)
        vesting_amount = amount - tge_amount
        
        schedule = VestingSchedule(
            id=uuid.uuid4(),
            beneficiary_id=beneficiary_id,
            category=category,
            total_amount=vesting_amount,
            released_amount=tge_amount,  # TGE already released
            start_time=start,
            cliff_duration_days=allocation.cliff_months * 30,
            vesting_duration_days=allocation.vesting_months * 30,
            revocable=revocable
        )
        
        self.schedules[schedule.id] = schedule
        return schedule
    
    def calculate_vested_amount(self, schedule_id: uuid.UUID) -> Decimal:
        """Calculate currently vested amount for a schedule"""
        schedule = self.schedules.get(schedule_id)
        if not schedule or schedule.revoked:
            return Decimal("0")
        
        now = datetime.utcnow()
        cliff_end = schedule.start_time + timedelta(days=schedule.cliff_duration_days)
        
        # Before cliff
        if now < cliff_end:
            return Decimal("0")
        
        # Calculate linear vesting
        elapsed = (now - schedule.start_time).days
        total_days = schedule.vesting_duration_days
        
        if elapsed >= total_days:
            return schedule.total_amount
        
        return schedule.total_amount * Decimal(elapsed) / Decimal(total_days)
    
    def calculate_releasable(self, schedule_id: uuid.UUID) -> Decimal:
        """Calculate amount that can be released now"""
        schedule = self.schedules.get(schedule_id)
        if not schedule:
            return Decimal("0")
        
        vested = self.calculate_vested_amount(schedule_id)
        return vested - schedule.released_amount
    
    def release(self, schedule_id: uuid.UUID) -> Decimal:
        """Release vested tokens"""
        releasable = self.calculate_releasable(schedule_id)
        if releasable <= 0:
            return Decimal("0")
        
        schedule = self.schedules[schedule_id]
        schedule.released_amount += releasable
        
        return releasable
    
    def revoke(self, schedule_id: uuid.UUID) -> Tuple[Decimal, Decimal]:
        """Revoke a vesting schedule, return (released, returned)"""
        schedule = self.schedules.get(schedule_id)
        if not schedule or not schedule.revocable:
            raise ValueError("Schedule not revocable")
        
        releasable = self.calculate_releasable(schedule_id)
        schedule.released_amount += releasable
        schedule.revoked = True
        
        returned = schedule.total_amount - schedule.released_amount
        return (releasable, returned)
    
    def get_distribution_summary(self) -> Dict[str, any]:
        """Get summary of token distribution"""
        summary = {}
        for category, allocation in self.allocations.items():
            category_schedules = [
                s for s in self.schedules.values() 
                if s.category == category and not s.revoked
            ]
            
            total_allocated = sum(s.total_amount for s in category_schedules)
            total_released = sum(s.released_amount for s in category_schedules)
            
            summary[category.value] = {
                "allocation_percent": float(allocation.total_supply_percent),
                "total_allocated": float(total_allocated),
                "total_released": float(total_released),
                "schedules_count": len(category_schedules),
                "cliff_months": allocation.cliff_months,
                "vesting_months": allocation.vesting_months
            }
        
        return summary


class EconomicSimulator:
    """
    Simulates token economics over time
    
    Models:
    - Supply schedule (vesting releases)
    - Staking dynamics
    - Burn rate
    - Velocity
    """
    
    def __init__(self, total_supply: Decimal, distribution_service: TokenDistributionService):
        self.total_supply = total_supply
        self.distribution = distribution_service
        
        # Economic parameters
        self.staking_apy = Decimal("0.12")  # 12% APY
        self.burn_rate = Decimal("0.001")   # 0.1% per transaction
        self.velocity_target = Decimal("4") # Target velocity (annual turnover)
    
    def simulate_supply_schedule(self, months: int = 48) -> List[Dict]:
        """Simulate circulating supply over time"""
        schedule = []
        
        for month in range(months + 1):
            date = datetime.utcnow() + timedelta(days=month * 30)
            circulating = Decimal("0")
            
            # Calculate released from each category
            for category, allocation in self.distribution.allocations.items():
                category_supply = self.total_supply * (allocation.total_supply_percent / 100)
                
                # TGE unlock
                tge = category_supply * (allocation.tge_unlock_percent / 100)
                
                # Vesting unlock
                if month >= allocation.cliff_months:
                    if allocation.vesting_months > 0:
                        vested_months = min(month - allocation.cliff_months, allocation.vesting_months)
                        vested = (category_supply - tge) * Decimal(vested_months) / Decimal(allocation.vesting_months)
                    else:
                        vested = category_supply - tge
                else:
                    vested = Decimal("0")
                
                circulating += tge + vested
            
            schedule.append({
                "month": month,
                "date": date.isoformat(),
                "circulating_supply": float(circulating),
                "circulating_percent": float(circulating / self.total_supply * 100),
                "locked_supply": float(self.total_supply - circulating)
            })
        
        return schedule
    
    def simulate_staking_scenario(
        self,
        initial_stake_percent: Decimal = Decimal("30"),
        growth_rate: Decimal = Decimal("0.02"),
        months: int = 24
    ) -> List[Dict]:
        """Simulate staking dynamics"""
        results = []
        stake_percent = initial_stake_percent
        
        supply_schedule = self.simulate_supply_schedule(months)
        
        for i, supply in enumerate(supply_schedule):
            circulating = Decimal(str(supply["circulating_supply"]))
            staked = circulating * (stake_percent / 100)
            
            # Staking rewards (simplified)
            monthly_rewards = staked * (self.staking_apy / 12)
            
            # Update stake percent (growth)
            stake_percent = min(Decimal("80"), stake_percent + growth_rate)
            
            results.append({
                "month": i,
                "circulating": float(circulating),
                "staked": float(staked),
                "stake_percent": float(stake_percent),
                "monthly_rewards": float(monthly_rewards),
                "effective_apy": float(self.staking_apy * 100)
            })
        
        return results
    
    def calculate_token_velocity(
        self,
        transaction_volume: Decimal,
        circulating_supply: Decimal
    ) -> Decimal:
        """Calculate token velocity (turnover rate)"""
        if circulating_supply == 0:
            return Decimal("0")
        return transaction_volume / circulating_supply


# Create singleton instances
distribution_service = TokenDistributionService()
economic_simulator = EconomicSimulator(Decimal("1000000000"), distribution_service)  # 1B supply
