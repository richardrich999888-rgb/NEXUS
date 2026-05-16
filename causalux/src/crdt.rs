// CRDT Layer - Conflict-Free Replicated Data Types
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Inventor: Katta Naga Sri Ganesh

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};

// ============================================================================
// RGA TEXT CRDT: Replicated Growable Array for collaborative text editing
// ============================================================================

/// Unique identifier for a character in the RGA
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CharacterId {
    pub timestamp: u64,
    pub node_id: String,
    pub sequence: u64,
}

impl CharacterId {
    pub fn new(timestamp: u64, node_id: String, sequence: u64) -> Self {
        Self { timestamp, node_id, sequence }
    }
}

/// A character in the RGA with tombstone for deletions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RGACharacter {
    pub id: CharacterId,
    pub value: char,
    pub deleted: bool,
    pub inserted_after: Option<CharacterId>,
}

/// RGA (Replicated Growable Array) CRDT for text
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RGAText {
    pub characters: BTreeMap<CharacterId, RGACharacter>,
    order: Vec<CharacterId>,
    node_id: String,
    sequence_counter: u64,
}

impl RGAText {
    pub fn new(node_id: String) -> Self {
        Self {
            characters: BTreeMap::new(),
            order: Vec::new(),
            node_id,
            sequence_counter: 0,
        }
    }

    /// Insert a character at a position
    pub fn insert(&mut self, position: usize, character: char) -> CharacterId {
        let timestamp = Self::current_timestamp();
        self.sequence_counter += 1;
        
        let id = CharacterId::new(timestamp, self.node_id.clone(), self.sequence_counter);
        
        let inserted_after = if position > 0 && !self.order.is_empty() {
            self.order.get(position - 1).cloned()
        } else {
            None
        };

        let char_obj = RGACharacter {
            id: id.clone(),
            value: character,
            deleted: false,
            inserted_after,
        };

        self.characters.insert(id.clone(), char_obj);
        let insert_pos = position.min(self.order.len());
        self.order.insert(insert_pos, id.clone());

        id
    }

    /// Delete a character at a position
    pub fn delete(&mut self, position: usize) -> Option<CharacterId> {
        if position >= self.order.len() {
            return None;
        }

        let id = self.order[position].clone();
        
        if let Some(char_obj) = self.characters.get_mut(&id) {
            char_obj.deleted = true;
        }

        self.order.remove(position);
        Some(id)
    }

    /// Insert a character received from another node
    pub fn insert_remote(&mut self, char_obj: RGACharacter) {
        let id = char_obj.id.clone();

        if self.characters.contains_key(&id) {
            return; // Already have this character
        }

        let position = if let Some(after_id) = &char_obj.inserted_after {
            self.order.iter().position(|cid| cid == after_id)
                .map(|pos| pos + 1)
                .unwrap_or(0)
        } else {
            0
        };

        // Handle concurrent insertions - deterministic ordering
        let mut insert_pos = position;
        while insert_pos < self.order.len() {
            let existing_id = &self.order[insert_pos];
            if &id < existing_id {
                break;
            }
            insert_pos += 1;
        }

        self.characters.insert(id.clone(), char_obj);
        self.order.insert(insert_pos, id);
    }

    /// Delete a character received from another node
    pub fn delete_remote(&mut self, char_id: &CharacterId) {
        if let Some(char_obj) = self.characters.get_mut(char_id) {
            char_obj.deleted = true;
        }
        self.order.retain(|id| id != char_id);
    }

    /// Get the current text
    pub fn to_string(&self) -> String {
        self.order
            .iter()
            .filter_map(|id| {
                self.characters.get(id).and_then(|ch| {
                    if !ch.deleted { Some(ch.value) } else { None }
                })
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn get_all_characters(&self) -> Vec<RGACharacter> {
        self.characters.values().cloned().collect()
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}

// ============================================================================
// G-COUNTER CRDT: Grow-only counter
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GCounter {
    counts: BTreeMap<String, u64>,
    node_id: String,
}

impl GCounter {
    pub fn new(node_id: String) -> Self {
        let mut counts = BTreeMap::new();
        counts.insert(node_id.clone(), 0);
        Self { counts, node_id }
    }

    pub fn increment(&mut self, amount: u64) {
        *self.counts.entry(self.node_id.clone()).or_insert(0) += amount;
    }

    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn merge(&mut self, other: &GCounter) {
        for (node, count) in &other.counts {
            self.counts
                .entry(node.clone())
                .and_modify(|c| *c = (*c).max(*count))
                .or_insert(*count);
        }
    }

    pub fn get_breakdown(&self) -> BTreeMap<String, u64> {
        self.counts.clone()
    }
}

// ============================================================================
// PN-COUNTER CRDT: Positive-Negative counter
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PNCounter {
    positive: GCounter,
    negative: GCounter,
}

impl PNCounter {
    pub fn new(node_id: String) -> Self {
        Self {
            positive: GCounter::new(node_id.clone()),
            negative: GCounter::new(node_id),
        }
    }

    pub fn increment(&mut self, amount: u64) {
        self.positive.increment(amount);
    }

    pub fn decrement(&mut self, amount: u64) {
        self.negative.increment(amount);
    }

    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }

    pub fn merge(&mut self, other: &PNCounter) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
}

// ============================================================================
// LWW-REGISTER CRDT: Last-Write-Wins register
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LWWRegister<T: Clone> {
    value: T,
    timestamp: u64,
    node_id: String,
}

impl<T: Clone> LWWRegister<T> {
    pub fn new(initial_value: T, node_id: String) -> Self {
        Self {
            value: initial_value,
            timestamp: 0,
            node_id,
        }
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn merge(&mut self, other: &LWWRegister<T>) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.node_id = other.node_id.clone();
        } else if other.timestamp == self.timestamp && other.node_id > self.node_id {
            self.value = other.value.clone();
            self.node_id = other.node_id.clone();
        }
    }
}

// ============================================================================
// OR-SET CRDT: Observed-Remove Set (add-wins)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ElementId {
    pub value_hash: String,
    pub timestamp: u64,
    pub node_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ORSet<T: Clone + Serialize> {
    elements: HashMap<String, BTreeSet<ElementId>>,
    removed: BTreeSet<ElementId>,
    node_id: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Serialize> ORSet<T> {
    pub fn new(node_id: String) -> Self {
        Self {
            elements: HashMap::new(),
            removed: BTreeSet::new(),
            node_id,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add(&mut self, value: T) -> ElementId {
        let value_hash = Self::hash_value(&value);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        let element_id = ElementId {
            value_hash: value_hash.clone(),
            timestamp,
            node_id: self.node_id.clone(),
        };

        self.elements
            .entry(value_hash)
            .or_insert_with(BTreeSet::new)
            .insert(element_id.clone());

        element_id
    }

    pub fn remove(&mut self, value: &T) {
        let value_hash = Self::hash_value(value);
        
        if let Some(element_ids) = self.elements.get(&value_hash) {
            for id in element_ids {
                self.removed.insert(id.clone());
            }
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        let value_hash = Self::hash_value(value);
        
        if let Some(element_ids) = self.elements.get(&value_hash) {
            element_ids.iter().any(|id| !self.removed.contains(id))
        } else {
            false
        }
    }

    pub fn elements(&self) -> Vec<String> {
        self.elements
            .iter()
            .filter(|(_, ids)| ids.iter().any(|id| !self.removed.contains(id)))
            .map(|(value_hash, _)| value_hash.clone())
            .collect()
    }

    pub fn merge(&mut self, other: &ORSet<T>) {
        for (value_hash, ids) in &other.elements {
            self.elements
                .entry(value_hash.clone())
                .or_insert_with(BTreeSet::new)
                .extend(ids.clone());
        }
        self.removed.extend(other.removed.clone());
    }

    fn hash_value(value: &T) -> String {
        let json = serde_json::to_string(value).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// ============================================================================
// LWW-MAP CRDT: Last-Write-Wins Map
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LWWMap<K: Clone + Eq + std::hash::Hash, V: Clone> {
    entries: HashMap<K, LWWRegister<Option<V>>>,
    node_id: String,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LWWMap<K, V> {
    pub fn new(node_id: String) -> Self {
        Self {
            entries: HashMap::new(),
            node_id,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let register = self.entries
            .entry(key)
            .or_insert_with(|| LWWRegister::new(None, self.node_id.clone()));
        register.set(Some(value));
    }

    pub fn remove(&mut self, key: &K) {
        if let Some(register) = self.entries.get_mut(key) {
            register.set(None);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|reg| reg.get().as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().filter_map(|(k, reg)| {
            reg.get().as_ref().map(|v| (k, v))
        })
    }

    pub fn merge(&mut self, other: &LWWMap<K, V>) {
        for (key, other_register) in &other.entries {
            let register = self.entries
                .entry(key.clone())
                .or_insert_with(|| LWWRegister::new(None, self.node_id.clone()));
            register.merge(other_register);
        }
    }
}

// ============================================================================
// CRDT DOCUMENT: Composite CRDT for rich documents
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CRDTDocument {
    pub id: String,
    pub title: LWWRegister<String>,
    pub content: RGAText,
    pub metadata: LWWMap<String, String>,
    pub collaborators: ORSet<String>,
    pub version_count: GCounter,
    pub node_id: String,
}

impl CRDTDocument {
    pub fn new(id: String, node_id: String) -> Self {
        Self {
            id: id.clone(),
            title: LWWRegister::new(String::new(), node_id.clone()),
            content: RGAText::new(node_id.clone()),
            metadata: LWWMap::new(node_id.clone()),
            collaborators: ORSet::new(node_id.clone()),
            version_count: GCounter::new(node_id.clone()),
            node_id,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title.set(title);
        self.version_count.increment(1);
    }

    pub fn insert_text(&mut self, position: usize, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.content.insert(position + i, ch);
        }
        self.version_count.increment(1);
    }

    pub fn delete_text(&mut self, position: usize, length: usize) {
        for _ in 0..length {
            self.content.delete(position);
        }
        self.version_count.increment(1);
    }

    pub fn add_collaborator(&mut self, collaborator: String) {
        self.collaborators.add(collaborator);
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn merge(&mut self, other: &CRDTDocument) {
        self.title.merge(&other.title);
        
        for char_obj in other.content.get_all_characters() {
            if !self.content.characters.contains_key(&char_obj.id) {
                self.content.insert_remote(char_obj);
            }
        }
        
        self.metadata.merge(&other.metadata);
        self.collaborators.merge(&other.collaborators);
        self.version_count.merge(&other.version_count);
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "title": self.title.get(),
            "content": self.content.to_string(),
            "metadata": self.metadata.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<_, _>>(),
            "collaborators": self.collaborators.elements(),
            "version": self.version_count.value(),
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rga_basic() {
        let mut doc = RGAText::new("node1".to_string());
        doc.insert(0, 'H');
        doc.insert(1, 'i');
        
        assert_eq!(doc.to_string(), "Hi");
    }

    #[test]
    fn test_rga_concurrent_inserts() {
        let mut doc1 = RGAText::new("node1".to_string());
        let mut doc2 = RGAText::new("node2".to_string());

        doc1.insert(0, 'A');
        doc2.insert(0, 'B');

        // Merge
        for char_obj in doc2.get_all_characters() {
            doc1.insert_remote(char_obj);
        }
        for char_obj in doc1.get_all_characters() {
            doc2.insert_remote(char_obj.clone());
        }

        // Both should converge
        assert_eq!(doc1.to_string(), doc2.to_string());
    }

    #[test]
    fn test_gcounter() {
        let mut c1 = GCounter::new("node1".to_string());
        let mut c2 = GCounter::new("node2".to_string());

        c1.increment(5);
        c2.increment(3);

        c1.merge(&c2);
        assert_eq!(c1.value(), 8);
    }

    #[test]
    fn test_pncounter() {
        let mut counter = PNCounter::new("node1".to_string());
        counter.increment(10);
        counter.decrement(3);
        
        assert_eq!(counter.value(), 7);
    }

    #[test]
    fn test_orset_add_wins() {
        let mut set1 = ORSet::<String>::new("node1".to_string());
        let mut set2 = ORSet::<String>::new("node2".to_string());

        set1.add("apple".to_string());
        set2.add("apple".to_string());
        set2.remove(&"apple".to_string());

        set1.merge(&set2);

        // Add wins!
        assert!(set1.contains(&"apple".to_string()));
    }

    #[test]
    fn test_lww_map() {
        let mut map1 = LWWMap::<String, i32>::new("node1".to_string());
        map1.insert("key".to_string(), 10);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let mut map2 = LWWMap::<String, i32>::new("node2".to_string());
        map2.insert("key".to_string(), 20);

        map1.merge(&map2);
        assert_eq!(map1.get(&"key".to_string()), Some(&20));
    }

    #[test]
    fn test_crdt_document() {
        let mut doc1 = CRDTDocument::new("doc".to_string(), "alice".to_string());
        let mut doc2 = CRDTDocument::new("doc".to_string(), "bob".to_string());

        doc1.set_title("Hello".to_string());
        doc1.insert_text(0, "World");

        doc2.set_title("Hi".to_string());
        doc2.insert_text(0, "There");

        doc1.merge(&doc2);
        doc2.merge(&doc1);

        // Should converge
        assert_eq!(doc1.content.to_string(), doc2.content.to_string());
    }
}
