//! CausalAtom - The Universal Primitive
//! 
//! A CausalAtom is the fundamental building block of CAUSALUX.
//! Every data type (documents, counters, tensors) can be expressed as atoms.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use crate::version_vector::VersionVector;
use crate::content_address::ContentAddress;

/// The value contained in an atom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomValue {
    /// Null/empty value
    Null,
    /// Boolean
    Bool(bool),
    /// Integer
    Int(i64),
    /// Float
    Float(f64),
    /// String
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// List of atoms
    List(Vec<CausalAtom>),
    /// Map of atoms
    Map(Vec<(String, CausalAtom)>),
    /// Reference to another atom
    Ref(String),
}

impl AtomValue {
    /// Get type name
    pub fn type_name(&self) -> &str {
        match self {
            AtomValue::Null => "null",
            AtomValue::Bool(_) => "bool",
            AtomValue::Int(_) => "int",
            AtomValue::Float(_) => "float",
            AtomValue::String(_) => "string",
            AtomValue::Bytes(_) => "bytes",
            AtomValue::List(_) => "list",
            AtomValue::Map(_) => "map",
            AtomValue::Ref(_) => "ref",
        }
    }

    /// Check if null
    pub fn is_null(&self) -> bool {
        matches!(self, AtomValue::Null)
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AtomValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            AtomValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            AtomValue::Float(f) => Some(*f),
            AtomValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
}

/// Metadata for an atom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomMeta {
    /// Content address (hash-based ID)
    pub id: String,
    /// Version vector
    pub version: VersionVector,
    /// Creator node
    pub creator: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Dependencies (atoms this depends on)
    pub dependencies: BTreeSet<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl AtomMeta {
    /// Create new metadata
    pub fn new(creator: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        Self {
            id: String::new(), // Set after content hash
            version: VersionVector::new(),
            creator,
            created_at: now,
            modified_at: now,
            dependencies: BTreeSet::new(),
            tags: Vec::new(),
        }
    }

    /// Update modification time
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
    }
}

/// CausalAtom - The Universal Primitive
/// 
/// Contains:
/// - A value (any type)
/// - Metadata (version, creator, etc.)
/// - Provenance (causal history)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalAtom {
    /// The value
    pub value: AtomValue,
    /// Metadata
    pub meta: AtomMeta,
}

impl CausalAtom {
    /// Create a new atom
    pub fn new(value: AtomValue, creator: String) -> Self {
        let mut meta = AtomMeta::new(creator.clone());
        meta.version.increment(&creator);
        
        let mut atom = Self { value, meta };
        atom.update_id();
        atom
    }

    /// Create null atom
    pub fn null(creator: String) -> Self {
        Self::new(AtomValue::Null, creator)
    }

    /// Create bool atom
    pub fn bool(value: bool, creator: String) -> Self {
        Self::new(AtomValue::Bool(value), creator)
    }

    /// Create int atom
    pub fn int(value: i64, creator: String) -> Self {
        Self::new(AtomValue::Int(value), creator)
    }

    /// Create float atom
    pub fn float(value: f64, creator: String) -> Self {
        Self::new(AtomValue::Float(value), creator)
    }

    /// Create string atom
    pub fn string(value: impl Into<String>, creator: String) -> Self {
        Self::new(AtomValue::String(value.into()), creator)
    }

    /// Create bytes atom
    pub fn bytes(value: Vec<u8>, creator: String) -> Self {
        Self::new(AtomValue::Bytes(value), creator)
    }

    /// Create list atom
    pub fn list(values: Vec<CausalAtom>, creator: String) -> Self {
        Self::new(AtomValue::List(values), creator)
    }

    /// Create map atom
    pub fn map(entries: Vec<(String, CausalAtom)>, creator: String) -> Self {
        Self::new(AtomValue::Map(entries), creator)
    }

    /// Create reference atom
    pub fn reference(target_id: impl Into<String>, creator: String) -> Self {
        Self::new(AtomValue::Ref(target_id.into()), creator)
    }

    /// Update content ID based on value
    fn update_id(&mut self) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        // Hash the serialized value
        let value_json = serde_json::to_string(&self.value).unwrap_or_default();
        hasher.update(value_json.as_bytes());
        hasher.update(self.meta.creator.as_bytes());
        hasher.update(self.meta.created_at.to_le_bytes());
        
        self.meta.id = format!("atom_{}", &format!("{:x}", hasher.finalize())[..16]);
    }

    /// Get atom ID
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get value
    pub fn value(&self) -> &AtomValue {
        &self.value
    }

    /// Get value type
    pub fn value_type(&self) -> &str {
        self.value.type_name()
    }

    /// Update value
    pub fn set_value(&mut self, value: AtomValue) {
        self.value = value;
        self.meta.touch();
        self.meta.version.increment(&self.meta.creator);
        self.update_id();
    }

    /// Add dependency
    pub fn add_dependency(&mut self, atom_id: impl Into<String>) {
        self.meta.dependencies.insert(atom_id.into());
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.meta.tags.push(tag.into());
    }

    /// Check if this atom is newer than another
    pub fn is_newer_than(&self, other: &CausalAtom) -> bool {
        // Compare by total operations - more operations = newer
        self.meta.version.total_operations() > other.meta.version.total_operations()
    }

    /// Merge with another atom (CRDT merge)
    pub fn merge(&self, other: &CausalAtom) -> CausalAtom {
        // Use the one with the later timestamp (LWW semantics)
        if self.meta.modified_at >= other.meta.modified_at {
            let mut result = self.clone();
            result.meta.version = self.meta.version.merge(&other.meta.version);
            result.meta.dependencies.extend(other.meta.dependencies.iter().cloned());
            result
        } else {
            let mut result = other.clone();
            result.meta.version = self.meta.version.merge(&other.meta.version);
            result.meta.dependencies.extend(self.meta.dependencies.iter().cloned());
            result
        }
    }

    /// Deep merge for nested structures
    pub fn deep_merge(&self, other: &CausalAtom) -> CausalAtom {
        match (&self.value, &other.value) {
            // Merge lists by concatenation
            (AtomValue::List(a), AtomValue::List(b)) => {
                let mut merged = a.clone();
                for item in b {
                    if !merged.iter().any(|x| x.id() == item.id()) {
                        merged.push(item.clone());
                    } else {
                        // Recursively merge existing items
                        for existing in merged.iter_mut() {
                            if existing.id() == item.id() {
                                *existing = existing.deep_merge(item);
                            }
                        }
                    }
                }
                let mut result = self.clone();
                result.value = AtomValue::List(merged);
                result
            }
            
            // Merge maps by key
            (AtomValue::Map(a), AtomValue::Map(b)) => {
                let mut merged: Vec<(String, CausalAtom)> = a.clone();
                for (key, value) in b {
                    if let Some((_, existing)) = merged.iter_mut().find(|(k, _)| k == key) {
                        *existing = existing.deep_merge(value);
                    } else {
                        merged.push((key.clone(), value.clone()));
                    }
                }
                let mut result = self.clone();
                result.value = AtomValue::Map(merged);
                result
            }
            
            // For other types, use LWW
            _ => self.merge(other),
        }
    }
}

/// Document as atoms
pub type AtomDocument = CausalAtom; // With AtomValue::List containing character atoms

/// Counter as atom
impl CausalAtom {
    /// Create a counter atom
    pub fn counter(initial: i64, creator: String) -> Self {
        Self::int(initial, creator)
    }

    /// Increment counter (creates new atom)
    pub fn increment(&self, amount: i64, creator: &str) -> Self {
        let current = self.value.as_int().unwrap_or(0);
        let mut new_atom = Self::int(current + amount, creator.to_string());
        new_atom.add_dependency(self.id());
        new_atom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_creation() {
        let atom = CausalAtom::string("hello", "node1".to_string());
        assert_eq!(atom.value_type(), "string");
        assert_eq!(atom.value.as_string(), Some("hello"));
        assert!(!atom.id().is_empty());
    }

    #[test]
    fn test_atom_update() {
        let mut atom = CausalAtom::int(42, "node1".to_string());
        let original_id = atom.id().to_string();
        
        atom.set_value(AtomValue::Int(100));
        
        assert_ne!(atom.id(), &original_id);
        assert_eq!(atom.value.as_int(), Some(100));
    }

    #[test]
    fn test_atom_merge() {
        let atom1 = CausalAtom::string("hello", "node1".to_string());
        
        // Simulate delay
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let atom2 = CausalAtom::string("world", "node2".to_string());
        
        let merged = atom1.merge(&atom2);
        
        // Should have the later one's value
        assert_eq!(merged.value.as_string(), Some("world"));
    }

    #[test]
    fn test_list_atom() {
        let items = vec![
            CausalAtom::int(1, "node1".to_string()),
            CausalAtom::int(2, "node1".to_string()),
            CausalAtom::int(3, "node1".to_string()),
        ];
        
        let list = CausalAtom::list(items, "node1".to_string());
        
        match &list.value {
            AtomValue::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_map_atom() {
        let entries = vec![
            ("name".to_string(), CausalAtom::string("Alice", "node1".to_string())),
            ("age".to_string(), CausalAtom::int(30, "node1".to_string())),
        ];
        
        let map = CausalAtom::map(entries, "node1".to_string());
        
        match &map.value {
            AtomValue::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_counter_atom() {
        let counter = CausalAtom::counter(0, "node1".to_string());
        let incremented = counter.increment(5, "node1");
        
        assert_eq!(incremented.value.as_int(), Some(5));
        assert!(incremented.meta.dependencies.contains(counter.id()));
    }

    #[test]
    fn test_deep_merge_maps() {
        let map1 = CausalAtom::map(vec![
            ("a".to_string(), CausalAtom::int(1, "node1".to_string())),
        ], "node1".to_string());
        
        let map2 = CausalAtom::map(vec![
            ("b".to_string(), CausalAtom::int(2, "node2".to_string())),
        ], "node2".to_string());
        
        let merged = map1.deep_merge(&map2);
        
        match &merged.value {
            AtomValue::Map(entries) => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Expected map"),
        }
    }
}
