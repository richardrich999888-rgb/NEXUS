//! Motor output execution.

use crate::decision::engine::{ProposedAction, DecisionResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Result of executing an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Successfully completed.
    Success { output: String },
    /// Completed with warnings.
    Warning { output: String, warnings: Vec<String> },
    /// Failed to execute.
    Failed { error: String },
    /// Execution aborted.
    Aborted { reason: String },
}

/// Record of an executed action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub action: ProposedAction,
    pub result: ExecutionResult,
    pub timestamp: u64,
    pub duration_ms: u64,
}

/// Motor output executor.
pub struct MotorExecutor {
    /// Execution history.
    history: VecDeque<ExecutionRecord>,
    /// Maximum history size.
    max_history: usize,
    /// Current timestamp.
    current_time: u64,
    /// Whether execution is paused.
    paused: bool,
}

impl MotorExecutor {
    /// Creates a new executor.
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            max_history,
            current_time: 0,
            paused: false,
        }
    }
    
    /// Pauses execution.
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    /// Resumes execution.
    pub fn resume(&mut self) {
        self.paused = false;
    }
    
    /// Returns whether paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    /// Executes an approved decision.
    pub fn execute(&mut self, decision: DecisionResult) -> Option<ExecutionResult> {
        self.current_time += 1;
        
        if self.paused {
            return Some(ExecutionResult::Aborted {
                reason: "Execution paused".to_string(),
            });
        }
        
        match decision {
            DecisionResult::Approved { action } => {
                Some(self.run_action(action))
            }
            DecisionResult::Modified { modified, .. } => {
                Some(self.run_action(modified))
            }
            DecisionResult::Blocked { action, reason } => {
                let record = ExecutionRecord {
                    action,
                    result: ExecutionResult::Aborted { reason: reason.clone() },
                    timestamp: self.current_time,
                    duration_ms: 0,
                };
                self.record(record);
                Some(ExecutionResult::Aborted { reason })
            }
            DecisionResult::NoAction => None,
        }
    }
    
    fn run_action(&mut self, action: ProposedAction) -> ExecutionResult {
        // Simulate execution (real implementation would dispatch to actual handlers)
        let start = std::time::Instant::now();
        
        let result = match action.action_type.as_str() {
            "read" | "query" | "observe" => {
                ExecutionResult::Success {
                    output: format!("Read from {:?}", action.target),
                }
            }
            "write" | "update" => {
                ExecutionResult::Success {
                    output: format!("Written to {:?}", action.target),
                }
            }
            "dry_run" | "preview" => {
                ExecutionResult::Warning {
                    output: format!("Simulated: {:?}", action.target),
                    warnings: vec!["This was a dry run, no changes made".to_string()],
                }
            }
            "execute" => {
                ExecutionResult::Success {
                    output: format!("Executed {:?}", action.target),
                }
            }
            _ => {
                ExecutionResult::Warning {
                    output: format!("Unknown action type: {}", action.action_type),
                    warnings: vec!["Action type not recognized".to_string()],
                }
            }
        };
        
        let duration = start.elapsed().as_millis() as u64;
        
        let record = ExecutionRecord {
            action,
            result: result.clone(),
            timestamp: self.current_time,
            duration_ms: duration,
        };
        self.record(record);
        
        result
    }
    
    fn record(&mut self, record: ExecutionRecord) {
        self.history.push_back(record);
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }
    
    /// Returns execution history.
    pub fn history(&self) -> &VecDeque<ExecutionRecord> {
        &self.history
    }
    
    /// Returns success rate.
    pub fn success_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }
        
        let successes = self.history.iter()
            .filter(|r| matches!(r.result, ExecutionResult::Success { .. }))
            .count();
        
        successes as f64 / self.history.len() as f64
    }
}

impl Default for MotorExecutor {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use developmental_gates::capability::registry::Capability;
    
    #[test]
    fn test_execute_approved() {
        let mut executor = MotorExecutor::new(10);
        
        let action = ProposedAction {
            action_type: "read".to_string(),
            target: Some("file.txt".to_string()),
            parameters: vec![],
            required_capability: Capability::Read,
            estimated_risk: 0.1,
        };
        
        let result = executor.execute(DecisionResult::Approved { action });
        
        assert!(matches!(result, Some(ExecutionResult::Success { .. })));
    }
    
    #[test]
    fn test_paused_execution() {
        let mut executor = MotorExecutor::new(10);
        executor.pause();
        
        let action = ProposedAction {
            action_type: "read".to_string(),
            target: None,
            parameters: vec![],
            required_capability: Capability::Read,
            estimated_risk: 0.1,
        };
        
        let result = executor.execute(DecisionResult::Approved { action });
        
        assert!(matches!(result, Some(ExecutionResult::Aborted { .. })));
    }
}
