//! # Virtual Gland System
//!
//! PATENT CLAIM 9: Bio-inspired Hormone Secretion
//!
//! Implements virtual glands that produce hormones based on stimuli:
//! - Hypothalamic Controller (master regulator)
//! - Pituitary Router (signal distribution)
//! - Specialized glands for each hormone class

use crate::endocrine::{Hormone, EndocrineState};
use serde::{Serialize, Deserialize};

/// Stimulus that triggers hormone secretion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stimulus {
    /// Task completed successfully (triggers accuracy/cortisol)
    TaskSuccess { difficulty: f64, latency_ms: u64 },
    /// Task failed (triggers stress response)
    TaskFailure { difficulty: f64, error_severity: f64 },
    /// Collaboration with other agents (triggers oxytocin)
    Collaboration { partner_count: usize, success_rate: f64 },
    /// Novel solution generated (triggers dopamine)
    NovelSolution { novelty_score: f64 },
    /// Urgent request (triggers adrenaline)
    Urgency { deadline_pressure: f64 },
    /// Ethical constraint satisfied (triggers endorphins)
    EthicalCompliance { constraint_difficulty: f64 },
    /// Exploration behavior (triggers norepinephrine)
    Exploration { risk_taken: f64 },
    /// Long-term consistency (triggers growth hormone)
    Consistency { days_stable: u32 },
}

impl Stimulus {
    /// Extract the primary hormone this stimulus affects
    pub fn primary_hormone(&self) -> Hormone {
        match self {
            Stimulus::TaskSuccess { .. } => Hormone::Cortisol,
            Stimulus::TaskFailure { .. } => Hormone::Cortisol,
            Stimulus::Collaboration { .. } => Hormone::Oxytocin,
            Stimulus::NovelSolution { .. } => Hormone::Dopamine,
            Stimulus::Urgency { .. } => Hormone::Adrenaline,
            Stimulus::EthicalCompliance { .. } => Hormone::Endorphins,
            Stimulus::Exploration { .. } => Hormone::Norepinephrine,
            Stimulus::Consistency { .. } => Hormone::GrowthHormone,
        }
    }

    /// Calculate stimulus strength [0.0, 1.0]
    pub fn strength(&self) -> f64 {
        match self {
            Stimulus::TaskSuccess { difficulty, .. } => *difficulty,
            Stimulus::TaskFailure { error_severity, .. } => *error_severity,
            Stimulus::Collaboration { success_rate, partner_count } => {
                success_rate * (1.0 + (*partner_count as f64).ln().max(0.0) * 0.2)
            }
            Stimulus::NovelSolution { novelty_score } => *novelty_score,
            Stimulus::Urgency { deadline_pressure } => *deadline_pressure,
            Stimulus::EthicalCompliance { constraint_difficulty } => *constraint_difficulty,
            Stimulus::Exploration { risk_taken } => *risk_taken,
            Stimulus::Consistency { days_stable } => (*days_stable as f64 / 30.0).min(1.0),
        }
    }
}

/// Trait for all virtual glands
pub trait Gland {
    /// Process a stimulus and return hormone secretions
    fn process(&self, stimulus: &Stimulus, state: &EndocrineState) -> Vec<(Hormone, f64)>;

    /// Get the hormones this gland produces
    fn produces(&self) -> Vec<Hormone>;
}

/// Hypothalamic Controller - master regulator
///
/// Like the biological hypothalamus, coordinates the entire endocrine system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypothalamicController {
    /// Stress sensitivity
    pub stress_threshold: f64,
    /// Overall activity level
    pub activity: f64,
}

impl HypothalamicController {
    pub fn new() -> Self {
        Self {
            stress_threshold: 0.7,
            activity: 1.0,
        }
    }

    /// Evaluate system stress and adjust activity
    pub fn evaluate_stress(&mut self, state: &EndocrineState) {
        // Check if cortisol is elevated
        let cortisol = state.levels.get(&Hormone::Cortisol)
            .map(|l| l.level)
            .unwrap_or(0.5);

        if cortisol > self.stress_threshold {
            // Reduce activity to protect system (like biological HPA axis)
            self.activity = (self.activity * 0.9).max(0.3);
        } else {
            // Slowly recover activity
            self.activity = (self.activity * 1.02).min(1.0);
        }
    }

    /// Generate releasing hormone signals for pituitary
    pub fn releasing_signals(&self, stimulus: &Stimulus) -> Vec<(Hormone, f64)> {
        let strength = stimulus.strength() * self.activity;
        let primary = stimulus.primary_hormone();

        let mut signals = vec![(primary, strength)];

        // Cross-talk: some stimuli affect multiple hormones
        match stimulus {
            Stimulus::TaskSuccess { latency_ms, .. } => {
                // Fast task also triggers adrenaline drop
                if *latency_ms < 100 {
                    signals.push((Hormone::Adrenaline, strength * 0.3));
                }
            }
            Stimulus::Collaboration { partner_count, .. } => {
                // Social activity also stabilizes mood
                if *partner_count > 2 {
                    signals.push((Hormone::Serotonin, strength * 0.2));
                }
            }
            Stimulus::NovelSolution { .. } => {
                // Novelty also triggers exploration
                signals.push((Hormone::Norepinephrine, strength * 0.4));
            }
            _ => {}
        }

        signals
    }
}

/// Pituitary Router - distributes signals to peripheral glands
///
/// Like the biological pituitary, connects hypothalamus to endocrine glands
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PituitaryRouter {
    /// Amplification factor for each hormone
    pub amplification: std::collections::HashMap<Hormone, f64>,
}

impl PituitaryRouter {
    pub fn new() -> Self {
        let mut amplification = std::collections::HashMap::new();

        // Default amplification factors
        for hormone in Hormone::all() {
            amplification.insert(hormone, 1.0);
        }

        Self { amplification }
    }

    /// Route signals with amplification
    pub fn route(&self, signals: Vec<(Hormone, f64)>) -> Vec<(Hormone, f64)> {
        signals
            .into_iter()
            .map(|(h, strength)| {
                let amp = self.amplification.get(&h).copied().unwrap_or(1.0);
                (h, strength * amp)
            })
            .collect()
    }

    /// Adjust amplification based on feedback
    pub fn adjust_amplification(&mut self, hormone: Hormone, factor: f64) {
        if let Some(amp) = self.amplification.get_mut(&hormone) {
            *amp = (*amp * factor).clamp(0.5, 2.0);
        }
    }
}

/// Performance Gland - produces stress and speed hormones
///
/// Secretes: Cortisol, Adrenaline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceGland {
    /// Fatigue accumulation
    pub fatigue: f64,
}

impl PerformanceGland {
    pub fn new() -> Self {
        Self { fatigue: 0.0 }
    }

    /// Update fatigue based on workload
    pub fn update_fatigue(&mut self, workload: f64, rest_time: f64) {
        // Accumulate fatigue from work
        self.fatigue += workload * 0.1;
        // Recover during rest
        self.fatigue -= rest_time * 0.05;
        self.fatigue = self.fatigue.clamp(0.0, 1.0);
    }
}

impl Gland for PerformanceGland {
    fn process(&self, stimulus: &Stimulus, state: &EndocrineState) -> Vec<(Hormone, f64)> {
        let mut secretions = Vec::new();

        match stimulus {
            Stimulus::TaskSuccess { difficulty, latency_ms } => {
                // Good performance → cortisol reward (eustress)
                let cortisol = difficulty * (1.0 - self.fatigue);
                secretions.push((Hormone::Cortisol, cortisol));

                // Fast → adrenaline boost
                if *latency_ms < 100 {
                    secretions.push((Hormone::Adrenaline, 0.3));
                }
            }
            Stimulus::TaskFailure { difficulty, error_severity } => {
                // Failure → stress cortisol (distress)
                let stress = error_severity * 0.5;
                secretions.push((Hormone::Cortisol, stress));
            }
            Stimulus::Urgency { deadline_pressure } => {
                // Urgency → adrenaline surge
                let current_adrenaline = state.levels.get(&Hormone::Adrenaline)
                    .map(|l| l.level)
                    .unwrap_or(0.5);

                // Apply negative feedback
                let feedback = if current_adrenaline > 0.7 { 0.5 } else { 1.0 };
                secretions.push((Hormone::Adrenaline, deadline_pressure * feedback));
            }
            _ => {}
        }

        secretions
    }

    fn produces(&self) -> Vec<Hormone> {
        vec![Hormone::Cortisol, Hormone::Adrenaline]
    }
}

/// Cooperation Gland - produces social bonding hormones
///
/// Secretes: Oxytocin
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CooperationGland {
    /// Social memory (trust accumulation)
    pub trust_bank: f64,
}

impl CooperationGland {
    pub fn new() -> Self {
        Self { trust_bank: 0.5 }
    }

    /// Update trust based on interaction outcomes
    pub fn update_trust(&mut self, success: bool) {
        if success {
            self.trust_bank = (self.trust_bank + 0.1).min(1.0);
        } else {
            self.trust_bank = (self.trust_bank - 0.2).max(0.0);
        }
    }
}

impl Gland for CooperationGland {
    fn process(&self, stimulus: &Stimulus, _state: &EndocrineState) -> Vec<(Hormone, f64)> {
        match stimulus {
            Stimulus::Collaboration { partner_count, success_rate } => {
                // More partners and higher success = more oxytocin
                let social_multiplier = 1.0 + (*partner_count as f64).ln().max(0.0) * 0.3;
                let oxytocin = success_rate * social_multiplier * self.trust_bank;
                vec![(Hormone::Oxytocin, oxytocin.min(1.0))]
            }
            _ => vec![],
        }
    }

    fn produces(&self) -> Vec<Hormone> {
        vec![Hormone::Oxytocin]
    }
}

/// Stability Gland - produces mood stabilization hormones
///
/// Secretes: Serotonin, Endorphins
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StabilityGland {
    /// Mood baseline
    pub mood_baseline: f64,
}

impl StabilityGland {
    pub fn new() -> Self {
        Self { mood_baseline: 0.5 }
    }
}

impl Gland for StabilityGland {
    fn process(&self, stimulus: &Stimulus, state: &EndocrineState) -> Vec<(Hormone, f64)> {
        let mut secretions = Vec::new();

        match stimulus {
            Stimulus::Consistency { days_stable } => {
                // Long-term stability boosts serotonin
                let stability_factor = (*days_stable as f64 / 30.0).min(1.0);
                secretions.push((Hormone::Serotonin, stability_factor * 0.5));
            }
            Stimulus::EthicalCompliance { constraint_difficulty } => {
                // Ethics satisfaction releases endorphins
                secretions.push((Hormone::Endorphins, *constraint_difficulty));
            }
            _ => {}
        }

        secretions
    }

    fn produces(&self) -> Vec<Hormone> {
        vec![Hormone::Serotonin, Hormone::Endorphins]
    }
}

/// Exploration Gland - produces novelty-seeking hormones
///
/// Secretes: Dopamine, Norepinephrine
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplorationGland {
    /// Recent novelty exposure (affects tolerance)
    pub novelty_tolerance: f64,
}

impl ExplorationGland {
    pub fn new() -> Self {
        Self { novelty_tolerance: 1.0 }
    }

    /// Update tolerance (habituation)
    pub fn habituate(&mut self, novelty_exposure: f64) {
        // Repeated novelty reduces response (tolerance)
        self.novelty_tolerance = (self.novelty_tolerance - novelty_exposure * 0.1).max(0.3);
    }

    /// Recover tolerance
    pub fn recover(&mut self, time: f64) {
        self.novelty_tolerance = (self.novelty_tolerance + time * 0.01).min(1.0);
    }
}

impl Gland for ExplorationGland {
    fn process(&self, stimulus: &Stimulus, _state: &EndocrineState) -> Vec<(Hormone, f64)> {
        let mut secretions = Vec::new();

        match stimulus {
            Stimulus::NovelSolution { novelty_score } => {
                // Novel solution → dopamine burst (with tolerance)
                let dopamine = novelty_score * self.novelty_tolerance;
                secretions.push((Hormone::Dopamine, dopamine));

                // Also triggers alertness
                secretions.push((Hormone::Norepinephrine, dopamine * 0.5));
            }
            Stimulus::Exploration { risk_taken } => {
                // Risk-taking → norepinephrine
                secretions.push((Hormone::Norepinephrine, *risk_taken));
            }
            _ => {}
        }

        secretions
    }

    fn produces(&self) -> Vec<Hormone> {
        vec![Hormone::Dopamine, Hormone::Norepinephrine]
    }
}

/// Development Gland - produces long-term growth hormones
///
/// Secretes: GrowthHormone
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevelopmentGland {
    /// Accumulated experience
    pub experience: u64,
}

impl DevelopmentGland {
    pub fn new() -> Self {
        Self { experience: 0 }
    }

    /// Add experience
    pub fn add_experience(&mut self, amount: u64) {
        self.experience = self.experience.saturating_add(amount);
    }
}

impl Gland for DevelopmentGland {
    fn process(&self, stimulus: &Stimulus, _state: &EndocrineState) -> Vec<(Hormone, f64)> {
        match stimulus {
            Stimulus::Consistency { days_stable } => {
                // Pulsatile release based on stability
                let pulse = if *days_stable % 7 == 0 { 0.3 } else { 0.05 };
                let base = (*days_stable as f64 / 365.0).min(0.5);
                vec![(Hormone::GrowthHormone, base + pulse)]
            }
            _ => vec![],
        }
    }

    fn produces(&self) -> Vec<Hormone> {
        vec![Hormone::GrowthHormone]
    }
}

/// Complete glandular system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlandularSystem {
    pub hypothalamus: HypothalamicController,
    pub pituitary: PituitaryRouter,
    pub performance: PerformanceGland,
    pub cooperation: CooperationGland,
    pub stability: StabilityGland,
    pub exploration: ExplorationGland,
    pub development: DevelopmentGland,
}

impl Default for GlandularSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl GlandularSystem {
    pub fn new() -> Self {
        Self {
            hypothalamus: HypothalamicController::new(),
            pituitary: PituitaryRouter::new(),
            performance: PerformanceGland::new(),
            cooperation: CooperationGland::new(),
            stability: StabilityGland::new(),
            exploration: ExplorationGland::new(),
            development: DevelopmentGland::new(),
        }
    }

    /// Process a stimulus through the entire system
    pub fn process(&mut self, stimulus: &Stimulus, state: &mut EndocrineState) {
        // 1. Hypothalamus evaluates stress and generates releasing signals
        self.hypothalamus.evaluate_stress(state);
        let releasing_signals = self.hypothalamus.releasing_signals(stimulus);

        // 2. Pituitary routes and amplifies signals
        let routed_signals = self.pituitary.route(releasing_signals);

        // 3. Collect secretions from all glands
        let mut all_secretions: Vec<(Hormone, f64)> = Vec::new();
        all_secretions.extend(self.performance.process(stimulus, state));
        all_secretions.extend(self.cooperation.process(stimulus, state));
        all_secretions.extend(self.stability.process(stimulus, state));
        all_secretions.extend(self.exploration.process(stimulus, state));
        all_secretions.extend(self.development.process(stimulus, state));

        // 4. Apply routed signals and secretions to state
        for (hormone, strength) in routed_signals.iter().chain(all_secretions.iter()) {
            // Apply negative feedback before secretion
            let feedback = state.apply_negative_feedback(*hormone);
            let adjusted = strength * feedback;

            if adjusted > 0.01 {
                state.secrete(*hormone, adjusted);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_success_triggers_cortisol() {
        let mut system = GlandularSystem::new();
        let mut state = EndocrineState::new([0u8; 32]);

        let initial = state.levels.get(&Hormone::Cortisol).map(|l| l.level).unwrap_or(0.5);

        system.process(
            &Stimulus::TaskSuccess { difficulty: 0.8, latency_ms: 50 },
            &mut state,
        );

        let after = state.levels.get(&Hormone::Cortisol).map(|l| l.level).unwrap_or(0.5);
        assert!(after > initial, "Task success should increase cortisol");
    }

    #[test]
    fn test_collaboration_triggers_oxytocin() {
        let mut system = GlandularSystem::new();
        let mut state = EndocrineState::new([0u8; 32]);

        let initial = state.levels.get(&Hormone::Oxytocin).map(|l| l.level).unwrap_or(0.5);

        system.process(
            &Stimulus::Collaboration { partner_count: 3, success_rate: 0.9 },
            &mut state,
        );

        let after = state.levels.get(&Hormone::Oxytocin).map(|l| l.level).unwrap_or(0.5);
        assert!(after > initial, "Collaboration should increase oxytocin");
    }

    #[test]
    fn test_novelty_triggers_dopamine() {
        let mut system = GlandularSystem::new();
        let mut state = EndocrineState::new([0u8; 32]);

        system.process(
            &Stimulus::NovelSolution { novelty_score: 0.9 },
            &mut state,
        );

        let dopamine = state.levels.get(&Hormone::Dopamine).map(|l| l.level).unwrap_or(0.5);
        let norepinephrine = state.levels.get(&Hormone::Norepinephrine).map(|l| l.level).unwrap_or(0.5);

        assert!(dopamine > 0.5, "Novelty should increase dopamine");
        assert!(norepinephrine > 0.5, "Novelty should also increase norepinephrine");
    }

    #[test]
    fn test_glandular_system_complete() {
        let system = GlandularSystem::new();

        // All glands should be initialized
        assert_eq!(system.hypothalamus.activity, 1.0);
        assert!(!system.pituitary.amplification.is_empty());
    }
}
