mod common;

use common::TestServer;
use sas::network::{NetworkConfig, NetworkClient};
use std::thread;
use std::time::Duration;

#[test]
fn test_multiple_clients_can_connect() {
    const PORT: u16 = 27962;
    
    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");
    
    thread::sleep(Duration::from_millis(100));

    let mut clients: Vec<NetworkClient> = Vec::new();
    
    for i in 0..3 {
        let mut client = NetworkClient::new(NetworkConfig::default());
        client.connect(format!("Player{}", i), &format!("127.0.0.1:{}", PORT))
            .expect("Client failed to connect");
        
        thread::sleep(Duration::from_millis(50));
        server.update();
        thread::sleep(Duration::from_millis(50));
        client.update();
        
        assert!(client.is_connected(), "Client {} should be connected", i);
        clients.push(client);
    }

    for _ in 0..10 {
        server.update();
        for client in &mut clients {
            client.update();
        }
        thread::sleep(Duration::from_millis(16));
    }

    for (i, client) in clients.iter().enumerate() {
        let snapshot = client.last_snapshot().expect("Client should have snapshot");
        assert_eq!(snapshot.players.len(), 3, "Client {} should see all 3 players", i);
    }
}

#[test]
fn test_client_disconnect_and_reconnect() {
    const PORT: u16 = 27963;
    
    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");
    
    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    client1.connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Failed to connect");
    
    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();
    
    assert!(client1.is_connected());

    client1.disconnect();
    thread::sleep(Duration::from_millis(50));
    server.update();
    
    assert!(!client1.is_connected());

    let mut client2 = NetworkClient::new(NetworkConfig::default());
    client2.connect("Player1_Reconnected".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Failed to reconnect");
    
    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client2.update();
    
    assert!(client2.is_connected());
}

