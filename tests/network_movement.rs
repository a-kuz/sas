mod common;

use common::TestServer;
use sas::network::{NetworkClient, NetworkConfig};
use std::thread;
use std::time::Duration;

#[test]
fn test_two_clients_movement_synchronization() {
    const PORT: u16 = 27961;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Starting network integration test ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    println!("Client 1 connecting...");
    client1
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();

    assert!(client1.is_connected(), "Client 1 should be connected");
    let player1_id = client1.player_id().expect("Client 1 should have player ID");
    println!("Client 1 connected with ID: {}", player1_id);

    println!("Client 2 connecting...");
    client2
        .connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 2 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client2.update();

    assert!(client2.is_connected(), "Client 2 should be connected");
    let player2_id = client2.player_id().expect("Client 2 should have player ID");
    println!("Client 2 connected with ID: {}", player2_id);

    println!("\n=== Waiting 3 seconds for players to settle ===\n");
    for _ in 0..180 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let initial_pos_server = server
        .get_player_position(player1_id)
        .expect("Player 1 should exist on server");
    println!(
        "Player 1 initial position (server): ({:.2}, {:.2})",
        initial_pos_server.0, initial_pos_server.1
    );

    println!("\n=== Player 1 starts moving right ===\n");

    for i in 0..30 {
        client1.send_input(0.0, 1.0, 0.0, 0).ok();

        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);

        if i % 5 == 0 {
            let pos = server.get_player_position(player1_id).unwrap();
            println!(
                "  [Frame {}] Player 1 position on server: ({:.2}, {:.2})",
                i, pos.0, pos.1
            );
        }
    }

    for _ in 0..10 {
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let final_pos_server = server
        .get_player_position(player1_id)
        .expect("Player 1 should still exist on server");
    println!(
        "\nPlayer 1 final position (server): ({:.2}, {:.2})",
        final_pos_server.0, final_pos_server.1
    );

    let snapshot = client2
        .last_snapshot()
        .expect("Client 2 should have received snapshot");
    println!(
        "Client 2 received snapshot with {} players",
        snapshot.players.len()
    );

    let player1_state = snapshot
        .players
        .iter()
        .find(|p| p.player_id == player1_id)
        .expect("Client 2 should see Player 1 in snapshot");

    println!(
        "Player 1 position seen by Client 2: ({:.2}, {:.2})",
        player1_state.position.0, player1_state.position.1
    );

    let distance_moved = final_pos_server.0 - initial_pos_server.0;
    println!("\nPlayer 1 moved {:.2} units to the right", distance_moved);

    assert!(
        distance_moved > 10.0,
        "Player 1 should have moved significantly (moved: {:.2})",
        distance_moved
    );

    let pos_diff_x = (player1_state.position.0 - final_pos_server.0).abs();
    let pos_diff_y = (player1_state.position.1 - final_pos_server.1).abs();
    println!(
        "Position difference: x={:.2}, y={:.2}",
        pos_diff_x, pos_diff_y
    );

    assert!(pos_diff_x < 50.0 && pos_diff_y < 50.0,
            "Client 2 should see Player 1 at approximately the same position as server (diff: {:.2}, {:.2})",
            pos_diff_x, pos_diff_y);

    println!("\n=== Test PASSED ===\n");
}

#[test]
fn test_bidirectional_movement_visibility() {
    const PORT: u16 = 27964;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing bidirectional movement visibility ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    client1
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();

    let player1_id = client1.player_id().expect("Client 1 should have player ID");
    println!("Player 1 connected with ID: {}", player1_id);

    client2
        .connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
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

    let initial_snapshot_c1 = client1
        .last_snapshot()
        .expect("Client 1 should have snapshot");
    let initial_snapshot_c2 = client2
        .last_snapshot()
        .expect("Client 2 should have snapshot");

    let p1_initial_pos = initial_snapshot_c2
        .players
        .iter()
        .find(|p| p.player_id == player1_id)
        .map(|p| p.position)
        .expect("Player 1 should be visible to Client 2");

    let p2_initial_pos = initial_snapshot_c1
        .players
        .iter()
        .find(|p| p.player_id == player2_id)
        .map(|p| p.position)
        .expect("Player 2 should be visible to Client 1");

    println!("Initial state:");
    println!(
        "  Player 1 position (seen by Client 2): ({:.2}, {:.2})",
        p1_initial_pos.0, p1_initial_pos.1
    );
    println!(
        "  Player 2 position (seen by Client 1): ({:.2}, {:.2})",
        p2_initial_pos.0, p2_initial_pos.1
    );

    println!("\n=== Player 1 jumps (button=2), Player 2 moves right ===\n");

    for i in 0..40 {
        client1.send_input(0.0, 0.0, 0.0, 2).ok();
        client2.send_input(0.0, 1.0, 0.0, 0).ok();

        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);

        if i % 10 == 0 {
            let p1_pos = server.get_player_position(player1_id).unwrap();
            let p2_pos = server.get_player_position(player2_id).unwrap();
            println!(
                "  [Frame {}] Player 1: ({:.2}, {:.2}), Player 2: ({:.2}, {:.2})",
                i, p1_pos.0, p1_pos.1, p2_pos.0, p2_pos.1
            );
        }
    }

    for _ in 0..10 {
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let final_snapshot_c1 = client1
        .last_snapshot()
        .expect("Client 1 should have final snapshot");
    let final_snapshot_c2 = client2
        .last_snapshot()
        .expect("Client 2 should have final snapshot");

    let p1_final_seen_by_c2 = final_snapshot_c2
        .players
        .iter()
        .find(|p| p.player_id == player1_id)
        .expect("Player 1 should still be visible to Client 2");

    let p2_final_seen_by_c1 = final_snapshot_c1
        .players
        .iter()
        .find(|p| p.player_id == player2_id)
        .expect("Player 2 should still be visible to Client 1");

    println!("\nFinal state:");
    println!(
        "  Player 1 position (seen by Client 2): ({:.2}, {:.2})",
        p1_final_seen_by_c2.position.0, p1_final_seen_by_c2.position.1
    );
    println!(
        "  Player 2 position (seen by Client 1): ({:.2}, {:.2})",
        p2_final_seen_by_c1.position.0, p2_final_seen_by_c1.position.1
    );

    let p1_moved_y = p1_final_seen_by_c2.position.1 - p1_initial_pos.1;
    println!(
        "\nPlayer 1 vertical movement (jump): {:.2} units",
        p1_moved_y
    );

    assert!(
        p1_moved_y < -5.0,
        "Player 1 should have jumped up (negative Y), but moved {:.2} units",
        p1_moved_y
    );

    let p2_moved_x = p2_final_seen_by_c1.position.0 - p2_initial_pos.0;
    println!("Player 2 horizontal movement: {:.2} units", p2_moved_x);

    assert!(
        p2_moved_x > 10.0,
        "Player 2 should have moved right significantly, but only moved {:.2} units",
        p2_moved_x
    );

    println!("\n=== Both players see each other's movement ===");
    println!("✓ Client 2 sees Player 1 jumping");
    println!("✓ Client 1 sees Player 2 moving right");
    println!("\n=== Test PASSED ===\n");
}

#[test]
#[should_panic(expected = "Player 1 should have moved")]
fn test_no_movement_should_fail_player1() {
    const PORT: u16 = 27965;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing that NO movement is detected (should fail) ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    client1
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();

    let player1_id = client1.player_id().expect("Client 1 should have player ID");

    client2
        .connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
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

    let initial_pos_server = server
        .get_player_position(player1_id)
        .expect("Player 1 should exist on server");
    println!(
        "Player 1 settled position: ({:.2}, {:.2})",
        initial_pos_server.0, initial_pos_server.1
    );

    println!("\n=== Player 1 does NOT move (sends no input) ===\n");

    for i in 0..30 {
        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);

        if i % 10 == 0 {
            let pos = server.get_player_position(player1_id).unwrap();
            println!(
                "  [Frame {}] Player 1 position: ({:.2}, {:.2})",
                i, pos.0, pos.1
            );
        }
    }

    for _ in 0..10 {
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let final_pos_server = server
        .get_player_position(player1_id)
        .expect("Player 1 should still exist on server");

    let distance_moved = (final_pos_server.0 - initial_pos_server.0).abs();
    println!(
        "\nPlayer 1 moved {:.2} units (should be minimal)",
        distance_moved
    );

    assert!(
        distance_moved > 10.0,
        "Player 1 should have moved significantly (moved: {:.2})",
        distance_moved
    );
}

#[test]
#[should_panic(expected = "should have jumped")]
fn test_no_jump_should_fail() {
    const PORT: u16 = 27966;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing that NO jump is detected (should fail) ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    client1
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();

    let player1_id = client1.player_id().expect("Client 1 should have player ID");

    client2
        .connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 2 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client2.update();

    for _ in 0..180 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let initial_snapshot_c2 = client2
        .last_snapshot()
        .expect("Client 2 should have snapshot");
    let p1_initial_pos = initial_snapshot_c2
        .players
        .iter()
        .find(|p| p.player_id == player1_id)
        .map(|p| p.position)
        .expect("Player 1 should be visible to Client 2");

    println!(
        "Player 1 settled position: ({:.2}, {:.2})",
        p1_initial_pos.0, p1_initial_pos.1
    );
    println!("\n=== Player 1 does NOT jump (button=0 instead of 2) ===\n");

    for i in 0..40 {
        client1.send_input(0.0, 0.0, 0.0, 0).ok();

        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);

        if i % 10 == 0 {
            let pos = server.get_player_position(player1_id).unwrap();
            println!("  [Frame {}] Player 1: ({:.2}, {:.2})", i, pos.0, pos.1);
        }
    }

    for _ in 0..10 {
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let final_snapshot_c2 = client2
        .last_snapshot()
        .expect("Client 2 should have final snapshot");
    let p1_final_seen_by_c2 = final_snapshot_c2
        .players
        .iter()
        .find(|p| p.player_id == player1_id)
        .expect("Player 1 should still be visible to Client 2");

    let p1_moved_y = p1_final_seen_by_c2.position.1 - p1_initial_pos.1;
    println!(
        "\nPlayer 1 vertical movement: {:.2} units (should be minimal)",
        p1_moved_y
    );

    assert!(
        p1_moved_y < -5.0,
        "Player 1 should have jumped (moved vertically), but only moved {:.2} units",
        p1_moved_y
    );
}

#[test]
#[should_panic(expected = "should have moved right")]
fn test_player2_no_movement_should_fail() {
    const PORT: u16 = 27967;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing that Player 2 NO movement is detected (should fail) ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client1 = NetworkClient::new(NetworkConfig::default());
    let mut client2 = NetworkClient::new(NetworkConfig::default());

    client1
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 1 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client1.update();

    let _player1_id = client1.player_id().expect("Client 1 should have player ID");

    client2
        .connect("Player2".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client 2 failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client2.update();

    let player2_id = client2.player_id().expect("Client 2 should have player ID");

    println!("\n=== Waiting 3 seconds for players to settle ===\n");
    for _ in 0..180 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    let initial_snapshot_c1 = client1
        .last_snapshot()
        .expect("Client 1 should have snapshot");
    let p2_initial_pos = initial_snapshot_c1
        .players
        .iter()
        .find(|p| p.player_id == player2_id)
        .map(|p| p.position)
        .expect("Player 2 should be visible to Client 1");

    println!(
        "Player 2 settled position: ({:.2}, {:.2})",
        p2_initial_pos.0, p2_initial_pos.1
    );
    println!("\n=== Player 2 does NOT move right (sends 0.0 instead of 1.0) ===\n");

    for i in 0..40 {
        client2.send_input(0.0, 0.0, 0.0, 0).ok();

        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);

        if i % 10 == 0 {
            let pos = server.get_player_position(player2_id).unwrap();
            println!("  [Frame {}] Player 2: ({:.2}, {:.2})", i, pos.0, pos.1);
        }
    }

    for _ in 0..10 {
        client1.update();
        thread::sleep(Duration::from_millis(16));
    }

    let final_snapshot_c1 = client1
        .last_snapshot()
        .expect("Client 1 should have final snapshot");
    let p2_final_seen_by_c1 = final_snapshot_c1
        .players
        .iter()
        .find(|p| p.player_id == player2_id)
        .expect("Player 2 should still be visible to Client 1");

    let p2_moved_x = p2_final_seen_by_c1.position.0 - p2_initial_pos.0;
    println!(
        "\nPlayer 2 horizontal movement: {:.2} units (should be minimal)",
        p2_moved_x
    );

    assert!(
        p2_moved_x > 10.0,
        "Player 2 should have moved right significantly, but only moved {:.2} units",
        p2_moved_x
    );
}
