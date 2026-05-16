use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn of(data: &[u8]) -> Self {
        let result = Sha256::digest(data);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Hash(bytes)
    }

    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}
