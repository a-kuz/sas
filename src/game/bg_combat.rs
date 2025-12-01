pub struct CombatEvent {
    pub attacker_id: u16,
    pub target_id: u16,
    pub damage: i32,
}

pub struct KillEvent {
    pub killer_id: u16,
    pub victim_id: u16,
}

pub struct CombatResult {
    pub damage_events: Vec<CombatEvent>,
    pub kill_events: Vec<KillEvent>,
    pub respawn_events: Vec<u16>,
}

pub fn apply_damage(
    target_health: &mut i32,
    damage: i32,
    attacker_id: u16,
    target_id: u16,
) -> (bool, CombatEvent) {
    *target_health -= damage;

    let died = *target_health <= 0;

    (
        died,
        CombatEvent {
            attacker_id,
            target_id,
            damage,
        },
    )
}

pub fn respawn_player(
    health: &mut i32,
    x: &mut f32,
    y: &mut f32,
    vel_x: &mut f32,
    vel_y: &mut f32,
    spawn_x: f32,
    spawn_y: f32,
) {
    *health = 100;
    *x = spawn_x;
    *y = spawn_y;
    *vel_x = 0.0;
    *vel_y = 0.0;
}
