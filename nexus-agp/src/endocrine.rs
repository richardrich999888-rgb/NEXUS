//! # Artificial Human Endocrine System (AHES)
//!
//! PATENT CLAIMS 8-12: Bio-inspired Computational Governance
//!
//! Maps 8 reputation dimensions to hormone analogs with biological kinetics:
//! - Half-life decay (first-order kinetics)
//! - Receptor saturation (Michaelis-Menten)
//! - Negative feedback inhibition
//! - Circadian rhythm modulation

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// The 8 hormones of the Artificial Endocrine System
/// 
/// Each maps to a reputation dimension with biological analog behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hormone {
    /// Cortisol (Accuracy) - Stress/performance hormone
    /// Increases under task pressure, drives accuracy
    Cortisol,
    /// Oxytocin (Cooperation) - Social bonding hormone
    /// Released during successful multi-agent collaboration
    Oxytocin,
    /// Serotonin (Stability) - Mood regulation hormone
    /// Maintains consistent, predictable behavior
    Serotonin,
    /// Dopamine (Uniqueness) - Reward/novelty hormone
    /// Surges when novel solutions are generated
    Dopamine,
    /// Adrenaline (Latency) - Fight-or-flight hormone
    /// Controls response speed and urgency
    Adrenaline,
    /// Endorphins (Ethics) - Pain/pleasure balance hormone
    /// Released when ethical constraints are satisfied
    Endorphins,
    /// Norepinephrine (Novelty) - Alertness hormone
    /// Modulates exploration vs exploitation
    Norepinephrine,
    /// GrowthHormone (Longevity) - Development hormone
    /// Accumulates with long-term consistent performance
    GrowthHormone,
}

impl Hormone {
    /// Get biological half-life in seconds
    /// 
    /// Based on actual hormone half-lives scaled for computation
    pub fn half_life(&self) -> f64 {
        match self {
            Hormone::Cortisol => 90.0 * 60.0,        // 90 min (like real cortisol)
            Hormone::Oxytocin => 3.0 * 60.0,         // 3 min (very short-lived)
            Hormone::Serotonin => 24.0 * 60.0 * 60.0, // 24h (stable)
            Hormone::Dopamine => 5.0 * 60.0,         // 5 min (transient)
            Hormone::Adrenaline => 2.0 * 60.0,       // 2 min (burst hormone)
            Hormone::Endorphins => 20.0 * 60.0,      // 20 min
            Hormone::Norepinephrine => 1.5 * 60.0,   // 1.5 min
            Hormone::GrowthHormone => 15.0 * 60.0,   // 15 min pulses
        }
    }

    /// Get receptor binding affinity (Km in Michaelis-Menten)
    /// 
    /// Lower Km = higher affinity = more sensitive
    pub fn km(&self) -> f64 {
        match self {
            Hormone::Cortisol => 0.3,      // Medium affinity
            Hormone::Oxytocin => 0.1,      // High affinity (strong social response)
            Hormone::Serotonin => 0.5,     // Lower affinity (buffered)
            Hormone::Dopamine => 0.2,      // High affinity (reward sensitivity)
            Hormone::Adrenaline => 0.05,   // Very high (emergency response)
            Hormone::Endorphins => 0.4,    // Medium
            Hormone::Norepinephrine => 0.15, // High
            Hormone::GrowthHormone => 0.6, // Low (slow development)
        }
    }

    /// Maximum secretion rate per action
    pub fn max_secretion_rate(&self) -> f64 {
        match self {
            Hormone::Cortisol => 0.4,
            Hormone::Oxytocin => 0.6,       // Strong social signals
            Hormone::Serotonin => 0.1,      // Slow accumulation
            Hormone::Dopamine => 0.5,       // Burst rewards
            Hormone::Adrenaline => 0.8,     // Emergency bursts
            Hormone::Endorphins => 0.3,
            Hormone::Norepinephrine => 0.5,
            Hormone::GrowthHormone => 0.05, // Very slow growth
        }
    }

    /// Get all 8 hormones
    pub fn all() -> [Hormone; 8] {
        [
            Hormone::Cortisol,
            Hormone::Oxytocin,
            Hormone::Serotonin,
            Hormone::Dopamine,
            Hormone::Adrenaline,
            Hormone::Endorphins,
            Hormone::Norepinephrine,
            Hormone::GrowthHormone,
        ]
    }
}

impl std::fmt::Display for Hormone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hormone::Cortisol => write!(f, "Cortisol (Accuracy)"),
            Hormone::Oxytocin => write!(f, "Oxytocin (Cooperation)"),
            Hormone::Serotonin => write!(f, "Serotonin (Stability)"),
            Hormone::Dopamine => write!(f, "Dopamine (Uniqueness)"),
            Hormone::Adrenaline => write!(f, "Adrenaline (Latency)"),
            Hormone::Endorphins => write!(f, "Endorphins (Ethics)"),
            Hormone::Norepinephrine => write!(f, "Norepinephrine (Novelty)"),
            Hormone::GrowthHormone => write!(f, "GrowthHormone (Longevity)"),
        }
    }
}

/// Hormone level with biological kinetics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormoneLevel {
    /// Current concentration [0.0, 1.0]
    pub level: f64,
    /// Peak level ever achieved (for normalization)
    pub peak: f64,
    /// Last update timestamp (seconds)
    pub last_updated: f64,
    /// Circadian phase offset (radians)
    pub circadian_phase: f64,
}

impl Default for HormoneLevel {
    fn default() -> Self {
        Self {
            level: 0.5, // Baseline
            peak: 0.5,
            last_updated: 0.0,
            circadian_phase: 0.0,
        }
    }
}

impl HormoneLevel {
    pub fn new(initial: f64) -> Self {
        Self {
            level: initial.clamp(0.0, 1.0),
            peak: initial.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Apply first-order decay: level *= 0.5^(Δt / half_life)
    /// 
    /// PATENT CLAIM: Biological half-life decay for continuous governance
    pub fn decay(&mut self, delta_time: f64, half_life: f64) {
        if delta_time > 0.0 && half_life > 0.0 {
            let decay_factor = 0.5_f64.powf(delta_time / half_life);
            // Decay towards baseline (0.5), not zero
            let baseline = 0.5;
            self.level = baseline + (self.level - baseline) * decay_factor;
            self.last_updated += delta_time;
        }
    }

    /// Secrete hormone (increase level)
    pub fn secrete(&mut self, amount: f64) {
        self.level = (self.level + amount).clamp(0.0, 1.0);
        if self.level > self.peak {
            self.peak = self.level;
        }
    }

    /// Apply circadian modulation
    /// 
    /// Time-of-day variation similar to biological rhythms
    pub fn circadian_factor(&self, time_of_day: f64) -> f64 {
        // 24-hour cycle, amplitude 20%
        let phase = (time_of_day / 86400.0) * 2.0 * std::f64::consts::PI + self.circadian_phase;
        1.0 + 0.2 * phase.sin()
    }

    /// Get circadian-adjusted level
    pub fn effective_level(&self, time_of_day: f64) -> f64 {
        (self.level * self.circadian_factor(time_of_day)).clamp(0.0, 1.0)
    }
}

/// Receptor for a specific hormone
/// 
/// Implements Michaelis-Menten saturation kinetics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormoneReceptor {
    /// Receptor density [0.0, 1.0] - higher = more sensitive
    pub density: f64,
    /// Maximum response (Vmax)
    pub vmax: f64,
    /// Binding affinity (Km from hormone)
    pub km: f64,
    /// Downregulation factor (reduces with overexposure)
    pub downregulation: f64,
}

impl HormoneReceptor {
    pub fn new(hormone: Hormone) -> Self {
        Self {
            density: 1.0,
            vmax: 1.0,
            km: hormone.km(),
            downregulation: 1.0,
        }
    }

    /// Calculate response using Michaelis-Menten kinetics
    /// 
    /// response = (Vmax × [H] × density × downreg) / (Km + [H])
    /// 
    /// PATENT CLAIM: Receptor saturation curves for privilege levels
    pub fn response(&self, hormone_level: f64) -> f64 {
        if hormone_level <= 0.0 {
            return 0.0;
        }
        let effective_vmax = self.vmax * self.density * self.downregulation;
        (effective_vmax * hormone_level) / (self.km + hormone_level)
    }

    /// Downregulate receptor after prolonged exposure
    /// 
    /// High hormone levels reduce receptor sensitivity (biological adaptation)
    pub fn downregulate(&mut self, exposure_duration: f64, hormone_level: f64) {
        if hormone_level > 0.7 && exposure_duration > 60.0 {
            // Reduce by 10% per minute of high exposure
            let reduction = 0.1 * (exposure_duration / 60.0) * (hormone_level - 0.7) / 0.3;
            self.downregulation = (self.downregulation - reduction).clamp(0.1, 1.0);
        }
    }

    /// Upregulate receptor (recovery from downregulation)
    pub fn upregulate(&mut self, recovery_time: f64) {
        // Recover 5% per hour
        let recovery = 0.05 * (recovery_time / 3600.0);
        self.downregulation = (self.downregulation + recovery).clamp(0.1, 1.0);
    }
}

/// Complete endocrine state of an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndocrineState {
    /// Hormone levels for each type
    pub levels: HashMap<Hormone, HormoneLevel>,
    /// Receptor states for each hormone
    pub receptors: HashMap<Hormone, HormoneReceptor>,
    /// System time (seconds since agent creation)
    pub system_time: f64,
    /// Agent ID for tracking
    pub agent_id: [u8; 32],
}

impl EndocrineState {
    /// Create new endocrine state for an agent
    pub fn new(agent_id: [u8; 32]) -> Self {
        let mut levels = HashMap::new();
        let mut receptors = HashMap::new();

        for hormone in Hormone::all() {
            levels.insert(hormone, HormoneLevel::default());
            receptors.insert(hormone, HormoneReceptor::new(hormone));
        }

        Self {
            levels,
            receptors,
            system_time: 0.0,
            agent_id,
        }
    }

    /// Advance time and apply decay to all hormones
    pub fn tick(&mut self, delta_time: f64) {
        self.system_time += delta_time;

        for hormone in Hormone::all() {
            if let Some(level) = self.levels.get_mut(&hormone) {
                level.decay(delta_time, hormone.half_life());
            }
        }
    }

    /// Secrete a hormone in response to an event
    pub fn secrete(&mut self, hormone: Hormone, stimulus_strength: f64) {
        let max_rate = hormone.max_secretion_rate();
        let amount = (stimulus_strength * max_rate).clamp(0.0, max_rate);

        if let Some(level) = self.levels.get_mut(&hormone) {
            level.secrete(amount);
        }
    }

    /// Get privilege level for an action (receptor-mediated response)
    /// 
    /// PATENT CLAIM: Privilege as receptor density and response curve
    pub fn privilege(&self, hormone: Hormone) -> f64 {
        let level = self.levels.get(&hormone).map(|l| l.level).unwrap_or(0.5);
        let receptor = self.receptors.get(&hormone);

        match receptor {
            Some(r) => r.response(level),
            None => level, // Fallback to raw level
        }
    }

    /// Compute overall alignment (homeostasis indicator)
    /// 
    /// High alignment = all hormones near baseline = stable system
    pub fn alignment(&self) -> f64 {
        let baseline = 0.5;
        let mut deviation_sum = 0.0;
        let mut count = 0.0;

        for level in self.levels.values() {
            deviation_sum += (level.level - baseline).abs();
            count += 1.0;
        }

        if count > 0.0 {
            1.0 - (deviation_sum / count) // Higher when closer to baseline
        } else {
            1.0
        }
    }

    /// Get the dominant hormone (highest above baseline)
    pub fn dominant_hormone(&self) -> Option<Hormone> {
        let baseline = 0.5;
        self.levels
            .iter()
            .filter(|(_, l)| l.level > baseline)
            .max_by(|(_, a), (_, b)| {
                a.level.partial_cmp(&b.level).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(h, _)| *h)
    }

    /// Apply negative feedback: high hormone levels inhibit further secretion
    /// 
    /// PATENT CLAIM: Negative feedback loop for self-regulation
    pub fn apply_negative_feedback(&mut self, hormone: Hormone) -> f64 {
        let level = self.levels.get(&hormone).map(|l| l.level).unwrap_or(0.5);

        // Feedback factor: reduces secretion at high levels
        // At level 0.5 → factor 1.0 (no feedback)
        // At level 0.9 → factor 0.2 (strong inhibition)
        if level > 0.5 {
            1.0 - 0.8 * ((level - 0.5) / 0.5).powi(2)
        } else {
            1.0
        }
    }

    /// Convert to 8-dimensional reputation vector
    pub fn to_reputation_vector(&self) -> [f64; 8] {
        [
            self.levels.get(&Hormone::Cortisol).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::Oxytocin).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::Serotonin).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::Dopamine).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::Adrenaline).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::Endorphins).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::Norepinephrine).map(|l| l.level).unwrap_or(0.5),
            self.levels.get(&Hormone::GrowthHormone).map(|l| l.level).unwrap_or(0.5),
        ]
    }

    /// Create from 8-dimensional reputation vector
    pub fn from_reputation_vector(agent_id: [u8; 32], vector: [f64; 8]) -> Self {
        let mut state = Self::new(agent_id);

        let hormones = Hormone::all();
        for (i, hormone) in hormones.iter().enumerate() {
            if let Some(level) = state.levels.get_mut(hormone) {
                level.level = vector[i].clamp(0.0, 1.0);
                level.peak = level.level;
            }
        }

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_life_decay() {
        let mut level = HormoneLevel::new(0.9);
        let half_life = Hormone::Dopamine.half_life(); // 5 min

        // After one half-life, should be halfway to baseline
        level.decay(half_life, half_life);

        // Started at 0.9, baseline 0.5, so 0.5 + (0.9-0.5)*0.5 = 0.7
        assert!((level.level - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_receptor_saturation() {
        let receptor = HormoneReceptor::new(Hormone::Oxytocin);

        // At low levels, response is nearly linear
        let low_response = receptor.response(0.1);

        // At high levels, response saturates
        let high_response = receptor.response(0.9);

        // Saturation: diminishing returns
        assert!(high_response / low_response < 9.0);
    }

    #[test]
    fn test_negative_feedback() {
        let mut state = EndocrineState::new([0u8; 32]);

        // Secrete dopamine
        state.secrete(Hormone::Dopamine, 0.8);
        let fb1 = state.apply_negative_feedback(Hormone::Dopamine);

        // Higher level = stronger feedback inhibition
        state.secrete(Hormone::Dopamine, 0.5);
        let fb2 = state.apply_negative_feedback(Hormone::Dopamine);

        assert!(fb2 < fb1, "Higher levels should have stronger feedback");
    }

    #[test]
    fn test_alignment_at_baseline() {
        let state = EndocrineState::new([0u8; 32]);

        // All at baseline = perfect alignment
        let alignment = state.alignment();
        assert_eq!(alignment, 1.0);
    }

    #[test]
    fn test_circadian_rhythm() {
        let level = HormoneLevel {
            level: 0.5,
            peak: 0.5,
            last_updated: 0.0,
            circadian_phase: 0.0,
        };

        // At 6 AM (21600) vs 6 PM (64800) - opposite phases of the day
        let morning = level.effective_level(21600.0);  // 6 AM - rising phase
        let evening = level.effective_level(64800.0);  // 6 PM - falling phase

        // There should be variation across the day
        assert!(
            (morning - evening).abs() > 0.01 || morning != evening,
            "Circadian should vary: morning={}, evening={}",
            morning,
            evening
        );
    }
}
