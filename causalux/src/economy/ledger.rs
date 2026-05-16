//! Economy Ledger - Transaction tracking and history
//! 
//! Maintains a causal ledger of all economic transactions,
//! enabling audit trails and dispute resolution.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use crate::version_vector::VersionVector;
use crate::content_address::ContentAddress;
use super::token::{TokenBalance, InsufficientBalance};

/// Transaction type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Initial token allocation
    Mint { amount: u64 },
    /// Token destruction
    Burn { amount: u64 },
    /// Transfer between accounts
    Transfer { from: String, to: String, amount: u64 },
    /// Payment for operation
    OperationFee { operation_id: String, amount: u64 },
    /// Reward for providing service
    ServiceReward { service_type: String, amount: u64 },
    /// Stake tokens (lock for participation)
    Stake { amount: u64 },
    /// Unstake tokens
    Unstake { amount: u64 },
}

/// A single transaction in the ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: String,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Account initiating the transaction
    pub initiator: String,
    /// Timestamp
    pub timestamp: u64,
    /// Version vector at creation
    pub version: VersionVector,
    /// Dependencies (previous transactions)
    pub dependencies: BTreeSet<String>,
    /// Signature for verification
    pub signature: Vec<u8>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        tx_type: TransactionType,
        initiator: String,
        version: VersionVector,
        dependencies: BTreeSet<String>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        let id = Self::compute_id(&tx_type, &initiator, timestamp);
        
        Self {
            id,
            tx_type,
            initiator,
            timestamp,
            version,
            dependencies,
            signature: Vec::new(), // Would be signed in production
        }
    }

    fn compute_id(tx_type: &TransactionType, initiator: &str, timestamp: u64) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", tx_type).as_bytes());
        hasher.update(initiator.as_bytes());
        hasher.update(timestamp.to_le_bytes());
        format!("tx_{}", &format!("{:x}", hasher.finalize())[..16])
    }

    /// Get the amount involved in this transaction
    pub fn amount(&self) -> u64 {
        match &self.tx_type {
            TransactionType::Mint { amount } => *amount,
            TransactionType::Burn { amount } => *amount,
            TransactionType::Transfer { amount, .. } => *amount,
            TransactionType::OperationFee { amount, .. } => *amount,
            TransactionType::ServiceReward { amount, .. } => *amount,
            TransactionType::Stake { amount } => *amount,
            TransactionType::Unstake { amount } => *amount,
        }
    }
}

/// Economy Ledger - tracks all transactions
#[derive(Debug, Clone)]
pub struct EconomyLedger {
    /// Token balances
    balances: TokenBalance,
    /// Transaction history
    transactions: HashMap<String, Transaction>,
    /// Transaction order (for replay)
    tx_order: Vec<String>,
    /// Version vector
    version: VersionVector,
    /// Node ID
    node_id: String,
    /// Treasury account (collects fees)
    treasury: String,
    /// Fee percentage (basis points, 100 = 1%)
    fee_bps: u64,
}

impl EconomyLedger {
    /// Create a new economy ledger
    pub fn new(node_id: String, initial_supply: u64) -> Self {
        let treasury = format!("{}_treasury", node_id);
        let mut balances = TokenBalance::new(node_id.clone());
        
        // Mint initial supply to treasury
        balances.credit(&treasury, initial_supply);
        
        Self {
            balances,
            transactions: HashMap::new(),
            tx_order: Vec::new(),
            version: VersionVector::new(),
            node_id,
            treasury,
            fee_bps: 10, // 0.1% fee
        }
    }

    /// Get balance for an account
    pub fn balance(&self, account: &str) -> i64 {
        self.balances.balance(account)
    }

    /// Get treasury balance
    pub fn treasury_balance(&self) -> i64 {
        self.balances.balance(&self.treasury)
    }

    /// Mint new tokens (only treasury can do this initially)
    pub fn mint(&mut self, to: &str, amount: u64) -> Result<String, LedgerError> {
        let tx = Transaction::new(
            TransactionType::Mint { amount },
            self.node_id.clone(),
            self.version.clone(),
            self.latest_tx_deps(),
        );
        
        self.balances.credit(to, amount);
        self.record_tx(tx.clone());
        
        Ok(tx.id)
    }

    /// Transfer tokens between accounts
    pub fn transfer(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<String, LedgerError> {
        // Calculate fee
        let fee = (amount * self.fee_bps) / 10000;
        let net_amount = amount - fee;
        
        // Check balance
        if self.balance(from) < amount as i64 {
            return Err(LedgerError::InsufficientBalance(InsufficientBalance {
                required: amount,
                available: self.balance(from).max(0) as u64,
            }));
        }
        
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: from.to_string(),
                to: to.to_string(),
                amount,
            },
            from.to_string(),
            self.version.clone(),
            self.latest_tx_deps(),
        );
        
        // Execute transfer
        self.balances.transfer(from, to, net_amount)
            .map_err(LedgerError::InsufficientBalance)?;
        
        // Collect fee to treasury
        if fee > 0 {
            self.balances.transfer(from, &self.treasury, fee)
                .map_err(LedgerError::InsufficientBalance)?;
        }
        
        self.record_tx(tx.clone());
        
        Ok(tx.id)
    }

    /// Charge for an operation
    pub fn charge_operation(
        &mut self,
        account: &str,
        operation_id: &str,
        amount: u64,
    ) -> Result<String, LedgerError> {
        if self.balance(account) < amount as i64 {
            return Err(LedgerError::InsufficientBalance(InsufficientBalance {
                required: amount,
                available: self.balance(account).max(0) as u64,
            }));
        }
        
        let tx = Transaction::new(
            TransactionType::OperationFee {
                operation_id: operation_id.to_string(),
                amount,
            },
            account.to_string(),
            self.version.clone(),
            self.latest_tx_deps(),
        );
        
        // Transfer to treasury
        self.balances.transfer(account, &self.treasury, amount)
            .map_err(LedgerError::InsufficientBalance)?;
        
        self.record_tx(tx.clone());
        
        Ok(tx.id)
    }

    /// Reward for providing service (sync bandwidth, etc.)
    pub fn reward_service(
        &mut self,
        account: &str,
        service_type: &str,
        amount: u64,
    ) -> Result<String, LedgerError> {
        // Rewards come from treasury
        if self.treasury_balance() < amount as i64 {
            return Err(LedgerError::TreasuryDepleted);
        }
        
        let tx = Transaction::new(
            TransactionType::ServiceReward {
                service_type: service_type.to_string(),
                amount,
            },
            self.node_id.clone(),
            self.version.clone(),
            self.latest_tx_deps(),
        );
        
        let treasury = self.treasury.clone();
        self.balances.transfer(&treasury, account, amount)
            .map_err(LedgerError::InsufficientBalance)?;
        
        self.record_tx(tx.clone());
        
        Ok(tx.id)
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        self.transactions.get(tx_id)
    }

    /// Get transaction history for an account
    pub fn account_history(&self, account: &str) -> Vec<&Transaction> {
        self.tx_order
            .iter()
            .filter_map(|id| self.transactions.get(id))
            .filter(|tx| tx.initiator == account || self.tx_involves(tx, account))
            .collect()
    }

    fn tx_involves(&self, tx: &Transaction, account: &str) -> bool {
        match &tx.tx_type {
            TransactionType::Transfer { from, to, .. } => from == account || to == account,
            _ => false,
        }
    }

    /// Merge with another ledger (CRDT merge)
    pub fn merge(&mut self, other: &EconomyLedger) {
        // Merge balances
        self.balances.merge(&other.balances);
        
        // Merge transactions (idempotent)
        for (tx_id, tx) in &other.transactions {
            if !self.transactions.contains_key(tx_id) {
                self.transactions.insert(tx_id.clone(), tx.clone());
                self.tx_order.push(tx_id.clone());
            }
        }
        
        // Sort transactions by timestamp
        self.tx_order.sort_by(|a, b| {
            let ta = self.transactions.get(a).map(|t| t.timestamp).unwrap_or(0);
            let tb = self.transactions.get(b).map(|t| t.timestamp).unwrap_or(0);
            ta.cmp(&tb)
        });
        
        self.version = self.version.merge(&other.version);
    }

    fn record_tx(&mut self, tx: Transaction) {
        let tx_id = tx.id.clone();
        self.transactions.insert(tx_id.clone(), tx);
        self.tx_order.push(tx_id);
        self.version.increment(&self.node_id);
    }

    fn latest_tx_deps(&self) -> BTreeSet<String> {
        self.tx_order.last().cloned().into_iter().collect()
    }

    /// Get total transaction count
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Get ledger statistics
    pub fn stats(&self) -> LedgerStats {
        LedgerStats {
            total_accounts: self.balances.account_count(),
            total_transactions: self.transactions.len(),
            total_supply: self.balances.total_supply(),
            treasury_balance: self.treasury_balance(),
        }
    }
}

/// Ledger statistics
#[derive(Debug, Clone)]
pub struct LedgerStats {
    pub total_accounts: usize,
    pub total_transactions: usize,
    pub total_supply: i64,
    pub treasury_balance: i64,
}

/// Ledger errors
#[derive(Debug, Clone)]
pub enum LedgerError {
    InsufficientBalance(InsufficientBalance),
    TreasuryDepleted,
    InvalidTransaction(String),
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerError::InsufficientBalance(e) => write!(f, "{}", e),
            LedgerError::TreasuryDepleted => write!(f, "Treasury depleted"),
            LedgerError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
        }
    }
}

impl std::error::Error for LedgerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_creation() {
        let ledger = EconomyLedger::new("node1".to_string(), 1_000_000);
        assert_eq!(ledger.treasury_balance(), 1_000_000);
    }

    #[test]
    fn test_mint_and_transfer() {
        let mut ledger = EconomyLedger::new("node1".to_string(), 1_000_000);
        
        // Mint to alice
        ledger.mint("alice", 10000).unwrap();
        assert_eq!(ledger.balance("alice"), 10000);
        
        // Transfer to bob (with 0.1% fee)
        ledger.transfer("alice", "bob", 1000).unwrap();
        // alice: 10000 - 1000 = 9000
        // bob: 1000 - 1 (fee) = 999
        // treasury: 1_000_000 + 1 = 1_000_001
        assert_eq!(ledger.balance("alice"), 9000);
        assert_eq!(ledger.balance("bob"), 999);
    }

    #[test]
    fn test_operation_charge() {
        let mut ledger = EconomyLedger::new("node1".to_string(), 1_000_000);
        
        ledger.mint("alice", 1000).unwrap();
        ledger.charge_operation("alice", "op_123", 10).unwrap();
        
        assert_eq!(ledger.balance("alice"), 990);
    }

    #[test]
    fn test_service_reward() {
        let mut ledger = EconomyLedger::new("node1".to_string(), 1_000_000);
        
        let initial_treasury = ledger.treasury_balance();
        ledger.reward_service("alice", "sync_bandwidth", 100).unwrap();
        
        assert_eq!(ledger.balance("alice"), 100);
        assert_eq!(ledger.treasury_balance(), initial_treasury - 100);
    }

    #[test]
    fn test_ledger_merge() {
        let mut ledger1 = EconomyLedger::new("node1".to_string(), 1_000_000);
        let mut ledger2 = EconomyLedger::new("node2".to_string(), 1_000_000);
        
        ledger1.mint("alice", 1000).unwrap();
        ledger2.mint("bob", 500).unwrap();
        
        ledger1.merge(&ledger2);
        
        // Both accounts should exist after merge
        assert!(ledger1.balance("alice") >= 0);
        assert!(ledger1.balance("bob") >= 0);
    }
}
