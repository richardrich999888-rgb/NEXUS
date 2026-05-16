// Causal Presence Protocol - Real-time collaboration awareness
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Inventor: Katta Naga Sri Ganesh
//
// Patent Claim: "A method for synchronizing user cursor positions in a 
// distributed document editing system using content-addressed references 
// that remain valid across concurrent text modifications."

use crate::content_address::ContentAddress;
use crate::crdt::{LWWRegister, ORSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// USER PRESENCE STATE
// ============================================================================

/// User presence status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceStatus {
    /// User is actively editing
    Active,
    /// User is viewing but not editing
    Viewing,
    /// User is typing (show indicator)
    Typing,
    /// User is idle (no activity for > 5 min)
    Idle,
    /// User is offline
    Offline,
}

/// User presence information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPresence {
    /// User identifier
    pub user_id: String,
    
    /// Display name
    pub display_name: String,
    
    /// Current status
    pub status: LWWRegister<PresenceStatus>,
    
    /// Last activity timestamp
    pub last_activity: u64,
    
    /// User color (for cursor display)
    pub color: String,
}

impl UserPresence {
    pub fn new(user_id: String, display_name: String, color: String, node_id: String) -> Self {
        Self {
            user_id,
            display_name,
            status: LWWRegister::new(PresenceStatus::Active, node_id),
            last_activity: Self::now(),
            color,
        }
    }

    /// Update activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Self::now();
    }

    /// Set status
    pub fn set_status(&mut self, status: PresenceStatus) {
        self.status.set(status);
        self.touch();
    }

    /// Check if user is considered idle (no activity for > 5 minutes)
    pub fn is_idle(&self) -> bool {
        let idle_threshold = 5 * 60; // 5 minutes
        Self::now() - self.last_activity > idle_threshold
    }

    /// Merge with another presence (for sync)
    pub fn merge(&mut self, other: &UserPresence) {
        self.status.merge(&other.status);
        if other.last_activity > self.last_activity {
            self.last_activity = other.last_activity;
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// ============================================================================
// CURSOR POSITION (CONTENT-ADDRESSED)
// ============================================================================

/// Cursor position using content-addressed references
/// 
/// Unlike traditional character offsets, this cursor position remains
/// valid even when other users insert/delete text around it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalCursor {
    /// User who owns this cursor
    pub user_id: String,
    
    /// Content address of character BEFORE cursor
    /// None means cursor is at document start
    pub anchor: Option<ContentAddress>,
    
    /// Content address of character AFTER cursor (for selections)
    /// If same as anchor, no selection
    pub focus: Option<ContentAddress>,
    
    /// Timestamp for LWW semantics
    pub timestamp: u64,
    
    /// Node that last updated this cursor
    pub node_id: String,
}

impl CausalCursor {
    /// Create cursor at document start
    pub fn at_start(user_id: String, node_id: String) -> Self {
        Self {
            user_id,
            anchor: None,
            focus: None,
            timestamp: Self::now(),
            node_id,
        }
    }

    /// Create cursor at specific position (after character with given address)
    pub fn at_position(
        user_id: String, 
        after_char: ContentAddress, 
        node_id: String,
    ) -> Self {
        Self {
            user_id,
            anchor: Some(after_char.clone()),
            focus: Some(after_char),
            timestamp: Self::now(),
            node_id,
        }
    }

    /// Create cursor with selection
    pub fn with_selection(
        user_id: String,
        anchor: ContentAddress,
        focus: ContentAddress,
        node_id: String,
    ) -> Self {
        Self {
            user_id,
            anchor: Some(anchor),
            focus: Some(focus),
            timestamp: Self::now(),
            node_id,
        }
    }

    /// Check if cursor has a selection
    pub fn has_selection(&self) -> bool {
        match (&self.anchor, &self.focus) {
            (Some(a), Some(f)) => a.id() != f.id(),
            _ => false,
        }
    }

    /// Move cursor to new position
    pub fn move_to(&mut self, after_char: Option<ContentAddress>) {
        self.anchor = after_char.clone();
        self.focus = after_char;
        self.timestamp = Self::now();
    }

    /// Extend selection to new position
    pub fn extend_to(&mut self, to_char: Option<ContentAddress>) {
        self.focus = to_char;
        self.timestamp = Self::now();
    }

    /// Merge with another cursor (LWW - later timestamp wins)
    pub fn merge(&mut self, other: &CausalCursor) {
        if other.timestamp > self.timestamp {
            self.anchor = other.anchor.clone();
            self.focus = other.focus.clone();
            self.timestamp = other.timestamp;
            self.node_id = other.node_id.clone();
        } else if other.timestamp == self.timestamp && other.node_id > self.node_id {
            // Tie-breaker: higher node_id wins
            self.anchor = other.anchor.clone();
            self.focus = other.focus.clone();
            self.node_id = other.node_id.clone();
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}

// ============================================================================
// PRESENCE MANAGER
// ============================================================================

/// Manages presence and cursor state for all users in a session
#[derive(Clone, Debug)]
pub struct PresenceManager {
    /// All user presences (user_id -> presence)
    pub presences: HashMap<String, UserPresence>,
    
    /// All cursors (user_id -> cursor)
    pub cursors: HashMap<String, CausalCursor>,
    
    /// Active users in this document (OR-Set for add-wins)
    pub active_users: ORSet<String>,
    
    /// This node's ID
    node_id: String,
    
    /// Idle timeout duration
    idle_timeout: Duration,
}

impl PresenceManager {
    pub fn new(node_id: String) -> Self {
        Self {
            presences: HashMap::new(),
            cursors: HashMap::new(),
            active_users: ORSet::new(node_id.clone()),
            node_id,
            idle_timeout: Duration::from_secs(5 * 60), // 5 minutes
        }
    }

    /// Join session as a user
    pub fn join(&mut self, user_id: String, display_name: String, color: String) {
        let presence = UserPresence::new(
            user_id.clone(),
            display_name,
            color,
            self.node_id.clone(),
        );
        
        self.presences.insert(user_id.clone(), presence);
        self.cursors.insert(
            user_id.clone(),
            CausalCursor::at_start(user_id.clone(), self.node_id.clone()),
        );
        self.active_users.add(user_id);
    }

    /// Leave session
    pub fn leave(&mut self, user_id: &str) {
        if let Some(presence) = self.presences.get_mut(user_id) {
            presence.set_status(PresenceStatus::Offline);
        }
        self.active_users.remove(&user_id.to_string());
    }

    /// Update cursor position
    pub fn update_cursor(&mut self, user_id: &str, after_char: Option<ContentAddress>) {
        if let Some(cursor) = self.cursors.get_mut(user_id) {
            cursor.move_to(after_char);
        }
        if let Some(presence) = self.presences.get_mut(user_id) {
            presence.touch();
            presence.set_status(PresenceStatus::Active);
        }
    }

    /// Update cursor selection
    pub fn update_selection(
        &mut self, 
        user_id: &str, 
        anchor: ContentAddress, 
        focus: ContentAddress,
    ) {
        if let Some(cursor) = self.cursors.get_mut(user_id) {
            *cursor = CausalCursor::with_selection(
                user_id.to_string(),
                anchor,
                focus,
                self.node_id.clone(),
            );
        }
    }

    /// Mark user as typing
    pub fn set_typing(&mut self, user_id: &str) {
        if let Some(presence) = self.presences.get_mut(user_id) {
            presence.set_status(PresenceStatus::Typing);
        }
    }

    /// Mark user as idle (called by background timer)
    pub fn check_idle(&mut self) {
        for presence in self.presences.values_mut() {
            if presence.is_idle() && *presence.status.get() != PresenceStatus::Offline {
                presence.set_status(PresenceStatus::Idle);
            }
        }
    }

    /// Get all active users
    pub fn get_active_users(&self) -> Vec<&UserPresence> {
        self.presences
            .values()
            .filter(|p| *p.status.get() != PresenceStatus::Offline)
            .collect()
    }

    /// Get cursor for user
    pub fn get_cursor(&self, user_id: &str) -> Option<&CausalCursor> {
        self.cursors.get(user_id)
    }

    /// Get all cursors (for rendering)
    pub fn get_all_cursors(&self) -> Vec<(&UserPresence, &CausalCursor)> {
        self.presences
            .iter()
            .filter_map(|(user_id, presence)| {
                self.cursors.get(user_id).map(|cursor| (presence, cursor))
            })
            .filter(|(p, _)| *p.status.get() != PresenceStatus::Offline)
            .collect()
    }

    /// Merge with another presence manager (for sync)
    pub fn merge(&mut self, other: &PresenceManager) {
        // Merge presences
        for (user_id, other_presence) in &other.presences {
            if let Some(presence) = self.presences.get_mut(user_id) {
                presence.merge(other_presence);
            } else {
                self.presences.insert(user_id.clone(), other_presence.clone());
            }
        }

        // Merge cursors
        for (user_id, other_cursor) in &other.cursors {
            if let Some(cursor) = self.cursors.get_mut(user_id) {
                cursor.merge(other_cursor);
            } else {
                self.cursors.insert(user_id.clone(), other_cursor.clone());
            }
        }

        // Merge active users set
        self.active_users.merge(&other.active_users);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_lifecycle() {
        let mut manager = PresenceManager::new("node1".to_string());

        // User joins
        manager.join("alice".to_string(), "Alice".to_string(), "#FF5733".to_string());
        
        assert_eq!(manager.get_active_users().len(), 1);
        
        // User types
        manager.set_typing("alice");
        let alice = manager.presences.get("alice").unwrap();
        assert_eq!(*alice.status.get(), PresenceStatus::Typing);

        // User leaves
        manager.leave("alice");
        assert_eq!(manager.get_active_users().len(), 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut manager = PresenceManager::new("node1".to_string());
        manager.join("alice".to_string(), "Alice".to_string(), "#FF5733".to_string());

        // Initial cursor at start
        let cursor = manager.get_cursor("alice").unwrap();
        assert!(cursor.anchor.is_none());

        // Move cursor
        let addr = ContentAddress::new("hello", 5, "op1".to_string());
        manager.update_cursor("alice", Some(addr.clone()));

        let cursor = manager.get_cursor("alice").unwrap();
        assert!(cursor.anchor.is_some());
        assert_eq!(cursor.anchor.as_ref().unwrap().offset, 5);
    }

    #[test]
    fn test_cursor_merge() {
        let mut cursor1 = CausalCursor::at_start("alice".to_string(), "node1".to_string());
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let addr = ContentAddress::new("test", 3, "op1".to_string());
        let cursor2 = CausalCursor::at_position("alice".to_string(), addr, "node2".to_string());

        cursor1.merge(&cursor2);

        // cursor2 has higher timestamp, should win
        assert!(cursor1.anchor.is_some());
    }

    #[test]
    fn test_presence_merge() {
        let mut manager1 = PresenceManager::new("node1".to_string());
        let mut manager2 = PresenceManager::new("node2".to_string());

        manager1.join("alice".to_string(), "Alice".to_string(), "#FF5733".to_string());
        manager2.join("bob".to_string(), "Bob".to_string(), "#33FF57".to_string());

        // Merge
        manager1.merge(&manager2);

        // Both users should be present
        assert_eq!(manager1.presences.len(), 2);
        assert!(manager1.presences.contains_key("alice"));
        assert!(manager1.presences.contains_key("bob"));
    }

    #[test]
    fn test_selection() {
        let mut manager = PresenceManager::new("node1".to_string());
        manager.join("alice".to_string(), "Alice".to_string(), "#FF5733".to_string());

        let anchor = ContentAddress::new("hello", 0, "op1".to_string());
        let focus = ContentAddress::new("hello", 5, "op1".to_string());

        manager.update_selection("alice", anchor, focus);

        let cursor = manager.get_cursor("alice").unwrap();
        assert!(cursor.has_selection());
    }
}
