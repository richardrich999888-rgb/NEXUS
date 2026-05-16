//! Atom Composition - Building complex structures from atoms
//! 
//! Enables composing atoms into larger structures while maintaining
//! causal properties throughout.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::version_vector::VersionVector;
use super::causal_atom::{CausalAtom, AtomValue, AtomMeta};

/// Reference to an atom (by ID)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AtomRef {
    /// Atom ID
    pub id: String,
    /// Optional version constraint
    pub version: Option<u64>,
}

impl AtomRef {
    /// Create a new reference
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: None,
        }
    }

    /// Create a versioned reference
    pub fn versioned(id: impl Into<String>, version: u64) -> Self {
        Self {
            id: id.into(),
            version: Some(version),
        }
    }
}

/// A composite made of multiple atoms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeAtom {
    /// Root atom
    pub root: CausalAtom,
    /// Nested atoms (by ID)
    pub atoms: HashMap<String, CausalAtom>,
    /// Version vector for the composite
    pub version: VersionVersion,
    /// Node ID
    node_id: String,
}

/// Alias for version vector (avoiding naming conflict)
type VersionVersion = VersionVector;

impl CompositeAtom {
    /// Create a new composite from a root atom
    pub fn new(root: CausalAtom, node_id: String) -> Self {
        let mut atoms = HashMap::new();
        atoms.insert(root.id().to_string(), root.clone());
        
        Self {
            root,
            atoms,
            version: VersionVector::new(),
            node_id,
        }
    }

    /// Add an atom to the composite
    pub fn add(&mut self, atom: CausalAtom) {
        self.atoms.insert(atom.id().to_string(), atom);
        self.version.increment(&self.node_id);
    }

    /// Get an atom by ID
    pub fn get(&self, id: &str) -> Option<&CausalAtom> {
        self.atoms.get(id)
    }

    /// Get mutable atom by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CausalAtom> {
        self.atoms.get_mut(id)
    }

    /// Resolve a reference to an atom
    pub fn resolve(&self, atom_ref: &AtomRef) -> Option<&CausalAtom> {
        self.atoms.get(&atom_ref.id)
    }

    /// Update an atom in the composite
    pub fn update(&mut self, id: &str, value: AtomValue) -> Option<String> {
        if let Some(atom) = self.atoms.get_mut(id) {
            atom.set_value(value);
            self.version.increment(&self.node_id);
            Some(atom.id().to_string())
        } else {
            None
        }
    }

    /// Merge with another composite
    pub fn merge(&mut self, other: &CompositeAtom) {
        // Merge atoms
        for (id, other_atom) in &other.atoms {
            if let Some(self_atom) = self.atoms.get_mut(id) {
                // Deep merge existing atoms
                *self_atom = self_atom.deep_merge(other_atom);
            } else {
                // Add new atoms
                self.atoms.insert(id.clone(), other_atom.clone());
            }
        }
        
        // Merge root
        self.root = self.root.deep_merge(&other.root);
        
        // Merge versions
        self.version = self.version.merge(&other.version);
    }

    /// Get all atom IDs
    pub fn atom_ids(&self) -> Vec<String> {
        self.atoms.keys().cloned().collect()
    }

    /// Get atom count
    pub fn atom_count(&self) -> usize {
        self.atoms.len()
    }

    /// Flatten to a list of atoms
    pub fn flatten(&self) -> Vec<&CausalAtom> {
        self.atoms.values().collect()
    }
}

/// Atom Composer - builds complex structures
#[derive(Debug)]
pub struct AtomComposer {
    /// Current composite being built
    composite: CompositeAtom,
    /// Stack for nested building
    stack: Vec<String>,
}

impl AtomComposer {
    /// Start a new composition
    pub fn new(node_id: &str) -> Self {
        let root = CausalAtom::map(vec![], node_id.to_string());
        let composite = CompositeAtom::new(root, node_id.to_string());
        
        Self {
            composite,
            stack: vec![],
        }
    }

    /// Start from an existing composite
    pub fn from_composite(composite: CompositeAtom) -> Self {
        let root_id = composite.root.id().to_string();
        Self {
            composite,
            stack: vec![root_id],
        }
    }

    /// Add a field to the current level
    pub fn field(mut self, name: &str, value: CausalAtom) -> Self {
        let parent_id = self.stack.last()
            .cloned()
            .unwrap_or_else(|| self.composite.root.id().to_string());
        
        // Add the atom
        self.composite.add(value.clone());
        
        // Update parent to include reference
        if let Some(parent) = self.composite.atoms.get_mut(&parent_id) {
            if let AtomValue::Map(ref mut entries) = parent.value {
                entries.push((name.to_string(), CausalAtom::reference(value.id(), parent.meta.creator.clone())));
            }
        }
        
        self
    }

    /// Add a string field
    pub fn string_field(self, name: &str, value: &str, node_id: &str) -> Self {
        self.field(name, CausalAtom::string(value, node_id.to_string()))
    }

    /// Add an int field
    pub fn int_field(self, name: &str, value: i64, node_id: &str) -> Self {
        self.field(name, CausalAtom::int(value, node_id.to_string()))
    }

    /// Add a nested object
    pub fn object(mut self, name: &str, node_id: &str) -> Self {
        let obj = CausalAtom::map(vec![], node_id.to_string());
        let obj_id = obj.id().to_string();
        self.composite.add(obj);
        self.stack.push(obj_id);
        self
    }

    /// End the current nested object
    pub fn end_object(mut self) -> Self {
        self.stack.pop();
        self
    }

    /// Build the final composite
    pub fn build(self) -> CompositeAtom {
        self.composite
    }

    /// Get current atom count
    pub fn atom_count(&self) -> usize {
        self.composite.atom_count()
    }
}

/// Schema for typed atoms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomSchema {
    /// Schema name
    pub name: String,
    /// Field definitions
    pub fields: Vec<FieldDef>,
    /// Required fields
    pub required: Vec<String>,
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: String,
    pub description: Option<String>,
}

impl AtomSchema {
    /// Create a new schema
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            required: vec![],
        }
    }

    /// Add a field
    pub fn field(mut self, name: &str, field_type: &str) -> Self {
        self.fields.push(FieldDef {
            name: name.to_string(),
            field_type: field_type.to_string(),
            description: None,
        });
        self
    }

    /// Mark a field as required
    pub fn require(mut self, name: &str) -> Self {
        self.required.push(name.to_string());
        self
    }

    /// Validate an atom against this schema
    pub fn validate(&self, atom: &CausalAtom) -> Result<(), SchemaError> {
        match &atom.value {
            AtomValue::Map(entries) => {
                // Check required fields
                for req in &self.required {
                    if !entries.iter().any(|(k, _)| k == req) {
                        return Err(SchemaError::MissingRequired(req.clone()));
                    }
                }
                Ok(())
            }
            _ => Err(SchemaError::TypeMismatch {
                expected: "map".to_string(),
                got: atom.value_type().to_string(),
            }),
        }
    }
}

/// Schema validation error
#[derive(Debug, Clone)]
pub enum SchemaError {
    MissingRequired(String),
    TypeMismatch { expected: String, got: String },
    InvalidValue(String),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::MissingRequired(field) => write!(f, "Missing required field: {}", field),
            SchemaError::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            }
            SchemaError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for SchemaError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_creation() {
        let root = CausalAtom::string("root", "node1".to_string());
        let composite = CompositeAtom::new(root, "node1".to_string());
        
        assert_eq!(composite.atom_count(), 1);
    }

    #[test]
    fn test_composite_add() {
        let root = CausalAtom::map(vec![], "node1".to_string());
        let mut composite = CompositeAtom::new(root, "node1".to_string());
        
        composite.add(CausalAtom::int(42, "node1".to_string()));
        composite.add(CausalAtom::string("hello", "node1".to_string()));
        
        assert_eq!(composite.atom_count(), 3);
    }

    #[test]
    fn test_composer() {
        let composite = AtomComposer::new("node1")
            .string_field("name", "Alice", "node1")
            .int_field("age", 30, "node1")
            .build();
        
        assert!(composite.atom_count() >= 1);
    }

    #[test]
    fn test_composite_merge() {
        let root1 = CausalAtom::map(vec![
            ("a".to_string(), CausalAtom::int(1, "node1".to_string())),
        ], "node1".to_string());
        let mut comp1 = CompositeAtom::new(root1, "node1".to_string());
        
        let root2 = CausalAtom::map(vec![
            ("b".to_string(), CausalAtom::int(2, "node2".to_string())),
        ], "node2".to_string());
        let comp2 = CompositeAtom::new(root2, "node2".to_string());
        
        comp1.merge(&comp2);
        
        assert!(comp1.atom_count() >= 2);
    }

    #[test]
    fn test_schema_validation() {
        let schema = AtomSchema::new("Person")
            .field("name", "string")
            .field("age", "int")
            .require("name");
        
        let valid = CausalAtom::map(vec![
            ("name".to_string(), CausalAtom::string("Alice", "node1".to_string())),
        ], "node1".to_string());
        
        assert!(schema.validate(&valid).is_ok());
        
        let invalid = CausalAtom::map(vec![], "node1".to_string());
        assert!(schema.validate(&invalid).is_err());
    }
}
