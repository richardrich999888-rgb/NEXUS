use crate::log::LogEntry;
use crate::errors::{NexusError, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;

pub struct Storage {
    file: File,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)
            .map_err(|e| NexusError::StorageError(e.to_string()))?;
        
        Ok(Storage { file })
    }

    pub fn append(&mut self, entry: &LogEntry) -> Result<()> {
        let bytes = entry.serialize();
        let len = (bytes.len() as u32).to_le_bytes();
        
        self.file.write_all(&len)
            .map_err(|e| NexusError::StorageError(e.to_string()))?;
        self.file.write_all(&bytes)
            .map_err(|e| NexusError::StorageError(e.to_string()))?;
        self.file.sync_all()
            .map_err(|e| NexusError::StorageError(e.to_string()))?;
        
        Ok(())
    }

    pub fn read_all(&mut self) -> Result<Vec<LogEntry>> {
        self.file.seek(SeekFrom::Start(0))
            .map_err(|e| NexusError::StorageError(e.to_string()))?;

        let mut entries = Vec::new();
        loop {
            let mut len_bytes = [0u8; 4];
            match self.file.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(NexusError::StorageError(e.to_string())),
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            let mut buf = vec![0u8; len];
            self.file.read_exact(&mut buf)
                .map_err(|e| NexusError::StorageError(e.to_string()))?;

            let entry = LogEntry::deserialize(&buf)?;
            entries.push(entry);
        }

        Ok(entries)
    }
}
