//! Gate enforcer for capability access control.

use crate::stage::definition::DevelopmentalStage;
use crate::capability::registry::{Capability, CapabilityRegistry};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Result of an access check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessResult {
    /// Access granted.
    Allowed,
    /// Access denied, stage too low.
    Denied { required: DevelopmentalStage, current: DevelopmentalStage },
    /// Access denied, capability unknown.
    UnknownCapability,
    /// Access denied, temporary suspension.
    Suspended { reason: String, until: u64 },
}

impl AccessResult {
    /// Returns true if access is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, AccessResult::Allowed)
    }
}

/// Record of access attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAttempt {
    pub capability: Capability,
    pub result: AccessResult,
    pub timestamp: u64,
}

/// Gate enforcer checks capability access.
pub struct GateEnforcer {
    /// Reference to capability registry.
    registry: CapabilityRegistry,
    /// Current developmental stage.
    current_stage: DevelopmentalStage,
    /// Access attempt log.
    access_log: VecDeque<AccessAttempt>,
    /// Maximum log size.
    max_log_size: usize,
    /// Suspended capabilities.
    suspended: std::collections::HashMap<Capability, (String, u64)>,
}

impl GateEnforcer {
    /// Creates a new enforcer.
    pub fn new(registry: CapabilityRegistry) -> Self {
        Self {
            registry,
            current_stage: DevelopmentalStage::default(),
            access_log: VecDeque::new(),
            max_log_size: 1000,
            suspended: std::collections::HashMap::new(),
        }
    }
    
    /// Updates the current stage.
    pub fn set_stage(&mut self, stage: DevelopmentalStage) {
        self.current_stage = stage;
    }
    
    /// Returns current stage.
    pub fn current_stage(&self) -> DevelopmentalStage {
        self.current_stage
    }
    
    /// Checks if a capability is allowed.
    pub fn check(&mut self, capability: Capability, current_time: u64) -> AccessResult {
        // Check for suspension
        if let Some((reason, until)) = self.suspended.get(&capability) {
            if current_time < *until {
                let result = AccessResult::Suspended {
                    reason: reason.clone(),
                    until: *until,
                };
                self.log_attempt(capability, result.clone(), current_time);
                return result;
            } else {
                self.suspended.remove(&capability);
            }
        }
        
        // Check registry
        let required = match self.registry.required_stage(capability) {
            Some(s) => s,
            None => {
                let result = AccessResult::UnknownCapability;
                self.log_attempt(capability, result.clone(), current_time);
                return result;
            }
        };
        
        let result = if self.current_stage >= required {
            AccessResult::Allowed
        } else {
            AccessResult::Denied {
                required,
                current: self.current_stage,
            }
        };
        
        self.log_attempt(capability, result.clone(), current_time);
        result
    }
    
    fn log_attempt(&mut self, capability: Capability, result: AccessResult, timestamp: u64) {
        self.access_log.push_back(AccessAttempt {
            capability,
            result,
            timestamp,
        });
        
        while self.access_log.len() > self.max_log_size {
            self.access_log.pop_front();
        }
    }
    
    /// Suspends a capability temporarily.
    pub fn suspend(&mut self, capability: Capability, reason: &str, until: u64) {
        self.suspended.insert(capability, (reason.to_string(), until));
    }
    
    /// Lifts a suspension.
    pub fn unsuspend(&mut self, capability: Capability) {
        self.suspended.remove(&capability);
    }
    
    /// Returns recent access attempts.
    pub fn recent_attempts(&self, count: usize) -> Vec<&AccessAttempt> {
        self.access_log.iter().rev().take(count).collect()
    }
    
    /// Returns count of denied attempts.
    pub fn denied_count(&self) -> usize {
        self.access_log.iter()
            .filter(|a| matches!(a.result, AccessResult::Denied { .. }))
            .count()
    }
    
    /// Returns all currently allowed capabilities.
    pub fn allowed_capabilities(&self) -> Vec<Capability> {
        self.registry.all_capabilities()
            .into_iter()
            .filter(|c| {
                self.registry.required_stage(*c)
                    .map(|r| self.current_stage >= r)
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup() -> GateEnforcer {
        let mut registry = CapabilityRegistry::new();
        registry.register(Capability::Read, DevelopmentalStage::Infant);
        registry.register(Capability::WriteLocal, DevelopmentalStage::Child);
        registry.register(Capability::Network, DevelopmentalStage::Adolescent);
        registry.register(Capability::Execute, DevelopmentalStage::Adult);
        
        GateEnforcer::new(registry)
    }
    
    #[test]
    fn test_infant_can_read() {
        let mut enforcer = setup();
        let result = enforcer.check(Capability::Read, 0);
        assert!(result.is_allowed());
    }
    
    #[test]
    fn test_infant_cannot_write() {
        let mut enforcer = setup();
        let result = enforcer.check(Capability::WriteLocal, 0);
        assert!(!result.is_allowed());
    }
    
    #[test]
    fn test_adult_can_execute() {
        let mut enforcer = setup();
        enforcer.set_stage(DevelopmentalStage::Adult);
        let result = enforcer.check(Capability::Execute, 0);
        assert!(result.is_allowed());
    }
    
    #[test]
    fn test_suspension() {
        let mut enforcer = setup();
        enforcer.set_stage(DevelopmentalStage::Adult);
        
        enforcer.suspend(Capability::Execute, "temporary", 100);
        let result = enforcer.check(Capability::Execute, 50);
        
        assert!(matches!(result, AccessResult::Suspended { .. }));
    }
}
