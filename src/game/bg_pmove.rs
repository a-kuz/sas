use super::map::Map;
use super::constants::*;
use super::collision;

#[derive(Clone, Debug)]
pub struct PmoveState {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub was_in_air: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct PmoveCmd {
    pub move_right: f32,
    pub jump: bool,
    pub crouch: bool,
    pub haste_active: bool,
}

#[derive(Clone, Debug)]
pub struct PmoveResult {
    pub new_x: f32,
    pub new_y: f32,
    pub new_vel_x: f32,
    pub new_vel_y: f32,
    pub new_was_in_air: bool,
    pub jumped: bool,
    pub landed: bool,
    pub had_impulse: bool,
    pub impulse_type: String,
    pub hit_jumppad: bool,
}

pub fn pmove(state: &PmoveState, cmd: &PmoveCmd, dt: f32, map: &Map) -> PmoveResult {
    const MAX_DT: f32 = 0.05;
    
    let x = state.x;
    let y = state.y;
    let mut vel_x = state.vel_x;
    let mut vel_y = state.vel_y;
    let was_in_air = state.was_in_air;

    let on_ground = collision::check_on_ground(x, y, map);

    let dt_clamped = dt.min(MAX_DT);
    let dt_norm = dt_clamped * 60.0;

    let base_max_speed = if cmd.crouch {
        MAX_SPEED_GROUND * CROUCH_SPEED_MULT
    } else {
        MAX_SPEED_GROUND
    };
    let max_speed = if cmd.haste_active {
        base_max_speed * HASTE_SPEED_MULT
    } else {
        base_max_speed
    };

    let accel = if on_ground { GROUND_ACCEL } else { AIR_ACCEL };
    let change_dir_accel = accel * 2.3;

    if cmd.move_right < -0.01 {
        if vel_x > 0.0 {
            vel_x -= change_dir_accel * dt_norm;
        }
        if vel_x > -max_speed {
            vel_x -= accel * dt_norm;
        }
        if vel_x < -max_speed {
            vel_x = -max_speed;
        }
    } else if cmd.move_right > 0.01 {
        if vel_x < 0.0 {
            vel_x += change_dir_accel * dt_norm;
        }
        if vel_x < max_speed {
            vel_x += accel * dt_norm;
        }
        if vel_x > max_speed {
            vel_x = max_speed;
        }
    }

    let mut jumped = false;
    if cmd.jump && on_ground && vel_y >= -0.5 {
        let jump_force = if cmd.haste_active {
            JUMP_FORCE * HASTE_JUMP_MULT
        } else {
            JUMP_FORCE
        };
        vel_y = jump_force;
        jumped = true;
    }

    vel_y += GRAVITY * dt_norm;

    if vel_y > -1.0 && vel_y < 0.0 {
        vel_y /= 1.0 + (0.11 * dt_norm);
    }
    if vel_y > 0.0 && vel_y < 5.0 {
        vel_y *= 1.0 + (0.1 * dt_norm);
    }

    if cmd.move_right.abs() < 0.01 {
        if vel_x.abs() > 0.01 {
            if on_ground {
                vel_x /= 1.0 + (0.14 * dt_norm);
            } else {
                vel_x /= 1.0 + (0.025 * dt_norm);
            }
            if vel_x.abs() < 0.01 {
                vel_x = 0.0;
            }
        }
    }

    if vel_y > MAX_FALL_SPEED {
        vel_y = MAX_FALL_SPEED;
    }

    if vel_y < -15.0 {
        vel_y = -15.0;
    }

    if vel_x.abs() > MAX_SPEED_AIR {
        vel_x = vel_x.signum() * MAX_SPEED_AIR;
    }

    let mut coll = collision::move_with_collision(
        x,
        y,
        vel_x,
        vel_y,
        cmd.crouch,
        dt_norm,
        map,
    );

    let mut had_impulse = false;
    let mut impulse_type = String::new();
    let mut hit_jumppad = false;

    for jumppad in &map.jumppads {
        if jumppad.check_collision(coll.new_x, coll.new_y) && coll.new_vel_y >= -1.0 {
            coll.new_vel_x += jumppad.force_x;
            coll.new_vel_y = jumppad.force_y;
            had_impulse = true;
            hit_jumppad = true;
            impulse_type = format!("jumppad(fx={:.1},fy={:.1})", jumppad.force_x, jumppad.force_y);
        }
    }

    let landed = coll.on_ground && !was_in_air && vel_y > 2.0;

    PmoveResult {
        new_x: coll.new_x,
        new_y: coll.new_y,
        new_vel_x: coll.new_vel_x,
        new_vel_y: coll.new_vel_y,
        new_was_in_air: !coll.on_ground,
        jumped,
        landed,
        had_impulse,
        impulse_type,
        hit_jumppad,
    }
}

