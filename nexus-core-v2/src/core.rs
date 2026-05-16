use crate::executor::Executor;
use crate::hash::Hash;
use crate::log::LogEntry;
use crate::merge::create_merge_entry;
use crate::op::{Lamport, Operation};
use crate::replay::State;
use crate::storage::Storage;
use crate::errors::{NexusError, Result};
use std::collections::HashMap;

pub struct NexusCore {
    log: Vec<LogEntry>,
    index: HashMap<Hash, usize>,
    state: State,
    lamport: Lamport,
    functions: HashMap<Hash, Vec<u8>>,
    executor: Executor,
    storage: Option<Storage>,
}

impl NexusCore {
    pub fn new() -> Self {
        NexusCore {
            log: Vec::new(),
            index: HashMap::new(),
            state: State::new(),
            lamport: 0,
            functions: HashMap::new(),
            executor: Executor::new(),
            storage: None,
        }
    }

    pub fn with_storage(mut self, mut storage: Storage) -> Result<Self> {
        let entries = storage.read_all()?;
        for entry in entries {
            self.append_entry(entry)?;
        }
        self.storage = Some(storage);
        Ok(self)
    }

    pub fn register_function(&mut self, wasm_bytes: Vec<u8>) -> Hash {
        let hash = Hash::of(&wasm_bytes);
        self.functions.insert(hash, wasm_bytes);
        hash
    }

    pub fn execute(&mut self, wasm_hash: Hash, input: Vec<u8>, parents: Vec<Hash>) -> Result<Hash> {
        for parent in &parents {
            if !self.index.contains_key(parent) {
                return Err(NexusError::MissingParent(*parent));
            }
        }

        self.lamport += 1;
        let operation = Operation::new(wasm_hash, input.clone(), parents, self.lamport);

        let wasm_bytes = self.functions.get(&wasm_hash)
            .ok_or_else(|| NexusError::FunctionNotFound(wasm_hash))?;

        let output = self.executor.execute(wasm_bytes, &input)?;
        let entry = LogEntry::new(operation, output);

        let entry_id = self.append_entry(entry.clone())?;

        if let Some(ref mut storage) = self.storage {
            storage.append(&entry)?;
        }

        Ok(entry_id)
    }

    pub fn merge(&mut self, local_id: Hash, remote_id: Hash) -> Result<Hash> {
        let local_pos = *self.index.get(&local_id)
            .ok_or_else(|| NexusError::EntryNotFound(local_id))?;
        let remote_pos = *self.index.get(&remote_id)
            .ok_or_else(|| NexusError::EntryNotFound(remote_id))?;

        let local = self.log[local_pos].clone();
        let remote = self.log[remote_pos].clone();

        self.lamport += 1;
        let merge_entry = create_merge_entry(&local, &remote, self.lamport);
        
        let merge_id = self.append_entry(merge_entry.clone())?;

        if let Some(ref mut storage) = self.storage {
            storage.append(&merge_entry)?;
        }

        Ok(merge_id)
    }

    pub fn replay(&mut self) -> Result<()> {
        self.state.replay(&self.log)
    }

    pub fn ingest_entry(&mut self, entry: LogEntry) -> Result<Hash> {
        if !entry.verify() {
            return Err(NexusError::InvalidProof);
        }

        self.append_entry(entry)
    }

    fn append_entry(&mut self, entry: LogEntry) -> Result<Hash> {
        if !entry.verify() {
            return Err(NexusError::InvalidProof);
        }

        let entry_id = entry.id();
        
        if self.index.contains_key(&entry_id) {
            return Ok(entry_id);
        }

        let position = self.log.len();
        self.log.push(entry.clone());
        self.index.insert(entry_id, position);

        if entry.operation.lamport > self.lamport {
            self.lamport = entry.operation.lamport;
        }

        Ok(entry_id)
    }

    pub fn log_entries(&self) -> &[LogEntry] {
        &self.log
    }

    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    pub fn get_entry(&self, id: Hash) -> Option<&LogEntry> {
        self.index.get(&id).map(|&pos| &self.log[pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism_hash() {
        // Test that hash function is deterministic
        let data = b"same input";
        let h1 = Hash::of(data);
        let h2 = Hash::of(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_log_entry_proof() {
        // Test that log entries have verifiable proofs
        let op = Operation::new(Hash::of(b"fn"), vec![1, 2, 3], vec![], 1);
        let entry = LogEntry::new(op, vec![42]);
        
        assert!(entry.verify());
    }

    #[test]
    fn test_invalid_proof_rejected() {
        let mut core = NexusCore::new();

        let op = Operation::new(Hash::of(b"fn"), vec![1], vec![], 1);
        let mut entry = LogEntry::new(op, vec![42]);
        
        // Tamper with proof
        entry.proof = Hash::of(b"wrong");

        assert!(core.ingest_entry(entry).is_err());
    }

    #[test]
    fn test_entry_id_determinism() {
        // Same operation + output produces same entry ID
        let op1 = Operation::new(Hash::of(b"fn"), vec![1, 2, 3], vec![], 1);
        let op2 = Operation::new(Hash::of(b"fn"), vec![1, 2, 3], vec![], 1);
        
        let entry1 = LogEntry::new(op1, vec![42]);
        let entry2 = LogEntry::new(op2, vec![42]);
        
        assert_eq!(entry1.id(), entry2.id());
    }

    #[test]
    fn test_ingest_and_replay() {
        let mut core = NexusCore::new();
        
        let op1 = Operation::new(Hash::of(b"fn"), vec![1], vec![], 1);
        let entry1 = LogEntry::new(op1, vec![10]);
        let id1 = core.ingest_entry(entry1).unwrap();
        
        let op2 = Operation::new(Hash::of(b"fn"), vec![2], vec![id1], 2);
        let entry2 = LogEntry::new(op2, vec![20]);
        core.ingest_entry(entry2).unwrap();
        
        assert_eq!(core.log_len(), 2);
        assert!(core.replay().is_ok());
    }

    #[test]
    fn test_idempotent_ingest() {
        let mut core = NexusCore::new();
        
        let op = Operation::new(Hash::of(b"fn"), vec![1], vec![], 1);
        let entry = LogEntry::new(op, vec![42]);
        
        let id1 = core.ingest_entry(entry.clone()).unwrap();
        let id2 = core.ingest_entry(entry).unwrap();
        
        assert_eq!(id1, id2);
        assert_eq!(core.log_len(), 1);
    }
}

