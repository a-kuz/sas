use super::{PlayerState, ProjectileState};
use serde::{Serialize, Deserialize};

pub struct DummySnapshot;

impl DummySnapshot {
    pub fn player_state() -> PlayerState {
        PlayerState::default()
    }
    
    pub fn projectile_state() -> ProjectileState {
        ProjectileState::default()
    }
}

pub struct SnapshotDelta {
    dummy_player: PlayerState,
    dummy_projectile: ProjectileState,
}

impl SnapshotDelta {
    pub fn new() -> Self {
        Self {
            dummy_player: PlayerState::default(),
            dummy_projectile: ProjectileState::default(),
        }
    }
    
    pub fn get_dummy_player(&self) -> &PlayerState {
        &self.dummy_player
    }
    
    pub fn get_dummy_projectile(&self) -> &ProjectileState {
        &self.dummy_projectile
    }
    
    pub fn compare_players(&self, old: &PlayerState, new: &PlayerState) -> PlayerStateDelta {
        PlayerStateDelta {
            player_id: new.player_id,
            command_time: if old.command_time != new.command_time { Some(new.command_time) } else { None },
            position: if old.position != new.position { Some(new.position) } else { None },
            velocity: if old.velocity != new.velocity { Some(new.velocity) } else { None },
            angle: if old.angle != new.angle { Some(new.angle) } else { None },
            health: if old.health != new.health { Some(new.health) } else { None },
            armor: if old.armor != new.armor { Some(new.armor) } else { None },
            weapon: if old.weapon != new.weapon { Some(new.weapon) } else { None },
            ammo: if old.ammo != new.ammo { Some(new.ammo) } else { None },
            frags: if old.frags != new.frags { Some(new.frags) } else { None },
            deaths: if old.deaths != new.deaths { Some(new.deaths) } else { None },
            powerup_quad: if old.powerup_quad != new.powerup_quad { Some(new.powerup_quad) } else { None },
            on_ground: if old.on_ground != new.on_ground { Some(new.on_ground) } else { None },
            is_crouching: if old.is_crouching != new.is_crouching { Some(new.is_crouching) } else { None },
            is_attacking: if old.is_attacking != new.is_attacking { Some(new.is_attacking) } else { None },
            is_dead: if old.is_dead != new.is_dead { Some(new.is_dead) } else { None },
        }
    }
    
    pub fn compare_projectiles(&self, old: &ProjectileState, new: &ProjectileState) -> ProjectileStateDelta {
        ProjectileStateDelta {
            id: new.id,
            trajectory: if old.trajectory != new.trajectory { Some(new.trajectory.clone()) } else { None },
            weapon_type: if old.weapon_type != new.weapon_type { Some(new.weapon_type) } else { None },
            owner_id: if old.owner_id != new.owner_id { Some(new.owner_id) } else { None },
            spawn_time: if old.spawn_time != new.spawn_time { Some(new.spawn_time) } else { None },
        }
    }
}

impl Default for SnapshotDelta {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStateDelta {
    pub player_id: u16,
    pub command_time: Option<u32>,
    pub position: Option<(f32, f32)>,
    pub velocity: Option<(f32, f32)>,
    pub angle: Option<f32>,
    pub health: Option<i32>,
    pub armor: Option<i32>,
    pub weapon: Option<u8>,
    pub ammo: Option<[u16; 10]>,
    pub frags: Option<i32>,
    pub deaths: Option<i32>,
    pub powerup_quad: Option<u16>,
    pub on_ground: Option<bool>,
    pub is_crouching: Option<bool>,
    pub is_attacking: Option<bool>,
    pub is_dead: Option<bool>,
}

impl PlayerStateDelta {
    pub fn count_changed_fields(&self) -> usize {
        let mut count = 0;
        if self.command_time.is_some() { count += 1; }
        if self.position.is_some() { count += 1; }
        if self.is_dead.is_some() { count += 1; }
        if self.velocity.is_some() { count += 1; }
        if self.angle.is_some() { count += 1; }
        if self.health.is_some() { count += 1; }
        if self.armor.is_some() { count += 1; }
        if self.weapon.is_some() { count += 1; }
        if self.ammo.is_some() { count += 1; }
        if self.frags.is_some() { count += 1; }
        if self.deaths.is_some() { count += 1; }
        if self.powerup_quad.is_some() { count += 1; }
        if self.on_ground.is_some() { count += 1; }
        if self.is_crouching.is_some() { count += 1; }
        if self.is_attacking.is_some() { count += 1; }
        count
    }
    
    pub fn is_full_update(&self) -> bool {
        self.count_changed_fields() >= 10
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectileStateDelta {
    pub id: u32,
    pub trajectory: Option<super::Trajectory>,
    pub weapon_type: Option<u8>,
    pub owner_id: Option<u16>,
    pub spawn_time: Option<u32>,
}

impl ProjectileStateDelta {
    pub fn count_changed_fields(&self) -> usize {
        let mut count = 0;
        if self.trajectory.is_some() { count += 1; }
        if self.weapon_type.is_some() { count += 1; }
        if self.owner_id.is_some() { count += 1; }
        if self.spawn_time.is_some() { count += 1; }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dummy_snapshot_all_zeros() {
        let player = DummySnapshot::player_state();
        assert_eq!(player.player_id, 0);
        assert_eq!(player.position, (0.0, 0.0));
        assert_eq!(player.health, 0);
        assert_eq!(player.armor, 0);
    }
    
    #[test]
    fn test_delta_no_changes() {
        let delta_gen = SnapshotDelta::new();
        let player1 = PlayerState {
            player_id: 1,
            position: (100.0, 200.0),
            health: 100,
            ..Default::default()
        };
        let player2 = player1.clone();
        
        let delta = delta_gen.compare_players(&player1, &player2);
        assert_eq!(delta.count_changed_fields(), 0);
    }
    
    #[test]
    fn test_delta_position_changed() {
        let delta_gen = SnapshotDelta::new();
        let player1 = PlayerState {
            player_id: 1,
            position: (100.0, 200.0),
            health: 100,
            ..Default::default()
        };
        let player2 = PlayerState {
            position: (150.0, 200.0),
            ..player1
        };
        
        let delta = delta_gen.compare_players(&player1, &player2);
        assert_eq!(delta.count_changed_fields(), 1);
        assert!(delta.position.is_some());
        assert_eq!(delta.position.unwrap(), (150.0, 200.0));
    }
    
    #[test]
    fn test_full_update_from_dummy() {
        let delta_gen = SnapshotDelta::new();
        let dummy = delta_gen.get_dummy_player();
        let player = PlayerState {
            player_id: 1,
            position: (100.0, 200.0),
            velocity: (10.0, 5.0),
            angle: 1.5,
            health: 100,
            armor: 50,
            weapon: 2,
            ammo: [10, 20, 5, 5, 5, 0, 0, 0, 0, 0],
            frags: 5,
            deaths: 2,
            powerup_quad: 100,
            on_ground: true,
            is_crouching: false,
            is_attacking: true,
            command_time: 0,
            is_dead: false,
        };
        
        let delta = delta_gen.compare_players(dummy, &player);
        let changed = delta.count_changed_fields();
        assert!(changed >= 10, "Expected at least 10 changed fields, got {}", changed);
    }
}

