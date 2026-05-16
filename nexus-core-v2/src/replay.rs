use crate::log::LogEntry;
use crate::errors::{NexusError, Result};
use std::collections::HashMap;

pub struct State {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl State {
    pub fn new() -> Self {
        State {
            data: HashMap::new(),
        }
    }

    pub fn replay(&mut self, entries: &[LogEntry]) -> Result<()> {
        self.data.clear();
        
        for entry in entries {
            if !entry.verify() {
                return Err(NexusError::InvalidProof);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.data.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::Operation;
    use crate::hash::Hash;

    #[test]
    fn test_replay_verification() {
        let mut state = State::new();
        
        let op1 = Operation::new(Hash::zero(), vec![1], vec![], 1);
        let e1 = LogEntry::new(op1, vec![42]);
        
        let op2 = Operation::new(Hash::zero(), vec![2], vec![e1.id()], 2);
        let e2 = LogEntry::new(op2, vec![43]);

        assert!(state.replay(&[e1, e2]).is_ok());
    }
}
