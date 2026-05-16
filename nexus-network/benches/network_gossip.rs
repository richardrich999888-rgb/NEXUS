// NEXUS Network: Gossip Benchmarks
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use nexus_network::{QuicTransport, GossipProtocol, CausalMessage};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_network_gossip(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9101);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9102);

    let transport1 = Arc::new(QuicTransport::new_dev(addr1, "node1").unwrap());
    let transport2 = Arc::new(QuicTransport::new_dev(addr2, "node2").unwrap());
    
    let gossip1 = GossipProtocol::new(transport1);
    let _gossip2 = GossipProtocol::new(transport2);
    
    gossip1.add_peer(addr2);

    let signing_key = generate_signing_key();
    let mut clock = VectorClock::new();
    let tensor = CausalTensor::new(
        vec![0u8; 1024],
        vec![],
        1,
        &mut clock,
        &signing_key,
    ).unwrap();
    let msg = CausalMessage::Tensor(tensor);

    c.bench_function("gossip_broadcast_1kb", |b| {
        b.iter(|| {
            rt.block_on(async {
                gossip1.broadcast(black_box(msg.clone())).await.unwrap();
            })
        });
    });
}

criterion_group!(benches, bench_network_gossip);
criterion_main!(benches);
