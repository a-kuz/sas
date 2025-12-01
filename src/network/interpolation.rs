use super::{PlayerState, ProjectileState};
use std::collections::VecDeque;

const MAX_SNAPSHOTS: usize = 16;

pub struct SnapshotBuffer {
    snapshots: VecDeque<SnapshotEntry>,
}

#[derive(Clone, Debug)]
struct SnapshotEntry {
    pub tick: u32,
    pub timestamp: f64,
    pub players: Vec<PlayerState>,
    pub projectiles: Vec<ProjectileState>,
}

impl SnapshotBuffer {
    pub fn new() -> Self {
        Self {
            snapshots: VecDeque::with_capacity(MAX_SNAPSHOTS),
        }
    }

    pub fn add_snapshot(
        &mut self,
        tick: u32,
        timestamp: f64,
        players: Vec<PlayerState>,
        projectiles: Vec<ProjectileState>,
    ) {
        let entry = SnapshotEntry {
            tick,
            timestamp,
            players,
            projectiles,
        };

        self.snapshots.push_back(entry);

        while self.snapshots.len() > MAX_SNAPSHOTS {
            self.snapshots.pop_front();
        }
    }

    pub fn interpolate_player(
        &self,
        player_id: u16,
        render_time: f64,
    ) -> Option<InterpolatedPlayer> {
        if self.snapshots.is_empty() {
            return None;
        }

        if self.snapshots.len() == 1 {
            let snap = &self.snapshots[0];
            let player = snap.players.iter().find(|p| p.player_id == player_id)?;
            let time_ahead = (render_time - snap.timestamp) as f32;

            return Some(InterpolatedPlayer {
                position: (
                    player.position.0 + player.velocity.0 * time_ahead,
                    player.position.1 + player.velocity.1 * time_ahead,
                ),
                angle: player.angle,
                velocity: player.velocity,
                health: player.health,
                armor: player.armor,
                weapon: player.weapon,
                on_ground: player.on_ground,
                is_crouching: player.is_crouching,
                is_attacking: player.is_attacking,
            });
        }

        let latest_snap = self.snapshots.back().unwrap();

        if render_time > latest_snap.timestamp {
            let player = latest_snap
                .players
                .iter()
                .find(|p| p.player_id == player_id)?;
            let time_ahead = (render_time - latest_snap.timestamp) as f32;
            let max_extrapolation = 0.05;

            static mut LAST_EXTRAP_PRINT: f64 = 0.0;
            unsafe {
                if super::get_network_time() - LAST_EXTRAP_PRINT > 2.0 {
                    println!("[INTERP] EXTRAPOLATING player {} render={:.3} latest={:.3} ahead={:.0}ms snaps={}", 
                        player_id, render_time, latest_snap.timestamp, time_ahead * 1000.0, self.snapshots.len());
                    LAST_EXTRAP_PRINT = super::get_network_time();
                }
            }

            if time_ahead < max_extrapolation {
                return Some(InterpolatedPlayer {
                    position: (
                        player.position.0 + player.velocity.0 * time_ahead,
                        player.position.1 + player.velocity.1 * time_ahead,
                    ),
                    angle: player.angle,
                    velocity: player.velocity,
                    health: player.health,
                    armor: player.armor,
                    weapon: player.weapon,
                    on_ground: player.on_ground,
                    is_crouching: player.is_crouching,
                    is_attacking: player.is_attacking,
                });
            } else {
                return Some(InterpolatedPlayer {
                    position: player.position,
                    angle: player.angle,
                    velocity: player.velocity,
                    health: player.health,
                    armor: player.armor,
                    weapon: player.weapon,
                    on_ground: player.on_ground,
                    is_crouching: player.is_crouching,
                    is_attacking: player.is_attacking,
                });
            }
        }

        let (from_idx, to_idx) = self.find_bracketing_snapshots(render_time)?;

        let from_snap = &self.snapshots[from_idx];
        let to_snap = &self.snapshots[to_idx];

        let from_player = from_snap
            .players
            .iter()
            .find(|p| p.player_id == player_id)?;
        let to_player = to_snap.players.iter().find(|p| p.player_id == player_id)?;

        let time_delta = to_snap.timestamp - from_snap.timestamp;
        if time_delta <= 0.0 {
            return Some(InterpolatedPlayer {
                position: to_player.position,
                angle: to_player.angle,
                velocity: to_player.velocity,
                health: to_player.health,
                armor: to_player.armor,
                weapon: to_player.weapon,
                on_ground: to_player.on_ground,
                is_crouching: to_player.is_crouching,
                is_attacking: to_player.is_attacking,
            });
        }

        let fraction = ((render_time - from_snap.timestamp) / time_delta).clamp(0.0, 1.0);
        let smoothed_fraction = smoothstep(fraction as f32);

        Some(InterpolatedPlayer {
            position: (
                from_player.position.0
                    + (to_player.position.0 - from_player.position.0) * smoothed_fraction,
                from_player.position.1
                    + (to_player.position.1 - from_player.position.1) * smoothed_fraction,
            ),
            angle: lerp_angle(from_player.angle, to_player.angle, smoothed_fraction),
            velocity: (
                from_player.velocity.0
                    + (to_player.velocity.0 - from_player.velocity.0) * smoothed_fraction,
                from_player.velocity.1
                    + (to_player.velocity.1 - from_player.velocity.1) * smoothed_fraction,
            ),
            health: to_player.health,
            armor: to_player.armor,
            weapon: to_player.weapon,
            on_ground: to_player.on_ground,
            is_crouching: to_player.is_crouching,
            is_attacking: to_player.is_attacking,
        })
    }

    pub fn interpolate_projectile(
        &self,
        projectile_id: u32,
        render_time: f64,
    ) -> Option<InterpolatedProjectile> {
        if let Some(snap) = self.snapshots.back() {
            let proj = snap.projectiles.iter().find(|p| p.id == projectile_id)?;
            let current_time_ms = (render_time * 1000.0) as u32;
            let pos = proj.trajectory.evaluate(current_time_ms);
            let vel = proj.trajectory.evaluate_velocity(current_time_ms);

            return Some(InterpolatedProjectile {
                position: pos,
                velocity: vel,
            });
        }
        None
    }

    fn find_bracketing_snapshots(&self, render_time: f64) -> Option<(usize, usize)> {
        for i in 0..self.snapshots.len().saturating_sub(1) {
            let from_snap = &self.snapshots[i];
            let to_snap = &self.snapshots[i + 1];

            // include equality on the upper bound to avoid brief gaps at boundaries
            if from_snap.timestamp <= render_time && to_snap.timestamp >= render_time {
                return Some((i, i + 1));
            }
        }

        None
    }

    pub fn get_latest_snapshot_time(&self) -> Option<f64> {
        self.snapshots.back().map(|s| s.timestamp)
    }
}

impl Default for SnapshotBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct InterpolatedPlayer {
    pub position: (f32, f32),
    pub angle: f32,
    pub velocity: (f32, f32),
    pub health: i32,
    pub armor: i32,
    pub weapon: u8,
    pub on_ground: bool,
    pub is_crouching: bool,
    pub is_attacking: bool,
}

#[derive(Clone, Debug)]
pub struct InterpolatedProjectile {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
}

fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let mut diff = to - from;

    while diff > std::f32::consts::PI {
        diff -= 2.0 * std::f32::consts::PI;
    }
    while diff < -std::f32::consts::PI {
        diff += 2.0 * std::f32::consts::PI;
    }

    from + diff * t
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_buffer_add() {
        let mut buffer = SnapshotBuffer::new();

        let player = PlayerState {
            player_id: 1,
            position: (100.0, 200.0),
            ..Default::default()
        };

        buffer.add_snapshot(1, 0.0, vec![player], vec![]);

        assert_eq!(buffer.snapshots.len(), 1);
    }

    #[test]
    fn test_interpolation_basic() {
        let mut buffer = SnapshotBuffer::new();

        std::thread::sleep(std::time::Duration::from_millis(10));
        let t1 = super::super::get_network_time();

        let player1 = PlayerState {
            player_id: 1,
            position: (0.0, 0.0),
            ..Default::default()
        };
        let t1 = super::super::get_network_time();
        buffer.add_snapshot(1, t1, vec![player1], vec![]);

        std::thread::sleep(std::time::Duration::from_millis(100));
        let t2 = super::super::get_network_time();

        let player2 = PlayerState {
            player_id: 1,
            position: (100.0, 0.0),
            ..Default::default()
        };
        let t2 = super::super::get_network_time();
        buffer.add_snapshot(2, t2, vec![player2], vec![]);

        let mid_time = (t1 + t2) / 2.0;
        let interpolated = buffer.interpolate_player(1, mid_time).unwrap();

        assert!(interpolated.position.0 > 25.0 && interpolated.position.0 < 75.0);
    }

    #[test]
    fn test_lerp_angle() {
        let angle1 = lerp_angle(0.0, std::f32::consts::PI, 0.5);
        assert!((angle1 - std::f32::consts::PI / 2.0).abs() < 0.01);

        let angle2 = lerp_angle(0.1, 6.2, 0.5);
        assert!(angle2 < 1.0 || angle2 > 5.0);
    }
}
