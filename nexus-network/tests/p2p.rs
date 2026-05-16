// NEXUS Network: P2P Integration Tests
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use nexus_network::{QuicTransport, GossipProtocol, CausalMessage};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_p2p_message_exchange() {
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9001);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9002);

    let transport1 = Arc::new(QuicTransport::new_dev(addr1, "node1").expect("Failed to create transport 1"));
    let transport2 = Arc::new(QuicTransport::new_dev(addr2, "node2").expect("Failed to create transport 2"));

    let gossip1 = GossipProtocol::new(transport1.clone());
    let _gossip2 = GossipProtocol::new(transport2.clone());

    gossip1.add_peer(addr2);

    // Create a dummy tensor
    let signing_key = generate_signing_key();
    let mut clock = VectorClock::new();
    let tensor = CausalTensor::new(
        b"Hello P2P".to_vec(),
        vec![],
        1,
        &mut clock,
        &signing_key,
    ).unwrap();

    let msg = CausalMessage::Tensor(tensor.clone());

    // Setup receiver on node 2
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let transport2_clone = transport2.clone();
    tokio::spawn(async move {
        transport2_clone.listen(move |received_msg| {
            let tx = tx.clone();
            async move {
                if let CausalMessage::Tensor(t) = received_msg {
                    tx.send(t).await.unwrap();
                }
            }
        }).await.unwrap();
    });

    // Broadcast from node 1 to node 2
    gossip1.broadcast(msg).await.expect("Failed to broadcast");

    // Verify reception
    let received = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for message")
        .expect("Failed to receive message");

    assert_eq!(received.id, tensor.id);
    assert_eq!(received.data, b"Hello P2P");
}
