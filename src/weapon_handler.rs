use macroquad::prelude::*;
use crate::game::{GameState, weapon::Weapon, constants::*};
use crate::input::{Input, LocalMultiplayerInput};
use crate::audio;

pub struct WeaponHandler;

impl WeaponHandler {
    pub fn handle_weapon_input_multiplayer(
        game_state: &mut GameState,
        input: &Input,
        player_id: u16,
    ) -> Option<(f32, f32, f32, u8)> {
        let player_idx = game_state.players.iter().position(|p| p.id == player_id)?;
        
        let player_x = game_state.players[player_idx].x;
        let player_y = game_state.players[player_idx].y;
        let player_model = game_state.players[player_idx].model.clone();
        let player_lower_frame = game_state.players[player_idx].lower_frame;
        let player_upper_frame = game_state.players[player_idx].upper_frame;
        let player_weapon = game_state.players[player_idx].weapon;
        let player_vel_x = game_state.players[player_idx].vel_x;
        let player_vel_y = game_state.players[player_idx].vel_y;
        
        let mut projectile_to_create: Option<(f32, f32, f32, u8)> = None;
        
        if let Some(player) = game_state.players.get_mut(player_idx) {
            if input.shoot && player.refire <= 0.0 && player.weapon_switch_time <= 0.0 {
                let weapon_idx = player.weapon as usize;
                if player.ammo[weapon_idx] >= player.weapon.ammo_per_shot() || player.weapon as u8 == 0 {
                    player.refire = player.weapon.refire_time_seconds();
                    if player.weapon as u8 > 0 {
                        player.ammo[weapon_idx] = player.ammo[weapon_idx]
                            .saturating_sub(player.weapon.ammo_per_shot());
                    }

                    let angle = input.aim_angle;
                    player.angle = angle;

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

                    let (shoot_x, shoot_y) = Self::calculate_barrel_position(
                        player_x,
                        player_y,
                        angle,
                        flip,
                        pitch,
                        player_lower_frame as f32,
                        player_upper_frame as f32,
                        &mut game_state.model_cache,
                        &game_state.weapon_model_cache,
                        &player_model,
                        player_weapon,
                    );

                    game_state.audio_events.push(audio::events::AudioEvent::WeaponFire {
                        weapon: player.weapon,
                        x: player.x,
                        has_quad: player.powerups.quad > 0,
                    });
                    
                    projectile_to_create = Some((shoot_x, shoot_y, angle, player.weapon as u8));
                }
            }
        }
        
        if let Some((shoot_x, shoot_y, angle, weapon_u8)) = projectile_to_create {
            let projectile = crate::game::projectile::Projectile::new(
                shoot_x,
                shoot_y,
                angle,
                player_id,
                player_weapon,
                player_vel_x,
                player_vel_y,
            );
            let projectile = game_state.create_projectile_with_id(projectile);
            game_state.projectiles.push(projectile);
            return Some((shoot_x, shoot_y, angle, weapon_u8));
        }
        
        None
    }

    pub fn handle_shooting(
        game_state: &mut GameState,
        input: &Input,
        local_mp_input: &LocalMultiplayerInput,
    ) -> Vec<(f32, f32, f32, u16, Weapon)> {
        let mut shoot_actions = Vec::new();

        if game_state.is_local_multiplayer {
            let player_inputs = [&local_mp_input.player1, &local_mp_input.player2];
            for (player_idx, player_input) in player_inputs.iter().enumerate() {
                if player_idx < game_state.players.len() {
                    let player_x = game_state.players[player_idx].x;
                    let player_y = game_state.players[player_idx].y;
                    let player_id = game_state.players[player_idx].id;
                    let player_weapon = game_state.players[player_idx].weapon;
                    let player_model = game_state.players[player_idx].model.clone();
                    let player_lower_frame = game_state.players[player_idx].lower_frame;
                    let player_upper_frame = game_state.players[player_idx].upper_frame;
                    
                    if let Some(player) = game_state.players.get_mut(player_idx) {
                        if player_input.shoot && player.refire <= 0.0 && player.weapon_switch_time <= 0.0 {
                            let weapon_idx = player.weapon as usize;
                            if player.ammo[weapon_idx] >= player.weapon.ammo_per_shot() || player.weapon as u8 == 0 {
                                player.refire = player.weapon.refire_time_seconds();
                                if player.weapon as u8 > 0 {
                                    player.ammo[weapon_idx] = player.ammo[weapon_idx]
                                        .saturating_sub(player.weapon.ammo_per_shot());
                                }

                                game_state.audio_events.push(audio::events::AudioEvent::WeaponFire {
                                    weapon: player.weapon,
                                    x: player.x,
                                    has_quad: player.powerups.quad > 0,
                                });

                                let angle = player_input.aim_angle;
                                player.angle = angle;

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

                                let (shoot_x, shoot_y) = Self::calculate_barrel_position(
                                    player_x,
                                    player_y,
                                    angle,
                                    flip,
                                    pitch,
                                    player_lower_frame as f32,
                                    player_upper_frame as f32,
                                    &mut game_state.model_cache,
                                    &game_state.weapon_model_cache,
                                    &player_model,
                                    player_weapon,
                                );

                                shoot_actions.push((shoot_x, shoot_y, angle, player_id, player_weapon));
                            }
                        }

                        let angle = player_input.aim_angle;
                        player.angle = angle;
                    }
                }
            }
        } else if let Some(player) = game_state.players.get_mut(0) {
            if input.shoot && player.refire <= 0.0 && player.weapon_switch_time <= 0.0 {
                let weapon_idx = player.weapon as usize;
                if player.ammo[weapon_idx] >= player.weapon.ammo_per_shot() || player.weapon as u8 == 0 {
                    player.refire = player.weapon.refire_time_seconds();
                    if player.weapon as u8 > 0 {
                        player.ammo[weapon_idx] = player.ammo[weapon_idx]
                            .saturating_sub(player.weapon.ammo_per_shot());
                    }

                    game_state.audio_events.push(audio::events::AudioEvent::WeaponFire {
                        weapon: player.weapon,
                        x: player.x,
                        has_quad: player.powerups.quad > 0,
                    });

                    let angle = input.aim_angle;
                    player.angle = angle;

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

                    let player_x = player.x;
                    let player_y = player.y;
                    let lower_frame = player.lower_frame;
                    let upper_frame = player.upper_frame;
                    let player_model = player.model.clone();
                    let weapon = player.weapon;
                    
                    let (shoot_x, shoot_y) = Self::calculate_barrel_position(
                        player_x,
                        player_y,
                        angle,
                        flip,
                        pitch,
                        lower_frame as f32,
                        upper_frame as f32,
                        &mut game_state.model_cache,
                        &game_state.weapon_model_cache,
                        &player_model,
                        weapon,
                    );

                    shoot_actions.push((shoot_x, shoot_y, angle, player.id, player.weapon));
                }
            }

            let angle = input.aim_angle;
            player.angle = angle;
        }

        shoot_actions
    }


    fn calculate_barrel_position(
        player_x: f32,
        player_y: f32,
        angle: f32,
        flip: bool,
        pitch: f32,
        lower_frame: f32,
        upper_frame: f32,
        model_cache: &mut crate::game::model_cache::ModelCache,
        weapon_model_cache: &crate::game::weapon_model_cache::WeaponModelCache,
        player_model: &str,
        weapon: crate::game::weapon::Weapon,
    ) -> (f32, f32) {
        let weapon_model = weapon_model_cache.get(weapon);
        
        if let Some(player_model) = model_cache.get_mut(player_model) {
            player_model.get_barrel_position(
                player_x,
                player_y,
                flip,
                pitch,
                angle,
                lower_frame as usize,
                upper_frame as usize,
                weapon_model,
            )
        } else {
            let weapon_offset = 20.0;
            (
                player_x + angle.cos() * weapon_offset,
                player_y - 24.0 + angle.sin() * weapon_offset,
            )
        }
    }

    pub fn handle_weapon_switching(
        game_state: &mut GameState,
        input: &Input,
        local_mp_input: &LocalMultiplayerInput,
    ) {
        if game_state.is_local_multiplayer {
            for (player_idx, player_input) in [&local_mp_input.player1, &local_mp_input.player2]
                .iter()
                .enumerate()
            {
                let weapon_switch = player_input.weapon_switch;
                if let Some(player) = game_state.players.get_mut(player_idx) {
                    Self::process_weapon_switch_simple(weapon_switch, player, &mut game_state.audio_events);
                }
            }
        } else {
            let weapon_switch = input.weapon_switch;
            if let Some(player) = game_state.players.get_mut(0) {
                Self::process_weapon_switch_simple(weapon_switch, player, &mut game_state.audio_events);
            }
        }
    }

    fn process_weapon_switch_simple(
        weapon_switch: Option<u8>,
        player: &mut crate::game::player::Player,
        audio_events: &mut crate::audio::events::AudioEventQueue,
    ) {
        if let Some(weapon_idx) = weapon_switch {
            let new_weapon = if weapon_idx == 255 {
                let current = player.weapon as usize;
                let mut next = if current == 0 { 8 } else { current - 1 };
                while next != current {
                    if player.has_weapon[next] {
                        break;
                    }
                    next = if next == 0 { 8 } else { next - 1 };
                }
                match next {
                    0 => Weapon::Gauntlet,
                    1 => Weapon::MachineGun,
                    2 => Weapon::Shotgun,
                    3 => Weapon::GrenadeLauncher,
                    4 => Weapon::RocketLauncher,
                    5 => Weapon::Lightning,
                    6 => Weapon::Railgun,
                    7 => Weapon::Plasmagun,
                    8 => Weapon::BFG,
                    _ => Weapon::Gauntlet,
                }
            } else if weapon_idx == 254 {
                let current = player.weapon as usize;
                let mut next = if current == 8 { 0 } else { current + 1 };
                while next != current {
                    if player.has_weapon[next] {
                        break;
                    }
                    next = if next == 8 { 0 } else { next + 1 };
                }
                match next {
                    0 => Weapon::Gauntlet,
                    1 => Weapon::MachineGun,
                    2 => Weapon::Shotgun,
                    3 => Weapon::GrenadeLauncher,
                    4 => Weapon::RocketLauncher,
                    5 => Weapon::Lightning,
                    6 => Weapon::Railgun,
                    7 => Weapon::Plasmagun,
                    8 => Weapon::BFG,
                    _ => Weapon::Gauntlet,
                }
            } else if weapon_idx < 9 && player.has_weapon[weapon_idx as usize] {
                match weapon_idx {
                    0 => Weapon::Gauntlet,
                    1 => Weapon::MachineGun,
                    2 => Weapon::Shotgun,
                    3 => Weapon::GrenadeLauncher,
                    4 => Weapon::RocketLauncher,
                    5 => Weapon::Lightning,
                    6 => Weapon::Railgun,
                    7 => Weapon::Plasmagun,
                    8 => Weapon::BFG,
                    _ => Weapon::Gauntlet,
                }
            } else {
                return;
            };
            
            if new_weapon as u8 != player.weapon as u8 {
                audio_events.push(audio::events::AudioEvent::WeaponSwitch);
                player.weapon = new_weapon;
                player.weapon_switch_time = player.weapon.switch_time_seconds();
                player.refire = player.weapon_switch_time;
            }
        }
    }

    pub fn process_weapon_fire(
        game_state: &mut GameState,
        shoot_actions: Vec<(f32, f32, f32, u16, Weapon)>,
    ) {
        for (shoot_x, shoot_y, angle, player_id, weapon) in shoot_actions {
            let (muzzle_x, muzzle_y) = if matches!(weapon, Weapon::Railgun) {
                (shoot_x, shoot_y)
            } else {
                (shoot_x, shoot_y)
            };
            
            game_state.muzzle_flashes.push(crate::game::muzzle::MuzzleFlash::new(
                muzzle_x, muzzle_y, angle, weapon,
            ));
            game_state.lights.push(crate::game::light::LightPulse::from_weapon(
                shoot_x + angle.cos() * 18.0,
                shoot_y + angle.sin() * 18.0,
                weapon as u8,
            ));

            Self::handle_weapon_specific_effects(game_state, shoot_x, shoot_y, angle, player_id, weapon);
        }
    }

    fn handle_weapon_specific_effects(
        game_state: &mut GameState,
        shoot_x: f32,
        shoot_y: f32,
        angle: f32,
        player_id: u16,
        weapon: Weapon,
    ) {
        match weapon {
            Weapon::RocketLauncher | Weapon::GrenadeLauncher | Weapon::Plasmagun | Weapon::BFG => {
                let player_idx = game_state.players.iter().position(|p| p.id == player_id);
                let (vel_x, vel_y, has_quad) = if let Some(idx) = player_idx {
                    (game_state.players[idx].vel_x, game_state.players[idx].vel_y, game_state.players[idx].powerups.quad > 0)
                } else {
                    (0.0, 0.0, false)
                };
                
                let mut projectile = crate::game::projectile::Projectile::new(
                    shoot_x,
                    shoot_y,
                    angle,
                    player_id,
                    weapon,
                    vel_x,
                    vel_y,
                );
                if has_quad {
                    projectile.damage *= 3;
                }
                projectile = game_state.create_projectile_with_id(projectile);
                println!("[{:.3}] [WEAPON] Created projectile [{}]: weapon={:?} owner_id={} damage={} has_quad={} pos=({:.1}, {:.1})", 
                    macroquad::prelude::get_time(),
                    projectile.id, weapon, player_id, projectile.damage, has_quad, shoot_x, shoot_y);
                game_state.projectiles.push(projectile);
            }
            Weapon::Railgun => {
                Self::handle_railgun_fire(game_state, shoot_x, shoot_y, angle, player_id);
            }
            Weapon::Shotgun => {
                Self::handle_shotgun_fire(game_state, shoot_x, shoot_y, angle, player_id);
            }
            Weapon::MachineGun => {
                Self::handle_machinegun_fire(game_state, shoot_x, shoot_y, angle, player_id);
            }
            Weapon::Lightning | Weapon::Gauntlet => {
                Self::handle_lightning_gauntlet_fire(game_state, shoot_x, shoot_y, angle, player_id, weapon);
            }
        }
    }

    fn handle_railgun_fire(
        game_state: &mut GameState,
        shoot_x: f32,
        shoot_y: f32,
        angle: f32,
        player_id: u16,
    ) {
        let player_color = crate::game::railgun::get_player_railgun_color(player_id);
        
        let barrel_x = shoot_x;
        let barrel_y = shoot_y;
        
        let (end_x, end_y, hits) = crate::game::railgun::fire_railgun_hitscan(
            barrel_x,
            barrel_y,
            angle,
            player_id,
            &game_state.map,
        );

        game_state.railgun_effects.fire_railgun(
            barrel_x,
            barrel_y,
            end_x,
            end_y,
            player_color,
        );

        let mut damage = Weapon::Railgun.damage();
        if game_state.players.iter().any(|p| p.id == player_id && p.powerups.quad > 0) {
            damage *= 3;
        }

        for (target_idx, hit_damage, hit_x, hit_y) in hits {
            game_state.pending_hits.push((target_idx, hit_damage, hit_x, hit_y, player_id));
        }

        for (i, target) in game_state.players.iter().enumerate() {
            if target.id != player_id && !target.dead {
                let hitbox_height = if target.crouch { PLAYER_HITBOX_HEIGHT_CROUCH } else { PLAYER_HITBOX_HEIGHT };
                let hitbox_width = PLAYER_HITBOX_WIDTH;
                let target_pos = Vec2::new(target.x - hitbox_width / 2.0, target.y - hitbox_height);
                let target_size = Vec2::new(hitbox_width, hitbox_height);

                if crate::game::collision::line_rect_intersect(
                    Vec2::new(barrel_x, barrel_y),
                    Vec2::new(end_x, end_y),
                    target_pos,
                    target_size,
                ) {
                    let dx = target.x - barrel_x;
                    let dy = target.y - barrel_y;
                    let target_dist = (dx * dx + dy * dy).sqrt();
                    
                    let wall_dx = end_x - barrel_x;
                    let wall_dy = end_y - barrel_y;
                    let wall_dist = (wall_dx * wall_dx + wall_dy * wall_dy).sqrt();
                    
                    if target_dist < wall_dist {
                        game_state.pending_hits.push((i, damage, target.x, target.y, player_id));
                        game_state.audio_events.push(
                            crate::audio::events::AudioEvent::RailgunHit { x: target.x },
                        );
                        break;
                    }
                }
            }
        }
        
        for (idx, corpse) in game_state.corpses.iter().enumerate() {
            let hitbox_height = PLAYER_HITBOX_HEIGHT_CROUCH;
            let hitbox_width = PLAYER_HITBOX_WIDTH;
            let target_pos = Vec2::new(corpse.player.x - hitbox_width / 2.0, corpse.player.y - hitbox_height);
            let target_size = Vec2::new(hitbox_width, hitbox_height);

            if crate::game::collision::line_rect_intersect(
                Vec2::new(barrel_x, barrel_y),
                Vec2::new(end_x, end_y),
                target_pos,
                target_size,
            ) {
                let dx = corpse.player.x - barrel_x;
                let dy = corpse.player.y - barrel_y;
                let target_dist = (dx * dx + dy * dy).sqrt();
                
                let wall_dx = end_x - barrel_x;
                let wall_dy = end_y - barrel_y;
                let wall_dist = (wall_dx * wall_dx + wall_dy * wall_dy).sqrt();
                
                if target_dist < wall_dist {
                    let gib_x = corpse.player.x;
                    let gib_y = corpse.player.y;
                    
                    game_state.weapon_hit_effects.push(
                        crate::game::weapon_hit_effect::WeaponHitEffect::new_blood(gib_x, gib_y),
                    );
                    
                    game_state.audio_events.push(crate::audio::events::AudioEvent::PlayerGib { x: gib_x });
                    
                    for _ in 0..15 {
                        game_state.particles.push(crate::game::particle::Particle::new(
                            gib_x,
                            gib_y,
                            rand::gen_range(-6.0, 6.0),
                            rand::gen_range(-9.0, -3.0),
                            true,
                        ));
                    }
                    
                    game_state.gibs.extend(crate::game::gib::spawn_gibs(gib_x, gib_y));
                    game_state.liquid_blood.add_blood(gib_x, gib_y, 3.0, rand::gen_range(-2.0, 2.0), rand::gen_range(-1.0, 1.0));
                    game_state.corpses.remove(idx);
                    break;
                }
            }
        }

        game_state.bullet_holes.push(crate::game::hitscan::BulletHole::new(end_x, end_y));

        let explosion_particles = game_state.railgun_effects.create_explosion_particles();
        game_state.particles.extend(explosion_particles);
    }

    fn handle_shotgun_fire(
        game_state: &mut GameState,
        shoot_x: f32,
        shoot_y: f32,
        angle: f32,
        player_id: u16,
    ) {
        let rays = crate::game::hitscan::fire_hitscan(
            shoot_x, shoot_y, angle, 800.0, 0.15, 10, player_id, 10,
        );

        let mut hits: Vec<(usize, i32)> = Vec::new();

        for (start_x, start_y, ray_angle, owner, dmg) in rays {
            let mut hit_this_ray = false;
            let mut final_end_x = start_x + ray_angle.cos() * 800.0;
            let mut final_end_y = start_y + ray_angle.sin() * 800.0;

            let steps = 60;
            for step in 0..steps {
                let t = (step as f32) / (steps as f32);
                let check_x = start_x + (final_end_x - start_x) * t;
                let check_y = start_y + (final_end_y - start_y) * t;

                if game_state.map.is_solid((check_x / 32.0) as i32, (check_y / 16.0) as i32) {
                    final_end_x = check_x;
                    final_end_y = check_y;
                    game_state.bullet_holes.push(crate::game::hitscan::BulletHole::new(check_x, check_y));
                    hit_this_ray = true;
                    break;
                }
            }

            if !hit_this_ray {
                for (i, target) in game_state.players.iter().enumerate() {
                    if target.id != owner && !target.dead {
                        let hitbox_height = if target.crouch { PLAYER_HITBOX_HEIGHT_CROUCH } else { PLAYER_HITBOX_HEIGHT };
                        let hitbox_width = PLAYER_HITBOX_WIDTH;
                        let target_pos = Vec2::new(
                            target.x - hitbox_width / 2.0,
                            target.y - hitbox_height,
                        );
                        let target_size = Vec2::new(hitbox_width, hitbox_height);

                        if crate::game::collision::line_rect_intersect(
                            Vec2::new(start_x, start_y),
                            Vec2::new(final_end_x, final_end_y),
                            target_pos,
                            target_size,
                        ) {
                            hits.push((i, dmg));
                            hit_this_ray = true;
                            break;
                        }
                    }
                }
                
                if !hit_this_ray {
                    for (idx, corpse) in game_state.corpses.iter().enumerate() {
                        let hitbox_height = PLAYER_HITBOX_HEIGHT_CROUCH;
                        let hitbox_width = PLAYER_HITBOX_WIDTH;
                        let target_pos = Vec2::new(
                            corpse.player.x - hitbox_width / 2.0,
                            corpse.player.y - hitbox_height,
                        );
                        let target_size = Vec2::new(hitbox_width, hitbox_height);

                        if crate::game::collision::line_rect_intersect(
                            Vec2::new(start_x, start_y),
                            Vec2::new(final_end_x, final_end_y),
                            target_pos,
                            target_size,
                        ) {
                            let gib_x = corpse.player.x;
                            let gib_y = corpse.player.y;
                            
                            game_state.weapon_hit_effects.push(
                                crate::game::weapon_hit_effect::WeaponHitEffect::new_blood(gib_x, gib_y),
                            );
                            
                            game_state.audio_events.push(crate::audio::events::AudioEvent::PlayerGib { x: gib_x });
                            
                            for _ in 0..15 {
                                game_state.particles.push(crate::game::particle::Particle::new(
                                    gib_x,
                                    gib_y,
                                    rand::gen_range(-6.0, 6.0),
                                    rand::gen_range(-9.0, -3.0),
                                    true,
                                ));
                            }
                            
                            game_state.liquid_blood.add_blood(gib_x, gib_y, 3.0, rand::gen_range(-2.0, 2.0), rand::gen_range(-1.0, 1.0));
                            game_state.gibs.extend(crate::game::gib::spawn_gibs(gib_x, gib_y));
                            game_state.corpses.remove(idx);
                            hit_this_ray = true;
                            break;
                        }
                    }
                }
            }

            game_state.debug_rays.push(crate::game::hitscan::DebugRay::new(
                start_x,
                start_y,
                final_end_x,
                final_end_y,
                hit_this_ray,
            ));
        }

        for (idx, mut dmg) in hits {
            if let Some(player_idx) = game_state.players.iter().position(|p| p.id == player_id) {
                if game_state.players[player_idx].powerups.quad > 0 {
                    dmg *= 3;
                }
            }
            let target_x = game_state.players[idx].x;
            let target_y = game_state.players[idx].y;
            game_state.pending_hits.push((idx, dmg, target_x, target_y, player_id));
        }
    }

    fn handle_machinegun_fire(
        game_state: &mut GameState,
        shoot_x: f32,
        shoot_y: f32,
        angle: f32,
        player_id: u16,
    ) {
        let rays = crate::game::hitscan::fire_hitscan(
            shoot_x, shoot_y, angle, 1000.0, 0.05, 1, player_id, 7,
        );

        for (start_x, start_y, ray_angle, owner, mut dmg) in rays {
            if let Some(player_idx) = game_state.players.iter().position(|p| p.id == player_id) {
                if game_state.players[player_idx].powerups.quad > 0 {
                    dmg *= 3;
                }
            }
            
            let mut final_end_x = start_x + ray_angle.cos() * 1000.0;
            let mut final_end_y = start_y + ray_angle.sin() * 1000.0;
            let mut hit_wall = false;

            let steps = 60;
            for step in 0..steps {
                let t = (step as f32) / (steps as f32);
                let check_x = start_x + (final_end_x - start_x) * t;
                let check_y = start_y + (final_end_y - start_y) * t;

                if game_state.map.is_solid((check_x / 32.0) as i32, (check_y / 16.0) as i32) {
                    final_end_x = check_x;
                    final_end_y = check_y;
                    game_state.bullet_holes.push(crate::game::hitscan::BulletHole::new(check_x, check_y));
                    hit_wall = true;
                    break;
                }
            }

            if !hit_wall {
                let mut hit_player_idx: Option<usize> = None;
                let mut min_dist = f32::MAX;

                for (i, target) in game_state.players.iter().enumerate() {
                    if target.id != owner && !target.dead {
                        let hitbox_height = if target.crouch { PLAYER_HITBOX_HEIGHT_CROUCH } else { PLAYER_HITBOX_HEIGHT };
                        let hitbox_width = PLAYER_HITBOX_WIDTH;
                        let target_pos = Vec2::new(
                            target.x - hitbox_width / 2.0,
                            target.y - hitbox_height,
                        );
                        let target_size = Vec2::new(hitbox_width, hitbox_height);

                        if crate::game::collision::line_rect_intersect(
                            Vec2::new(start_x, start_y),
                            Vec2::new(final_end_x, final_end_y),
                            target_pos,
                            target_size,
                        ) {
                            let dx = target.x - start_x;
                            let dy = target.y - start_y;
                            let dist = dx * dx + dy * dy;
                            if dist < min_dist {
                                min_dist = dist;
                                hit_player_idx = Some(i);
                            }
                        }
                    }
                }

                if let Some(idx) = hit_player_idx {
                    let target = &game_state.players[idx];
                    game_state.pending_hits.push((idx, dmg, target.x, target.y, player_id));
                } else {
                    for (idx, corpse) in game_state.corpses.iter().enumerate() {
                        let hitbox_height = PLAYER_HITBOX_HEIGHT_CROUCH;
                        let hitbox_width = PLAYER_HITBOX_WIDTH;
                        let target_pos = Vec2::new(
                            corpse.player.x - hitbox_width / 2.0,
                            corpse.player.y - hitbox_height,
                        );
                        let target_size = Vec2::new(hitbox_width, hitbox_height);

                        if crate::game::collision::line_rect_intersect(
                            Vec2::new(start_x, start_y),
                            Vec2::new(final_end_x, final_end_y),
                            target_pos,
                            target_size,
                        ) {
                            let gib_x = corpse.player.x;
                            let gib_y = corpse.player.y;
                            
                            game_state.weapon_hit_effects.push(
                                crate::game::weapon_hit_effect::WeaponHitEffect::new_blood(gib_x, gib_y),
                            );
                            
                            game_state.audio_events.push(crate::audio::events::AudioEvent::PlayerGib { x: gib_x });
                            
                            for _ in 0..15 {
                                game_state.particles.push(crate::game::particle::Particle::new(
                                    gib_x,
                                    gib_y,
                                    rand::gen_range(-6.0, 6.0),
                                    rand::gen_range(-9.0, -3.0),
                                    true,
                                ));
                            }
                            game_state.liquid_blood.add_blood(gib_x, gib_y, 3.0, rand::gen_range(-2.0, 2.0), rand::gen_range(-1.0, 1.0));
                            
                            game_state.gibs.extend(crate::game::gib::spawn_gibs(gib_x, gib_y));
                            game_state.corpses.remove(idx);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn handle_lightning_gauntlet_fire(
        game_state: &mut GameState,
        shoot_x: f32,
        shoot_y: f32,
        angle: f32,
        player_id: u16,
        weapon: Weapon,
    ) {
        let (range, mut damage) = if matches!(weapon, Weapon::Gauntlet) {
            (32.0, 50)
        } else {
            (300.0, 8)
        };
        if let Some(player_idx) = game_state.players.iter().position(|p| p.id == player_id) {
            if game_state.players[player_idx].powerups.quad > 0 {
                damage *= 3;
            }
        }

        let mut hit_idx: Option<usize> = None;
        let mut hit_x = 0.0;
        let mut hit_y = 0.0;

        for (i, target) in game_state.players.iter().enumerate() {
            if target.id != player_id && !target.dead {
                let dx = target.x - shoot_x;
                let dy = target.y - shoot_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < range {
                    let angle_to_target = dy.atan2(dx);
                    let angle_diff = if matches!(weapon, Weapon::Gauntlet) {
                        1.0
                    } else {
                        0.1
                    };

                    if (angle - angle_to_target).abs() < angle_diff || matches!(weapon, Weapon::Gauntlet) {
                        hit_idx = Some(i);
                        hit_x = target.x;
                        hit_y = target.y;
                        break;
                    }
                }
            }
        }

        if let Some(idx) = hit_idx {
            game_state.pending_hits.push((idx, damage, hit_x, hit_y, player_id));

            if matches!(weapon, Weapon::Gauntlet) {
                for _ in 0..5 {
                    game_state.particles.push(crate::game::particle::Particle::new(
                        hit_x,
                        hit_y,
                        rand::gen_range(-3.0, 3.0),
                        rand::gen_range(-3.0, 1.0),
                        false,
                    ));
                }
                game_state.liquid_blood.add_blood(hit_x, hit_y, 1.0, rand::gen_range(-1.0, 1.0), rand::gen_range(-0.5, 0.5));
            }
        }
    }
}
