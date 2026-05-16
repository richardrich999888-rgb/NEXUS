// PCU Compression - Compress PCU data using VECTRA
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh

use nexus_pcu::{PCU, ContentHash};
use vectra::{vectra_encode, vectra_decode, EncodeResult, Artifact, Payload, VectraResult};
use serde::{Deserialize, Serialize};

/// Compressed PCU - a PCU with VECTRA-compressed data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedPCU {
    /// Original PCU ID (preserved for routing)
    pub pcu_id: ContentHash,
    
    /// Compressed inputs (VECTRA artifacts)
    pub compressed_inputs: Vec<CompressedData>,
    
    /// Compressed parameters
    pub compressed_params: Option<CompressedData>,
    
    /// Original code hash (WASM not compressed - already optimized)
    pub code_hash: ContentHash,
    
    /// Compression stats
    pub stats: CompressionStats,
}

/// Compressed data wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedData {
    /// Original content hash
    pub original_hash: ContentHash,
    
    /// Original size in bytes
    pub original_size: usize,
    
    /// Compressed artifact bytes
    pub artifact_bytes: Vec<u8>,
    
    /// Compressed size
    pub compressed_size: usize,
    
    /// Whether compression was applied (false = pass-through)
    pub was_compressed: bool,
}

/// Compression statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Total original bytes
    pub original_bytes: usize,
    
    /// Total compressed bytes
    pub compressed_bytes: usize,
    
    /// Number of inputs compressed
    pub inputs_compressed: usize,
    
    /// Number of inputs passed through
    pub inputs_passthrough: usize,
}

impl CompressionStats {
    /// Calculate compression ratio
    pub fn ratio(&self) -> f64 {
        if self.compressed_bytes == 0 {
            1.0
        } else {
            self.original_bytes as f64 / self.compressed_bytes as f64
        }
    }
    
    /// Calculate space savings percentage
    pub fn savings_percent(&self) -> f64 {
        if self.original_bytes == 0 {
            0.0
        } else {
            100.0 * (1.0 - (self.compressed_bytes as f64 / self.original_bytes as f64))
        }
    }
}

/// Compress a single data block
pub fn compress_data(data: &[u8]) -> CompressedData {
    let original_hash = ContentHash::compute(data);
    let original_size = data.len();
    
    // Create VECTRA Payload from bytes
    let payload = Payload::new(data.to_vec());
    
    match vectra_encode(payload) {
        EncodeResult::Encoded(artifact) => {
            let artifact_bytes = artifact.to_bytes();
            let compressed_size = artifact_bytes.len();
            
            CompressedData {
                original_hash,
                original_size,
                artifact_bytes,
                compressed_size,
                was_compressed: true,
            }
        }
        EncodeResult::PassThrough(original_payload) => {
            // PassThrough returns the original Payload, extract bytes
            let original_bytes = original_payload.as_bytes().to_vec();
            CompressedData {
                original_hash,
                original_size,
                artifact_bytes: original_bytes,
                compressed_size: original_size,
                was_compressed: false,
            }
        }
    }
}

/// Decompress a data block
pub fn decompress_data(compressed: &CompressedData) -> VectraResult<Vec<u8>> {
    if !compressed.was_compressed {
        return Ok(compressed.artifact_bytes.clone());
    }
    
    let artifact = Artifact::from_bytes(&compressed.artifact_bytes)?;
    let payload = vectra_decode(&artifact)?;
    Ok(payload.as_bytes().to_vec())
}

/// Compress PCU inputs and parameters
pub fn compress_pcu(pcu: &PCU, input_data: &[Vec<u8>]) -> CompressedPCU {
    let mut stats = CompressionStats::default();
    let mut compressed_inputs = Vec::new();
    
    for data in input_data {
        let compressed = compress_data(data);
        
        stats.original_bytes += compressed.original_size;
        stats.compressed_bytes += compressed.compressed_size;
        
        if compressed.was_compressed {
            stats.inputs_compressed += 1;
        } else {
            stats.inputs_passthrough += 1;
        }
        
        compressed_inputs.push(compressed);
    }
    
    // Compress parameters if present
    let compressed_params = if !pcu.parameters.is_empty() {
        let params = compress_data(&pcu.parameters);
        stats.original_bytes += params.original_size;
        stats.compressed_bytes += params.compressed_size;
        Some(params)
    } else {
        None
    };
    
    CompressedPCU {
        pcu_id: pcu.id,
        compressed_inputs,
        compressed_params,
        code_hash: pcu.code.hash,
        stats,
    }
}

/// Decompress all PCU data
pub fn decompress_pcu(compressed: &CompressedPCU) -> VectraResult<Vec<Vec<u8>>> {
    let mut inputs = Vec::new();
    
    for comp_data in &compressed.compressed_inputs {
        inputs.push(decompress_data(comp_data)?);
    }
    
    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_data_passthrough() {
        // Random data typically can't be compressed
        let random_data: Vec<u8> = (0..100).map(|i| (i * 17) as u8).collect();
        let compressed = compress_data(&random_data);
        
        // Either compressed or passthrough, should be lossless
        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(random_data, decompressed);
    }

    #[test]
    fn test_compress_structured_data() {
        // Structured JSON data - good candidate for VECTRA
        let structured = br#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"},{"id":3,"name":"Charlie"}]}"#;
        let compressed = compress_data(structured);
        
        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(structured.to_vec(), decompressed);
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats {
            original_bytes: 1000,
            compressed_bytes: 500,
            inputs_compressed: 3,
            inputs_passthrough: 1,
        };
        
        assert!((stats.ratio() - 2.0).abs() < 0.01);
        assert!((stats.savings_percent() - 50.0).abs() < 0.01);
    }
}
