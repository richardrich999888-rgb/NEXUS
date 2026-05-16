// CRDT-backed USO - Universal State Objects with automatic merge
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh

use causalux_v2::{GCounter, PNCounter, LWWRegister, ORSet, LWWMap, RGAText};
use nexus_pcu::{ContentHash, PrincipalId, SyncPolicy, AccessPolicy};
use serde::{Deserialize, Serialize};

/// Type of CRDT backing this USO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum USOType {
    /// Raw bytes with last-writer-wins
    Raw,
    /// JSON with last-writer-wins
    Json,
    /// Grow-only counter
    Counter,
    /// Positive-negative counter
    PNCounter,
    /// Last-writer-wins register
    Register,
    /// Observed-remove set
    Set,
    /// Last-writer-wins map
    Map,
    /// Collaborative text (RGA)
    Text,
}

/// USO with automatic CRDT-based merge
#[derive(Clone, Debug)]
pub struct CrdtUSO {
    /// Content-addressed ID
    pub id: ContentHash,
    
    /// USO type determines merge behavior
    pub uso_type: USOType,
    
    /// The CRDT state
    pub state: CrdtState,
    
    /// Node ID for CRDT operations
    node_id: String,
    
    /// Access policy
    pub access: AccessPolicy,
    
    /// Sync policy
    pub sync: SyncPolicy,
}

/// Underlying CRDT state
#[derive(Clone, Debug)]
pub enum CrdtState {
    Raw(Vec<u8>),
    Json(serde_json::Value),
    Counter(GCounter),
    PNCounter(PNCounter),
    Register(LWWRegister<serde_json::Value>),
    Set(ORSet<String>),
    Map(LWWMap<String, serde_json::Value>),
    Text(RGAText),
}

impl CrdtUSO {
    /// Create raw bytes USO
    pub fn raw(data: Vec<u8>, node_id: impl Into<String>, owner: PrincipalId) -> Self {
        let id = ContentHash::compute(&data);
        let node_id = node_id.into();
        
        CrdtUSO {
            id,
            uso_type: USOType::Raw,
            state: CrdtState::Raw(data),
            node_id,
            access: AccessPolicy::owner_only(owner),
            sync: SyncPolicy::default(),
        }
    }

    /// Create counter USO
    pub fn counter(node_id: impl Into<String>, owner: PrincipalId) -> Self {
        let node_id = node_id.into();
        let counter = GCounter::new(node_id.clone());
        let id = ContentHash::compute(b"counter");
        
        CrdtUSO {
            id,
            uso_type: USOType::Counter,
            state: CrdtState::Counter(counter),
            node_id,
            access: AccessPolicy::owner_only(owner),
            sync: SyncPolicy::default(),
        }
    }

    /// Create PN counter USO (supports increment and decrement)
    pub fn pn_counter(node_id: impl Into<String>, owner: PrincipalId) -> Self {
        let node_id = node_id.into();
        let counter = PNCounter::new(node_id.clone());
        let id = ContentHash::compute(b"pn_counter");
        
        CrdtUSO {
            id,
            uso_type: USOType::PNCounter,
            state: CrdtState::PNCounter(counter),
            node_id,
            access: AccessPolicy::owner_only(owner),
            sync: SyncPolicy::default(),
        }
    }

    /// Create set USO
    pub fn set(node_id: impl Into<String>, owner: PrincipalId) -> Self {
        let node_id = node_id.into();
        let set = ORSet::new(node_id.clone());
        let id = ContentHash::compute(b"set");
        
        CrdtUSO {
            id,
            uso_type: USOType::Set,
            state: CrdtState::Set(set),
            node_id,
            access: AccessPolicy::owner_only(owner),
            sync: SyncPolicy::default(),
        }
    }

    /// Create text USO for collaborative editing
    pub fn text(node_id: impl Into<String>, owner: PrincipalId) -> Self {
        let node_id = node_id.into();
        let text = RGAText::new(node_id.clone());
        let id = ContentHash::compute(b"text");
        
        CrdtUSO {
            id,
            uso_type: USOType::Text,
            state: CrdtState::Text(text),
            node_id,
            access: AccessPolicy::owner_only(owner),
            sync: SyncPolicy::default(),
        }
    }

    /// Increment counter (only works for Counter/PNCounter types)
    pub fn increment(&mut self, amount: u64) -> Result<(), CrdtError> {
        match &mut self.state {
            CrdtState::Counter(c) => {
                c.increment(amount);
                self.update_id();
                Ok(())
            }
            CrdtState::PNCounter(c) => {
                c.increment(amount);
                self.update_id();
                Ok(())
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Decrement counter (only works for PNCounter type)
    pub fn decrement(&mut self, amount: u64) -> Result<(), CrdtError> {
        match &mut self.state {
            CrdtState::PNCounter(c) => {
                c.decrement(amount);
                self.update_id();
                Ok(())
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Get counter value
    pub fn counter_value(&self) -> Result<i64, CrdtError> {
        match &self.state {
            CrdtState::Counter(c) => Ok(c.value() as i64),
            CrdtState::PNCounter(c) => Ok(c.value()),
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Add element to set
    pub fn add_to_set(&mut self, element: impl Into<String>) -> Result<(), CrdtError> {
        match &mut self.state {
            CrdtState::Set(s) => {
                s.add(element.into());
                self.update_id();
                Ok(())
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Remove element from set
    pub fn remove_from_set(&mut self, element: &str) -> Result<(), CrdtError> {
        match &mut self.state {
            CrdtState::Set(s) => {
                s.remove(&element.to_string());
                self.update_id();
                Ok(())
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Check if set contains element
    pub fn set_contains(&self, element: &str) -> Result<bool, CrdtError> {
        match &self.state {
            CrdtState::Set(s) => Ok(s.contains(&element.to_string())),
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Get all elements in set
    pub fn set_elements(&self) -> Result<Vec<String>, CrdtError> {
        match &self.state {
            CrdtState::Set(s) => {
                let mut elements: Vec<String> = s.elements().into_iter().collect();
                elements.sort();
                Ok(elements)
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Insert character in text
    pub fn insert_text(&mut self, position: usize, character: char) -> Result<(), CrdtError> {
        match &mut self.state {
            CrdtState::Text(t) => {
                t.insert(position, character);
                self.update_id();
                Ok(())
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Get text content
    pub fn get_text(&self) -> Result<String, CrdtError> {
        match &self.state {
            CrdtState::Text(t) => Ok(t.to_string()),
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Merge with another CRDT USO of the same type
    pub fn merge(&mut self, other: &CrdtUSO) -> Result<(), CrdtError> {
        match (&mut self.state, &other.state) {
            (CrdtState::Counter(a), CrdtState::Counter(b)) => {
                a.merge(b);
                self.update_id();
                Ok(())
            }
            (CrdtState::PNCounter(a), CrdtState::PNCounter(b)) => {
                a.merge(b);
                self.update_id();
                Ok(())
            }
            (CrdtState::Set(a), CrdtState::Set(b)) => {
                a.merge(b);
                self.update_id();
                Ok(())
            }
            _ => Err(CrdtError::TypeMismatch),
        }
    }

    /// Update content-addressed ID after mutation
    fn update_id(&mut self) {
        let bytes = match &self.state {
            CrdtState::Raw(data) => data.clone(),
            CrdtState::Json(v) => v.to_string().into_bytes(),
            CrdtState::Counter(c) => c.value().to_le_bytes().to_vec(),
            CrdtState::PNCounter(c) => c.value().to_le_bytes().to_vec(),
            CrdtState::Register(r) => serde_json::to_vec(r).unwrap_or_default(),
            CrdtState::Set(s) => serde_json::to_vec(s).unwrap_or_default(),
            CrdtState::Map(m) => serde_json::to_vec(m).unwrap_or_default(),
            CrdtState::Text(t) => t.to_string().into_bytes(),
        };
        self.id = ContentHash::compute(&bytes);
    }
}

/// CRDT operation errors
#[derive(Debug, Clone)]
pub enum CrdtError {
    TypeMismatch,
    InvalidOperation,
}

impl std::fmt::Display for CrdtError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CrdtError::TypeMismatch => write!(f, "CRDT type mismatch"),
            CrdtError::InvalidOperation => write!(f, "Invalid CRDT operation"),
        }
    }
}

impl std::error::Error for CrdtError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_uso() {
        let owner = PrincipalId::generate();
        let mut uso = CrdtUSO::counter("node1", owner);
        
        uso.increment(5).unwrap();
        uso.increment(3).unwrap();
        
        assert_eq!(uso.counter_value().unwrap(), 8);
    }

    #[test]
    fn test_pn_counter_uso() {
        let owner = PrincipalId::generate();
        let mut uso = CrdtUSO::pn_counter("node1", owner);
        
        uso.increment(10).unwrap();
        uso.decrement(3).unwrap();
        
        assert_eq!(uso.counter_value().unwrap(), 7);
    }

    #[test]
    fn test_set_uso() {
        let owner = PrincipalId::generate();
        let mut uso = CrdtUSO::set("node1", owner);
        
        uso.add_to_set("apple").unwrap();
        uso.add_to_set("banana").unwrap();
        
        assert!(uso.set_contains("apple").unwrap());
        assert!(uso.set_contains("banana").unwrap());
        assert!(!uso.set_contains("cherry").unwrap());
        
        uso.remove_from_set("apple").unwrap();
        assert!(!uso.set_contains("apple").unwrap());
    }

    #[test]
    fn test_text_uso() {
        let owner = PrincipalId::generate();
        let mut uso = CrdtUSO::text("node1", owner);
        
        uso.insert_text(0, 'H').unwrap();
        uso.insert_text(1, 'i').unwrap();
        
        assert_eq!(uso.get_text().unwrap(), "Hi");
    }

    #[test]
    fn test_counter_merge() {
        let owner = PrincipalId::generate();
        
        let mut uso1 = CrdtUSO::counter("node1", owner);
        uso1.increment(5).unwrap();
        
        let mut uso2 = CrdtUSO::counter("node2", owner);
        uso2.increment(3).unwrap();
        
        uso1.merge(&uso2).unwrap();
        
        // After merge, should have sum from both nodes
        assert_eq!(uso1.counter_value().unwrap(), 8);
    }
}
