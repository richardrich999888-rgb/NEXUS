//! Threat memory with deduplication and expiry.

use crate::threat::signature::SignedThreatReport;
use crate::threat::pattern::ThreatCategory;
use crate::identity::keypair::AsiId;
use crate::reputation::aggregation::ReputationAggregator;
use std::collections::{HashMap, HashSet};

/// Stores known threats with deduplication and expiry.
pub struct ThreatMemory {
    /// Active threats by report ID.
    threats: HashMap<[u8; 32], StoredThreat>,
    /// Index by category for fast lookup.
    by_category: HashMap<ThreatCategory, HashSet<[u8; 32]>>,
    /// Index by pattern hash for deduplication.
    by_pattern: HashMap<[u8; 32], HashSet<[u8; 32]>>,
    /// Maximum threats to store.
    capacity: usize,
    /// Threat expiry time.
    ttl: u64,
}

/// A stored threat with aggregated metadata.
#[derive(Debug, Clone)]
pub struct StoredThreat {
    /// Original report.
    pub report: SignedThreatReport,
    /// Aggregated confidence from multiple reporters.
    pub aggregated_confidence: f64,
    /// All reporters who confirmed this pattern.
    pub confirming_reporters: HashSet<AsiId>,
    /// Time this was first seen.
    pub first_seen: u64,
}

/// Result of attempting to add a threat.
#[derive(Debug)]
pub enum ThreatAddResult {
    /// New threat added.
    Added,
    /// Existing threat confirmed by additional reporter.
    Confirmed { new_confidence: f64 },
    /// Threat rejected (low reputation, invalid signature, etc.).
    Rejected { reason: String },
    /// Duplicate from same reporter.
    Duplicate,
}

impl ThreatMemory {
    /// Creates a new threat memory.
    pub fn new(capacity: usize, ttl: u64) -> Self {
        Self {
            threats: HashMap::new(),
            by_category: HashMap::new(),
            by_pattern: HashMap::new(),
            capacity,
            ttl,
        }
    }
    
    /// Returns the number of stored threats.
    pub fn len(&self) -> usize {
        self.threats.len()
    }
    
    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.threats.is_empty()
    }
    
    /// Adds a threat report, aggregating with existing reports if pattern matches.
    pub fn add(
        &mut self,
        report: SignedThreatReport,
        reputation: &ReputationAggregator,
        self_id: AsiId,
        current_time: u64,
    ) -> ThreatAddResult {
        // Check for exact duplicate
        let report_id = report.report_id();
        if self.threats.contains_key(&report_id) {
            return ThreatAddResult::Duplicate;
        }
        
        // Get reporter's reputation
        let reporter_rep = reputation.get_direct(self_id, report.reporter, current_time);
        
        // Reject if reputation too low
        const MIN_REPUTATION: f64 = 0.2;
        if reporter_rep < MIN_REPUTATION {
            return ThreatAddResult::Rejected {
                reason: format!("Reporter reputation {:.2} below threshold", reporter_rep),
            };
        }
        
        // Check if this pattern already exists
        let pattern_hash = report.pattern.pattern_hash;
        if let Some(existing_ids) = self.by_pattern.get(&pattern_hash) {
            // Find the primary report for this pattern
            if let Some(primary_id) = existing_ids.iter().next() {
                if let Some(stored) = self.threats.get_mut(primary_id) {
                    // Don't count same reporter twice
                    if stored.confirming_reporters.contains(&report.reporter) {
                        return ThreatAddResult::Duplicate;
                    }
                    
                    // Aggregate confidence
                    stored.confirming_reporters.insert(report.reporter);
                    
                    // Confidence increases with more reporters, weighted by reputation
                    let weighted_confidence = report.confidence * reporter_rep;
                    let n = stored.confirming_reporters.len() as f64;
                    stored.aggregated_confidence =
                        (stored.aggregated_confidence * (n - 1.0) + weighted_confidence) / n;
                    
                    // Cap at 1.0
                    stored.aggregated_confidence = stored.aggregated_confidence.min(1.0);
                    
                    return ThreatAddResult::Confirmed {
                        new_confidence: stored.aggregated_confidence,
                    };
                }
            }
        }
        
        // New threat - check capacity
        if self.threats.len() >= self.capacity {
            self.evict_oldest(current_time);
        }
        
        // Add new threat
        let mut confirming = HashSet::new();
        confirming.insert(report.reporter);
        
        let category = report.pattern.category;
        let stored = StoredThreat {
            aggregated_confidence: report.confidence * reporter_rep,
            confirming_reporters: confirming,
            first_seen: current_time,
            report,
        };
        
        self.threats.insert(report_id, stored);
        
        self.by_category
            .entry(category)
            .or_default()
            .insert(report_id);
        
        self.by_pattern
            .entry(pattern_hash)
            .or_default()
            .insert(report_id);
        
        ThreatAddResult::Added
    }
    
    /// Removes expired threats.
    pub fn expire(&mut self, current_time: u64) {
        let expired: Vec<[u8; 32]> = self.threats
            .iter()
            .filter(|(_, t)| current_time.saturating_sub(t.first_seen) > self.ttl)
            .map(|(id, _)| *id)
            .collect();
        
        for id in expired {
            self.remove(&id);
        }
    }
    
    /// Checks if a pattern hash is known as a threat above threshold.
    pub fn is_known_threat(&self, pattern_hash: &[u8; 32], threshold: f64) -> bool {
        self.by_pattern
            .get(pattern_hash)
            .and_then(|ids| ids.iter().next())
            .and_then(|id| self.threats.get(id))
            .map(|t| t.aggregated_confidence >= threshold)
            .unwrap_or(false)
    }
    
    /// Gets threats by category.
    pub fn get_by_category(&self, category: ThreatCategory) -> Vec<&StoredThreat> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.threats.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Returns all active threats above confidence threshold.
    pub fn active_threats(&self, threshold: f64) -> Vec<&SignedThreatReport> {
        self.threats
            .values()
            .filter(|t| t.aggregated_confidence >= threshold)
            .map(|t| &t.report)
            .collect()
    }
    
    /// Gets a specific threat by report ID.
    pub fn get(&self, report_id: &[u8; 32]) -> Option<&StoredThreat> {
        self.threats.get(report_id)
    }
    
    fn remove(&mut self, id: &[u8; 32]) {
        if let Some(stored) = self.threats.remove(id) {
            if let Some(set) = self.by_category.get_mut(&stored.report.pattern.category) {
                set.remove(id);
            }
            if let Some(set) = self.by_pattern.get_mut(&stored.report.pattern.pattern_hash) {
                set.remove(id);
            }
        }
    }
    
    fn evict_oldest(&mut self, current_time: u64) {
        // First try to evict expired
        self.expire(current_time);
        
        if self.threats.len() >= self.capacity {
            // Evict lowest confidence
            if let Some(id) = self.threats
                .iter()
                .min_by(|(_, a), (_, b)| {
                    a.aggregated_confidence.partial_cmp(&b.aggregated_confidence).unwrap()
                })
                .map(|(id, _)| *id)
            {
                self.remove(&id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair::AsiIdentity;
    use crate::threat::pattern::ThreatPattern;
    
    fn setup() -> (ThreatMemory, ReputationAggregator, AsiIdentity) {
        let memory = ThreatMemory::new(100, 3600);
        let reputation = ReputationAggregator::new(100);
        let identity = AsiIdentity::generate();
        (memory, reputation, identity)
    }
    
    #[test]
    fn test_add_threat() {
        let (mut memory, reputation, identity) = setup();
        
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
        let report = SignedThreatReport::new(&identity, pattern, 0.8, 0);
        
        let result = memory.add(report, &reputation, identity.id, 0);
        assert!(matches!(result, ThreatAddResult::Added));
        assert_eq!(memory.len(), 1);
    }
    
    #[test]
    fn test_duplicate_rejection() {
        let (mut memory, reputation, identity) = setup();
        
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
        let report = SignedThreatReport::new(&identity, pattern.clone(), 0.8, 0);
        
        memory.add(report.clone(), &reputation, identity.id, 0);
        
        // Same reporter, same pattern = duplicate
        let result = memory.add(report, &reputation, identity.id, 1);
        assert!(matches!(result, ThreatAddResult::Duplicate));
    }
    
    #[test]
    fn test_pattern_confirmation() {
        let (mut memory, mut reputation, identity1) = setup();
        let identity2 = AsiIdentity::generate();
        
        // Build reputation for identity2
        for t in 0..20 {
            reputation.record_positive(identity1.id, identity2.id, t);
        }
        
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
        
        // First report
        let report1 = SignedThreatReport::new(&identity1, pattern.clone(), 0.8, 0);
        memory.add(report1, &reputation, identity1.id, 0);
        
        // Second report from different identity confirms
        let report2 = SignedThreatReport::new(&identity2, pattern, 0.85, 1);
        let result = memory.add(report2, &reputation, identity1.id, 1);
        
        assert!(matches!(result, ThreatAddResult::Confirmed { .. }));
    }
    
    #[test]
    fn test_expiry() {
        let (mut memory, reputation, identity) = setup();
        
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
        let report = SignedThreatReport::new(&identity, pattern, 0.8, 0);
        
        memory.add(report, &reputation, identity.id, 0);
        assert_eq!(memory.len(), 1);
        
        // Expire after TTL
        memory.expire(4000);
        assert_eq!(memory.len(), 0);
    }
}
