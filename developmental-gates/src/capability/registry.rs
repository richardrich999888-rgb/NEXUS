//! Capability registry.

use crate::stage::definition::DevelopmentalStage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core capabilities that can be gated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Perception
    Read,
    Observe,
    Query,
    
    // Local actions
    WriteLocal,
    ComputeLocal,
    
    // Extended actions
    Network,
    FileSystem,
    
    // Autonomous actions
    Execute,
    Spawn,
    Modify,
    
    // Meta actions
    SelfModify,
    Delegate,
    Mentor,
    
    // Custom capability by ID
    Custom(u32),
}

impl Capability {
    /// Returns a human-readable name.
    pub fn name(&self) -> String {
        match self {
            Capability::Read => "read".to_string(),
            Capability::Observe => "observe".to_string(),
            Capability::Query => "query".to_string(),
            Capability::WriteLocal => "write_local".to_string(),
            Capability::ComputeLocal => "compute_local".to_string(),
            Capability::Network => "network".to_string(),
            Capability::FileSystem => "filesystem".to_string(),
            Capability::Execute => "execute".to_string(),
            Capability::Spawn => "spawn".to_string(),
            Capability::Modify => "modify".to_string(),
            Capability::SelfModify => "self_modify".to_string(),
            Capability::Delegate => "delegate".to_string(),
            Capability::Mentor => "mentor".to_string(),
            Capability::Custom(id) => format!("custom_{}", id),
        }
    }
    
    /// Returns default required stage.
    pub fn default_stage(&self) -> DevelopmentalStage {
        match self {
            // Infant can only perceive
            Capability::Read | Capability::Observe | Capability::Query => DevelopmentalStage::Infant,
            
            // Child can do local actions
            Capability::WriteLocal | Capability::ComputeLocal => DevelopmentalStage::Child,
            
            // Adolescent can do network/filesystem
            Capability::Network | Capability::FileSystem => DevelopmentalStage::Adolescent,
            
            // Adult can execute and spawn
            Capability::Execute | Capability::Spawn | Capability::Modify => DevelopmentalStage::Adult,
            
            // Elder can self-modify and mentor
            Capability::SelfModify | Capability::Delegate | Capability::Mentor => DevelopmentalStage::Elder,
            
            // Custom default to Adult
            Capability::Custom(_) => DevelopmentalStage::Adult,
        }
    }
}

/// Registry mapping capabilities to required stages.
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    /// Capability → required stage.
    requirements: HashMap<Capability, DevelopmentalStage>,
    /// Descriptions.
    descriptions: HashMap<Capability, String>,
}

impl CapabilityRegistry {
    /// Creates a new registry with default capabilities.
    pub fn new() -> Self {
        let mut registry = Self {
            requirements: HashMap::new(),
            descriptions: HashMap::new(),
        };
        
        // Register all standard capabilities
        let standard = [
            Capability::Read,
            Capability::Observe,
            Capability::Query,
            Capability::WriteLocal,
            Capability::ComputeLocal,
            Capability::Network,
            Capability::FileSystem,
            Capability::Execute,
            Capability::Spawn,
            Capability::Modify,
            Capability::SelfModify,
            Capability::Delegate,
            Capability::Mentor,
        ];
        
        for cap in standard {
            registry.register(cap, cap.default_stage());
        }
        
        registry
    }
    
    /// Registers a capability with its required stage.
    pub fn register(&mut self, capability: Capability, stage: DevelopmentalStage) {
        self.requirements.insert(capability, stage);
    }
    
    /// Registers with description.
    pub fn register_with_description(&mut self, capability: Capability, stage: DevelopmentalStage, description: &str) {
        self.requirements.insert(capability, stage);
        self.descriptions.insert(capability, description.to_string());
    }
    
    /// Returns required stage for a capability.
    pub fn required_stage(&self, capability: Capability) -> Option<DevelopmentalStage> {
        self.requirements.get(&capability).copied()
    }
    
    /// Returns all registered capabilities.
    pub fn all_capabilities(&self) -> Vec<Capability> {
        self.requirements.keys().copied().collect()
    }
    
    /// Returns capabilities at or below a stage.
    pub fn capabilities_at_stage(&self, stage: DevelopmentalStage) -> Vec<Capability> {
        self.requirements
            .iter()
            .filter(|(_, &s)| s <= stage)
            .map(|(&c, _)| c)
            .collect()
    }
    
    /// Returns capabilities that unlock at a specific stage.
    pub fn capabilities_unlocked_at(&self, stage: DevelopmentalStage) -> Vec<Capability> {
        self.requirements
            .iter()
            .filter(|(_, &s)| s == stage)
            .map(|(&c, _)| c)
            .collect()
    }
    
    /// Returns description of a capability.
    pub fn description(&self, capability: Capability) -> Option<&str> {
        self.descriptions.get(&capability).map(|s| s.as_str())
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_stages() {
        let registry = CapabilityRegistry::new();
        
        assert_eq!(registry.required_stage(Capability::Read), Some(DevelopmentalStage::Infant));
        assert_eq!(registry.required_stage(Capability::Execute), Some(DevelopmentalStage::Adult));
    }
    
    #[test]
    fn test_capabilities_at_stage() {
        let registry = CapabilityRegistry::new();
        
        let infant_caps = registry.capabilities_at_stage(DevelopmentalStage::Infant);
        assert!(infant_caps.contains(&Capability::Read));
        assert!(!infant_caps.contains(&Capability::Execute));
        
        let adult_caps = registry.capabilities_at_stage(DevelopmentalStage::Adult);
        assert!(adult_caps.contains(&Capability::Execute));
    }
    
    #[test]
    fn test_custom_registration() {
        let mut registry = CapabilityRegistry::new();
        registry.register(Capability::Custom(42), DevelopmentalStage::Child);
        
        assert_eq!(registry.required_stage(Capability::Custom(42)), Some(DevelopmentalStage::Child));
    }
}
