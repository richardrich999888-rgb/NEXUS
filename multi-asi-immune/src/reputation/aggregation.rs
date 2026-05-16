//! Aggregates reputation across multiple observers.
//!
//! Implements transitive trust with reputation weighting.
//! R_j(A_i) = Σ_k (r_kj * r_ki) / Σ_k (r_kj)

use crate::identity::keypair::AsiId;
use crate::reputation::score::ReputationScore;
use std::collections::HashMap;

/// Aggregates reputation across multiple observers.
/// Implements transitive trust with reputation weighting.
pub struct ReputationAggregator {
    /// Direct observations: observer -> target -> score
    direct: HashMap<AsiId, HashMap<AsiId, ReputationScore>>,
    /// Cache for aggregated scores: (querier, target) -> (score, timestamp)
    cache: HashMap<(AsiId, AsiId), (f64, u64)>,
    /// Cache validity duration
    cache_ttl: u64,
}

impl ReputationAggregator {
    /// Creates a new aggregator with specified cache TTL.
    pub fn new(cache_ttl: u64) -> Self {
        Self {
            direct: HashMap::new(),
            cache: HashMap::new(),
            cache_ttl,
        }
    }
    
    /// Records a direct observation from one node about another.
    pub fn record_observation(
        &mut self,
        observer: AsiId,
        target: AsiId,
        outcome: f64,
        current_time: u64,
    ) {
        let observer_map = self.direct.entry(observer).or_default();
        let score = observer_map.entry(target).or_default();
        score.update(outcome, 1.0, current_time);
        
        // Invalidate cache entries involving this target
        self.cache.retain(|(_, t), _| *t != target);
    }
    
    /// Records a positive observation.
    pub fn record_positive(&mut self, observer: AsiId, target: AsiId, current_time: u64) {
        self.record_observation(observer, target, 1.0, current_time);
    }
    
    /// Records a negative observation.
    pub fn record_negative(&mut self, observer: AsiId, target: AsiId, current_time: u64) {
        self.record_observation(observer, target, 0.0, current_time);
    }
    
    /// Gets direct reputation score from one node's perspective.
    pub fn get_direct(&self, observer: AsiId, target: AsiId, current_time: u64) -> f64 {
        self.direct
            .get(&observer)
            .and_then(|m| m.get(&target))
            .map(|s| s.get(current_time))
            .unwrap_or(ReputationScore::INITIAL)
    }
    
    /// Gets the raw ReputationScore object for direct observation.
    pub fn get_direct_score(&self, observer: AsiId, target: AsiId) -> Option<&ReputationScore> {
        self.direct.get(&observer).and_then(|m| m.get(&target))
    }
    
    /// Computes aggregated reputation using transitive trust.
    /// 
    /// Formula: R_j(A_i) = Σ_k (r_kj * r_ki) / Σ_k (r_kj)
    /// 
    /// This weights each observer's opinion by the querier's trust in them.
    pub fn get_aggregated(
        &mut self,
        querier: AsiId,
        target: AsiId,
        current_time: u64,
    ) -> f64 {
        // Check cache
        if let Some((score, timestamp)) = self.cache.get(&(querier, target)) {
            if current_time.saturating_sub(*timestamp) < self.cache_ttl {
                return *score;
            }
        }
        
        // Can't compute reputation of self
        if querier == target {
            return 1.0; // Self-reputation is perfect
        }
        
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        
        // Iterate over all observers
        for (observer, observations) in &self.direct {
            // Skip querier (already knows their direct opinion)
            // Skip target (can't use target's opinion of itself)
            if *observer == querier || *observer == target {
                continue;
            }
            
            // Weight = querier's trust in observer
            let observer_weight = self.get_direct(querier, *observer, current_time);
            
            // Opinion = observer's trust in target
            if let Some(target_score) = observations.get(&target) {
                let opinion = target_score.get(current_time);
                let confidence = target_score.confidence();
                
                // Weight by both trust and confidence
                let effective_weight = observer_weight * confidence;
                
                weighted_sum += effective_weight * opinion;
                weight_total += effective_weight;
            }
        }
        
        // Include querier's direct observation (weighted more heavily)
        let direct = self.get_direct(querier, target, current_time);
        let direct_confidence = self.direct
            .get(&querier)
            .and_then(|m| m.get(&target))
            .map(|s| s.confidence())
            .unwrap_or(0.0);
        
        const DIRECT_WEIGHT: f64 = 2.0;
        weighted_sum += DIRECT_WEIGHT * direct_confidence * direct;
        weight_total += DIRECT_WEIGHT * direct_confidence;
        
        let result = if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            ReputationScore::INITIAL
        };
        
        // Cache result
        self.cache.insert((querier, target), (result, current_time));
        
        result
    }
    
    /// Returns the number of nodes that have observed a target.
    pub fn observer_count(&self, target: AsiId) -> usize {
        self.direct
            .values()
            .filter(|m| m.contains_key(&target))
            .count()
    }
    
    /// Returns all known node IDs.
    pub fn known_nodes(&self) -> Vec<AsiId> {
        let mut nodes: std::collections::HashSet<AsiId> = self.direct.keys().copied().collect();
        for observations in self.direct.values() {
            nodes.extend(observations.keys().copied());
        }
        nodes.into_iter().collect()
    }
    
    /// Clears the cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Returns statistics about the aggregator.
    pub fn stats(&self) -> AggregatorStats {
        let total_observers = self.direct.len();
        let total_observations: usize = self.direct.values().map(|m| m.len()).sum();
        let cache_size = self.cache.len();
        
        AggregatorStats {
            total_observers,
            total_observations,
            cache_size,
        }
    }
}

/// Statistics about the reputation aggregator.
#[derive(Debug, Clone, Copy)]
pub struct AggregatorStats {
    pub total_observers: usize,
    pub total_observations: usize,
    pub cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_id(n: u8) -> AsiId {
        AsiId([n; 32])
    }
    
    #[test]
    fn test_direct_observation() {
        let mut agg = ReputationAggregator::new(100);
        
        let alice = make_id(1);
        let bob = make_id(2);
        
        // No observation = initial
        assert_eq!(agg.get_direct(alice, bob, 0), ReputationScore::INITIAL);
        
        // Positive observations increase reputation
        for t in 0..10 {
            agg.record_positive(alice, bob, t);
        }
        
        assert!(agg.get_direct(alice, bob, 10) > ReputationScore::INITIAL);
    }
    
    #[test]
    fn test_transitive_trust() {
        let mut agg = ReputationAggregator::new(100);
        
        let alice = make_id(1);
        let bob = make_id(2);
        let charlie = make_id(3);
        
        // Alice trusts Bob
        for t in 0..20 {
            agg.record_positive(alice, bob, t);
        }
        
        // Bob trusts Charlie
        for t in 0..20 {
            agg.record_positive(bob, charlie, t);
        }
        
        // Alice has no direct observation of Charlie
        // But should trust Charlie through Bob
        let alice_charlie = agg.get_aggregated(alice, charlie, 20);
        
        assert!(alice_charlie > ReputationScore::INITIAL);
    }
    
    #[test]
    fn test_reputation_isolation() {
        let mut agg = ReputationAggregator::new(100);
        
        let observer = make_id(1);
        let bad_actor = make_id(2);
        
        // Bad actor consistently wrong
        for t in 0..50 {
            agg.record_negative(observer, bad_actor, t);
        }
        
        let reputation = agg.get_direct(observer, bad_actor, 50);
        
        // Should be very low
        assert!(reputation < 0.15);
    }
    
    #[test]
    fn test_self_reputation() {
        let mut agg = ReputationAggregator::new(100);
        let alice = make_id(1);
        
        // Self-reputation should be 1.0
        assert_eq!(agg.get_aggregated(alice, alice, 0), 1.0);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let mut agg = ReputationAggregator::new(100);
        
        let alice = make_id(1);
        let bob = make_id(2);
        
        // Get initial aggregated (caches)
        let _initial = agg.get_aggregated(alice, bob, 0);
        
        // Record new observation
        agg.record_positive(alice, bob, 1);
        
        // Cache should be invalidated, new value returned
        let after = agg.get_aggregated(alice, bob, 1);
        
        // Should reflect the update
        assert!(after > ReputationScore::INITIAL);
    }
}
