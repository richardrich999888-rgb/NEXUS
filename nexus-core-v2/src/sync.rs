use crate::hash::Hash;
use crate::log::LogEntry;
use crate::errors::{NexusError, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::collections::HashSet;

pub struct SyncClient {
    stream: TcpStream,
}

impl SyncClient {
    pub fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;
        Ok(SyncClient { stream })
    }

    pub fn send_summary(&mut self, hashes: &[Hash]) -> Result<()> {
        let bytes = bincode::serialize(hashes)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;
        let len = (bytes.len() as u32).to_le_bytes();
        
        self.stream.write_all(&len)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;
        self.stream.write_all(&bytes)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;
        
        Ok(())
    }

    pub fn receive_entries(&mut self) -> Result<Vec<LogEntry>> {
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;

        let entries: Vec<LogEntry> = bincode::deserialize(&buf)
            .map_err(|e| NexusError::SyncError(e.to_string()))?;

        for entry in &entries {
            if !entry.verify() {
                return Err(NexusError::InvalidProof);
            }
        }

        Ok(entries)
    }
}

pub fn compute_diff(local: &[Hash], remote: &[Hash]) -> Vec<Hash> {
    let local_set: HashSet<_> = local.iter().copied().collect();
    remote.iter().filter(|h| !local_set.contains(h)).copied().collect()
}
