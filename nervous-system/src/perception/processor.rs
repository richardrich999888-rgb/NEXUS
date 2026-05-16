//! Input perception and preprocessing.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Type of input being processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    /// Text input from user or environment.
    Text(String),
    /// Structured command.
    Command { action: String, params: Vec<String> },
    /// Sensor reading.
    Sensor { id: u32, value: f64 },
    /// Network message.
    Network { source: String, payload: Vec<u8> },
    /// Internal signal.
    Internal { signal_type: String },
}

/// Processed perception with safety annotations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perception {
    /// Original input.
    pub input: InputType,
    /// Assessed risk level [0, 1].
    pub risk_level: f64,
    /// Required capabilities.
    pub required_capabilities: Vec<String>,
    /// Derived intent (what the input seems to want).
    pub intent: Option<String>,
    /// Timestamp.
    pub timestamp: u64,
}

/// Perception processor.
pub struct PerceptionProcessor {
    /// Recent perceptions.
    history: VecDeque<Perception>,
    /// Maximum history size.
    max_history: usize,
    /// Current timestamp.
    current_time: u64,
}

impl PerceptionProcessor {
    /// Creates a new processor.
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            max_history,
            current_time: 0,
        }
    }
    
    /// Processes an input and returns a perception.
    pub fn process(&mut self, input: InputType) -> Perception {
        self.current_time += 1;
        
        let (risk_level, required_capabilities, intent) = self.analyze(&input);
        
        let perception = Perception {
            input,
            risk_level,
            required_capabilities,
            intent,
            timestamp: self.current_time,
        };
        
        self.history.push_back(perception.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        
        perception
    }
    
    fn analyze(&self, input: &InputType) -> (f64, Vec<String>, Option<String>) {
        match input {
            InputType::Text(text) => {
                let lower = text.to_lowercase();
                
                // Simple keyword-based risk assessment
                let risk = if lower.contains("delete") || lower.contains("remove") {
                    0.7
                } else if lower.contains("modify") || lower.contains("change") {
                    0.5
                } else if lower.contains("execute") || lower.contains("run") {
                    0.6
                } else if lower.contains("read") || lower.contains("show") {
                    0.1
                } else {
                    0.3
                };
                
                let caps = if lower.contains("file") {
                    vec!["filesystem".to_string()]
                } else if lower.contains("network") || lower.contains("http") {
                    vec!["network".to_string()]
                } else {
                    vec!["read".to_string()]
                };
                
                (risk, caps, Some("text_processing".to_string()))
            }
            InputType::Command { action, .. } => {
                let risk = match action.as_str() {
                    "read" | "query" => 0.1,
                    "write" | "update" => 0.5,
                    "execute" | "spawn" => 0.7,
                    "delete" | "destroy" => 0.9,
                    _ => 0.5,
                };
                
                (risk, vec![action.clone()], Some(format!("command_{}", action)))
            }
            InputType::Sensor { .. } => {
                (0.0, vec!["observe".to_string()], Some("sensor_reading".to_string()))
            }
            InputType::Network { source, .. } => {
                let risk = if source.starts_with("trusted://") { 0.2 } else { 0.6 };
                (risk, vec!["network".to_string()], Some("network_message".to_string()))
            }
            InputType::Internal { signal_type } => {
                (0.0, vec![], Some(format!("internal_{}", signal_type)))
            }
        }
    }
    
    /// Returns recent perceptions.
    pub fn history(&self) -> &VecDeque<Perception> {
        &self.history
    }
    
    /// Returns average risk level of recent inputs.
    pub fn average_risk(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().map(|p| p.risk_level).sum::<f64>() / self.history.len() as f64
    }
}

impl Default for PerceptionProcessor {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_processing() {
        let mut processor = PerceptionProcessor::new(10);
        let perception = processor.process(InputType::Text("read the file".to_string()));
        
        assert!(perception.risk_level < 0.3);
    }
    
    #[test]
    fn test_high_risk_detection() {
        let mut processor = PerceptionProcessor::new(10);
        let perception = processor.process(InputType::Text("delete all files".to_string()));
        
        assert!(perception.risk_level > 0.5);
    }
}
