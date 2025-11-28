use super::animation::{AnimState, PlayerAnimation};
use super::bot_ai::BotAI;
use super::map::Map;
use super::sprite;
use super::weapon::Weapon;
use crate::audio::events::AudioEvent;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: u16,
    pub name: String,
    pub model: String,
    pub x: f32,
    pub y: f32,
    pub cx: f32,
    pub cy: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub prev_x: f32,
    pub prev_y: f32,
    pub interpolation_time: f32,
    pub should_interpolate: bool,
    pub angle: f32,
    pub direction: u8,
    pub health: i32,
    pub armor: i32,
    pub frags: i32,
    pub deaths: i32,
    pub team: u8,
    pub dead: bool,
    pub gibbed: bool,
    pub is_bot: bool,
    pub crouch: bool,
    pub weapon: Weapon,
    pub ammo: [u8; 9],
    pub has_weapon: [bool; 9],
    pub refire: f32,
    pub weapon_switch_time: f32,
    pub powerups: PowerUps,
    pub animation: PlayerAnimation,
    pub bot_ai: Option<BotAI>,
    pub was_in_air: bool,
    pub respawn_timer: f32,
    pub lower_frame: usize,
    pub upper_frame: usize,
    pub animation_time: f32,
    pub debug_anim: String,
    pub prev_legs_anim_id: u8,
    pub lower_next_frame: usize,
    pub upper_next_frame: usize,
    pub lower_fps: u32,
    pub upper_fps: u32,
    pub frame_timer: f32,
    pub upper_frame_timer: f32,
    pub shadow_lx: f32,
    pub shadow_ly: f32,
    pub shadow_lr: f32,
    pub idle_time: f32,
    pub idle_yaw: f32,
    pub somersault_time: f32,
    pub hp_decay_timer: f32,
    pub manual_flip_x: Option<bool>,
    pub excellent_count: u32,
    pub impressive_count: u32,
    pub barrel_spin_angle: f32,
    pub barrel_spin_speed: f32,
}

#[derive(Clone, Debug)]
pub struct PowerUps {
    pub quad: u16,
    pub regen: u16,
    pub battle: u16,
    pub flight: u16,
    pub haste: u16,
    pub invis: u16,
}

impl Player {
    fn fix_stuck_position(&mut self, map: &Map) {
        use super::constants::*;
        let hitbox_height = if self.crouch {
            PLAYER_HITBOX_HEIGHT_CROUCH
        } else {
            PLAYER_HITBOX_HEIGHT
        };
        let _hitbox_width = PLAYER_HITBOX_WIDTH / 2.0 - 0.5;

        for _ in 0..32 {
            let check_x_center = (self.x / 32.0) as i32;
            let check_y_feet = ((self.y + 16.0) / 16.0) as i32;
            let check_y_body = ((self.y) / 16.0) as i32;
            let check_y_head = ((self.y - hitbox_height) / 16.0) as i32;

            let stuck_in_floor = map.is_solid(check_x_center, check_y_feet)
                && map.is_solid(check_x_center, check_y_body);
            let stuck_in_ceiling = map.is_solid(check_x_center, check_y_head);

            if !stuck_in_floor && !stuck_in_ceiling {
                break;
            }

            if stuck_in_floor {
                self.y -= 4.0;
            }
            if stuck_in_ceiling {
                self.y += 4.0;
            }
        }
    }

    pub fn update_timers(&mut self, dt: f32) {
        if self.dead {
            return;
        }

        if self.refire > 0.0 {
            self.refire -= dt;
            if self.refire < 0.0 {
                self.refire = 0.0;
            }
        }

        if self.weapon_switch_time > 0.0 {
            self.weapon_switch_time -= dt;
            if self.weapon_switch_time < 0.0 {
                self.weapon_switch_time = 0.0;
            }
        }

        if self.powerups.quad > 0 {
            self.powerups.quad = self.powerups.quad.saturating_sub(1);
        }
        if self.powerups.regen > 0 {
            self.powerups.regen = self.powerups.regen.saturating_sub(1);
            if (get_time() * 50.0) as u64 % 10 == 0 && self.health < 200 {
                self.health += 1;
            }
        }
        if self.powerups.battle > 0 {
            self.powerups.battle = self.powerups.battle.saturating_sub(1);
        }
        if self.powerups.flight > 0 {
            self.powerups.flight = self.powerups.flight.saturating_sub(1);
        }
        if self.powerups.haste > 0 {
            self.powerups.haste = self.powerups.haste.saturating_sub(1);
        }
        if self.powerups.invis > 0 {
            self.powerups.invis = self.powerups.invis.saturating_sub(1);
        }

        if self.health > 100 {
            self.hp_decay_timer += dt;
            if self.hp_decay_timer >= 1.0 {
                self.health -= 1;
                self.hp_decay_timer = 0.0;
            }
        } else {
            self.hp_decay_timer = 0.0;
        }

        let is_moving = self.vel_x.abs() > 0.1 || self.vel_y.abs() > 0.5;
        if is_moving {
            self.idle_time = 0.0;
            self.idle_yaw = 0.0;
        } else {
            self.idle_time += dt;
            if self.idle_time > 1.0 {
                self.idle_yaw = ((self.idle_time - 1.0) * 1.2).sin() * 0.15;
            }
        }
        
        if self.somersault_time > 0.0 {
            self.somersault_time -= dt;
            if self.somersault_time < 0.0 {
                self.somersault_time = 0.0;
            }
        }

        if matches!(self.weapon, Weapon::MachineGun) {
            if self.barrel_spin_speed > 0.0 {
                self.barrel_spin_speed -= super::constants::BARREL_SPIN_FRICTION * dt;
                if self.barrel_spin_speed < 0.0 {
                    self.barrel_spin_speed = 0.0;
                }
            }
        } else {
            self.barrel_spin_speed = 0.0;
        }
        if self.barrel_spin_speed > 0.0 {
            self.barrel_spin_angle += self.barrel_spin_speed * dt;
            if self.barrel_spin_angle > std::f32::consts::TAU {
                self.barrel_spin_angle -= std::f32::consts::TAU;
            }
        }
    }

    pub fn pmove(&mut self, cmd: &super::usercmd::UserCmd, dt: f32, map: &Map) -> Vec<AudioEvent> {
        self.pmove_internal(cmd, dt, map, true)
    }

    pub fn pmove_no_teleport(&mut self, cmd: &super::usercmd::UserCmd, dt: f32, map: &Map) -> Vec<AudioEvent> {
        self.pmove_internal(cmd, dt, map, false)
    }

    fn pmove_internal(&mut self, cmd: &super::usercmd::UserCmd, dt: f32, map: &Map, allow_teleport: bool) -> Vec<AudioEvent> {
        let mut events = Vec::new();

        if self.dead {
            self.apply_death_physics(dt, map);
            return events;
        }

        use super::usercmd::*;

        self.angle = cmd.angles.0;
        self.crouch = (cmd.buttons & BUTTON_CROUCH) != 0;

        let pmove_state = super::bg_pmove::PmoveState {
            x: self.x,
            y: self.y,
            vel_x: self.vel_x,
            vel_y: self.vel_y,
            was_in_air: self.was_in_air,
        };

        let pmove_cmd = super::bg_pmove::PmoveCmd {
            move_right: cmd.right,
            jump: (cmd.buttons & BUTTON_JUMP) != 0,
            crouch: (cmd.buttons & BUTTON_CROUCH) != 0,
            haste_active: self.powerups.haste > 0,
        };

        let result = super::bg_pmove::pmove(&pmove_state, &pmove_cmd, dt, map);

        if result.jumped {
            events.push(AudioEvent::PlayerJump {
                x: self.x,
                model: self.model.clone(),
            });
        }

        let mut teleported = false;
        if allow_teleport {
            for teleporter in &map.teleporters {
                if result.new_x >= teleporter.x
                    && result.new_x <= teleporter.x + teleporter.width
                    && result.new_y >= teleporter.y
                    && result.new_y <= teleporter.y + teleporter.height
                {
                    println!("[{:.3}] [TELEPORT] p{} from ({:.1},{:.1}) to ({:.1},{:.1}) vel=({:.2},{:.2})",
                        macroquad::prelude::get_time(), self.id,
                        result.new_x, result.new_y,
                        teleporter.dest_x, teleporter.dest_y,
                        result.new_vel_x, result.new_vel_y);
                    events.push(AudioEvent::TeleportIn { x: result.new_x });
                    self.x = teleporter.dest_x;
                    self.y = teleporter.dest_y;
                    self.vel_x = result.new_vel_x;
                    self.vel_y = result.new_vel_y;
                    self.was_in_air = result.new_was_in_air;
                    self.fix_stuck_position(map);
                    events.push(AudioEvent::TeleportOut { x: self.x });
                    teleported = true;
                    break;
                }
            }
        }

        if teleported {
            let moving = self.vel_x.abs() > 0.5;
            let shooting = self.refire > 0.0 && self.refire > (self.weapon.refire_time_seconds() - (3.0 / 60.0));
            self.animation.update(!self.was_in_air, moving, shooting, self.vel_x.abs());
            return events;
        }

        if result.landed {
            events.push(AudioEvent::PlayerLand { x: result.new_x });
        }

        if result.hit_jumppad {
            use crate::network;
            println!("[{}] [PLAYER] p{} HIT JUMPPAD at ({:.1},{:.1})", 
                network::get_absolute_time(), self.id, result.new_x, result.new_y);
            events.push(AudioEvent::JumpPad { x: result.new_x });
            
            if macroquad::prelude::rand::gen_range(0, 10) == 0 {
                self.somersault_time = 1.0;
            }
        }

        self.x = result.new_x;
        self.y = result.new_y;
        self.vel_x = result.new_vel_x;
        self.vel_y = result.new_vel_y;
        self.was_in_air = result.new_was_in_air;

        let moving = self.vel_x.abs() > 0.5;
        let shooting = self.refire > 0.0 && self.refire > (self.weapon.refire_time_seconds() - (3.0 / 60.0));
        self.animation.update(!self.was_in_air, moving, shooting, self.vel_x.abs());

        events
    }

    pub fn new(id: u16, name: String, is_bot: bool) -> Self {
        Self {
            id,
            name,
            model: "sarge".to_string(),
            x: 0.0,
            y: 0.0,
            cx: 0.0,
            cy: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            prev_x: 0.0,
            prev_y: 0.0,
            interpolation_time: 0.0,
            should_interpolate: true,
            angle: 0.0,
            direction: 0,
            health: 100,
            armor: 0,
            frags: 0,
            deaths: 0,
            team: 0,
            dead: false,
            gibbed: false,
            is_bot,
            crouch: false,
            weapon: Weapon::Gauntlet,
            ammo: [0; 9],
            has_weapon: [true, false, false, false, false, false, false, false, false],
            refire: 0.0,
            weapon_switch_time: 0.0,
            powerups: PowerUps {
                quad: 0,
                regen: 0,
                battle: 0,
                flight: 0,
                haste: 0,
                invis: 0,
            },
            animation: PlayerAnimation::new(),
            bot_ai: if is_bot { Some(BotAI::new()) } else { None },
            was_in_air: false,
            respawn_timer: 0.0,
            lower_frame: 0,
            upper_frame: 0,
            animation_time: 0.0,
            debug_anim: String::new(),
            prev_legs_anim_id: 0,
            lower_next_frame: 0,
            upper_next_frame: 0,
            lower_fps: 15,
            upper_fps: 15,
            frame_timer: 0.0,
            upper_frame_timer: 0.0,
            shadow_lx: 0.0,
            shadow_ly: 0.0,
            shadow_lr: 0.0,
            idle_time: 0.0,
            idle_yaw: 0.0,
            somersault_time: 0.0,
            hp_decay_timer: 0.0,
            manual_flip_x: None,
            excellent_count: 0,
            impressive_count: 0,
            barrel_spin_angle: 0.0,
            barrel_spin_speed: 0.0,
        }
    }

    pub fn _update(&mut self, dt: f32, map: &Map) -> (Option<bool>, Vec<AudioEvent>) {
        let events = Vec::new();

        if self.dead {
            return (None, events);
        }

        use super::constants::*;

        let hitbox_height = if self.crouch { 20.0 } else { 36.0 };
        let hitbox_width = 9.0;
        let mut landed = false;

        let is_on_ground = super::collision::check_on_ground(self.x, self.y, map);

        let check_x_left_now = ((self.x - hitbox_width) / 32.0) as i32;
        let check_x_right_now = ((self.x + hitbox_width) / 32.0) as i32;
        let check_y_head = ((self.y - hitbox_height) / 16.0) as i32;
        let ceiling_above = map.is_solid(check_x_left_now, check_y_head)
            || map.is_solid(check_x_right_now, check_y_head);

        if ceiling_above && is_on_ground {
            self.crouch = true;
        }

        self.vel_y += GRAVITY;

        if self.vel_y > -1.0 && self.vel_y < 0.0 {
            self.vel_y /= 1.11;
        }
        if self.vel_y > 0.0 && self.vel_y < 5.0 {
            self.vel_y *= 1.1;
        }

        if self.vel_x.abs() > 0.2 {
            if is_on_ground {
                self.vel_x /= 1.14;
            } else {
                self.vel_x /= 1.025;
            }
        } else {
            self.vel_x = 0.0;
        }

        if self.vel_y > MAX_FALL_SPEED {
            self.vel_y = MAX_FALL_SPEED;
        }

        if self.vel_y < -15.0 {
            self.vel_y = -15.0;
        }

        if self.vel_x.abs() > MAX_SPEED_AIR {
            self.vel_x = self.vel_x.signum() * MAX_SPEED_AIR;
        }

        let new_x = self.x + self.vel_x;
        let new_y = self.y + self.vel_y;

        let check_x_left = ((new_x - hitbox_width) / 32.0) as i32;
        let check_x_right = ((new_x + hitbox_width) / 32.0) as i32;
        let check_y_top = ((new_y - hitbox_height) / 16.0) as i32;
        let check_y_bottom = ((new_y) / 16.0) as i32;
        let check_y_feet = ((new_y + 24.0) / 16.0) as i32;
        let check_y_body = ((new_y + 8.0) / 16.0) as i32;

        let can_move_x = !map.is_solid(check_x_left, check_y_top)
            && !map.is_solid(check_x_right, check_y_top)
            && !map.is_solid(check_x_left, check_y_bottom)
            && !map.is_solid(check_x_right, check_y_bottom);

        let can_move_y = !map.is_solid(check_x_left, check_y_bottom)
            && !map.is_solid(check_x_right, check_y_bottom);

        let on_ground = (map.is_solid(check_x_left, check_y_feet)
            && !map.is_solid(check_x_left, check_y_body))
            || (map.is_solid(check_x_right, check_y_feet)
                && !map.is_solid(check_x_right, check_y_body));

        if can_move_x {
            self.x = new_x;
        } else {
            self.vel_x = 0.0;

            if self.vel_x.abs() > 0.1 {
                let push_out = if new_x > self.x { -1.0 } else { 1.0 };
                self.x += push_out;
            }
        }

        if on_ground && self.vel_y > 0.0 {
            if self.was_in_air && self.vel_y > 2.0 {
                landed = true;
            }
            self.vel_y = 0.0;
            let grid_y = (self.y.round() / 16.0) as i32;
            self.y = (grid_y as f32 * 16.0) + 8.0;
        } else if can_move_y {
            self.y = new_y;
        } else {
            if self.vel_y > 0.0 {
                let grid_y = (self.y.round() / 16.0) as i32;
                self.y = (grid_y as f32 * 16.0) + 8.0;
            } else if self.vel_y < 0.0 {
                let grid_y = (self.y.round() / 16.0) as i32;
                self.y = (grid_y as f32 * 16.0) + 8.0;
            }
            self.vel_y = 0.0;
        }

        self.was_in_air = !on_ground;

        if self.refire > 0.0 {
            self.refire -= dt;
            if self.refire < 0.0 {
                self.refire = 0.0;
            }
        }

        if self.weapon_switch_time > 0.0 {
            self.weapon_switch_time -= dt;
            if self.weapon_switch_time < 0.0 {
                self.weapon_switch_time = 0.0;
            }
        }

        let moving = self.vel_x.abs() > 0.5;
        let shooting =
            self.refire > 0.0 && self.refire > (self.weapon.refire_time_seconds() - 3.0 / 60.0);
        self.animation
            .update(is_on_ground, moving, shooting, self.vel_x.abs());

        if self.powerups.quad > 0 {
            self.powerups.quad -= 1;
        }
        if self.powerups.regen > 0 {
            self.powerups.regen -= 1;
            if self._frame() % 10 == 0 && self.health < 200 {
                self.health += 1;
            }
        }
        if self.powerups.battle > 0 {
            self.powerups.battle -= 1;
        }
        if self.powerups.flight > 0 {
            self.powerups.flight -= 1;
        }
        if self.powerups.haste > 0 {
            self.powerups.haste -= 1;
        }
        if self.powerups.invis > 0 {
            self.powerups.invis -= 1;
        }

        (if landed { Some(true) } else { None }, events)
    }

    pub fn render(&self, camera_x: f32, camera_y: f32) {
        let screen_x = self.x - camera_x;
        let screen_y = self.y - camera_y;

        draw_ellipse(
            screen_x,
            screen_y + 18.0,
            12.0,
            4.0,
            0.0,
            Color::from_rgba(0, 0, 0, 100),
        );

        let base_color = if self.is_bot {
            Color::from_rgba(200, 100, 100, 255)
        } else {
            Color::from_rgba(100, 200, 100, 255)
        };

        let color = if self.powerups.quad > 0 {
            Color::from_rgba(150, 150, 255, 255)
        } else if self.powerups.invis > 0 {
            Color::from_rgba(
                (base_color.r * 255.0) as u8,
                (base_color.g * 255.0) as u8,
                (base_color.b * 255.0) as u8,
                100,
            )
        } else {
            base_color
        };

        let direction = if self.vel_x > 0.5 {
            0
        } else if self.vel_x < -0.5 {
            1
        } else {
            self.direction
        };

        sprite::draw_player_sprite(
            screen_x,
            screen_y,
            direction,
            color,
            self.animation.legs_frame,
            self.animation.state == AnimState::Attack,
            self.animation.state == AnimState::Walk || self.animation.state == AnimState::Run,
            self.crouch,
        );

        if self.powerups.quad > 0 {
            let time = (get_time() * 5.0) as f32;
            let glow_radius = 25.0 + (time.sin() * 5.0);
            draw_circle(
                screen_x,
                screen_y,
                glow_radius,
                Color::from_rgba(100, 150, 255, 30),
            );
        }

        let health_bar_y = screen_y - 24.0;
        draw_rectangle(
            screen_x - 16.0,
            health_bar_y,
            32.0,
            3.0,
            Color::from_rgba(30, 30, 34, 220),
        );

        if self.health > 0 {
            let health_width = (self.health.max(0) as f32 / super::constants::STARTING_HEALTH as f32) * 32.0;
            let health_color = if self.health > 75 {
                Color::from_rgba(0, 255, 0, 255)
            } else if self.health > 25 {
                Color::from_rgba(255, 255, 0, 255)
            } else {
                Color::from_rgba(255, 0, 0, 255)
            };
            draw_rectangle(
                screen_x - 16.0,
                health_bar_y,
                health_width,
                3.0,
                health_color,
            );
        }

        if self.armor > 0 {
            let armor_width = (self.armor.min(100) as f32 / 100.0) * 32.0;
            draw_rectangle(
                screen_x - 16.0,
                health_bar_y + 4.0,
                armor_width,
                2.0,
                Color::from_rgba(100, 200, 255, 200),
            );
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, map: &Map) {
        self.x = x;
        let aligned_y = ((y / 16.0).round() * 16.0) + 8.0;
        self.y = aligned_y;
        self.vel_x = 0.0;
        self.vel_y = 0.0;
        self.health = super::constants::STARTING_HEALTH;
        self.armor = 0;
        self.dead = false;
        self.gibbed = false;
        self.weapon = Weapon::MachineGun;
        self.refire = 0.0;
        self.weapon_switch_time = 0.0;
        self.animation = super::animation::PlayerAnimation::new();
        self.lower_frame = 0;
        self.upper_frame = 0;
        self.animation_time = 0.0;
        self.debug_anim.clear();
        self.prev_legs_anim_id = 0;
        self.lower_next_frame = 0;
        self.upper_next_frame = 0;
        self.lower_fps = 15;
        self.upper_fps = 15;
        self.frame_timer = 0.0;
        self.upper_frame_timer = 0.0;
        self.idle_time = 0.0;
        self.idle_yaw = 0.0;

        self.has_weapon = [true, true, false, false, false, false, false, false, false];
        self.ammo = [0, 100, 0, 0, 0, 0, 0, 0, 0];
        
        self.powerups.quad = 0;
        self.powerups.regen = 0;
        self.powerups.battle = 0;
        self.powerups.flight = 0;
        self.powerups.haste = 0;
        self.powerups.invis = 0;

        println!(
            "[Player] Before fix_stuck_position: x={}, y={}",
            self.x, self.y
        );
        self.fix_stuck_position(map);
        println!(
            "[Player] After fix_stuck_position: x={}, y={}",
            self.x, self.y
        );
        self.barrel_spin_angle = 0.0;
        self.barrel_spin_speed = 0.0;
    }

    pub fn take_damage(&mut self, damage: i32) -> (bool, bool) {
        const GIB_HEALTH: i32 = -150;

        if self.dead {
            let was_gibbed = self.gibbed;
            self.health -= damage;
            if self.health <= GIB_HEALTH && !was_gibbed {
                self.gibbed = true;
                return (false, true);
            }
            return (false, false);
        }

        let actual_damage = if self.armor > 0 {
            let absorbed = (damage as f32 * 0.66) as i32;
            self.armor -= absorbed;
            if self.armor < 0 {
                self.armor = 0;
            }
            damage - absorbed
        } else {
            damage
        };

        self.health -= actual_damage;

        if self.health <= GIB_HEALTH {
            self.dead = true;
            self.gibbed = true;
            self.deaths += 1;
            self.respawn_timer = 3.0;
            return (true, true);
        } else if self.health <= 0 {
            self.health = 0;
            self.dead = true;
            self.deaths += 1;
            self.respawn_timer = 3.0;
            return (true, false);
        }
        (false, false)
    }

    fn _frame(&self) -> u64 {
        (get_time() * 50.0) as u64
    }

    fn apply_death_physics(&mut self, dt: f32, map: &Map) {
        use super::bg_pmove::{pmove, PmoveState, PmoveCmd};

        let state = PmoveState {
            x: self.x,
            y: self.y,
            vel_x: self.vel_x,
            vel_y: self.vel_y,
            was_in_air: true,
        };

        let cmd = PmoveCmd {
            move_right: 0.0,
            jump: false,
            crouch: false,
            haste_active: false,
        };

        let result = pmove(&state, &cmd, dt, map);

        self.x = result.new_x;
        self.y = result.new_y;
        self.vel_x = result.new_vel_x;
        self.vel_y = result.new_vel_y;
    }
}
