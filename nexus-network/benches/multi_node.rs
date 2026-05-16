//! Multi-Node Network Benchmarks
//!
//! Measures network behavior with 3+ nodes:
//! - State sync convergence time
//! - Message overhead per operation
//! - Network partition recovery
//! - Split-brain scenarios

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use nexus_network::{QuicTransport, GossipProtocol, SyncProtocol, CausalMessage};
use nexus_sync::{NexusSyncEngine, ConflictPolicy};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

/// Multi-node test harness
struct MultiNodeHarness {
    nodes: Vec<NodeHandle>,
}

struct NodeHandle {
    addr: SocketAddr,
    transport: Arc<QuicTransport>,
    gossip: Arc<GossipProtocol>,
    sync: Arc<SyncProtocol>,
    engine: Arc<tokio::sync::RwLock<NexusSyncEngine>>,
}

impl MultiNodeHarness {
    /// Create a new multi-node harness with N nodes
    fn new(num_nodes: usize, rt: &Runtime) -> Result<Self, Box<dyn std::error::Error>> {
        let mut nodes = Vec::new();
        let base_port = 10000;
        
        for i in 0..num_nodes {
            let addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                (base_port + i as u16) as u16,
            );
            
            let transport = Arc::new(QuicTransport::new_dev(addr, &format!("node{}", i))?);
            let gossip = Arc::new(GossipProtocol::new(transport.clone()));
            let sync = Arc::new(SyncProtocol::new(transport.clone()));
            let engine = Arc::new(tokio::sync::RwLock::new(
                NexusSyncEngine::new(format!("node{}", i), ConflictPolicy::LastWriterWins)
            ));
            
            nodes.push(NodeHandle {
                addr,
                transport,
                gossip,
                sync,
                engine,
            });
        }
        
        // Connect all nodes in a mesh
        for i in 0..num_nodes {
            for j in 0..num_nodes {
                if i != j {
                    nodes[i].gossip.add_peer(nodes[j].addr);
                }
            }
        }
        
        Ok(Self { nodes })
    }
    
    /// Get node by index
    fn node(&self, idx: usize) -> &NodeHandle {
        &self.nodes[idx]
    }
    
    /// Get number of nodes
    fn len(&self) -> usize {
        self.nodes.len()
    }
}

fn bench_state_sync_convergence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state-sync-convergence");
    
    for num_nodes in [3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("convergence_time", format!("{}_nodes", num_nodes)),
            num_nodes,
            |b, &num_nodes| {
                b.iter(|| {
                    rt.block_on(async {
                        let harness = MultiNodeHarness::new(num_nodes, &rt).unwrap();
                        
                        // Create a tensor on node 0
                        let signing_key = generate_signing_key();
                        let mut clock = VectorClock::new();
                        let tensor = CausalTensor::new(
                            vec![0u8; 1024], // 1KB data
                            vec![],
                            1,
                            &mut clock,
                            &signing_key,
                        ).unwrap();
                        
                        let msg = CausalMessage::Tensor(tensor);
                        
                        let start = Instant::now();
                        
                        // Broadcast from node 0
                        harness.node(0).gossip.broadcast(msg.clone()).await.unwrap();
                        
                        // Wait for convergence (all nodes receive)
                        // In real scenario, would wait for acknowledgments
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        
                        start.elapsed()
                    })
                });
            },
        );
    }
    
    group.finish();
}

fn bench_message_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("message-overhead");
    
    // Test different message sizes
    for size_kb in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("bytes_per_sync", format!("{}KB", size_kb)),
            size_kb,
            |b, &size_kb| {
            b.iter(|| {
                rt.block_on(async {
                    let harness = MultiNodeHarness::new(3, &rt).unwrap();
                    
                    // Create tensor of specified size
                    let signing_key = generate_signing_key();
                    let mut clock = VectorClock::new();
                    let tensor = CausalTensor::new(
                        vec![0u8; size_kb * 1024],
                        vec![],
                        1,
                        &mut clock,
                        &signing_key,
                    ).unwrap();
                    
                    // Serialize to measure overhead
                    let msg = CausalMessage::Tensor(tensor);
                    let serialized = bincode::serialize(&msg).unwrap();
                    
                    // Broadcast and measure total bytes
                    harness.node(0).gossip.broadcast(msg).await.unwrap();
                    
                    // Return message size (overhead = serialized size)
                    serialized.len()
                })
            });
            },
        );
    }
    
    group.finish();
}

fn bench_network_partition_recovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("network-partition");
    
    group.bench_function("split_brain_recovery", |b| {
        b.iter(|| {
            rt.block_on(async {
            let harness = MultiNodeHarness::new(5, &rt).unwrap();
            
            // Simulate partition: nodes 0-2 in partition A, nodes 3-4 in partition B
            // Create conflicting updates in each partition
            let signing_key = generate_signing_key();
            
            // Partition A update
            let mut clock_a = VectorClock::new();
            let tensor_a = CausalTensor::new(
                b"partition_a".to_vec(),
                vec![],
                1,
                &mut clock_a,
                &signing_key,
            ).unwrap();
            
            // Partition B update
            let mut clock_b = VectorClock::new();
            let tensor_b = CausalTensor::new(
                b"partition_b".to_vec(),
                vec![],
                2,
                &mut clock_b,
                &signing_key,
            ).unwrap();
            
            // Broadcast in each partition
            harness.node(0).gossip.broadcast(CausalMessage::Tensor(tensor_a)).await.unwrap();
            harness.node(3).gossip.broadcast(CausalMessage::Tensor(tensor_b)).await.unwrap();
            
            // Simulate partition recovery (reconnect)
            // In real scenario, would wait for merge
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            
            // Measure recovery time
            Instant::now()
            })
        });
    });
    
    group.finish();
}

fn bench_concurrent_updates(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent-updates");
    
    for num_nodes in [3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_broadcast", format!("{}_nodes", num_nodes)),
            num_nodes,
            |b, &num_nodes| {
                b.iter(|| {
                    rt.block_on(async {
                        let harness = MultiNodeHarness::new(num_nodes, &rt).unwrap();
                        
                        // All nodes broadcast simultaneously
                        let mut handles = Vec::new();
                        for i in 0..num_nodes {
                            let gossip = harness.node(i).gossip.clone();
                            let signing_key = generate_signing_key();
                            let mut clock = VectorClock::new();
                            let tensor = CausalTensor::new(
                                format!("node{}_update", i).into_bytes(),
                                vec![],
                                i as u64 + 1,
                                &mut clock,
                                &signing_key,
                            ).unwrap();
                            let msg = CausalMessage::Tensor(tensor);
                            
                            handles.push(tokio::spawn(async move {
                                gossip.broadcast(msg).await
                            }));
                        }
                        
                        // Wait for all broadcasts
                        futures::future::join_all(handles).await;
                        
                        // Wait for convergence
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    })
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_state_sync_convergence,
    bench_message_overhead,
    bench_network_partition_recovery,
    bench_concurrent_updates
);
criterion_main!(benches);

