// USO Compression - Compress USO state using VECTRA
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh

use nexus_pcu::{USO, ContentHash, PrincipalId, SyncPolicy, AccessPolicy};
use vectra::{vectra_encode, vectra_decode, EncodeResult, Artifact, VectraResult};
use serde::{Deserialize, Serialize};
use crate::pcu_compress::{CompressedData, compress_data, decompress_data};

/// Compressed USO - a Universal State Object with VECTRA compression
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedUSO {
    /// Original USO ID
    pub uso_id: ContentHash,
    
    /// Compressed data
    pub compressed_data: CompressedData,
    
    /// Access policy (not compressed - small metadata)
    pub access: AccessPolicy,
    
    /// Sync policy
    pub sync: SyncPolicy,
    
    /// Original lamport timestamp
    pub lamport: u64,
}

impl CompressedUSO {
    /// Create compressed USO from regular USO
    pub fn from_uso(uso: &USO) -> Self {
        let compressed_data = compress_data(&uso.data);
        
        CompressedUSO {
            uso_id: uso.id,
            compressed_data,
            access: uso.access.clone(),
            sync: uso.sync.clone(),
            lamport: uso.history.lamport(),
        }
    }
    
    /// Decompress to get original data
    pub fn decompress_data(&self) -> VectraResult<Vec<u8>> {
        decompress_data(&self.compressed_data)
    }
    
    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.compressed_data.compressed_size == 0 {
            1.0
        } else {
            self.compressed_data.original_size as f64 / 
            self.compressed_data.compressed_size as f64
        }
    }
    
    /// Check if data was actually compressed
    pub fn is_compressed(&self) -> bool {
        self.compressed_data.was_compressed
    }
    
    /// Get space savings in bytes
    pub fn bytes_saved(&self) -> usize {
        if self.compressed_data.was_compressed {
            self.compressed_data.original_size - self.compressed_data.compressed_size
        } else {
            0
        }
    }
}

/// Compress multiple USOs for batch transmission
pub fn compress_uso_batch(usos: &[USO]) -> Vec<CompressedUSO> {
    usos.iter().map(CompressedUSO::from_uso).collect()
}

/// Get total compression stats for a batch
pub fn batch_compression_stats(compressed: &[CompressedUSO]) -> BatchStats {
    let mut stats = BatchStats::default();
    
    for uso in compressed {
        stats.total_count += 1;
        stats.original_bytes += uso.compressed_data.original_size;
        stats.compressed_bytes += uso.compressed_data.compressed_size;
        
        if uso.is_compressed() {
            stats.compressed_count += 1;
        }
    }
    
    stats
}

/// Batch compression statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BatchStats {
    pub total_count: usize,
    pub compressed_count: usize,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
}

impl BatchStats {
    pub fn ratio(&self) -> f64 {
        if self.compressed_bytes == 0 {
            1.0
        } else {
            self.original_bytes as f64 / self.compressed_bytes as f64
        }
    }
    
    pub fn savings_percent(&self) -> f64 {
        if self.original_bytes == 0 {
            0.0
        } else {
            100.0 * (1.0 - (self.compressed_bytes as f64 / self.original_bytes as f64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uso_compression() {
        let owner = PrincipalId::generate();
        let data = br#"{"key": "value", "nested": {"a": 1, "b": 2}}"#.to_vec();
        let uso = USO::new(data.clone(), owner);
        
        let compressed = CompressedUSO::from_uso(&uso);
        
        // Should be lossless
        let decompressed = compressed.decompress_data().unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_batch_compression() {
        let owner = PrincipalId::generate();
        
        let usos: Vec<USO> = (0..5).map(|i| {
            USO::new(format!(r#"{{"id": {}, "data": "test"}}"#, i).into_bytes(), owner)
        }).collect();
        
        let compressed = compress_uso_batch(&usos);
        let stats = batch_compression_stats(&compressed);
        
        assert_eq!(stats.total_count, 5);
    }
}
