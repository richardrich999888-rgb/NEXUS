// Content-based addressing for position-independent references

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Position-independent content address.
/// 
/// Instead of referring to positions by index (which breaks under concurrent
/// edits), we refer to content by its cryptographic hash. This preserves
/// user intent across concurrent modifications.
/// 
/// # Example
/// 
/// Bad:  "Insert 'World' at position 6"
/// Good: "Insert 'World' after content_hash='abc123' (which is 'Hello ')"
/// 
/// Even if other operations insert before position 6, the content-based
/// reference remains correct.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContentAddress {
    /// Cryptographic hash of the referenced content
    pub content_hash: String,
    
    /// Offset within that content (stable even if content moves)
    pub offset: usize,
    
    /// ID of the operation that created this content
    pub creator_op_id: String,
}

impl ContentAddress {
    /// Create a new content address
    /// 
    /// # Arguments
    /// 
    /// * `content` - The actual content being referenced
    /// * `offset` - Position within that content
    /// * `creator_op_id` - Operation that created this content
    pub fn new(content: &str, offset: usize, creator_op_id: String) -> Self {
        Self {
            content_hash: Self::hash_content(content),
            offset,
            creator_op_id,
        }
    }

    /// Compute SHA-256 hash of content
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify that given content matches this address
    pub fn verify(&self, content: &str) -> bool {
        Self::hash_content(content) == self.content_hash
    }

    /// Get a stable identifier for this address
    pub fn id(&self) -> String {
        format!("{}@{}", self.content_hash, self.offset)
    }
}

/// Special content addresses for common cases
impl ContentAddress {
    /// Address representing the start of a document
    pub fn document_start() -> Self {
        Self {
            content_hash: "START".to_string(),
            offset: 0,
            creator_op_id: "ROOT".to_string(),
        }
    }

    /// Address representing the end of a document
    pub fn document_end() -> Self {
        Self {
            content_hash: "END".to_string(),
            offset: 0,
            creator_op_id: "ROOT".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_address_creation() {
        let addr = ContentAddress::new("Hello, World!", 7, "op1".to_string());
        assert_eq!(addr.offset, 7);
        assert_eq!(addr.creator_op_id, "op1");
        assert!(!addr.content_hash.is_empty());
    }

    #[test]
    fn test_content_verification() {
        let addr = ContentAddress::new("Hello, World!", 0, "op1".to_string());
        assert!(addr.verify("Hello, World!"));
        assert!(!addr.verify("Different content"));
    }

    #[test]
    fn test_same_content_same_hash() {
        let addr1 = ContentAddress::new("Test", 0, "op1".to_string());
        let addr2 = ContentAddress::new("Test", 0, "op2".to_string());
        assert_eq!(addr1.content_hash, addr2.content_hash);
    }

    #[test]
    fn test_special_addresses() {
        let start = ContentAddress::document_start();
        let end = ContentAddress::document_end();
        
        assert_eq!(start.content_hash, "START");
        assert_eq!(end.content_hash, "END");
    }

    #[test]
    fn test_address_id() {
        let addr = ContentAddress::new("Test", 5, "op1".to_string());
        let id = addr.id();
        assert!(id.contains("@5"));
    }
}
