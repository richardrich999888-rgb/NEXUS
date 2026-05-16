//! Causal Tokens - CRDT-based economic tokens
//! 
//! Tokens are implemented as CRDTs, enabling distributed balance tracking
//! without consensus overhead. Each node maintains its view of balances,
//! merging naturally through CRDT semantics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::version_vector::VersionVector;
use crate::crdt::PNCounter;

/// Causal Token - a CRDT-based balance
/// 
/// Uses PN-Counter internally to allow both credits and debits
/// while maintaining eventual consistency across the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalToken {
    /// The underlying PN-Counter for balance
    counter: PNCounter,
    /// Token metadata
    pub metadata: TokenMetadata,
    /// Node ID for this token
    node_id: String,
}

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// Token symbol (e.g., "CAUS")
    pub symbol: String,
    /// Decimal places for display
    pub decimals: u8,
    /// Total supply tracking
    pub total_supply: u64,
}

impl Default for TokenMetadata {
    fn default() -> Self {
        Self {
            symbol: "CAUS".to_string(),
            decimals: 6,
            total_supply: 1_000_000_000_000, // 1 trillion tokens
        }
    }
}

impl CausalToken {
    /// Create a new token with initial balance
    pub fn new(node_id: &str, initial_balance: u64) -> Self {
        let mut counter = PNCounter::new(node_id.to_string());
        counter.increment(initial_balance);
        Self {
            counter,
            metadata: TokenMetadata::default(),
            node_id: node_id.to_string(),
        }
    }

    /// Create with zero balance
    pub fn zero(node_id: &str) -> Self {
        Self {
            counter: PNCounter::new(node_id.to_string()),
            metadata: TokenMetadata::default(),
            node_id: node_id.to_string(),
        }
    }

    /// Get current balance
    pub fn balance(&self) -> i64 {
        self.counter.value()
    }

    /// Credit tokens (add)
    pub fn credit(&mut self, amount: u64) {
        self.counter.increment(amount);
    }

    /// Debit tokens (subtract)
    pub fn debit(&mut self, amount: u64) -> Result<(), InsufficientBalance> {
        if self.balance() < amount as i64 {
            return Err(InsufficientBalance {
                required: amount,
                available: self.balance().max(0) as u64,
            });
        }
        self.counter.decrement(amount);
        Ok(())
    }

    /// Merge with another token (CRDT merge)
    pub fn merge(&mut self, other: &CausalToken) {
        self.counter.merge(&other.counter);
    }

    /// Transfer tokens to another balance
    pub fn transfer(
        &mut self,
        to: &mut CausalToken,
        amount: u64,
    ) -> Result<(), InsufficientBalance> {
        self.debit(amount)?;
        to.credit(amount);
        Ok(())
    }
}

/// Token balance tracker for multiple accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Balances per account
    accounts: HashMap<String, CausalToken>,
    /// Version vector for tracking updates
    version: VersionVector,
    /// Node ID
    node_id: String,
}

impl TokenBalance {
    /// Create a new token balance tracker
    pub fn new(node_id: String) -> Self {
        Self {
            accounts: HashMap::new(),
            version: VersionVector::new(),
            node_id,
        }
    }

    /// Get or create account
    pub fn account(&mut self, account_id: &str) -> &mut CausalToken {
        let node_id = self.node_id.clone();
        self.accounts
            .entry(account_id.to_string())
            .or_insert_with(|| CausalToken::zero(&node_id))
    }

    /// Get balance for account
    pub fn balance(&self, account_id: &str) -> i64 {
        self.accounts
            .get(account_id)
            .map(|t| t.balance())
            .unwrap_or(0)
    }

    /// Credit account
    pub fn credit(&mut self, account_id: &str, amount: u64) {
        self.account(account_id).credit(amount);
        self.version.increment(&self.node_id);
    }

    /// Debit account
    pub fn debit(&mut self, account_id: &str, amount: u64) -> Result<(), InsufficientBalance> {
        self.account(account_id).debit(amount)?;
        self.version.increment(&self.node_id);
        Ok(())
    }

    /// Transfer between accounts
    pub fn transfer(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<(), InsufficientBalance> {
        // Check balance first
        if self.balance(from) < amount as i64 {
            return Err(InsufficientBalance {
                required: amount,
                available: self.balance(from).max(0) as u64,
            });
        }
        
        self.account(from).debit(amount)?;
        self.account(to).credit(amount);
        self.version.increment(&self.node_id);
        Ok(())
    }

    /// Merge with another balance tracker (CRDT merge)
    pub fn merge(&mut self, other: &TokenBalance) {
        for (account_id, other_token) in &other.accounts {
            if let Some(self_token) = self.accounts.get_mut(account_id) {
                self_token.merge(other_token);
            } else {
                self.accounts.insert(account_id.clone(), other_token.clone());
            }
        }
        self.version = self.version.merge(&other.version);
    }

    /// Get total supply across all accounts
    pub fn total_supply(&self) -> i64 {
        self.accounts.values().map(|t| t.balance()).sum()
    }

    /// Get number of accounts
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }
}

/// Error for insufficient balance
#[derive(Debug, Clone)]
pub struct InsufficientBalance {
    pub required: u64,
    pub available: u64,
}

impl std::fmt::Display for InsufficientBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient balance: required {}, available {}",
            self.required, self.available
        )
    }
}

impl std::error::Error for InsufficientBalance {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = CausalToken::new("node1", 1000);
        assert_eq!(token.balance(), 1000);
    }

    #[test]
    fn test_credit_debit() {
        let mut token = CausalToken::new("node1", 100);
        
        token.credit(50);
        assert_eq!(token.balance(), 150);
        
        token.debit(30).unwrap();
        assert_eq!(token.balance(), 120);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut token = CausalToken::new("node1", 10);
        let result = token.debit(100);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_merge() {
        let mut token1 = CausalToken::new("node1", 100);
        let mut token2 = CausalToken::new("node2", 50);
        
        token1.credit(20);
        token2.credit(30);
        
        token1.merge(&token2);
        // After merge: 100 + 20 (node1) + 50 + 30 (node2) = 200
        assert_eq!(token1.balance(), 200);
    }

    #[test]
    fn test_balance_tracker() {
        let mut tracker = TokenBalance::new("node1".to_string());
        
        tracker.credit("alice", 1000);
        tracker.credit("bob", 500);
        
        assert_eq!(tracker.balance("alice"), 1000);
        assert_eq!(tracker.balance("bob"), 500);
        
        tracker.transfer("alice", "bob", 200).unwrap();
        
        assert_eq!(tracker.balance("alice"), 800);
        assert_eq!(tracker.balance("bob"), 700);
    }

    #[test]
    fn test_balance_merge() {
        let mut tracker1 = TokenBalance::new("node1".to_string());
        let mut tracker2 = TokenBalance::new("node2".to_string());
        
        tracker1.credit("alice", 100);
        tracker2.credit("alice", 50);
        
        tracker1.merge(&tracker2);
        
        assert_eq!(tracker1.balance("alice"), 150);
    }
}
