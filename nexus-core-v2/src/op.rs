use crate::hash::Hash;
use serde::{Deserialize, Serialize};

pub type Lamport = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub wasm_hash: Hash,
    pub input: Vec<u8>,
    pub parents: Vec<Hash>,
    pub lamport: Lamport,
}

impl Operation {
    pub fn new(wasm_hash: Hash, input: Vec<u8>, parents: Vec<Hash>, lamport: Lamport) -> Self {
        Operation { wasm_hash, input, parents, lamport }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
