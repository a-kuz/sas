use crate::game::{GameState, player::Player, weapon::Weapon};
use crate::audio;

pub struct BotHandler;

impl BotHandler {
    pub fn process_bot_ai(game_state: &mut GameState, dt: f32) -> Vec<(f32, f32, f32, u16, Weapon)> {
        if game_state.is_local_multiplayer {
            return Vec::new();
        }

        let players_snapshot = game_state.players.clone();
        let mut bot_actions = Vec::new();

        for i in 1..game_state.players.len() {
            if !game_state.players[i].is_bot || game_state.players[i].dead {
                continue;
            }

            let bot = &mut game_state.players[i];
            let bot_x = bot.x;
            let bot_y = bot.y;
            let bot_id = bot.id;
            let bot_weapon = bot.weapon;
            let bot_has_weapon = bot.has_weapon;
            let bot_ammo = bot.ammo;
            let bot_vel_y = bot.vel_y;

            let ai_move_direction;
            let ai_want_jump;
            let ai_want_weapon;
            let ai_want_shoot;
            let ai_target_player;
            let ai_rocket_jump_timer;

            let bot_name = bot.name.clone();
            let bot_model = bot.model.clone();
            let bot_cx = bot.cx;
            let bot_cy = bot.cy;
            let bot_vel_x = bot.vel_x;
            let bot_angle = bot.angle;
            let bot_direction = bot.direction as i8;
            let bot_health = bot.health;
            let bot_armor = bot.armor;
            let bot_frags = bot.frags;
            let bot_deaths = bot.deaths;
            let bot_team = bot.team;
            let bot_dead = bot.dead;
            let bot_gibbed = bot.gibbed;
            let bot_is_bot = bot.is_bot;
            let bot_crouch = bot.crouch;
            let bot_refire = bot.refire;
            let bot_weapon_switch_time = bot.weapon_switch_time;
            let bot_powerups = bot.powerups.clone();
            let bot_animation = bot.animation.clone();
            let bot_was_in_air = bot.was_in_air;
            let bot_respawn_timer = bot.respawn_timer;
            let bot_lower_frame = bot.lower_frame as f32;
            let bot_upper_frame = bot.upper_frame as f32;
            let bot_animation_time = bot.animation_time;
            let bot_debug_anim = bot.debug_anim.clone();
            let bot_prev_legs_anim_id = bot.prev_legs_anim_id as u32;
            let bot_lower_next_frame = bot.lower_next_frame as f32;
            let bot_upper_next_frame = bot.upper_next_frame as f32;
            let bot_lower_fps = bot.lower_fps as f32;
            let bot_upper_fps = bot.upper_fps as f32;
            let bot_frame_timer = bot.frame_timer;
            let bot_upper_frame_timer = bot.upper_frame_timer;
            let bot_shadow_lx = bot.shadow_lx;
            let bot_shadow_ly = bot.shadow_ly;
            let bot_shadow_lr = bot.shadow_lr;

            if let Some(ai) = &mut bot.bot_ai {
                let bot_snapshot = Self::create_bot_snapshot_from_values(
                    bot_id, bot_x, bot_y, bot_vel_y, bot_weapon, bot_has_weapon, bot_ammo,
                    bot_name, bot_model, bot_cx, bot_cy, bot_vel_x, bot_angle, bot_direction,
                    bot_health, bot_armor, bot_frags, bot_deaths, bot_team, bot_dead, bot_gibbed,
                    bot_is_bot, bot_crouch, bot_refire, bot_weapon_switch_time, bot_powerups,
                    bot_animation, bot_was_in_air, bot_respawn_timer, bot_lower_frame, bot_upper_frame,
                    bot_animation_time, bot_debug_anim, bot_prev_legs_anim_id, bot_lower_next_frame,
                    bot_upper_next_frame, bot_lower_fps, bot_upper_fps, bot_frame_timer,
                    bot_upper_frame_timer, bot_shadow_lx, bot_shadow_ly, bot_shadow_lr,
                );
                ai.think(&bot_snapshot, &players_snapshot, &game_state.map, &game_state.projectiles, game_state.nav_graph.as_ref());

                ai_move_direction = ai.move_direction;
                ai_want_jump = ai.want_jump;
                ai_want_weapon = ai.want_weapon;
                ai_want_shoot = ai.want_shoot;
                ai_target_player = ai.target_player;
                ai_rocket_jump_timer = ai.rocket_jump_timer;
            } else {
                continue;
            }

            let bot = &mut game_state.players[i];

            let mut bot_cmd = crate::game::usercmd::UserCmd::new();
            bot_cmd.right = ai_move_direction;
            if ai_want_jump {
                bot_cmd.buttons |= crate::game::usercmd::BUTTON_JUMP;
            }

            if let Some(target_id) = ai_target_player {
                if let Some(target) = players_snapshot.iter().find(|p| p.id == target_id) {
                    let dx = target.x - bot.x;
                    let dy = target.y - bot.y;
                    let angle = dy.atan2(dx);
                    bot_cmd.angles = (angle, 0.0);
                }
            }

            let bot_pmove_events = bot.pmove(&bot_cmd, dt, &game_state.map);
            for event in bot_pmove_events {
                game_state.audio_events.push(event);
            }

            if let Some(weapon) = ai_want_weapon {
                if bot.weapon != weapon && bot.has_weapon[weapon as usize] {
                    bot.weapon = weapon;
                    bot.weapon_switch_time = bot.weapon.switch_time_seconds();
                }
            }

            
            if let Some(action) = Self::handle_bot_shooting(
                bot,
                ai_rocket_jump_timer,
                ai_want_shoot,
                ai_target_player,
                &players_snapshot,
                &mut game_state.model_cache,
                &game_state.weapon_model_cache,
            ) {
                bot_actions.push(action);
            }
        }

        Self::process_bot_actions(game_state, bot_actions)
    }

    fn create_bot_snapshot_from_values(
        bot_id: u16, bot_x: f32, bot_y: f32, bot_vel_y: f32, bot_weapon: Weapon,
        bot_has_weapon: [bool; 9], bot_ammo: [u8; 9], bot_name: String, bot_model: String,
        bot_cx: f32, bot_cy: f32, bot_vel_x: f32, bot_angle: f32, bot_direction: i8,
        bot_health: i32, bot_armor: i32, bot_frags: i32, bot_deaths: i32, bot_team: u8,
        bot_dead: bool, bot_gibbed: bool, bot_is_bot: bool, bot_crouch: bool,
        bot_refire: f32, bot_weapon_switch_time: f32,         bot_powerups: crate::game::player::PowerUps,
        bot_animation: crate::game::animation::PlayerAnimation, bot_was_in_air: bool,
        bot_respawn_timer: f32, bot_lower_frame: f32, bot_upper_frame: f32,
        bot_animation_time: f32, bot_debug_anim: String, bot_prev_legs_anim_id: u32,
        bot_lower_next_frame: f32, bot_upper_next_frame: f32, bot_lower_fps: f32,
        bot_upper_fps: f32, bot_frame_timer: f32, bot_upper_frame_timer: f32,
        bot_shadow_lx: f32, bot_shadow_ly: f32, bot_shadow_lr: f32,
    ) -> Player {
        Player {
            id: bot_id,
            name: bot_name,
            model: bot_model,
            x: bot_x,
            y: bot_y,
            cx: bot_cx,
            cy: bot_cy,
            vel_x: bot_vel_x,
            vel_y: bot_vel_y,
            prev_x: bot_x,
            prev_y: bot_y,
            interpolation_time: 0.0,
            should_interpolate: true,
            angle: bot_angle,
            direction: bot_direction as u8,
            health: bot_health,
            armor: bot_armor,
            frags: bot_frags,
            deaths: bot_deaths,
            team: bot_team,
            dead: bot_dead,
            gibbed: bot_gibbed,
            is_bot: bot_is_bot,
            crouch: bot_crouch,
            weapon: bot_weapon,
            ammo: bot_ammo,
            has_weapon: bot_has_weapon,
            refire: bot_refire,
            weapon_switch_time: bot_weapon_switch_time,
            powerups: bot_powerups,
            animation: bot_animation,
            bot_ai: None,
            was_in_air: bot_was_in_air,
            respawn_timer: bot_respawn_timer,
            lower_frame: bot_lower_frame as usize,
            upper_frame: bot_upper_frame as usize,
            animation_time: bot_animation_time,
            debug_anim: bot_debug_anim,
            prev_legs_anim_id: bot_prev_legs_anim_id as u8,
            lower_next_frame: bot_lower_next_frame as usize,
            upper_next_frame: bot_upper_next_frame as usize,
            lower_fps: bot_lower_fps as u32,
            upper_fps: bot_upper_fps as u32,
            frame_timer: bot_frame_timer,
            upper_frame_timer: bot_upper_frame_timer,
            shadow_lx: bot_shadow_lx,
            shadow_ly: bot_shadow_ly,
            shadow_lr: bot_shadow_lr,
            idle_time: 0.0,
            idle_yaw: 0.0,
            somersault_time: 0.0,
            hp_decay_timer: 0.0,
            manual_flip_x: None,
            excellent_count: 0,
            impressive_count: 0,
        }
    }

    fn handle_bot_shooting(
        bot: &mut Player,
        ai_rocket_jump_timer: u32,
        ai_want_shoot: bool,
        ai_target_player: Option<u16>,
        players_snapshot: &[Player],
        model_cache: &mut crate::game::model_cache::ModelCache,
        weapon_model_cache: &crate::game::weapon_model_cache::WeaponModelCache,
    ) -> Option<(f32, f32, f32, u16, Weapon)> {
        if ai_rocket_jump_timer > 0 && ai_rocket_jump_timer > 180 && bot.refire <= 0.0 && bot.weapon_switch_time <= 0.0 {
            if bot.weapon != Weapon::RocketLauncher && bot.has_weapon[4] {
                bot.weapon = Weapon::RocketLauncher;
                bot.weapon_switch_time = bot.weapon.switch_time_seconds();
            } else if bot.weapon == Weapon::RocketLauncher && bot.ammo[4] > 0 {
                let angle = std::f32::consts::PI / 2.0;
                
                bot.refire = bot.weapon.refire_time_seconds();
                bot.ammo[4] = bot.ammo[4].saturating_sub(1);
                
                let flip = false;
                let pitch = 0.0;
                
                let weapon_model = weapon_model_cache.get(bot.weapon);
                
                let (shoot_x, shoot_y) = if let Some(bot_model) = model_cache.get_mut(&bot.model) {
                    bot_model.get_barrel_position(
                        bot.x,
                        bot.y,
                        flip,
                        pitch,
                        angle,
                        bot.lower_frame as usize,
                        bot.upper_frame as usize,
                        weapon_model,
                    )
                } else {
                    (bot.x, bot.y + 10.0)
                };
                
                return Some((shoot_x, shoot_y, angle, bot.id, bot.weapon));
            }
        } else if ai_want_shoot && bot.refire <= 0.0 && bot.weapon_switch_time <= 0.0 {
            if let Some(target_id) = ai_target_player {
                if let Some(target) = players_snapshot.iter().find(|p| p.id == target_id) {
                    let dx = target.x - bot.x;
                    let dy = (target.y - 24.0) - (bot.y - 24.0);
                    let angle = dy.atan2(dx);

                    let weapon_idx = bot.weapon as usize;
                    if bot.ammo[weapon_idx] >= bot.weapon.ammo_per_shot() || bot.weapon as u8 == 0 {
                        bot.refire = bot.weapon.refire_time_seconds();
                        if bot.weapon as u8 > 0 {
                            bot.ammo[weapon_idx] = bot.ammo[weapon_idx]
                                .saturating_sub(bot.weapon.ammo_per_shot());
                        }

                        let flip = angle.abs() > std::f32::consts::PI / 2.0;
                        let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                        let mut rel_angle = angle - base_dir;
                        while rel_angle > std::f32::consts::PI {
                            rel_angle -= 2.0 * std::f32::consts::PI;
                        }
                        while rel_angle < -std::f32::consts::PI {
                            rel_angle += 2.0 * std::f32::consts::PI;
                        }
                        let pitch = rel_angle;

                        let weapon_model = weapon_model_cache.get(bot.weapon);

                        let (shoot_x, shoot_y) = if let Some(bot_model) = model_cache.get_mut(&bot.model) {
                            bot_model.get_barrel_position(
                                bot.x,
                                bot.y,
                                flip,
                                pitch,
                                angle,
                                bot.lower_frame as usize,
                                bot.upper_frame as usize,
                                weapon_model,
                            )
                        } else {
                            let weapon_offset = 20.0;
                            (
                                bot.x + angle.cos() * weapon_offset,
                                bot.y - 24.0 + angle.sin() * weapon_offset,
                            )
                        };

                        return Some((shoot_x, shoot_y, angle, bot.id, bot.weapon));
                    }
                }
            }
        }

        None
    }

    fn process_bot_actions(
        game_state: &mut GameState,
        bot_actions: Vec<(f32, f32, f32, u16, Weapon)>,
    ) -> Vec<(f32, f32, f32, u16, Weapon)> {
        for (bot_x, _bot_y, _angle, bot_id, weapon) in &bot_actions {
            let has_quad = game_state.players.iter()
                .find(|p| p.id == *bot_id)
                .map(|p| p.powerups.quad > 0)
                .unwrap_or(false);
            
            game_state.audio_events.push(audio::events::AudioEvent::WeaponFire { 
                weapon: *weapon, 
                x: *bot_x,
                has_quad,
            });
        }

        bot_actions
    }
}
