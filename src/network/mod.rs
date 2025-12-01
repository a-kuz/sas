pub mod client;
pub mod client_prediction;
pub mod debug;
pub mod interpolation;
pub mod net_hud;
pub mod net_stats;
pub mod prediction;
pub mod prediction_debug;
pub mod protocol;
pub mod server;
pub mod snapshot_delta;
pub mod trajectory;

pub use client::NetworkClient;
pub use client_prediction::{ClientPrediction, PredictedPlayerState, PredictionError};
pub use debug::NetDebug;
pub use interpolation::{InterpolatedPlayer, InterpolatedProjectile, SnapshotBuffer};
pub use net_hud::NetHud;
pub use net_stats::NetStats;
pub use prediction::{CommandBuffer, UserCommand, CMD_BACKUP};
pub use prediction_debug::PredictionDebugRenderer;
pub use server::NetworkServer;
pub use snapshot_delta::{DummySnapshot, PlayerStateDelta, ProjectileStateDelta, SnapshotDelta};
pub use trajectory::{ProjectileTrajectory, Trajectory, TrajectoryType};

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Instant;

pub const PACKET_BACKUP: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerInputCmd {
    pub move_forward: f32,
    pub move_right: f32,
    pub angle: f32,
    pub buttons: u32,
    pub server_time: u32,
}

static START_TIME: OnceLock<Instant> = OnceLock::new();

pub fn get_network_time() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let start = START_TIME.get_or_init(|| Instant::now());
        start.elapsed().as_secs_f64()
    }

    #[cfg(target_arch = "wasm32")]
    {
        macroquad::prelude::get_time()
    }
}

pub fn get_absolute_time() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let total_secs = now.as_secs();
    let millis = now.subsec_millis();

    let secs_in_day = total_secs % 86400;
    let hours = (secs_in_day / 3600) % 24;
    let minutes = (secs_in_day / 60) % 60;
    let seconds = secs_in_day % 60;

    format!(
        "{:02}:{:02}:{:02}.{:02}",
        hours,
        minutes,
        seconds,
        millis / 10
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetMessage {
    ConnectRequest {
        player_name: String,
        protocol_version: u32,
    },
    ConnectResponse {
        player_id: u16,
        accepted: bool,
        reason: String,
    },
    Disconnect {
        player_id: u16,
        reason: String,
    },
    PlayerSnapshot {
        player_id: u16,
        position: (f32, f32),
        velocity: (f32, f32),
        angle: f32,
        health: i32,
        armor: i32,
        weapon: u8,
    },
    GameStateSnapshot {
        tick: u32,
        players: Vec<PlayerState>,
        projectiles: Vec<ProjectileState>,
    },
    GameStateDelta {
        tick: u32,
        base_message_num: u32,
        player_deltas: Vec<PlayerStateDelta>,
        projectile_deltas: Vec<ProjectileStateDelta>,
        new_projectiles: Vec<ProjectileState>,
        removed_projectiles: Vec<u32>,
    },
    SnapshotAck {
        player_id: u16,
        acknowledged_tick: u32,
    },
    WeaponSwitch {
        player_id: u16,
        weapon: u8,
    },
    PlayerInput {
        player_id: u16,
        input_sequence: u32,
        move_forward: f32,
        move_right: f32,
        angle: f32,
        buttons: u32,
        server_time: u32,
    },
    PlayerInputBatch {
        player_id: u16,
        commands: Vec<PlayerInputCmd>,
    },
    PlayerShoot {
        player_id: u16,
        weapon: u8,
        origin: (f32, f32),
        direction: f32,
    },
    ProjectileSpawned {
        id: u32,
        owner_id: u16,
        weapon: u8,
        x: f32,
        y: f32,
        vel_x: f32,
        vel_y: f32,
        spawn_time: u32,
    },
    PlayerDamaged {
        target_id: u16,
        attacker_id: u16,
        damage: i32,
        health_remaining: i32,
        knockback_x: f32,
        knockback_y: f32,
    },
    PlayerDied {
        player_id: u16,
        killer_id: u16,
        gibbed: bool,
        position: (f32, f32),
        velocity: (f32, f32),
    },
    PlayerGibbed {
        player_id: u16,
        position: (f32, f32),
    },
    PlayerRespawn {
        player_id: u16,
        position: (f32, f32),
    },
    Chat {
        player_id: u16,
        message: String,
    },
    ServerInfo {
        map_name: String,
        gametype: u8,
        max_players: u8,
        current_players: u8,
    },
    MapChange {
        map_name: String,
    },
    Heartbeat,
    Acknowledgement {
        sequence: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerState {
    pub player_id: u16,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub angle: f32,
    pub health: i32,
    pub armor: i32,
    pub weapon: u8,
    pub ammo: [u16; 10],
    pub frags: i32,
    pub deaths: i32,
    pub powerup_quad: u16,
    pub on_ground: bool,
    pub is_crouching: bool,
    pub is_attacking: bool,
    pub is_dead: bool,
    pub command_time: u32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            player_id: 0,
            command_time: 0,
            position: (0.0, 0.0),
            velocity: (0.0, 0.0),
            angle: 0.0,
            health: 0,
            armor: 0,
            weapon: 0,
            ammo: [0; 10],
            frags: 0,
            deaths: 0,
            powerup_quad: 0,
            on_ground: false,
            is_crouching: false,
            is_attacking: false,
            is_dead: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProjectileState {
    pub id: u32,
    pub trajectory: Trajectory,
    pub weapon_type: u8,
    pub owner_id: u16,
    pub spawn_time: u32,
}

impl Default for ProjectileState {
    fn default() -> Self {
        Self {
            id: 0,
            trajectory: Trajectory::default(),
            weapon_type: 0,
            owner_id: 0,
            spawn_time: 0,
        }
    }
}

pub struct NetworkConfig {
    pub server_address: String,
    pub server_port: u16,
    pub max_players: u8,
    pub tick_rate: u32,
    pub protocol_version: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0".to_string(),
            server_port: 27960,
            max_players: 16,
            tick_rate: 60,
            protocol_version: 1,
        }
    }
}
