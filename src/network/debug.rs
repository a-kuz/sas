use std::collections::VecDeque;

pub struct NetDebug {
    show_packets: bool,
    show_drop: bool,
    show_sync: bool,
    show_physics: bool,
    show_collision: bool,
    packet_log: VecDeque<PacketLogEntry>,
    sync_errors: VecDeque<SyncError>,
    physics_snapshots: VecDeque<PhysicsSnapshot>,
    max_log_entries: usize,
}

#[derive(Clone, Debug)]
pub struct PacketLogEntry {
    pub timestamp: f64,
    pub direction: PacketDirection,
    pub sequence: u32,
    pub size: usize,
    pub message_type: String,
}

#[derive(Clone, Debug)]
pub enum PacketDirection {
    Send,
    Recv,
}

#[derive(Clone, Debug)]
pub struct SyncError {
    pub timestamp: f64,
    pub error_type: SyncErrorType,
    pub details: String,
}

#[derive(Clone, Debug)]
pub enum SyncErrorType {
    OutOfOrder,
    Dropped,
    Duplicate,
    PositionMismatch,
    StateMismatch,
    VelocityMismatch,
    CollisionMismatch,
}

#[derive(Clone, Debug)]
pub struct PhysicsSnapshot {
    pub timestamp: f64,
    pub player_id: u16,
    pub location: PhysicsLocation,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub on_ground: bool,
    pub collided_x: bool,
    pub collided_y: bool,
    pub input_sequence: u32,
}

#[derive(Clone, Debug)]
pub enum PhysicsLocation {
    Client,
    Server,
    ClientPrediction,
}

impl NetDebug {
    pub fn new() -> Self {
        Self {
            show_packets: false,
            show_drop: false,
            show_sync: false,
            show_physics: false,
            show_collision: false,
            packet_log: VecDeque::new(),
            sync_errors: VecDeque::new(),
            physics_snapshots: VecDeque::new(),
            max_log_entries: 100,
        }
    }

    pub fn set_show_packets(&mut self, enabled: bool) {
        self.show_packets = enabled;
    }

    pub fn set_show_drop(&mut self, enabled: bool) {
        self.show_drop = enabled;
    }

    pub fn set_show_sync(&mut self, enabled: bool) {
        self.show_sync = enabled;
    }

    pub fn set_show_physics(&mut self, enabled: bool) {
        self.show_physics = enabled;
    }

    pub fn set_show_collision(&mut self, enabled: bool) {
        self.show_collision = enabled;
    }

    pub fn log_packet_send(&mut self, sequence: u32, size: usize, message_type: &str) {
        if self.show_packets {
            eprintln!("[{:.3}] [NET] send s={} size={} type={}", 
                super::get_network_time(), sequence, size, message_type);
        }

        let entry = PacketLogEntry {
            timestamp: super::get_network_time(),
            direction: PacketDirection::Send,
            sequence,
            size,
            message_type: message_type.to_string(),
        };

        self.packet_log.push_back(entry);
        if self.packet_log.len() > self.max_log_entries {
            self.packet_log.pop_front();
        }
    }

    pub fn log_packet_recv(&mut self, sequence: u32, size: usize, message_type: &str) {
        if self.show_packets {
            eprintln!("[{:.3}] [NET] recv s={} size={} type={}", 
                super::get_network_time(), sequence, size, message_type);
        }

        let entry = PacketLogEntry {
            timestamp: super::get_network_time(),
            direction: PacketDirection::Recv,
            sequence,
            size,
            message_type: message_type.to_string(),
        };

        self.packet_log.push_back(entry);
        if self.packet_log.len() > self.max_log_entries {
            self.packet_log.pop_front();
        }
    }

    pub fn log_out_of_order(&mut self, sequence: u32, expected: u32) {
        if self.show_drop {
            eprintln!("[{:.3}] [NET] Out of order packet {} (expected {})", 
                super::get_network_time(), sequence, expected);
        }

        let error = SyncError {
            timestamp: super::get_network_time(),
            error_type: SyncErrorType::OutOfOrder,
            details: format!("seq={} expected={}", sequence, expected),
        };

        self.sync_errors.push_back(error);
        if self.sync_errors.len() > self.max_log_entries {
            self.sync_errors.pop_front();
        }
    }

    pub fn log_dropped_packets(&mut self, count: u32, at_sequence: u32) {
        if self.show_drop {
            eprintln!("[{:.3}] [NET] Dropped {} packets at sequence {}", 
                super::get_network_time(), count, at_sequence);
        }

        let error = SyncError {
            timestamp: super::get_network_time(),
            error_type: SyncErrorType::Dropped,
            details: format!("count={} at={}", count, at_sequence),
        };

        self.sync_errors.push_back(error);
        if self.sync_errors.len() > self.max_log_entries {
            self.sync_errors.pop_front();
        }
    }

    pub fn log_position_mismatch(&mut self, 
        player_id: u16, 
        local_pos: (f32, f32), 
        server_pos: (f32, f32)
    ) {
        let distance = ((local_pos.0 - server_pos.0).powi(2) + (local_pos.1 - server_pos.1).powi(2)).sqrt();
        
        if self.show_sync {
            eprintln!(
                "[{:.3}] [SYNC] Position mismatch p{} local=({:.1},{:.1}) server=({:.1},{:.1}) dist={:.1}",
                super::get_network_time(), player_id, local_pos.0, local_pos.1, server_pos.0, server_pos.1, distance
            );
        }

        let error = SyncError {
            timestamp: super::get_network_time(),
            error_type: SyncErrorType::PositionMismatch,
            details: format!(
                "p{} local=({:.1},{:.1}) server=({:.1},{:.1}) dist={:.1}",
                player_id, local_pos.0, local_pos.1, server_pos.0, server_pos.1, distance
            ),
        };

        self.sync_errors.push_back(error);
        if self.sync_errors.len() > self.max_log_entries {
            self.sync_errors.pop_front();
        }
    }

    pub fn log_physics_state(
        &mut self,
        player_id: u16,
        location: PhysicsLocation,
        position: (f32, f32),
        velocity: (f32, f32),
        on_ground: bool,
        collided_x: bool,
        collided_y: bool,
        input_sequence: u32,
    ) {
        if self.show_physics {
            eprintln!(
                "[{:.3}] [PHYS] {:?} p{} seq={} pos=({:.1},{:.1}) vel=({:.2},{:.2}) ground={} coll_x={} coll_y={}",
                super::get_network_time(), location, player_id, input_sequence, 
                position.0, position.1, velocity.0, velocity.1,
                on_ground, collided_x, collided_y
            );
        }

        let snapshot = PhysicsSnapshot {
            timestamp: super::get_network_time(),
            player_id,
            location,
            position,
            velocity,
            on_ground,
            collided_x,
            collided_y,
            input_sequence,
        };

        self.physics_snapshots.push_back(snapshot);
        if self.physics_snapshots.len() > self.max_log_entries {
            self.physics_snapshots.pop_front();
        }
    }

    pub fn log_velocity_mismatch(
        &mut self,
        player_id: u16,
        local_vel: (f32, f32),
        server_vel: (f32, f32),
    ) {
        let vel_diff = (
            (local_vel.0 - server_vel.0).abs(),
            (local_vel.1 - server_vel.1).abs()
        );

        if self.show_sync && (vel_diff.0 > 0.5 || vel_diff.1 > 0.5) {
            eprintln!(
                "[{:.3}] [SYNC] Velocity mismatch p{} local=({:.2},{:.2}) server=({:.2},{:.2}) diff=({:.2},{:.2})",
                super::get_network_time(), player_id, local_vel.0, local_vel.1, server_vel.0, server_vel.1, vel_diff.0, vel_diff.1
            );
        }

        let error = SyncError {
            timestamp: super::get_network_time(),
            error_type: SyncErrorType::VelocityMismatch,
            details: format!(
                "p{} local=({:.2},{:.2}) server=({:.2},{:.2})",
                player_id, local_vel.0, local_vel.1, server_vel.0, server_vel.1
            ),
        };

        self.sync_errors.push_back(error);
        if self.sync_errors.len() > self.max_log_entries {
            self.sync_errors.pop_front();
        }
    }

    pub fn log_collision_event(
        &mut self,
        player_id: u16,
        collision_type: &str,
        position: (f32, f32),
        normal: (f32, f32),
    ) {
        if self.show_collision {
            eprintln!(
                "[{:.3}] [COLL] p{} {} at ({:.1},{:.1}) normal=({:.2},{:.2})",
                super::get_network_time(), player_id, collision_type, position.0, position.1, normal.0, normal.1
            );
        }
    }

    pub fn log_impulse_event(
        &mut self,
        player_id: u16,
        impulse_type: &str,
        old_vel: (f32, f32),
        new_vel: (f32, f32),
        position: (f32, f32),
    ) {
        let delta_vel = (new_vel.0 - old_vel.0, new_vel.1 - old_vel.1);
        let impulse_magnitude = (delta_vel.0 * delta_vel.0 + delta_vel.1 * delta_vel.1).sqrt();

        if self.show_physics || impulse_magnitude > 5.0 {
            eprintln!(
                "[{:.3}] [IMPULSE] p{} {} at ({:.1},{:.1}) vel ({:.2},{:.2}) -> ({:.2},{:.2}) delta=({:.2},{:.2}) mag={:.2}",
                super::get_network_time(), player_id, impulse_type,
                position.0, position.1,
                old_vel.0, old_vel.1,
                new_vel.0, new_vel.1,
                delta_vel.0, delta_vel.1,
                impulse_magnitude
            );
        }

        if impulse_magnitude > 10.0 {
            let error = SyncError {
                timestamp: super::get_network_time(),
                error_type: SyncErrorType::StateMismatch,
                details: format!(
                    "p{} {} impulse={:.1} at ({:.1},{:.1})",
                    player_id, impulse_type, impulse_magnitude, position.0, position.1
                ),
            };
            self.sync_errors.push_back(error);
            if self.sync_errors.len() > self.max_log_entries {
                self.sync_errors.pop_front();
            }
        }
    }

    pub fn compare_physics_snapshots(&self, player_id: u16, input_seq: u32) -> Option<PhysicsDiff> {
        let client = self.physics_snapshots.iter()
            .rev()
            .find(|s| s.player_id == player_id && 
                     s.input_sequence == input_seq &&
                     matches!(s.location, PhysicsLocation::Client))?;

        let server = self.physics_snapshots.iter()
            .rev()
            .find(|s| s.player_id == player_id && 
                     s.input_sequence == input_seq &&
                     matches!(s.location, PhysicsLocation::Server))?;

        let pos_diff = (
            (client.position.0 - server.position.0).abs(),
            (client.position.1 - server.position.1).abs()
        );

        let vel_diff = (
            (client.velocity.0 - server.velocity.0).abs(),
            (client.velocity.1 - server.velocity.1).abs()
        );

        Some(PhysicsDiff {
            position_diff: pos_diff,
            velocity_diff: vel_diff,
            on_ground_mismatch: client.on_ground != server.on_ground,
            collision_mismatch: client.collided_x != server.collided_x || 
                              client.collided_y != server.collided_y,
        })
    }

    pub fn get_packet_stats(&self) -> PacketStats {
        let total_packets = self.packet_log.len();
        let sent_packets = self.packet_log.iter()
            .filter(|p| matches!(p.direction, PacketDirection::Send))
            .count();
        let recv_packets = self.packet_log.iter()
            .filter(|p| matches!(p.direction, PacketDirection::Recv))
            .count();

        let dropped_count = self.sync_errors.iter()
            .filter(|e| matches!(e.error_type, SyncErrorType::Dropped))
            .count();

        let out_of_order_count = self.sync_errors.iter()
            .filter(|e| matches!(e.error_type, SyncErrorType::OutOfOrder))
            .count();

        PacketStats {
            total_packets,
            sent_packets,
            recv_packets,
            dropped_count,
            out_of_order_count,
        }
    }

    pub fn get_recent_errors(&self, count: usize) -> Vec<SyncError> {
        self.sync_errors.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    pub fn clear_logs(&mut self) {
        self.packet_log.clear();
        self.sync_errors.clear();
    }
}

#[derive(Clone, Debug)]
pub struct PacketStats {
    pub total_packets: usize,
    pub sent_packets: usize,
    pub recv_packets: usize,
    pub dropped_count: usize,
    pub out_of_order_count: usize,
}

#[derive(Clone, Debug)]
pub struct PhysicsDiff {
    pub position_diff: (f32, f32),
    pub velocity_diff: (f32, f32),
    pub on_ground_mismatch: bool,
    pub collision_mismatch: bool,
}

impl PhysicsDiff {
    pub fn is_significant(&self) -> bool {
        self.position_diff.0 > 1.0 || 
        self.position_diff.1 > 1.0 ||
        self.velocity_diff.0 > 0.5 ||
        self.velocity_diff.1 > 0.5 ||
        self.on_ground_mismatch ||
        self.collision_mismatch
    }

    pub fn max_position_error(&self) -> f32 {
        self.position_diff.0.max(self.position_diff.1)
    }

    pub fn max_velocity_error(&self) -> f32 {
        self.velocity_diff.0.max(self.velocity_diff.1)
    }
}

impl Default for NetDebug {
    fn default() -> Self {
        Self::new()
    }
}

