mod common;

use common::TestServer;
use sas::network::{NetworkConfig, NetworkClient};
use std::thread;
use std::time::Duration;

#[test]
fn test_opponent_idle_animation() {
    const PORT: u16 = 27968;
    const FIXED_DT: f32 = 1.0 / 60.0;
    
    println!("\n=== Testing opponent idle animation ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");
    
    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    client1.connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");
    
    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();
    
    let player1_id = client1.player_id().expect("Client 1 should have player ID");
    println!("Player 1 connected with ID: {}", player1_id);

    client2.connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 2 failed to connect");
    
    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client2.update();
    
    let player2_id = client2.player_id().expect("Client 2 should have player ID");
    println!("Player 2 connected with ID: {}", player2_id);

    println!("\n=== Waiting 3 seconds for players to settle ===\n");
    for _ in 0..180 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    println!("\n=== Player 1 stands still for 1 second ===\n");
    
    for i in 0..60 {
        client1.send_input(0.0, 0.0, 0.0, 0).ok();
        
        thread::sleep(Duration::from_millis(16));
        
        server.update();
        server.simulate_physics(FIXED_DT);
        
        if i % 20 == 0 {
            let pos = server.get_player_position(player1_id).unwrap();
            println!("  [Frame {}] Player 1 position: ({:.2}, {:.2})", i, pos.0, pos.1);
        }
    }

    for _ in 0..10 {
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let snapshot = client2.last_snapshot().expect("Client 2 should have snapshot");
    let player1_state = snapshot.players.iter()
        .find(|p| p.player_id == player1_id)
        .expect("Client 2 should see Player 1");

    println!("\nPlayer 1 state seen by Client 2:");
    println!("  Position: ({:.2}, {:.2})", player1_state.position.0, player1_state.position.1);
    println!("  Velocity: ({:.2}, {:.2})", player1_state.velocity.0, player1_state.velocity.1);
    println!("  On ground: {}", player1_state.on_ground);

    let velocity_magnitude = (player1_state.velocity.0.powi(2) + player1_state.velocity.1.powi(2)).sqrt();
    println!("  Velocity magnitude: {:.2}", velocity_magnitude);

    assert!(velocity_magnitude < 10.0, 
            "Player 1 should be standing still (low velocity), but has velocity {:.2}", velocity_magnitude);

    println!("\n=== Test PASSED: Opponent velocity correctly shows idle state ===\n");
}

#[test]
fn test_opponent_jump_animation() {
    const PORT: u16 = 27969;
    const FIXED_DT: f32 = 1.0 / 60.0;
    
    println!("\n=== Testing opponent animation during jump ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");
    
    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    client1.connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");
    
    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();
    
    let player1_id = client1.player_id().expect("Client 1 should have player ID");

    client2.connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 2 failed to connect");
    
    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client2.update();

    println!("\n=== Waiting 3 seconds for players to settle ===\n");
    for _ in 0..180 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    println!("\n=== Player 1 jumps (should be in air) ===\n");
    
    for i in 0..20 {
        client1.send_input(0.0, 0.0, 0.0, 2).ok();
        
        thread::sleep(Duration::from_millis(16));
        
        server.update();
        server.simulate_physics(FIXED_DT);
        
        if i % 10 == 0 {
            let pos = server.get_player_position(player1_id).unwrap();
            println!("  [Frame {}] Player 1 position: ({:.2}, {:.2})", i, pos.0, pos.1);
        }
    }

    for _ in 0..10 {
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let snapshot = client2.last_snapshot().expect("Client 2 should have snapshot");
    let player1_state = snapshot.players.iter()
        .find(|p| p.player_id == player1_id)
        .expect("Client 2 should see Player 1");

    println!("\nPlayer 1 state seen by Client 2 (during jump):");
    println!("  Position: ({:.2}, {:.2})", player1_state.position.0, player1_state.position.1);
    println!("  Velocity: ({:.2}, {:.2})", player1_state.velocity.0, player1_state.velocity.1);
    println!("  On ground: {}", player1_state.on_ground);

    let velocity_magnitude = (player1_state.velocity.0.powi(2) + player1_state.velocity.1.powi(2)).sqrt();
    println!("  Velocity magnitude: {:.2}", velocity_magnitude);

    assert!(!player1_state.on_ground || velocity_magnitude > 1.0, 
            "Player 1 should be in air (on_ground=false) or have velocity > 1.0 during jump. on_ground={}, vel={:.2}", 
            player1_state.on_ground, velocity_magnitude);

    println!("\n=== Test PASSED: Jump state correctly transmitted ===\n");
}


