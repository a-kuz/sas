mod common;

use common::TestServer;
use sas::network::{NetworkClient, NetworkConfig};
use std::thread;
use std::time::Duration;

#[test]
fn test_interpolation_smoothness() {
    const PORT: u16 = 27970;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing interpolation smoothness ===\n");

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

    let _player2_id = client2.player_id().expect("Client 2 should have player ID");

    println!("\n=== Settling ===\n");
    for _ in 0..60 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client1.update();
        client2.update();
        thread::sleep(Duration::from_millis(16));
    }

    println!("\n=== Player 1 moving right for 2 seconds ===\n");

    let mut interpolated_positions = Vec::new();
    let mut snapshot_count = 0;

    for frame in 0..120 {
        client1.send_input(0.0, 1.0, 0.0, 0).ok();

        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);
        client2.update();

        if client2.last_snapshot().is_some() {
            snapshot_count += 1;
        }

        let interp_time = client2.get_interpolation_time();
        if let Some(interpolated) = client2.interpolate_player(player1_id, interp_time) {
            interpolated_positions.push((frame, interpolated.position));

            if frame % 30 == 0 {
                println!(
                    "  [Frame {}] Interpolated position: ({:.2}, {:.2})",
                    frame, interpolated.position.0, interpolated.position.1
                );
            }
        }
    }

    println!("\n=== Analysis ===");
    println!("Total frames: 120");
    println!("Snapshots received: {}", snapshot_count);
    println!(
        "Successful interpolations: {}",
        interpolated_positions.len()
    );

    assert!(
        snapshot_count > 50,
        "Should receive many snapshots (got {})",
        snapshot_count
    );
    assert!(
        interpolated_positions.len() > 100,
        "Should interpolate most frames (got {})",
        interpolated_positions.len()
    );

    let mut max_frame_jump = 0.0f32;
    for i in 1..interpolated_positions.len() {
        let (_, pos_prev) = interpolated_positions[i - 1];
        let (frame_curr, pos_curr) = interpolated_positions[i];

        let dx = pos_curr.0 - pos_prev.0;
        let dy = pos_curr.1 - pos_prev.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > max_frame_jump {
            max_frame_jump = dist;
            if dist > 10.0 {
                println!("  Large jump at frame {}: {:.2}px", frame_curr, dist);
            }
        }
    }

    println!(
        "\nMax position jump between frames: {:.2}px",
        max_frame_jump
    );

    assert!(
        max_frame_jump < 20.0,
        "Position should not jump more than 20px between frames (max: {:.2}px)",
        max_frame_jump
    );

    let total_distance = {
        let first_pos = interpolated_positions.first().unwrap().1;
        let last_pos = interpolated_positions.last().unwrap().1;
        (last_pos.0 - first_pos.0).abs()
    };

    println!("Total distance moved: {:.2}px", total_distance);
    assert!(
        total_distance > 50.0,
        "Player should have moved significantly (moved: {:.2}px)",
        total_distance
    );

    println!("\n=== Test PASSED ===\n");
}

#[test]
fn test_base_cmd_updates_regularly() {
    const PORT: u16 = 27971;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing that base_cmd updates regularly ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client = NetworkClient::new(NetworkConfig::default());

    client
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client.update();

    let player_id = client.player_id().expect("Client should have player ID");
    println!("Player connected with ID: {}", player_id);

    println!("\n=== Settling ===\n");
    for _ in 0..60 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client.update();
        thread::sleep(Duration::from_millis(16));
    }

    let initial_snapshot = client.last_snapshot().expect("Should have snapshot");
    let initial_cmd_time = initial_snapshot
        .players
        .iter()
        .find(|p| p.player_id == player_id)
        .map(|p| p.command_time)
        .expect("Should find player");

    println!("Initial command_time: {}", initial_cmd_time);
    println!("\n=== Player NOT moving (no input) for 1 second ===\n");

    let mut cmd_time_updates = Vec::new();

    for frame in 0..60 {
        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);
        client.update();

        if let Some(snapshot) = client.last_snapshot() {
            if let Some(player) = snapshot.players.iter().find(|p| p.player_id == player_id) {
                if player.command_time != initial_cmd_time {
                    cmd_time_updates.push((frame, player.command_time));
                }

                if frame % 30 == 0 {
                    println!("  [Frame {}] command_time: {}", frame, player.command_time);
                }
            }
        }
    }

    println!("\n=== Analysis ===");
    println!("command_time updates: {}", cmd_time_updates.len());

    if !cmd_time_updates.is_empty() {
        let final_cmd_time = cmd_time_updates.last().unwrap().1;
        let cmd_time_delta = final_cmd_time - initial_cmd_time;
        println!(
            "command_time changed from {} to {} (delta: {}ms)",
            initial_cmd_time, final_cmd_time, cmd_time_delta
        );

        assert!(
            cmd_time_delta > 500,
            "command_time should update even without input (delta: {}ms)",
            cmd_time_delta
        );
    } else {
        panic!("command_time NEVER updated! This causes choppy movement!");
    }

    println!("\n=== Test PASSED ===\n");
}

#[test]
fn test_prediction_error_stays_low() {
    const PORT: u16 = 27972;
    const FIXED_DT: f32 = 1.0 / 60.0;

    println!("\n=== Testing prediction error stays low ===\n");

    let mut server = TestServer::new(PORT);
    server.start().expect("Failed to start server");

    thread::sleep(Duration::from_millis(100));

    let mut client = NetworkClient::new(NetworkConfig::default());

    client
        .connect("Player1".to_string(), &format!("127.0.0.1:{}", PORT))
        .expect("Client failed to connect");

    thread::sleep(Duration::from_millis(50));
    server.update();
    thread::sleep(Duration::from_millis(50));
    client.update();

    let _player_id = client.player_id().expect("Client should have player ID");

    println!("\n=== Settling ===\n");
    for _ in 0..60 {
        server.update();
        server.simulate_physics(FIXED_DT);
        client.update();
        thread::sleep(Duration::from_millis(16));
    }

    println!("\n=== Moving right for 2 seconds, tracking prediction error ===\n");

    let mut max_error = 0.0f32;
    let mut error_samples = Vec::new();

    for frame in 0..120 {
        client.send_input(0.0, 1.0, 0.0, 0).ok();

        thread::sleep(Duration::from_millis(16));

        server.update();
        server.simulate_physics(FIXED_DT);
        client.update();

        if let Some(error) = client.get_prediction().get_prediction_error() {
            if error.magnitude > max_error {
                max_error = error.magnitude;
            }
            error_samples.push(error.magnitude);

            if frame % 30 == 0 {
                println!(
                    "  [Frame {}] Prediction error: {:.2}px",
                    frame, error.magnitude
                );
            }
        }
    }

    println!("\n=== Analysis ===");
    println!("Error samples collected: {}", error_samples.len());

    if !error_samples.is_empty() {
        let avg_error: f32 = error_samples.iter().sum::<f32>() / error_samples.len() as f32;
        println!("Average prediction error: {:.2}px", avg_error);
        println!("Max prediction error: {:.2}px", max_error);

        assert!(
            max_error < 50.0,
            "Max prediction error should be < 50px (got {:.2}px)",
            max_error
        );

        assert!(
            avg_error < 20.0,
            "Average prediction error should be < 20px (got {:.2}px)",
            avg_error
        );
    }

    println!("\n=== Test PASSED ===\n");
}
