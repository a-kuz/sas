use super::constants::*;
use super::map::Map;
use glam::Vec2;

pub struct CollisionResult {
    pub new_x: f32,
    pub new_y: f32,
    pub new_vel_x: f32,
    pub new_vel_y: f32,
    pub on_ground: bool,
}

pub fn check_on_ground(x: f32, y: f32, map: &Map) -> bool {
    let hitbox_width = PLAYER_HITBOX_WIDTH / 2.0;
    let check_x_left = ((x - hitbox_width) / 32.0) as i32;
    let check_x_right = ((x + hitbox_width) / 32.0) as i32;
    let check_y_feet = ((y + 24.0) / 16.0) as i32;
    let check_y_body = ((y + 8.0) / 16.0) as i32;

    (map.is_solid(check_x_left, check_y_feet) && !map.is_solid(check_x_left, check_y_body))
        || (map.is_solid(check_x_right, check_y_feet) && !map.is_solid(check_x_right, check_y_body))
}

pub fn move_with_collision(
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    crouch: bool,
    dt_norm: f32,
    map: &Map,
) -> CollisionResult {
    let hitbox_height = if crouch {
        PLAYER_HITBOX_HEIGHT_CROUCH
    } else {
        PLAYER_HITBOX_HEIGHT
    };
    let hitbox_width = PLAYER_HITBOX_WIDTH / 2.0 - 0.5;

    let mut new_x = x;
    let mut new_y = y;
    let mut new_vel_x = vel_x;
    let mut new_vel_y = vel_y;

    let delta_x = vel_x * dt_norm;
    let delta_y = vel_y * dt_norm;

    if delta_x.abs() > 0.01 {
        let target_x = x + delta_x;

        let check_y_top = ((y - hitbox_height) / 16.0) as i32;
        let check_y_mid = ((y) / 16.0) as i32;
        let check_y_bottom = ((y + 16.0) / 16.0) as i32;

        let check_x_left = ((target_x - hitbox_width) / 32.0) as i32;
        let check_x_right = ((target_x + hitbox_width) / 32.0) as i32;

        let x_blocked = map.is_solid(check_x_left, check_y_top)
            || map.is_solid(check_x_right, check_y_top)
            || map.is_solid(check_x_left, check_y_mid)
            || map.is_solid(check_x_right, check_y_mid)
            || map.is_solid(check_x_left, check_y_bottom)
            || map.is_solid(check_x_right, check_y_bottom);

        if !x_blocked {
            new_x = target_x;
        } else {
            let check_ahead_x = if vel_x > 0.0 {
                x + hitbox_width + 2.0
            } else {
                x - hitbox_width - 2.0
            };

            let step_tile_x = (check_ahead_x / 32.0) as i32;
            let step_tile_y_feet = ((y + 16.0) / 16.0) as i32;
            let step_tile_y_body = ((y) / 16.0) as i32;
            let step_tile_y_head = ((y - hitbox_height) / 16.0) as i32;

            let can_step_up = vel_y <= 0.5
                && map.is_solid(step_tile_x, step_tile_y_feet)
                && !map.is_solid(step_tile_x, step_tile_y_body)
                && !map.is_solid(step_tile_x, step_tile_y_head);

            if can_step_up {
                let step_height = 16.0;
                new_x = target_x;
                new_y = y - step_height;
            } else {
                new_vel_x = 0.0;
            }
        }
    }

    if delta_y.abs() > 0.01 {
        let target_y = new_y + delta_y;

        let check_x_left = ((new_x - hitbox_width) / 32.0) as i32;
        let check_x_right = ((new_x + hitbox_width) / 32.0) as i32;

        if delta_y > 0.0 {
            let check_y_feet = ((target_y + 24.0) / 16.0) as i32;
            let check_y_body = ((target_y + 8.0) / 16.0) as i32;

            let on_ground_test = (map.is_solid(check_x_left, check_y_feet)
                && !map.is_solid(check_x_left, check_y_body))
                || (map.is_solid(check_x_right, check_y_feet)
                    && !map.is_solid(check_x_right, check_y_body));

            if on_ground_test {
                new_vel_y = 0.0;
                let grid_y = (target_y / 16.0).floor() as i32;
                new_y = (grid_y as f32 * 16.0) + 8.0;
            } else {
                new_y = target_y;
            }
        } else {
            let check_y_top = ((target_y - hitbox_height) / 16.0) as i32;

            if map.is_solid(check_x_left, check_y_top) || map.is_solid(check_x_right, check_y_top) {
                new_vel_y = 0.0;
                let grid_y = ((target_y - hitbox_height) / 16.0).ceil() as i32;
                new_y = (grid_y as f32 * 16.0) + hitbox_height;
            } else {
                new_y = target_y;
            }
        }
    }

    let check_x_left_final = ((new_x - hitbox_width) / 32.0) as i32;
    let check_x_right_final = ((new_x + hitbox_width) / 32.0) as i32;
    let check_y_feet_final = ((new_y + 24.0) / 16.0) as i32;
    let check_y_body_final = ((new_y + 8.0) / 16.0) as i32;

    let on_ground = (map.is_solid(check_x_left_final, check_y_feet_final)
        && !map.is_solid(check_x_left_final, check_y_body_final))
        || (map.is_solid(check_x_right_final, check_y_feet_final)
            && !map.is_solid(check_x_right_final, check_y_body_final));

    CollisionResult {
        new_x,
        new_y,
        new_vel_x,
        new_vel_y,
        on_ground,
    }
}

pub fn line_rect_intersect(p1: Vec2, p2: Vec2, r_pos: Vec2, r_size: Vec2) -> bool {
    let r_min = r_pos;
    let r_max = r_pos + r_size;

    if p1.x >= r_min.x && p1.x <= r_max.x && p1.y >= r_min.y && p1.y <= r_max.y {
        return true;
    }

    let d = (p2 - p1).normalize_or_zero();
    if d.x == 0.0 && d.y == 0.0 {
        return false;
    }

    let inv_dir = 1.0 / d;
    let mut tmin = (r_min - p1) * inv_dir;
    let mut tmax = (r_max - p1) * inv_dir;

    if tmin.x > tmax.x {
        std::mem::swap(&mut tmin.x, &mut tmax.x);
    }
    if tmin.y > tmax.y {
        std::mem::swap(&mut tmin.y, &mut tmax.y);
    }

    if (tmin.x > tmax.y) || (tmin.y > tmax.x) {
        return false;
    }

    let t_enter = tmin.x.max(tmin.y);
    let t_exit = tmax.x.min(tmax.y);

    t_enter < t_exit && t_exit > 0.0
}
