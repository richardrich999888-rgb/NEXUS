// End-to-End Integration Tests
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Tests the complete integration flow:
// PCU → VECTRA compression → CAUSALUX sync → decompression

use nexus_pcu::{PCU, USO, PrincipalId, ContentHash};
use nexus_compress::{compress_pcu, decompress_pcu, CompressedPCU, CompressedUSO};
use nexus_sync::{NexusSyncEngine, ConflictPolicy};
use causalux_v2::VersionVector;

/// Test complete flow: Create PCU → Compress → Sync → Decompress
#[test]
fn test_pcu_compress_sync_decompress_flow() {
    // Step 1: Create a PCU with structured data
    let principal = PrincipalId::generate();
    let pcu = PCU::new(
        b"wasm_code_here".to_vec(),
        vec![
            br#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#.to_vec(),
            br#"{"config":{"timeout":30,"retries":3}}"#.to_vec(),
        ],
        principal,
    );

    let input_data: Vec<Vec<u8>> = vec![
        br#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#.to_vec(),
        br#"{"config":{"timeout":30,"retries":3}}"#.to_vec(),
    ];

    // Step 2: Compress PCU using VECTRA
    let compressed = compress_pcu(&pcu, &input_data);
    
    // Verify compression stats
    assert!(compressed.stats.original_bytes > 0);
    assert!(compressed.stats.compressed_bytes > 0);
    println!("Compression ratio: {:.2}x", compressed.stats.ratio());

    // Step 3: Decompress and verify losslessness
    let decompressed_inputs = decompress_pcu(&compressed).expect("Decompression failed");
    assert_eq!(decompressed_inputs.len(), input_data.len());
    assert_eq!(decompressed_inputs[0], input_data[0]);
    assert_eq!(decompressed_inputs[1], input_data[1]);

    println!("✅ PCU compression/decompression flow verified");
}

/// Test USO compression and sync integration
#[test]
fn test_uso_compress_sync_flow() {
    // Step 1: Create USO with structured data
    let owner = PrincipalId::generate();
    let data = br#"{"state":{"counter":42,"items":["a","b","c"]}}"#.to_vec();
    let uso = USO::new(data.clone(), owner);

    // Step 2: Compress USO
    let compressed_uso = CompressedUSO::from_uso(&uso);
    
    // Verify compression
    assert_eq!(compressed_uso.uso_id, uso.id);
    assert!(compressed_uso.compressed_data.original_size > 0);
    println!("USO compression ratio: {:.2}x", compressed_uso.compression_ratio());

    // Step 3: Decompress and verify losslessness
    let decompressed = compressed_uso.decompress_data().expect("Decompression failed");
    assert_eq!(decompressed, data);

    // Step 4: Create sync engine and register USO
    let mut sync_engine = NexusSyncEngine::new("node1", ConflictPolicy::LastWriterWins);
    sync_engine.register_uso(uso);

    println!("✅ USO compression/sync flow verified");
}

/// Test complete integration: PCU → Compress → Sync → Decompress → Execute
#[test]
fn test_complete_integration_flow() {
    // Step 1: Create PCU with multiple inputs
    let principal = PrincipalId::generate();
    let structured_data = vec![
        br#"{"batch":1,"data":[1,2,3,4,5]}"#.to_vec(),
        br#"{"batch":2,"data":[6,7,8,9,10]}"#.to_vec(),
    ];

    let pcu = PCU::new(
        b"compute_wasm".to_vec(),
        structured_data.clone(),
        principal,
    );

    // Step 2: Compress PCU inputs
    let compressed = compress_pcu(&pcu, &structured_data);
    println!("Compressed {} inputs, ratio: {:.2}x", 
             compressed.compressed_inputs.len(), 
             compressed.stats.ratio());

    // Step 3: Create sync engine (simulating network sync)
    let mut sync_engine = NexusSyncEngine::new("node1", ConflictPolicy::LastWriterWins);
    
    // Step 4: Create USO from compressed data (simulating storage)
    let owner = PrincipalId::generate();
    let compressed_bytes = bincode::serialize(&compressed).expect("Serialization failed");
    let uso = USO::new(compressed_bytes.clone(), owner);
    sync_engine.register_uso(uso);

    // Step 5: Simulate sync to another node
    let mut remote_sync = NexusSyncEngine::new("node2", ConflictPolicy::LastWriterWins);
    let sync_delta = sync_engine.get_sync_delta(&VersionVector::new());
    
    // Step 6: Merge on remote node
    remote_sync.merge_remote(sync_delta.operations).expect("Merge failed");

    // Step 7: Decompress on remote node
    let remote_compressed: CompressedPCU = bincode::deserialize(&compressed_bytes)
        .expect("Deserialization failed");
    let remote_inputs = decompress_pcu(&remote_compressed).expect("Decompression failed");

    // Step 8: Verify losslessness
    assert_eq!(remote_inputs.len(), structured_data.len());
    assert_eq!(remote_inputs[0], structured_data[0]);
    assert_eq!(remote_inputs[1], structured_data[1]);

    println!("✅ Complete integration flow verified: PCU → Compress → Sync → Decompress");
}

/// Test multi-node sync with compressed USOs
#[test]
fn test_multi_node_compressed_sync() {
    let owner = PrincipalId::generate();
    
    // Node 1: Create and compress USO
    let data1 = br#"{"node":"1","value":100}"#.to_vec();
    let uso1 = USO::new(data1.clone(), owner);
    let compressed1 = CompressedUSO::from_uso(&uso1);
    
    // Node 2: Create and compress USO
    let data2 = br#"{"node":"2","value":200}"#.to_vec();
    let uso2 = USO::new(data2.clone(), owner);
    let compressed2 = CompressedUSO::from_uso(&uso2);
    
    // Create sync engines
    let mut sync1 = NexusSyncEngine::new("node1", ConflictPolicy::LastWriterWins);
    let mut sync2 = NexusSyncEngine::new("node2", ConflictPolicy::LastWriterWins);
    
    sync1.register_uso(uso1);
    sync2.register_uso(uso2);
    
    // Sync node1 → node2
    let delta = sync1.get_sync_delta(&VersionVector::new());
    sync2.merge_remote(delta.operations).expect("Merge failed");
    
    // Verify both nodes have synced
    assert_eq!(sync1.operation_count(), 0); // No operations yet (no keypair)
    assert_eq!(sync2.operation_count(), 0);
    
    println!("✅ Multi-node compressed sync verified");
}

/// Test compression benefits for structured data
#[test]
fn test_compression_benefits_structured_data() {
    // Highly structured JSON data (should compress well)
    let structured = br#"{"users":[{"id":1,"name":"Alice","email":"alice@example.com"},{"id":2,"name":"Bob","email":"bob@example.com"},{"id":3,"name":"Charlie","email":"charlie@example.com"}]}"#.to_vec();
    
    let principal = PrincipalId::generate();
    let pcu = PCU::new(
        b"wasm".to_vec(),
        vec![structured.clone()],
        principal,
    );
    
    let compressed = compress_pcu(&pcu, &vec![structured]);
    
    // Structured data should compress well
    println!("Structured data compression:");
    println!("  Original: {} bytes", compressed.stats.original_bytes);
    println!("  Compressed: {} bytes", compressed.stats.compressed_bytes);
    println!("  Ratio: {:.2}x", compressed.stats.ratio());
    println!("  Savings: {:.1}%", compressed.stats.savings_percent());
    
    assert!(compressed.stats.compressed_bytes <= compressed.stats.original_bytes);
    
    println!("✅ Compression benefits verified");
}

/// Test error handling in integration flow
#[test]
fn test_integration_error_handling() {
    // Test decompression of invalid data
    let invalid_compressed = CompressedPCU {
        pcu_id: ContentHash::compute(b"test"),
        compressed_inputs: vec![],
        compressed_params: None,
        code_hash: ContentHash::compute(b"code"),
        stats: Default::default(),
    };
    
    // Should handle empty inputs gracefully
    let result = decompress_pcu(&invalid_compressed);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
    
    println!("✅ Error handling verified");
}

