//! Operation Pricing - Dynamic pricing for CAUSALUX operations
//! 
//! Implements pricing policies for different operation types,
//! enabling fair resource allocation and spam prevention.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Operation types for pricing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Document operations
    DocumentInsert,
    DocumentDelete,
    DocumentUpdate,
    
    /// Counter operations (cheaper - simpler)
    CounterIncrement,
    CounterDecrement,
    
    /// Set operations
    SetAdd,
    SetRemove,
    
    /// Map operations
    MapPut,
    MapDelete,
    
    /// Sync operations
    SyncRequest,
    SyncResponse,
    
    /// Compute operations (GPU)
    GradientCompute,
    GradientSync,
    
    /// Storage operations
    StorageWrite,
    StorageRead,
    
    /// Custom operation
    Custom(String),
}

/// Pricing policy determines how prices are calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingPolicy {
    /// Fixed price per operation
    Fixed(u64),
    /// Price per byte
    PerByte { base: u64, per_byte: u64 },
    /// Price per compute unit
    PerCompute { base: u64, per_flop: u64 },
    /// Dynamic pricing based on network load
    Dynamic { base: u64, load_multiplier: f64 },
    /// Free (for essential operations)
    Free,
}

impl PricingPolicy {
    /// Calculate price for given size/compute
    pub fn calculate(&self, size_bytes: usize, compute_units: u64, load: f64) -> u64 {
        match self {
            PricingPolicy::Fixed(price) => *price,
            PricingPolicy::PerByte { base, per_byte } => {
                base + (size_bytes as u64 * per_byte)
            }
            PricingPolicy::PerCompute { base, per_flop } => {
                base + (compute_units * per_flop)
            }
            PricingPolicy::Dynamic { base, load_multiplier } => {
                let multiplier = 1.0 + (load * load_multiplier);
                (*base as f64 * multiplier) as u64
            }
            PricingPolicy::Free => 0,
        }
    }
}

/// Operation pricing configuration
#[derive(Debug, Clone)]
pub struct OperationPricing {
    /// Policies per operation type
    policies: HashMap<OperationType, PricingPolicy>,
    /// Default policy for unlisted operations
    default_policy: PricingPolicy,
    /// Current network load (0.0 - 1.0)
    network_load: f64,
    /// Minimum price floor
    min_price: u64,
    /// Maximum price ceiling
    max_price: u64,
}

impl Default for OperationPricing {
    fn default() -> Self {
        let mut policies = HashMap::new();
        
        // Document operations: per-byte pricing
        policies.insert(OperationType::DocumentInsert, PricingPolicy::PerByte {
            base: 10,
            per_byte: 1,
        });
        policies.insert(OperationType::DocumentDelete, PricingPolicy::Fixed(5));
        policies.insert(OperationType::DocumentUpdate, PricingPolicy::PerByte {
            base: 5,
            per_byte: 1,
        });
        
        // Counter operations: very cheap
        policies.insert(OperationType::CounterIncrement, PricingPolicy::Fixed(1));
        policies.insert(OperationType::CounterDecrement, PricingPolicy::Fixed(1));
        
        // Set operations
        policies.insert(OperationType::SetAdd, PricingPolicy::Fixed(2));
        policies.insert(OperationType::SetRemove, PricingPolicy::Fixed(2));
        
        // Map operations
        policies.insert(OperationType::MapPut, PricingPolicy::PerByte {
            base: 3,
            per_byte: 1,
        });
        policies.insert(OperationType::MapDelete, PricingPolicy::Fixed(2));
        
        // Sync operations: free to encourage participation
        policies.insert(OperationType::SyncRequest, PricingPolicy::Free);
        policies.insert(OperationType::SyncResponse, PricingPolicy::Free);
        
        // Compute operations: per-compute pricing
        policies.insert(OperationType::GradientCompute, PricingPolicy::PerCompute {
            base: 100,
            per_flop: 1,
        });
        policies.insert(OperationType::GradientSync, PricingPolicy::PerByte {
            base: 50,
            per_byte: 1,
        });
        
        // Storage: per-byte
        policies.insert(OperationType::StorageWrite, PricingPolicy::PerByte {
            base: 5,
            per_byte: 2,
        });
        policies.insert(OperationType::StorageRead, PricingPolicy::Free);
        
        Self {
            policies,
            default_policy: PricingPolicy::Fixed(10),
            network_load: 0.0,
            min_price: 0,
            max_price: 1_000_000,
        }
    }
}

impl OperationPricing {
    /// Create new pricing with default policies
    pub fn new() -> Self {
        Self::default()
    }

    /// Set policy for an operation type
    pub fn set_policy(&mut self, op_type: OperationType, policy: PricingPolicy) {
        self.policies.insert(op_type, policy);
    }

    /// Get policy for an operation type
    pub fn get_policy(&self, op_type: &OperationType) -> &PricingPolicy {
        self.policies.get(op_type).unwrap_or(&self.default_policy)
    }

    /// Calculate price for an operation
    pub fn price(
        &self,
        op_type: &OperationType,
        size_bytes: usize,
        compute_units: u64,
    ) -> u64 {
        let policy = self.get_policy(op_type);
        let raw_price = policy.calculate(size_bytes, compute_units, self.network_load);
        raw_price.max(self.min_price).min(self.max_price)
    }

    /// Update network load
    pub fn update_load(&mut self, load: f64) {
        self.network_load = load.max(0.0).min(1.0);
    }

    /// Get current network load
    pub fn network_load(&self) -> f64 {
        self.network_load
    }

    /// Estimate cost for a batch of operations
    pub fn estimate_batch(&self, operations: &[(OperationType, usize, u64)]) -> u64 {
        operations
            .iter()
            .map(|(op, size, compute)| self.price(op, *size, *compute))
            .sum()
    }

    /// Get price summary for all operation types
    pub fn price_summary(&self) -> HashMap<String, u64> {
        self.policies
            .keys()
            .map(|op| {
                let name = format!("{:?}", op);
                let price = self.price(op, 100, 1000); // Sample price
                (name, price)
            })
            .collect()
    }
}

/// Reward calculation for services
#[derive(Debug, Clone)]
pub struct RewardCalculator {
    /// Reward per KB of sync bandwidth
    pub bandwidth_reward_per_kb: u64,
    /// Reward per operation validated
    pub validation_reward: u64,
    /// Reward for staying online (per hour)
    pub uptime_reward_per_hour: u64,
    /// Bonus multiplier for high-reputation nodes
    pub reputation_multiplier: f64,
}

impl Default for RewardCalculator {
    fn default() -> Self {
        Self {
            bandwidth_reward_per_kb: 1,
            validation_reward: 5,
            uptime_reward_per_hour: 100,
            reputation_multiplier: 1.5,
        }
    }
}

impl RewardCalculator {
    /// Calculate reward for sync bandwidth contribution
    pub fn bandwidth_reward(&self, bytes: u64, reputation: f64) -> u64 {
        let kb = bytes / 1024;
        let base = kb * self.bandwidth_reward_per_kb;
        self.apply_reputation(base, reputation)
    }

    /// Calculate reward for operation validation
    pub fn validation_reward(&self, operations: u64, reputation: f64) -> u64 {
        let base = operations * self.validation_reward;
        self.apply_reputation(base, reputation)
    }

    /// Calculate uptime reward
    pub fn uptime_reward(&self, hours: u64, reputation: f64) -> u64 {
        let base = hours * self.uptime_reward_per_hour;
        self.apply_reputation(base, reputation)
    }

    fn apply_reputation(&self, base: u64, reputation: f64) -> u64 {
        if reputation > 0.8 {
            (base as f64 * self.reputation_multiplier) as u64
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_pricing() {
        let pricing = OperationPricing::new();
        let price = pricing.price(&OperationType::CounterIncrement, 0, 0);
        assert_eq!(price, 1);
    }

    #[test]
    fn test_per_byte_pricing() {
        let pricing = OperationPricing::new();
        // DocumentInsert: base 10 + 1 per byte
        let price = pricing.price(&OperationType::DocumentInsert, 100, 0);
        assert_eq!(price, 110); // 10 + 100*1
    }

    #[test]
    fn test_compute_pricing() {
        let pricing = OperationPricing::new();
        // GradientCompute: base 100 + 1 per FLOP
        let price = pricing.price(&OperationType::GradientCompute, 0, 1000);
        assert_eq!(price, 1100); // 100 + 1000*1
    }

    #[test]
    fn test_free_operations() {
        let pricing = OperationPricing::new();
        let price = pricing.price(&OperationType::SyncRequest, 1000, 1000);
        assert_eq!(price, 0);
    }

    #[test]
    fn test_dynamic_pricing() {
        let mut pricing = OperationPricing::new();
        pricing.set_policy(
            OperationType::Custom("test".to_string()),
            PricingPolicy::Dynamic { base: 100, load_multiplier: 2.0 },
        );
        
        // No load
        pricing.update_load(0.0);
        let price_low = pricing.price(&OperationType::Custom("test".to_string()), 0, 0);
        
        // High load
        pricing.update_load(1.0);
        let price_high = pricing.price(&OperationType::Custom("test".to_string()), 0, 0);
        
        assert!(price_high > price_low);
    }

    #[test]
    fn test_reward_calculation() {
        let calc = RewardCalculator::default();
        
        // Low reputation
        let reward1 = calc.bandwidth_reward(10240, 0.5); // 10KB
        assert_eq!(reward1, 10);
        
        // High reputation (1.5x)
        let reward2 = calc.bandwidth_reward(10240, 0.9);
        assert_eq!(reward2, 15);
    }
}
