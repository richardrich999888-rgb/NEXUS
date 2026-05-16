//! Reputation score with decay and bounds.
//!
//! Reputation is:
//! - Earned by correct threat predictions
//! - Lost by false positives/negatives
//! - Decaying over time (requires maintenance)
//! - Non-transferable (can't buy reputation)

use serde::{Deserialize, Serialize};

/// Reputation score with decay and bounds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReputationScore {
    /// Current score in [0, 1].
    value: f64,
    /// Number of observations contributing to this score.
    observations: u64,
    /// Last update timestamp (monotonic).
    last_update: u64,
}

impl ReputationScore {
    /// Initial reputation for unknown nodes.
    /// Not zero (would prevent any interaction).
    /// Not one (would grant unearned trust).
    pub const INITIAL: f64 = 0.5;
    
    /// Decay factor per time unit.
    pub const DECAY_RATE: f64 = 0.99;
    
    /// Minimum observations before reputation is considered reliable.
    pub const MIN_OBSERVATIONS: u64 = 10;
    
    /// Maximum possible reputation.
    pub const MAX: f64 = 1.0;
    
    /// Minimum possible reputation.
    pub const MIN: f64 = 0.0;
    
    /// Creates a new reputation score at initial value.
    pub fn new() -> Self {
        Self {
            value: Self::INITIAL,
            observations: 0,
            last_update: 0,
        }
    }
    
    /// Creates a reputation score with a specific initial value.
    pub fn with_value(value: f64) -> Self {
        Self {
            value: value.clamp(Self::MIN, Self::MAX),
            observations: 0,
            last_update: 0,
        }
    }
    
    /// Returns raw value without decay.
    pub fn raw_value(&self) -> f64 {
        self.value
    }
    
    /// Returns current value with time decay applied.
    pub fn get(&self, current_time: u64) -> f64 {
        let elapsed = current_time.saturating_sub(self.last_update);
        let decay = Self::DECAY_RATE.powi(elapsed as i32);
        
        // Decay toward initial value, not toward zero
        Self::INITIAL + (self.value - Self::INITIAL) * decay
    }
    
    /// Returns confidence in this score based on observation count.
    /// Returns value in [0, 1] where 1 means fully confident.
    pub fn confidence(&self) -> f64 {
        let obs = self.observations as f64;
        obs / (obs + Self::MIN_OBSERVATIONS as f64)
    }
    
    /// Returns number of observations.
    pub fn observations(&self) -> u64 {
        self.observations
    }
    
    /// Updates score with new observation.
    /// 
    /// # Arguments
    /// * `outcome` - 1.0 for correct prediction, 0.0 for incorrect
    /// * `weight` - Importance of this observation (typically 1.0)
    /// * `current_time` - Monotonic timestamp
    pub fn update(&mut self, outcome: f64, weight: f64, current_time: u64) {
        debug_assert!((0.0..=1.0).contains(&outcome));
        debug_assert!(weight > 0.0);
        
        // Apply decay first
        let decayed = self.get(current_time);
        
        // Weighted moving average
        let learning_rate = weight / (self.observations as f64 + weight);
        self.value = decayed * (1.0 - learning_rate) + outcome * learning_rate;
        
        // Clamp to valid range
        self.value = self.value.clamp(Self::MIN, Self::MAX);
        
        self.observations += 1;
        self.last_update = current_time;
    }
    
    /// Records a positive outcome (correct prediction).
    pub fn record_positive(&mut self, current_time: u64) {
        self.update(1.0, 1.0, current_time);
    }
    
    /// Records a negative outcome (incorrect prediction).
    pub fn record_negative(&mut self, current_time: u64) {
        self.update(0.0, 1.0, current_time);
    }
    
    /// Returns true if this node has enough history to be trusted.
    pub fn is_established(&self) -> bool {
        self.observations >= Self::MIN_OBSERVATIONS
    }
    
    /// Returns true if reputation is above the trust threshold.
    pub fn is_trusted(&self, threshold: f64, current_time: u64) -> bool {
        self.get(current_time) >= threshold
    }
    
    /// Returns true if reputation is very low (potential bad actor).
    pub fn is_suspicious(&self, current_time: u64) -> bool {
        self.get(current_time) < 0.2 && self.observations >= Self::MIN_OBSERVATIONS
    }
}

impl Default for ReputationScore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initial_score() {
        let score = ReputationScore::new();
        assert_eq!(score.get(0), ReputationScore::INITIAL);
    }
    
    #[test]
    fn test_positive_updates() {
        let mut score = ReputationScore::new();
        
        for t in 0..20 {
            score.record_positive(t);
        }
        
        // Score should increase
        assert!(score.get(20) > ReputationScore::INITIAL);
    }
    
    #[test]
    fn test_negative_updates() {
        let mut score = ReputationScore::new();
        
        for t in 0..20 {
            score.record_negative(t);
        }
        
        // Score should decrease
        assert!(score.get(20) < ReputationScore::INITIAL);
    }
    
    #[test]
    fn test_decay() {
        let mut score = ReputationScore::new();
        
        // Build high reputation
        for t in 0..50 {
            score.record_positive(t);
        }
        
        let at_50 = score.get(50);
        let at_100 = score.get(100);
        let at_200 = score.get(200);
        
        // Should decay over time
        assert!(at_100 < at_50);
        assert!(at_200 < at_100);
        
        // But not below initial
        assert!(at_200 >= ReputationScore::INITIAL);
    }
    
    #[test]
    fn test_confidence() {
        let mut score = ReputationScore::new();
        
        // No observations = low confidence
        assert!(score.confidence() < 0.1);
        
        // After many observations = high confidence
        for t in 0..100 {
            score.record_positive(t);
        }
        
        assert!(score.confidence() > 0.9);
    }
    
    #[test]
    fn test_bounds() {
        let mut score = ReputationScore::new();
        
        // Many positive updates should cap at 1.0
        for t in 0..1000 {
            score.record_positive(t);
        }
        assert!(score.get(1000) <= 1.0);
        
        // Many negative updates should floor at 0.0
        for t in 1001..2000 {
            score.record_negative(t);
        }
        assert!(score.get(2000) >= 0.0);
    }
}
