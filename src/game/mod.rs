pub mod animation;
pub mod award;
pub mod award_shader;
pub mod batched_effects;
pub mod bg_pmove;
pub mod bg_combat;
pub mod bot_ai;
pub mod collision;
pub mod constants;
pub mod damage_number;
pub mod deferred_renderer;
pub mod defrag;
pub mod file_loader;
pub mod gib;
pub mod hitscan;
pub mod item_model;
pub mod light;
pub mod light_grid;
pub mod lightmap;
pub mod liquid_blood;
pub mod map;
pub mod map_loader;
pub mod md3;
pub mod md3_anim;
pub mod md3_render;
pub mod message;
pub mod model_cache;
pub mod model_shader;
pub mod muzzle;
pub mod nav_graph;
pub mod nav_graph_generator;
pub mod particle;
pub mod player;
pub mod player_model;
pub mod procedural_tiles;
pub mod projectile;
pub mod projectile_model_cache;
pub mod q3_shader_parser;
pub mod railgun;
pub mod shader;
pub mod skin_loader;
pub mod smoke;
pub mod sprite;
pub mod story_mode;
pub mod teleport;
pub mod tile_borders;
pub mod tile_rendering;
pub mod tile_shader;
pub mod tile_shader_materials;
pub mod tile_textures;
pub mod trail;
pub mod usercmd;
pub mod weapon;
pub mod weapon_hit_effect;
pub mod weapon_model_cache;

use crate::audio::events::AudioEventQueue;
use crate::network::{NetworkClient, NetworkConfig, NetHud};
use macroquad::prelude::*;

pub struct Corpse {
    pub player: player::Player,
    pub lifetime: f32,
}

pub struct GameState {
    pub players: Vec<player::Player>,
    pub corpses: Vec<Corpse>,
    pub particles: Vec<particle::Particle>,
    pub projectiles: Vec<projectile::Projectile>,
    pub gibs: Vec<gib::Gib>,
    pub gib_model_cache: gib::GibModelCache,
    pub liquid_blood: liquid_blood::LiquidBloodManager,
    pub smokes: Vec<smoke::Smoke>,
    pub trails: Vec<trail::Trail>,
    pub teleports: Vec<teleport::Teleport>,
    pub muzzle_flashes: Vec<muzzle::MuzzleFlash>,
    pub lights: Vec<light::LightPulse>,
    pub explosion_flashes: Vec<light::ExplosionFlash>,
    pub bullet_holes: Vec<hitscan::BulletHole>,
    pub debug_rays: Vec<hitscan::DebugRay>,
    pub pending_hits: Vec<(usize, i32, f32, f32, u16)>,
    pub messages: Vec<message::GameMessage>,
    pub map: map::Map,
    pub time: f64,
    pub frame: u64,
    pub match_time: f32,
    pub time_limit: f32,
    pub model_cache: model_cache::ModelCache,
    pub item_model_cache: item_model::ItemModelCache,
    pub weapon_model_cache: weapon_model_cache::WeaponModelCache,
    pub projectile_model_cache: projectile_model_cache::ProjectileModelCache,
    pub tile_textures: tile_textures::TileTextureCache,
    pub shader_renderer: tile_shader::TileShaderRenderer,
    pub border_renderer: tile_borders::TileBorderRenderer,
    pub audio_events: AudioEventQueue,
    pub debug_md3: bool,
    pub debug_hitboxes: bool,
    pub deferred_renderer: Option<deferred_renderer::DeferredRenderer>,
    pub ambient_light: f32,
    pub is_local_multiplayer: bool,
    pub railgun_effects: railgun::RailgunEffects,
    pub linear_lights: Vec<map::LinearLight>,
    pub weapon_hit_effects: Vec<weapon_hit_effect::WeaponHitEffect>,
    pub weapon_hit_texture_cache: weapon_hit_effect::WeaponHitTextureCache,
    pub muzzle_flash_cache: muzzle::MuzzleFlashCache,
    pub use_item_icons: bool,
    pub nav_graph: Option<nav_graph::NavGraph>,
    pub damage_numbers: Vec<damage_number::DamageNumber>,
    pub disable_shadows: bool,
    pub disable_dynamic_lights: bool,
    pub disable_particles: bool,
    pub disable_deferred: bool,
    pub render_scale: f32,
    pub cartoon_shader: bool,
    pub defrag_mode: Option<defrag::DefragMode>,
    pub story_mode: Option<story_mode::StoryMode>,
    pub network_client: Option<NetworkClient>,
    pub is_multiplayer: bool,
    pub awards: Vec<award::Award>,
    pub award_trackers: std::collections::HashMap<u16, award::AwardTracker>,
    pub award_icon_cache: award::AwardIconCache,
    pub time_announcements: award::TimeAnnouncement,
    pub lead_announcements: award::LeadAnnouncement,
    pub team_advantage_announcements: award::TeamAdvantageAnnouncement,
    pub game_results: award::GameResults,
    pub net_hud: NetHud,
    pub next_projectile_id: u32,
    pub shadow_target: Option<RenderTarget>,
}

impl GameState {
    pub fn create_projectile_with_id(&mut self, mut projectile: projectile::Projectile) -> projectile::Projectile {
        projectile.id = self.next_projectile_id;
        self.next_projectile_id += 1;
        projectile
    }

    fn create_corpse(&mut self, player: &player::Player) {
        if !player.gibbed {
            let corpse_player = player.clone();
            self.corpses.push(Corpse {
                player: corpse_player,
                lifetime: 10.0,
            });
        }
    }
    
    pub fn end_match(&mut self) {
        if !self.game_results.show {
            self.game_results.trigger(&self.players, self.match_time);
        }
    }

    pub fn add_damage_number(&mut self, player_id: u32, target_id: u16, x: f32, y: f32, damage: i32, target_health: i32, target_armor: i32) {
        let combine_time = 1.0f32;

        if let Some(existing) = self.damage_numbers.iter_mut().find(|d| {
            d.target_id == target_id
                && d.lifetime < combine_time
        }) {
            existing.add_damage(damage, target_health, target_armor, x, y);
            return;
        }

        self.damage_numbers
            .push(damage_number::DamageNumber::new(player_id, target_id, x, y, damage, target_health, target_armor));
    }

    fn check_and_award(&mut self, killer_id: u16, _victim_id: u16, _was_airborne: bool, weapon: weapon::Weapon) {
        let tracker = self.award_trackers.entry(killer_id).or_insert_with(award::AwardTracker::new);
        
        let award_type = if matches!(weapon, weapon::Weapon::Railgun) {
            Some(award::AwardType::Impressive)
        } else if tracker.check_excellent(self.match_time) {
            Some(award::AwardType::Excellent)
        } else {
            None
        };

        if let Some(award_type) = award_type {
            self.awards.push(award::Award::new(award_type.clone(), killer_id));
            self.audio_events.push(crate::audio::events::AudioEvent::Award { award_type: award_type.clone() });
            
            if let Some(player) = self.players.iter_mut().find(|p| p.id == killer_id) {
                match award_type {
                    award::AwardType::Excellent => player.excellent_count += 1,
                    award::AwardType::Impressive => player.impressive_count += 1,
                    _ => {}
                }
            }
        }
    }

    pub fn connect_to_server(
        &mut self,
        server_address: &str,
        player_name: &str,
    ) -> Result<(), String> {
        let config = NetworkConfig::default();
        let mut client = NetworkClient::new(config);
        client.connect(player_name.to_string(), server_address)?;
        self.network_client = Some(client);
        self.is_multiplayer = true;
        Ok(())
    }

    pub fn disconnect_from_server(&mut self) {
        if let Some(ref mut client) = self.network_client {
            client.disconnect();
        }
        self.network_client = None;
        self.is_multiplayer = false;
    }

    pub fn send_chat(&mut self, message: String) -> Result<(), String> {
        if let Some(ref mut client) = self.network_client {
            if let Some(player_id) = client.get_player_id() {
                let chat_msg = crate::network::NetMessage::Chat {
                    player_id,
                    message,
                };
                client.send_message(chat_msg)
            } else {
                Err("Not connected to server".to_string())
            }
        } else {
            Err("No network client".to_string())
        }
    }

    pub fn is_connected_to_server(&self) -> bool {
        self.network_client.as_ref()
            .map(|client| client.get_player_id().is_some())
            .unwrap_or(false)
    }

    pub fn update_network(&mut self) {
        if let Some(ref mut client) = self.network_client {
            let messages = client.update();
            for msg in messages {
                self.handle_network_message(msg);
            }
        }
    }

    fn handle_network_message(&mut self, msg: crate::network::NetMessage) {
        use crate::network::NetMessage;
        
        if !matches!(msg, NetMessage::GameStateSnapshot { .. } | NetMessage::GameStateDelta { .. } | NetMessage::Heartbeat | NetMessage::Acknowledgement { .. }) {
            println!("[{:.3}] [CLIENT] Received: {:?}", macroquad::prelude::get_time(), 
                match &msg {
                    NetMessage::PlayerDied { player_id, killer_id, gibbed, .. } => 
                        format!("PlayerDied(player={}, killer={}, gibbed={})", player_id, killer_id, gibbed),
                    NetMessage::PlayerGibbed { player_id, .. } => 
                        format!("PlayerGibbed(player={})", player_id),
                    NetMessage::PlayerRespawn { player_id, .. } => 
                        format!("PlayerRespawn(player={})", player_id),
                    _ => format!("{:?}", msg),
                });
        }
 
        match msg {
            NetMessage::ConnectResponse {
                accepted, reason, ..
            } => {
                if accepted {
                    println!("Connected to server");
                } else {
                    println!("Connection rejected: {}", reason);
                    self.disconnect_from_server();
                }
            }
            NetMessage::MapChange { map_name } => {
                println!("Server changing map to: {}", map_name);
                self.map = map::Map::load_from_file(&map_name)
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to load map '{}': {}", map_name, e);
                        map::Map::new(&map_name)
                    });
                self.players.clear();
                self.projectiles.clear();
                self.particles.clear();
                self.gibs.clear();
                self.smokes.clear();
                self.lights.clear();
                println!("Map loaded: {} spawn points", self.map.spawn_points.len());
            }
            NetMessage::PlayerRespawn { player_id: _player_id, position } => {
                if let Some(ref client) = self.network_client {
                    if Some(_player_id) == client.player_id() {
                        if self.players.is_empty() || !self.players.iter().any(|p| p.id == _player_id) {
                            let mut player = player::Player::new(_player_id, "LocalPlayer".to_string(), false);
                            player.x = position.0;
                            player.y = position.1;
                            player.model = "sarge".to_string();
                            player.should_interpolate = false;
                            self.players.push(player);
                            println!("Local player spawned at ({}, {})", position.0, position.1);
                        }
                    } else {
                        if let Some(player) = self.players.iter_mut().find(|p| p.id == _player_id) {
                            player.x = position.0;
                            player.y = position.1;
                            player.prev_x = position.0;
                            player.prev_y = position.1;
                            player.dead = false;
                            player.gibbed = false;
                            player.health = 100;
                            player.should_interpolate = false;
                            println!("Remote player {} respawned at ({}, {})", _player_id, position.0, position.1);
                        } else {
                            let mut player = player::Player::new(_player_id, format!("Player{}", _player_id), false);
                            player.x = position.0;
                            player.y = position.1;
                            player.model = "visor".to_string();
                            player.should_interpolate = false;
                            self.players.push(player);
                            println!("Remote player {} spawned at ({}, {})", _player_id, position.0, position.1);
                        }
                    }
                }
            }
            NetMessage::GameStateSnapshot { .. } | NetMessage::GameStateDelta { .. } => {
                if let Some(ref mut client) = self.network_client {
                    if let Some(snapshot) = client.get_new_snapshot() {
                        self.sync_players_from_network(snapshot.players);
                        self.sync_projectiles_from_network(snapshot.projectiles);
                    }
                }
            }
            NetMessage::PlayerShoot { player_id, weapon, origin, direction } => {
                self.muzzle_flashes.push(muzzle::MuzzleFlash::new(
                    origin.0,
                    origin.1,
                    direction,
                    unsafe { std::mem::transmute(weapon) },
                ));
                
                self.lights.push(light::LightPulse::from_weapon(
                    origin.0 + direction.cos() * 18.0,
                    origin.1 + direction.sin() * 18.0,
                    weapon,
                ));
            }
            NetMessage::PlayerDamaged { target_id, damage, knockback_x, knockback_y, health_remaining, attacker_id } => {
                let mut player_pos = None;
                let mut target_armor = 0;
                let mut target_model = String::new();
                
                if let Some(player) = self.players.iter_mut().find(|p| p.id == target_id) {
                    let is_local = self.network_client.as_ref().and_then(|c| c.player_id()) == Some(target_id);
                    
                    if is_local {
                        let vel_before = (player.vel_x, player.vel_y);
                        player.vel_x = knockback_x;
                        player.vel_y = knockback_y;
                        
                        println!("[{:.3}] [CLIENT] Knockback applied: ({:.2},{:.2}) -> ({:.2},{:.2})", 
                            macroquad::prelude::get_time(),
                            vel_before.0, vel_before.1,
                            player.vel_x, player.vel_y);
                    }
                    
                    player_pos = Some((player.x, player.y));
                    target_armor = player.armor;
                    target_model = player.model.clone();
                }
                
                if let Some((player_x, player_y)) = player_pos {
                    let local_player_id = self.network_client.as_ref().and_then(|c| c.player_id());
                    
                    if local_player_id == Some(target_id) {
                        self.audio_events.push(crate::audio::events::AudioEvent::PlayerPain {
                            health: health_remaining,
                            x: player_x,
                            model: target_model.clone(),
                        });
                    }
                    
                    if local_player_id == Some(attacker_id) {
                        self.audio_events.push(crate::audio::events::AudioEvent::PlayerHit { damage });
                    }
                    
                    if local_player_id == Some(attacker_id) || local_player_id == Some(target_id) {
                        self.add_damage_number(attacker_id as u32, target_id, player_x, player_y, damage, health_remaining, target_armor);
                    }
                    
                    self.weapon_hit_effects.push(weapon_hit_effect::WeaponHitEffect::new_blood(
                        player_x,
                        player_y,
                    ));
                    
                    for _ in 0..3 {
                        self.particles.push(particle::Particle::new(
                            player_x,
                            player_y,
                            crate::compat_rand::gen_range_f32(-3.0, 3.0),
                            crate::compat_rand::gen_range_f32(-5.0, -1.0),
                            true,
                        ));
                    }
                }
            }
            NetMessage::Chat { message, .. } => {
                println!("[Chat] {}", message);
            }
            NetMessage::PlayerDied {
                player_id,
                killer_id,
                gibbed,
                position,
                velocity,
            } => {
                println!("[{:.3}] Player {} was killed by {} (gibbed: {})", 
                    macroquad::prelude::get_time(), player_id, killer_id, gibbed);
                
                if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                    println!("[{:.3}] [CLIENT] Setting player {} dead=true, gibbed={}, pos=({:.1},{:.1}), vel=({:.2},{:.2})", 
                        macroquad::prelude::get_time(), player_id, gibbed, position.0, position.1, velocity.0, velocity.1);
                    
                    if !gibbed {
                        let corpse_player = player.clone();
                        self.corpses.push(Corpse {
                            player: corpse_player,
                            lifetime: 10.0,
                        });
                    }
                    
                    player.dead = true;
                    player.gibbed = gibbed;
                    player.x = position.0;
                    player.y = position.1;
                    player.vel_x = velocity.0;
                    player.vel_y = velocity.1;
                    player.animation_time = 0.0;
                    
                    if gibbed {
                        self.audio_events.push(crate::audio::events::AudioEvent::PlayerGib {
                            x: position.0,
                        });
                    } else {
                        self.audio_events.push(crate::audio::events::AudioEvent::PlayerDeath {
                            x: position.0,
                            model: player.model.clone(),
                        });
                    }
                    
                    let particle_count = if gibbed { 20 } else { 10 };
                    for _ in 0..particle_count {
                        self.particles.push(particle::Particle::new(
                            position.0,
                            position.1,
                            crate::compat_rand::gen_range_f32(-5.0, 5.0),
                            crate::compat_rand::gen_range_f32(-8.0, -2.0),
                            true,
                        ));
                    }
                    
                    if gibbed {
                        self.gibs.extend(gib::spawn_gibs(position.0, position.1));
                    }
                }
            }
            NetMessage::PlayerGibbed {
                player_id,
                position,
            } => {
                println!("[{:.3}] Player {} was gibbed at ({:.1}, {:.1})", 
                    macroquad::prelude::get_time(), player_id, position.0, position.1);
                
                if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                    player.gibbed = true;
                    
                    self.audio_events.push(crate::audio::events::AudioEvent::PlayerGib {
                        x: position.0,
                    });
                    
                    for _ in 0..15 {
                        self.particles.push(particle::Particle::new(
                            position.0,
                            position.1,
                            crate::compat_rand::gen_range_f32(-6.0, 6.0),
                            crate::compat_rand::gen_range_f32(-9.0, -3.0),
                            true,
                        ));
                    }
                    
                    self.gibs.extend(gib::spawn_gibs(position.0, position.1));
                }
            }
            NetMessage::Disconnect { player_id, reason } => {
                println!("[{:.3}] Player {} disconnected: {}", 
                    macroquad::prelude::get_time(), player_id, reason);
                
                self.players.retain(|p| p.id != player_id);
                
                println!("[{:.3}] Removed player {}, {} players remain", 
                    macroquad::prelude::get_time(), player_id, self.players.len());
            }
            _ => {}
        }
    }

    fn sync_players_from_network(&mut self, network_players: Vec<crate::network::PlayerState>) {
        let local_player_id = self.network_client.as_ref().and_then(|c| c.player_id());
        
        if let Some(ref mut net_client) = self.network_client {
            let interp_time = net_client.get_interpolation_time();
            
            for net_player in network_players {
                if Some(net_player.player_id) == local_player_id {
                    if let Some(player) = self.players.iter_mut().find(|p| p.id == net_player.player_id) {
                        let error_x = (player.x - net_player.position.0).abs();
                        let error_y = (player.y - net_player.position.1).abs();
                        let error = (error_x * error_x + error_y * error_y).sqrt();
                        
                        if error > 50.0 {
                            println!("[CLIENT] Large error {:.1}px, snapping to server", error);
                            player.x = net_player.position.0;
                            player.y = net_player.position.1;
                            player.vel_x = net_player.velocity.0;
                            player.vel_y = net_player.velocity.1;
                        }
                        
                        player.health = net_player.health;
                        player.armor = net_player.armor;
                        player.frags = net_player.frags;
                        player.deaths = net_player.deaths;
                        player.powerups.quad = net_player.powerup_quad;
                    }
                    continue;
                }
                
                if let Some(player) = self.players.iter_mut().find(|p| p.id == net_player.player_id) {
                    if player.dead || net_player.is_dead {
                        continue;
                    }
                    
                    player.health = net_player.health;
                    player.armor = net_player.armor;
                    player.weapon = unsafe { std::mem::transmute(net_player.weapon) };
                    player.frags = net_player.frags;
                    player.deaths = net_player.deaths;
                    player.powerups.quad = net_player.powerup_quad;
                    player.crouch = net_player.is_crouching;
                    player.refire = if net_player.is_attacking { 0.1 } else { 0.0 };
                    player.should_interpolate = true;
                } else {
                    let player_id = net_player.player_id;
                    // println!("[{:.3}] *** [CLIENT] CREATING NEW PLAYER {} at ({:.1},{:.1}) ***", 
                        // macroquad::prelude::get_time(), player_id, net_player.position.0, net_player.position.1);
                    
                    let mut player = player::Player::new(player_id, format!("Player{}", player_id), false);
                    player.x = net_player.position.0;
                    player.y = net_player.position.1;
                    player.prev_x = net_player.position.0;
                    player.prev_y = net_player.position.1;
                    player.vel_x = net_player.velocity.0;
                    player.vel_y = net_player.velocity.1;
                    player.angle = net_player.angle;
                    player.health = net_player.health;
                    player.armor = net_player.armor;
                    player.was_in_air = !net_player.on_ground;
                    player.crouch = net_player.is_crouching;
                    player.model = "visor".to_string();
                    player.should_interpolate = false;
                    self.players.push(player);
                }
            }
        }
    }
    
    fn sync_projectiles_from_network(&mut self, network_projectiles: Vec<crate::network::ProjectileState>) {
        static mut LAST_SYNC_CALL: f64 = 0.0;
        unsafe {
            if macroquad::prelude::get_time() - LAST_SYNC_CALL > 2.0 {
                println!("[SYNC] sync_projectiles called with {} network projectiles, have {} local", 
                    network_projectiles.len(), self.projectiles.len());
                LAST_SYNC_CALL = macroquad::prelude::get_time();
            }
        }
        
        let current_time = (macroquad::prelude::get_time() * 1000.0) as u32;
        let local_player_id = self.network_client.as_ref().and_then(|c| c.player_id()).unwrap_or(0);
        
        for net_proj in &network_projectiles {
            if let Some(projectile) = self.projectiles.iter_mut().find(|p| p.id == net_proj.id) {
                let pos = net_proj.trajectory.evaluate(current_time);
                let vel = net_proj.trajectory.evaluate_velocity(current_time);
                
                projectile.x = pos.0;
                projectile.y = pos.1;
                projectile.vel_x = vel.0 / 1000.0;
                projectile.vel_y = vel.1 / 1000.0;
                projectile.active = true;
            } else {
                let pos = net_proj.trajectory.evaluate(current_time);
                let vel = net_proj.trajectory.evaluate_velocity(current_time);
                
                let weapon: weapon::Weapon = unsafe { std::mem::transmute(net_proj.weapon_type) };
                let mut proj = projectile::Projectile::new(
                    pos.0,
                    pos.1,
                    0.0,
                    net_proj.owner_id,
                    weapon,
                    0.0,
                    0.0,
                );
                proj.id = net_proj.id;
                proj.vel_x = vel.0 / 1000.0;
                proj.vel_y = vel.1 / 1000.0;
                
                static mut LAST_CREATE_PRINT: f64 = 0.0;
                unsafe {
                    if macroquad::prelude::get_time() - LAST_CREATE_PRINT > 1.0 {
                        println!("[SYNC] Creating NEW projectile {} from snapshot (owner={} weapon={:?})", 
                            proj.id, net_proj.owner_id, weapon);
                        LAST_CREATE_PRINT = macroquad::prelude::get_time();
                    }
                }
                
                self.projectiles.push(proj);
            }
        }
        
        let before_count = self.projectiles.len();
        self.projectiles.retain(|p| {
            if p.owner_id == local_player_id {
                return p.active;
            }
            let keep = network_projectiles.iter().any(|np| np.id == p.id);
            if !keep {
                static mut LAST_REMOVE_PRINT: f64 = 0.0;
                unsafe {
                    if macroquad::prelude::get_time() - LAST_REMOVE_PRINT > 1.0 {
                        println!("[SYNC] Removing projectile {} (not in snapshot)", p.id);
                        LAST_REMOVE_PRINT = macroquad::prelude::get_time();
                    }
                }
            }
            keep
        });
        let after_count = self.projectiles.len();
        
        if before_count != after_count {
            static mut LAST_SYNC_PRINT: f64 = 0.0;
            unsafe {
                if macroquad::prelude::get_time() - LAST_SYNC_PRINT > 1.0 {
                    println!("[SYNC] Projectiles: {} -> {} (removed {})", 
                        before_count, after_count, before_count - after_count);
                    LAST_SYNC_PRINT = macroquad::prelude::get_time();
                }
            }
        }
    }
    
    pub fn new(map_name: &str) -> Self {
        let teleports = Vec::new();

        Self {
            players: Vec::new(),
            corpses: Vec::new(),
            particles: Vec::new(),
            projectiles: Vec::new(),
            gibs: Vec::new(),
            gib_model_cache: gib::GibModelCache::new(),
            liquid_blood: liquid_blood::LiquidBloodManager::new(),
            smokes: Vec::new(),
            trails: Vec::new(),
            teleports,
            muzzle_flashes: Vec::new(),
            lights: Vec::new(),
            explosion_flashes: Vec::new(),
            bullet_holes: Vec::new(),
            debug_rays: Vec::new(),
            pending_hits: Vec::new(),
            messages: Vec::new(),
            map: map::Map::new(map_name),
            time: 0.0,
            frame: 0,
            match_time: 0.0,
            debug_md3: false,
            debug_hitboxes: false,
            time_limit: 600.0,
            model_cache: model_cache::ModelCache::new(),
            item_model_cache: item_model::ItemModelCache::new(),
            weapon_model_cache: weapon_model_cache::WeaponModelCache::new(),
            projectile_model_cache: projectile_model_cache::ProjectileModelCache::new(),
            tile_textures: tile_textures::TileTextureCache::new(),
            shader_renderer: tile_shader::TileShaderRenderer::new(),
            border_renderer: tile_borders::TileBorderRenderer::new(),
            audio_events: AudioEventQueue::new(),
            deferred_renderer: None,
            ambient_light: 0.15,
            is_local_multiplayer: false,
            railgun_effects: railgun::RailgunEffects::new(),
            linear_lights: Vec::new(),
            weapon_hit_effects: Vec::new(),
            weapon_hit_texture_cache: weapon_hit_effect::WeaponHitTextureCache::new(),
            muzzle_flash_cache: muzzle::MuzzleFlashCache::new(),
            use_item_icons: false,
            nav_graph: Self::load_nav_graph(map_name),
            damage_numbers: Vec::new(),
            disable_shadows: false,
            disable_dynamic_lights: false,
            disable_particles: false,
            disable_deferred: false,
            render_scale: 1.0,
            cartoon_shader: false,
            defrag_mode: Self::load_defrag_mode(map_name),
            story_mode: None,
            network_client: None,
            is_multiplayer: false,
            awards: Vec::new(),
            award_trackers: std::collections::HashMap::new(),
            award_icon_cache: award::AwardIconCache::new(),
            time_announcements: award::TimeAnnouncement::new(),
            lead_announcements: award::LeadAnnouncement::new(),
            team_advantage_announcements: award::TeamAdvantageAnnouncement::new(),
            game_results: award::GameResults::new(),
            net_hud: NetHud::new(),
            next_projectile_id: 0,
            shadow_target: None,
        }
    }

    pub async fn new_async(map_name: &str) -> Self {
        let teleports = Vec::new();
        let mut award_icon_cache = award::AwardIconCache::new();
        award_icon_cache.load().await;

        Self {
            players: Vec::new(),
            corpses: Vec::new(),
            particles: Vec::new(),
            projectiles: Vec::new(),
            gibs: Vec::new(),
            gib_model_cache: {
                let mut cache = gib::GibModelCache::new();
                cache.load().await;
                cache
            },
            liquid_blood: liquid_blood::LiquidBloodManager::new(),
            smokes: Vec::new(),
            trails: Vec::new(),
            teleports,
            muzzle_flashes: Vec::new(),
            lights: Vec::new(),
            explosion_flashes: Vec::new(),
            bullet_holes: Vec::new(),
            debug_rays: Vec::new(),
            pending_hits: Vec::new(),
            messages: Vec::new(),
            map: map::Map::new_async(map_name).await,
            time: 0.0,
            frame: 0,
            match_time: 0.0,
            debug_md3: false,
            debug_hitboxes: false,
            time_limit: 600.0,
            model_cache: model_cache::ModelCache::new(),
            item_model_cache: item_model::ItemModelCache::new(),
            weapon_model_cache: weapon_model_cache::WeaponModelCache::new(),
            projectile_model_cache: projectile_model_cache::ProjectileModelCache::new(),
            tile_textures: tile_textures::TileTextureCache::new(),
            shader_renderer: tile_shader::TileShaderRenderer::new(),
            border_renderer: tile_borders::TileBorderRenderer::new(),
            audio_events: AudioEventQueue::new(),
            deferred_renderer: None,
            ambient_light: 0.06,
            is_local_multiplayer: false,
            railgun_effects: railgun::RailgunEffects::new(),
            linear_lights: Vec::new(),
            weapon_hit_effects: Vec::new(),
            weapon_hit_texture_cache: weapon_hit_effect::WeaponHitTextureCache::new(),
            muzzle_flash_cache: muzzle::MuzzleFlashCache::new(),
            use_item_icons: false,
            nav_graph: Self::load_nav_graph(map_name),
            damage_numbers: Vec::new(),
            disable_shadows: false,
            disable_dynamic_lights: false,
            disable_particles: false,
            disable_deferred: false,
            render_scale: 1.0,
            cartoon_shader: false,
            defrag_mode: Self::load_defrag_mode(map_name),
            story_mode: None,
            network_client: None,
            is_multiplayer: false,
            awards: Vec::new(),
            award_trackers: std::collections::HashMap::new(),
            award_icon_cache,
            time_announcements: award::TimeAnnouncement::new(),
            lead_announcements: award::LeadAnnouncement::new(),
            team_advantage_announcements: award::TeamAdvantageAnnouncement::new(),
            game_results: award::GameResults::new(),
            net_hud: NetHud::new(),
            next_projectile_id: 0,
            shadow_target: None,
        }
    }

    fn load_defrag_mode(map_name: &str) -> Option<defrag::DefragMode> {
        let path = format!("maps/{}_defrag.json", map_name);

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(mode) = serde_json::from_str::<defrag::DefragMode>(&json) {
                    println!(
                        "[Defrag] Loaded defrag mode: {} checkpoints",
                        mode.checkpoints.len()
                    );
                    return Some(mode);
                }
            }
        }

        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_nav_graph(map_name: &str) -> Option<nav_graph::NavGraph> {
        let path = format!("maps/{}_navgraph.json", map_name);
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(graph) = serde_json::from_str::<nav_graph::NavGraph>(&json) {
                println!(
                    "[Nav] Loaded navigation graph: {} nodes, {} edges",
                    graph.nodes.len(),
                    graph.edges.len()
                );
                return Some(graph);
            }
        }

        println!("[Nav] No navigation graph found, generating...");
        if let Ok(map) = map::Map::load_from_file(map_name) {
            let generator = nav_graph_generator::NavGraphGenerator::new(map);
            let graph = generator.generate();

            let json = serde_json::to_string_pretty(&graph).unwrap();
            let _ = std::fs::write(&path, json);
            println!(
                "[Nav] Generated and saved: {} nodes, {} edges",
                graph.nodes.len(),
                graph.edges.len()
            );

            return Some(graph);
        }

        println!("[Nav] Failed to generate navigation graph");
        None
    }

    #[cfg(target_arch = "wasm32")]
    fn load_nav_graph(_map_name: &str) -> Option<nav_graph::NavGraph> {
        None
    }

    pub fn update(&mut self, dt: f32) {
        self.disable_shadows = !crate::cvar::get_cvar_bool("cg_shadows");
        self.disable_dynamic_lights = !crate::cvar::get_cvar_bool("r_dynamiclight");
        self.use_item_icons = crate::cvar::get_cvar_bool("cg_simpleItems");

        if self.is_multiplayer {
            let local_player_id = self.network_client.as_ref().and_then(|c| c.player_id());
            
            if let Some(ref mut client) = self.network_client {
                let interp_time = client.get_interpolation_time();
                let debug_interp = crate::cvar::get_cvar_bool("net_showinterp");
                
                for player in &mut self.players {
                    if Some(player.id) != local_player_id {
                        let will_interpolate = client.interpolate_player(player.id, interp_time).is_some();
                        
                        if self.frame % 60 == 0 && player.dead {
                            println!("[{:.3}] [DEATH DEBUG] p{} dead={} gibbed={} can_interp={}", 
                                macroquad::prelude::get_time(), player.id, player.dead, player.gibbed, will_interpolate);
                        }
                        
                        if player.dead && !player.gibbed {
                            use crate::game::bg_pmove::{pmove, PmoveState, PmoveCmd};
                            
                            if self.frame % 60 == 0 {
                                println!("[{:.3}] [CORPSE PHYSICS] p{} dead, applying gravity at ({:.1},{:.1}) vel=({:.2},{:.2})", 
                                    macroquad::prelude::get_time(), player.id, player.x, player.y, player.vel_x, player.vel_y);
                            }
                            
                            let state = PmoveState {
                                x: player.x,
                                y: player.y,
                                vel_x: player.vel_x,
                                vel_y: player.vel_y,
                                was_in_air: true,
                            };
                            
                            let cmd = PmoveCmd {
                                move_right: 0.0,
                                jump: false,
                                crouch: false,
                                haste_active: false,
                            };
                            
                            let result = pmove(&state, &cmd, dt, &self.map);
                            player.x = result.new_x;
                            player.y = result.new_y;
                            player.vel_x = result.new_vel_x;
                            player.vel_y = result.new_vel_y;
                            player.cx = player.x;
                            player.cy = player.y;
                        } else if let Some(interpolated) = client.interpolate_player(player.id, interp_time) {
                            if debug_interp && self.frame % 60 == 0 {
                                println!("[{:.3}] [INTERP] p{} pos=({:.1},{:.1}) vel=({:.2},{:.2})", 
                                    macroquad::prelude::get_time(), player.id,
                                    interpolated.position.0, interpolated.position.1,
                                    interpolated.velocity.0, interpolated.velocity.1);
                            }
                            
                            player.cx = interpolated.position.0;
                            player.cy = interpolated.position.1;
                            
                            player.x = interpolated.position.0;
                            player.y = interpolated.position.1;
                            player.vel_x = interpolated.velocity.0;
                            player.vel_y = interpolated.velocity.1;
                            player.angle = interpolated.angle;
                            player.was_in_air = !interpolated.on_ground;
                        } else {
                            if debug_interp && self.frame % 60 == 0 {
                                println!("[{:.3}] [INTERP] p{} - NO DATA", 
                                    macroquad::prelude::get_time(), player.id);
                            }
                            player.cx = player.x;
                            player.cy = player.y;
                        }
                    } else {
                        player.cx = player.x;
                        player.cy = player.y;
                    }
                }
            }
        } else {
            for player in &mut self.players {
                player.cx = player.x;
                player.cy = player.y;
            }
        }

        self.time += dt as f64;
        self.frame += 1;
        self.match_time += dt;

        if let Some(ref mut story) = self.story_mode {
            let (new_enemies, _should_change_level) =
                story.update(dt, &mut self.players, &self.map);
            for enemy in new_enemies {
                self.players.push(enemy);
            }
        }

        let local_player_id = if self.is_multiplayer {
            self.network_client.as_ref().and_then(|c| c.player_id())
        } else {
            None
        };

        for player in &mut self.players {
            player.update_timers(dt);
            
            if self.is_multiplayer && Some(player.id) != local_player_id {
                for jumppad in &self.map.jumppads {
                    if jumppad.check_collision(player.x, player.y) && player.vel_y >= -1.0 {
                        if player.somersault_time <= 0.0 && macroquad::prelude::rand::gen_range(0, 10) == 0 {
                            player.somersault_time = 1.0;
                        }
                    }
                }
            }

            if let Some(model) = self.model_cache.get_or_load(&player.model) {
                let on_ground = !player.was_in_air;
                
                let is_walking = if self.is_multiplayer && Some(player.id) != local_player_id {
                    if !on_ground {
                        false
                    } else {
                        let vel_mag = (player.vel_x.powi(2) + player.vel_y.powi(2)).sqrt();
                        vel_mag > 0.5
                    }
                } else {
                    on_ground && player.vel_x.abs() > 0.5
                };
                let is_attacking = player.refire > 0.0;

                if let Some(config) = &model.anim_config {
                    let (lf, uf, new_time) = player_model::PlayerModel::compute_frames(
                        config,
                        dt,
                        is_walking,
                        is_attacking,
                        on_ground,
                        player.dead,
                        player.crouch,
                        player.animation_time,
                    );
                    player.lower_frame = lf;
                    player.upper_frame = uf;
                    player.animation_time = new_time;
                } else {
                    player.animation_time += dt * 10.0;
                    if let Some(ref lower) = model.lower {
                        let num_frames = lower.header.num_bone_frames as usize;
                        if num_frames > 0 {
                            player.lower_frame =
                                ((player.animation_time as usize) % num_frames).min(190);
                            player.upper_frame = player.lower_frame.min(152);
                        }
                    }
                }
            }
        }

        for corpse in &mut self.corpses {
            corpse.lifetime -= dt;
            
            if let Some(model) = self.model_cache.get_or_load(&corpse.player.model) {
                if let Some(config) = &model.anim_config {
                    let (lf, uf, new_time) = player_model::PlayerModel::compute_frames(
                        config,
                        dt,
                        false,
                        false,
                        false,
                        true,
                        false,
                        corpse.player.animation_time,
                    );
                    corpse.player.lower_frame = lf;
                    corpse.player.upper_frame = uf;
                    corpse.player.animation_time = new_time;
                }
            }
            
            if !corpse.player.gibbed {
                use crate::game::bg_pmove::{pmove, PmoveState, PmoveCmd};
                
                let state = PmoveState {
                    x: corpse.player.x,
                    y: corpse.player.y,
                    vel_x: corpse.player.vel_x,
                    vel_y: corpse.player.vel_y,
                    was_in_air: true,
                };
                
                let cmd = PmoveCmd {
                    move_right: 0.0,
                    jump: false,
                    crouch: false,
                    haste_active: false,
                };
                
                let result = pmove(&state, &cmd, dt, &self.map);
                corpse.player.x = result.new_x;
                corpse.player.y = result.new_y;
                corpse.player.vel_x = result.new_vel_x;
                corpse.player.vel_y = result.new_vel_y;
            }
        }
        
        self.corpses.retain(|c| c.lifetime > 0.0);

        for jumppad in &mut self.map.jumppads {
            jumppad.update();
        }

        for teleport in &mut self.teleports {
            teleport.update();
        }

        let map_ptr = &self.map as *const map::Map;
        let dt_norm = dt * 60.0;
        for item in &mut self.map.items {
            if item.active && item.dropped {
                let is_moving = item.vel_x.abs() > 0.01 || item.vel_y.abs() > 0.01;
                
                if is_moving {
                    item.vel_y += 0.5 * dt_norm;
                    if item.vel_y > 15.0 {
                        item.vel_y = 15.0;
                    }
                    
                    item.pitch += item.spin_pitch * dt_norm;
                    item.yaw += item.spin_yaw * dt_norm;
                    item.roll += item.spin_roll * dt_norm;
                    
                    item.x += item.vel_x * dt_norm;
                    item.y += item.vel_y * dt_norm;
                    
                    let tile_x = (item.x / 32.0) as i32;
                    let tile_y = ((item.y + 8.0) / 16.0) as i32;
                    
                    let is_solid = unsafe { (*map_ptr).is_solid(tile_x, tile_y) };
                    
                    if is_solid {
                        item.vel_y = -item.vel_y * 0.4;
                        item.vel_x *= 0.7;
                        item.spin_pitch *= 0.8;
                        item.spin_yaw *= 0.8;
                        item.spin_roll *= 0.8;
                        
                        let max_corrections = 16;
                        let mut correction_count = 0;
                        while correction_count < max_corrections {
                            let check_y = ((item.y + 8.0) / 16.0) as i32;
                            if unsafe { (*map_ptr).is_solid(tile_x, check_y) } {
                                item.y -= 1.0;
                                correction_count += 1;
                            } else {
                                break;
                            }
                        }
                        
                        if item.vel_y.abs() < 0.5 && item.vel_x.abs() < 0.5 {
                            item.vel_y = 0.0;
                            item.vel_x *= 0.95;
                            if item.dropped {
                                item.y -= 30.0;
                            }
                        }
                    }
                    
                    let wall_tile_left = ((item.x - 8.0) / 32.0) as i32;
                    let wall_tile_right = ((item.x + 8.0) / 32.0) as i32;
                    let wall_y = (item.y / 16.0) as i32;
                    
                    let hit_wall = unsafe { 
                        (*map_ptr).is_solid(wall_tile_left, wall_y) || (*map_ptr).is_solid(wall_tile_right, wall_y)
                    };
                    
                    if hit_wall {
                        item.vel_x = -item.vel_x * 0.4;
                    }
                } else {
                    let target_spin = 0.02;
                    let lerp_factor = 0.05 * dt_norm;
                    
                    item.spin_pitch = item.spin_pitch * (1.0 - lerp_factor);
                    item.spin_yaw = item.spin_yaw * (1.0 - lerp_factor) + target_spin * lerp_factor;
                    item.spin_roll = item.spin_roll * (1.0 - lerp_factor);
                    
                    item.pitch += item.spin_pitch * dt_norm;
                    item.yaw += item.spin_yaw * dt_norm;
                    item.roll += item.spin_roll * dt_norm;
                    
                    let target_pitch = 0.0;
                    let target_roll = 0.0;
                    item.pitch = item.pitch * (1.0 - lerp_factor) + target_pitch * lerp_factor;
                    item.roll = item.roll * (1.0 - lerp_factor) + target_roll * lerp_factor;
                }
            }
        }

        for player in &mut self.players {
            if player.dead {
                continue;
            }
            
            for item in &mut self.map.items {
                if !item.active {
                    if item.respawn_time > 0 {
                        item.respawn_time -= 1;
                    } else {
                        item.active = true;
                    }
                    continue;
                }

                let dx = player.x - item.x;
                let dy = player.y - item.y;
                if (dx * dx + dy * dy).sqrt() < 24.0 {
                    use map::ItemType::*;
                    match item.item_type {
                        Health25 => {
                            if player.health < 100 {
                                player.health = (player.health + 25).min(100);
                                item.active = false;
                                item.respawn_time = self::constants::ITEM_RESPAWN_HEALTH;
                                self.audio_events.push(
                                    crate::audio::events::AudioEvent::ItemPickup { x: item.x },
                                );
                            }
                        }
                        Health50 => {
                            if player.health < 100 {
                                player.health = (player.health + 50).min(100);
                                item.active = false;
                                item.respawn_time = self::constants::ITEM_RESPAWN_HEALTH;
                                self.audio_events.push(
                                    crate::audio::events::AudioEvent::ItemPickup { x: item.x },
                                );
                            }
                        }
                        Health100 => {
                            if player.health < 200 {
                                player.health = (player.health + 100).min(200);
                                item.active = false;
                                item.respawn_time = self::constants::ITEM_RESPAWN_HEALTH;
                                self.audio_events.push(
                                    crate::audio::events::AudioEvent::ItemPickup { x: item.x },
                                );
                            }
                        }
                        Armor50 => {
                            if player.armor < 100 {
                                player.armor = (player.armor + 50).min(100);
                                item.active = false;
                                item.respawn_time = self::constants::ITEM_RESPAWN_ARMOR;
                                self.audio_events.push(
                                    crate::audio::events::AudioEvent::ArmorPickup { x: item.x },
                                );
                            }
                        }
                        Armor100 => {
                            if player.armor < 200 {
                                player.armor = (player.armor + 100).min(200);
                                item.active = false;
                                item.respawn_time = self::constants::ITEM_RESPAWN_ARMOR;
                                self.audio_events.push(
                                    crate::audio::events::AudioEvent::ArmorPickup { x: item.x },
                                );
                            }
                        }
                        RocketLauncher => {
                            if item.dropped {
                                println!("[ITEM PICKUP] Player {} picked up DROPPED RocketLauncher at ({:.1},{:.1})", 
                                    player.id, item.x, item.y);
                            }
                            player.has_weapon[4] = true;
                            player.ammo[4] = (player.ammo[4] + 10).min(100);
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_WEAPON;
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::WeaponPickup { x: item.x });
                        }
                        Railgun => {
                            if item.dropped {
                                println!("[ITEM PICKUP] Player {} picked up DROPPED Railgun at ({:.1},{:.1})", 
                                    player.id, item.x, item.y);
                            }
                            player.has_weapon[6] = true;
                            player.ammo[6] = (player.ammo[6] + 10).min(100);
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_WEAPON;
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::WeaponPickup { x: item.x });
                        }
                        Plasmagun => {
                            if item.dropped {
                                println!("[ITEM PICKUP] Player {} picked up DROPPED Plasmagun at ({:.1},{:.1})", 
                                    player.id, item.x, item.y);
                            }
                            player.has_weapon[7] = true;
                            player.ammo[7] = (player.ammo[7] + 50).min(200);
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_WEAPON;
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::WeaponPickup { x: item.x });
                        }
                        Quad => {
                            player.powerups.quad = self::constants::POWERUP_DURATION_QUAD;
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_POWERUP;
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::PowerupPickup { x: item.x },
                            );
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::QuadDamage,
                            );
                        }
                        Regen => {
                            player.powerups.regen = self::constants::POWERUP_DURATION_REGEN;
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_POWERUP;
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::PowerupPickup { x: item.x },
                            );
                        }
                        Battle => {
                            player.powerups.battle = self::constants::POWERUP_DURATION_BATTLE;
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_POWERUP;
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::PowerupPickup { x: item.x },
                            );
                        }
                        Flight => {
                            player.powerups.flight = self::constants::POWERUP_DURATION_FLIGHT;
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_POWERUP;
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::PowerupPickup { x: item.x },
                            );
                        }
                        Haste => {
                            player.powerups.haste = self::constants::POWERUP_DURATION_HASTE;
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_POWERUP;
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::PowerupPickup { x: item.x },
                            );
                        }
                        Invis => {
                            player.powerups.invis = self::constants::POWERUP_DURATION_INVIS;
                            item.active = false;
                            item.respawn_time = self::constants::ITEM_RESPAWN_POWERUP;
                            self.audio_events.push(
                                crate::audio::events::AudioEvent::PowerupPickup { x: item.x },
                            );
                        }
                        Shotgun => {
                            if item.dropped {
                                println!("[ITEM PICKUP] Player {} picked up DROPPED Shotgun at ({:.1},{:.1})", 
                                    player.id, item.x, item.y);
                            }
                            player.has_weapon[2] = true;
                            player.ammo[2] = (player.ammo[2] + 10).min(100);
                            item.active = false;
                            item.respawn_time = 300;
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::WeaponPickup { x: item.x });
                        }
                        GrenadeLauncher => {
                            if item.dropped {
                                println!("[ITEM PICKUP] Player {} picked up DROPPED GrenadeLauncher at ({:.1},{:.1})", 
                                    player.id, item.x, item.y);
                            }
                            player.has_weapon[3] = true;
                            player.ammo[3] = (player.ammo[3] + 10).min(100);
                            item.active = false;
                            item.respawn_time = 300;
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::WeaponPickup { x: item.x });
                        }
                        BFG => {
                            if item.dropped {
                                println!("[ITEM PICKUP] Player {} picked up DROPPED BFG at ({:.1},{:.1})", 
                                    player.id, item.x, item.y);
                            }
                            player.has_weapon[8] = true;
                            player.ammo[8] = (player.ammo[8] + 15).min(200);
                            item.active = false;
                            item.respawn_time = 600;
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::WeaponPickup { x: item.x });
                        }
                    }
                }
            }
        }

        self.map.items.retain(|item| {
            if item.dropped && !item.active {
                println!("[ITEM CLEANUP] Removing dropped {:?} at ({:.1},{:.1})", 
                    item.item_type, item.x, item.y);
                false
            } else {
                true
            }
        });

        self.particles.retain_mut(|p| p.update(dt, &self.map));
        self.gibs.retain_mut(|g| g.update(dt, &self.map));
        self.smokes.retain_mut(|s| s.update());
        self.trails.retain_mut(|t| t.update());
        self.muzzle_flashes.retain_mut(|m| m.update());
        self.bullet_holes.retain_mut(|b| b.update());
        self.debug_rays.retain_mut(|r| r.update());
        self.lights.retain_mut(|l| l.update());
        self.messages.retain_mut(|msg| msg.update());
        self.item_model_cache.update_all(dt);
        self.railgun_effects.update(dt);
        self.weapon_hit_effects.retain_mut(|e| e.update());
        self.damage_numbers.retain_mut(|d| d.update());


        if self.frame % 3 == 0 {
            for proj in &self.projectiles {
                if proj.active {
                    if matches!(proj.weapon_type, weapon::Weapon::RocketLauncher) {
                        let smoke_x = proj.x - proj.vel_x * 0.5;
                        let smoke_y = proj.y - proj.vel_y * 0.5;
                        self.smokes.push(smoke::Smoke::new(smoke_x, smoke_y, 8.0));
                    } else if matches!(proj.weapon_type, weapon::Weapon::GrenadeLauncher) {
                        let smoke_x = proj.x - proj.vel_x * 0.5;
                        let smoke_y = proj.y - proj.vel_y * 0.5;
                        self.smokes.push(smoke::Smoke::new(smoke_x, smoke_y, 6.0));
                    }
                }
            }
        }

        for proj in &self.projectiles {
            if proj.active && matches!(proj.weapon_type, weapon::Weapon::RocketLauncher) {
                self.lights.push(light::LightPulse::new(
                    proj.x,
                    proj.y,
                    120.0,
                    Color::from_rgba(255, 180, 80, 180),
                    80,
                ));
                self.lights.push(light::LightPulse::new(
                    proj.x - proj.vel_x * 2.0,
                    proj.y - proj.vel_y * 2.0,
                    30.0,
                    Color::from_rgba(255, 120, 40, 255),
                    60,
                ));
            } else if proj.active && matches!(proj.weapon_type, weapon::Weapon::GrenadeLauncher) {
                self.lights.push(light::LightPulse::new(
                    proj.x,
                    proj.y,
                    100.0,
                    Color::from_rgba(150, 255, 120, 170),
                    80,
                ));
                self.lights.push(light::LightPulse::new(
                    proj.x - proj.vel_x * 2.0,
                    proj.y - proj.vel_y * 2.0,
                    25.0,
                    Color::from_rgba(100, 220, 90, 220),
                    60,
                ));
            }
        }

        let mut exploded_projectiles: Vec<(f32, f32, weapon::Weapon, u16, i32, f32, Option<u16>)> = Vec::new();
        let mut kills = Vec::new();
        let mut pending_damage_numbers: Vec<(u32, u16, f32, f32, i32, i32, i32)> = Vec::new();
        
        let local_player_id = if self.is_multiplayer {
            self.network_client.as_ref().and_then(|c| c.player_id()).unwrap_or(0)
        } else {
            0
        };

        self.projectiles.retain_mut(|proj| {
            let should_update_locally = !self.is_multiplayer || proj.owner_id == local_player_id;
            
            let alive = if should_update_locally {
                proj.update(dt, &self.map)
            } else {
                true
            };

            if proj.just_bounced && matches!(proj.weapon_type, weapon::Weapon::GrenadeLauncher) {
                self.audio_events
                    .push(crate::audio::events::AudioEvent::GrenadeBounce { x: proj.x });
            }

            let trail_particles = proj.create_trail_particles();
            for particle in trail_particles {
                self.particles.push(particle);
            }

            if !alive && proj.active == false && should_update_locally {
                let has_explosion = matches!(
                    proj.weapon_type,
                    weapon::Weapon::RocketLauncher
                        | weapon::Weapon::GrenadeLauncher
                        | weapon::Weapon::Plasmagun
                        | weapon::Weapon::BFG
                );
                
                if has_explosion {
                    exploded_projectiles.push((
                        proj.x,
                        proj.y,
                        proj.weapon_type,
                        proj.owner_id,
                        proj.damage,
                        proj.explosion_radius(),
                        None,
                    ));
                }
            }
            alive
        });

        if !self.is_multiplayer {
            let projectiles_to_check = self.projectiles.clone();
            let mut projectiles_to_remove = Vec::new();
            
            for proj in &projectiles_to_check {
                if proj.active {
                    let mut corpse_to_create = None;
                    let mut hit_player_id = None;
                    let mut weapon_to_drop = None;

                    for player in &mut self.players {
                        if player.id != proj.owner_id && !player.gibbed {
                            let hitbox_height = if player.dead {
                                constants::PLAYER_HITBOX_HEIGHT_CROUCH
                            } else if player.crouch {
                                constants::PLAYER_HITBOX_HEIGHT_CROUCH
                            } else {
                                constants::PLAYER_HITBOX_HEIGHT
                            };
                            let hitbox_width = constants::PLAYER_HITBOX_WIDTH;
                            let hitbox_center_y = player.y - hitbox_height / 2.0 + 16.0;
                            if proj.check_collision(
                                player.x,
                                hitbox_center_y,
                                hitbox_width,
                                hitbox_height,
                            ) {
                                let was_alive = !player.dead;
                                
                                let weapon_drop_pos = if was_alive {
                                    if let Some(model) = self.model_cache.get(&player.model) {
                                        let flip = player.angle.abs() > std::f32::consts::PI / 2.0;
                                        let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                                        let mut rel_angle = player.angle - base_dir;
                                        while rel_angle > std::f32::consts::PI {
                                            rel_angle -= 2.0 * std::f32::consts::PI;
                                        }
                                        while rel_angle < -std::f32::consts::PI {
                                            rel_angle += 2.0 * std::f32::consts::PI;
                                        }
                                        let pitch = rel_angle;
                                        let weapon_model = self.weapon_model_cache.get(player.weapon);
                                        model.get_barrel_position(
                                            player.x,
                                            player.y,
                                            flip,
                                            pitch,
                                            player.angle,
                                            player.lower_frame,
                                            player.upper_frame,
                                            weapon_model,
                                        )
                                    } else {
                                        (player.x, player.y)
                                    }
                                } else {
                                    (player.x, player.y)
                                };
                                
                                let (died, gibbed) = player.take_damage(proj.damage);
                                
                                self.weapon_hit_effects.push(
                                    weapon_hit_effect::WeaponHitEffect::new_blood(proj.x, proj.y),
                                );
                                
                                if was_alive {
                                    for _ in 0..3 {
                                        let gib_type = match crate::compat_rand::gen_range_usize(0, 3) {
                                            0 => gib::GibType::Intestine,
                                            1 => gib::GibType::Brain,
                                            _ => gib::GibType::Fist,
                                        };
                                        self.gibs.push(gib::Gib::new(
                                            player.x,
                                            player.y,
                                            crate::compat_rand::gen_range_f32(-3.0, 3.0),
                                            crate::compat_rand::gen_range_f32(-5.0, -2.0),
                                            gib_type,
                                        ));
                                    }
                                }
                                
                                if proj.owner_id == 1 && was_alive {
                                    self.audio_events.push(
                                        crate::audio::events::AudioEvent::PlayerHit {
                                            damage: proj.damage,
                                        },
                                    );
                                    pending_damage_numbers.push((
                                        proj.owner_id as u32,
                                        player.id,
                                        player.x,
                                        player.y,
                                        proj.damage,
                                        player.health,
                                        player.armor,
                                    ));
                                }
                                
                                if died && was_alive {
                                    weapon_to_drop = Some((player.weapon, weapon_drop_pos.0, weapon_drop_pos.1));
                                    if !gibbed {
                                        corpse_to_create = Some(player.clone());
                                    } else {
                                        self.gibs.extend(gib::spawn_gibs(player.x, player.y));
                                        self.audio_events.push(
                                            crate::audio::events::AudioEvent::PlayerGib { x: player.x },
                                        );
                                    }
                                    kills.push((proj.owner_id, player.id, player.was_in_air, proj.weapon_type));
                                }
                                
                                hit_player_id = Some(player.id);
                                projectiles_to_remove.push(proj.id);
                                break;
                            }
                        }
                    }
                    
                    if let Some(corpse_player) = corpse_to_create {
                        self.corpses.push(Corpse {
                            player: corpse_player,
                            lifetime: 10.0,
                        });
                    }
                    
                    if let Some((weapon, x, y)) = weapon_to_drop {
                        if let Some(item_type) = weapon.to_item_type() {
                            let player = self.players.iter().find(|p| (p.x - x).abs() < 1.0 && (p.y - y).abs() < 1.0);
                            let (vel_x, vel_y) = if let Some(p) = player {
                                let base_x = (crate::compat_rand::gen_f32() - 0.5) * 6.0;
                                let base_y = -8.0 + (crate::compat_rand::gen_f32() - 0.5) * 4.0;
                                (p.vel_x.clamp(-10.0, 10.0) * 0.3 + base_x, p.vel_y.clamp(-10.0, 10.0) * 0.2 + base_y)
                            } else {
                                ((crate::compat_rand::gen_f32() - 0.5) * 6.0, -8.0 + (crate::compat_rand::gen_f32() - 0.5) * 4.0)
                            };
                            let pitch = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                            let yaw = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                            let roll = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                            let spin_pitch = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                            let spin_yaw = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                            let spin_roll = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                            self.map.items.push(map::Item {
                                x,
                                y: y - 10.0,
                                item_type,
                                respawn_time: 0,
                                active: true,
                                vel_x,
                                vel_y,
                                dropped: true,
                                yaw,
                                spin_yaw,
                                pitch,
                                roll,
                                spin_pitch,
                                spin_roll,
                            });
                        }
                    }
                    
                    if let Some(player_id) = hit_player_id {
                        let has_explosion = matches!(
                            proj.weapon_type,
                            weapon::Weapon::RocketLauncher
                                | weapon::Weapon::GrenadeLauncher
                                | weapon::Weapon::Plasmagun
                                | weapon::Weapon::BFG
                        );
                        
                        if has_explosion {
                            exploded_projectiles.push((
                                proj.x,
                                proj.y,
                                proj.weapon_type,
                                proj.owner_id,
                                proj.damage,
                                proj.explosion_radius(),
                                Some(player_id),
                            ));
                        }
                    }
                }
            }
            
            self.projectiles.retain(|p| !projectiles_to_remove.contains(&p.id));
        }

        if !self.is_multiplayer {
            let projectiles_to_check = self.projectiles.clone();
            for proj in &projectiles_to_check {
                if proj.active
                    && !matches!(
                        proj.weapon_type,
                        weapon::Weapon::RocketLauncher
                            | weapon::Weapon::GrenadeLauncher
                            | weapon::Weapon::Plasmagun
                            | weapon::Weapon::BFG
                    )
                {
                    let mut corpse_to_create = None;
                    let mut weapon_to_drop = None;
                    
                    for player in &mut self.players {
                        if player.id != proj.owner_id && !player.gibbed {
                            let hitbox_height = if player.dead {
                                constants::PLAYER_HITBOX_HEIGHT_CROUCH
                            } else if player.crouch {
                                constants::PLAYER_HITBOX_HEIGHT_CROUCH
                            } else {
                                constants::PLAYER_HITBOX_HEIGHT
                            };
                            let hitbox_width = constants::PLAYER_HITBOX_WIDTH;
                            let hitbox_center_y = player.y - hitbox_height / 2.0 + 16.0;
                            if proj.check_collision(
                                player.x,
                                hitbox_center_y,
                                hitbox_width,
                                hitbox_height,
                            ) {
                                if let Some(p) = self.projectiles.iter_mut().find(|p| {
                                    p.x == proj.x && p.y == proj.y && p.owner_id == proj.owner_id
                                }) {
                                    p.active = false;

                                    println!("[{:.3}] [COLLISION] Non-explosive projectile [{}] {:?} direct hit player {} for {} damage", 
                                        macroquad::prelude::get_time(), p.id, proj.weapon_type, player.id, proj.damage);
                                    
                                    let was_alive = !player.dead;
                                    
                                    let weapon_drop_pos = if was_alive {
                                        if let Some(model) = self.model_cache.get(&player.model) {
                                            let flip = player.angle.abs() > std::f32::consts::PI / 2.0;
                                            let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                                            let mut rel_angle = player.angle - base_dir;
                                            while rel_angle > std::f32::consts::PI {
                                                rel_angle -= 2.0 * std::f32::consts::PI;
                                            }
                                            while rel_angle < -std::f32::consts::PI {
                                                rel_angle += 2.0 * std::f32::consts::PI;
                                            }
                                            let pitch = rel_angle;
                                            let weapon_model = self.weapon_model_cache.get(player.weapon);
                                            model.get_barrel_position(
                                                player.x,
                                                player.y,
                                                flip,
                                                pitch,
                                                player.angle,
                                                player.lower_frame,
                                                player.upper_frame,
                                                weapon_model,
                                            )
                                        } else {
                                            (player.x, player.y)
                                        }
                                    } else {
                                        (player.x, player.y)
                                    };
                                    
                                    let (died, gibbed) = player.take_damage(proj.damage);

                                    self.weapon_hit_effects.push(
                                        weapon_hit_effect::WeaponHitEffect::new_blood(proj.x, proj.y),
                                    );
                                    
                                    if was_alive {
                                        for _ in 0..3 {
                                            let gib_type = match crate::compat_rand::gen_range_usize(0, 3) {
                                                0 => gib::GibType::Intestine,
                                                1 => gib::GibType::Brain,
                                                _ => gib::GibType::Fist,
                                            };
                                            self.gibs.push(gib::Gib::new(
                                                player.x,
                                                player.y,
                                                crate::compat_rand::gen_range_f32(-3.0, 3.0),
                                                crate::compat_rand::gen_range_f32(-5.0, -2.0),
                                                gib_type,
                                            ));
                                        }
                                    }

                                    if proj.owner_id == 1 && was_alive {
                                        self.audio_events.push(
                                            crate::audio::events::AudioEvent::PlayerHit {
                                                damage: proj.damage,
                                            },
                                        );
                                        pending_damage_numbers.push((
                                            proj.owner_id as u32,
                                            player.id,
                                            player.x,
                                            player.y,
                                            proj.damage,
                                            player.health,
                                            player.armor,
                                        ));
                                    }
                                    if died && was_alive {
                                        weapon_to_drop = Some((player.weapon, weapon_drop_pos.0, weapon_drop_pos.1));
                                        if !gibbed {
                                            corpse_to_create = Some(player.clone());
                                        } else {
                                            self.gibs.extend(gib::spawn_gibs(player.x, player.y));
                                            self.audio_events.push(
                                                crate::audio::events::AudioEvent::PlayerGib { x: player.x },
                                            );
                                        }
                                        println!("[PROJECTILE KILL] owner={} victim={} weapon={:?}", proj.owner_id, player.id, proj.weapon_type);
                                        kills.push((proj.owner_id, player.id, false, proj.weapon_type));
                                    }
                                }
                                if !matches!(proj.weapon_type, weapon::Weapon::Railgun) {
                                    break;
                                }
                            }
                        }
                    }
                    
                    if let Some(corpse_player) = corpse_to_create {
                        self.corpses.push(Corpse {
                            player: corpse_player,
                            lifetime: 10.0,
                        });
                    }
                    
                    if let Some((weapon, x, y)) = weapon_to_drop {
                        if let Some(item_type) = weapon.to_item_type() {
                            let player = self.players.iter().find(|p| p.x == x && p.y == y);
                            let (vel_x, vel_y) = if let Some(p) = player {
                                (p.vel_x * 0.5, p.vel_y * 0.5)
                            } else {
                                (0.0, 0.0)
                            };
                            let pitch = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                            let yaw = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                            let roll = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                            let spin_pitch = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                            let spin_yaw = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                            let spin_roll = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                            self.map.items.push(map::Item {
                                x,
                                y: y - 10.0,
                                item_type,
                                respawn_time: 0,
                                active: true,
                                vel_x,
                                vel_y,
                                dropped: true,
                                yaw,
                                spin_yaw,
                                pitch,
                                roll,
                                spin_pitch,
                                spin_roll,
                            });
                        }
                    }
                }
            }
            
            for proj in &projectiles_to_check {
                if proj.active
                    && !matches!(
                        proj.weapon_type,
                        weapon::Weapon::RocketLauncher
                            | weapon::Weapon::GrenadeLauncher
                            | weapon::Weapon::Plasmagun
                            | weapon::Weapon::BFG
                    )
                {
                    let mut corpse_hit_idx = None;
                    
                    for (idx, corpse) in self.corpses.iter().enumerate() {
                        let hitbox_height = constants::PLAYER_HITBOX_HEIGHT_CROUCH;
                        let hitbox_width = constants::PLAYER_HITBOX_WIDTH;
                        let hitbox_center_y = corpse.player.y - hitbox_height / 2.0 + 16.0;
                        
                        if proj.check_collision(
                            corpse.player.x,
                            hitbox_center_y,
                            hitbox_width,
                            hitbox_height,
                        ) {
                            corpse_hit_idx = Some(idx);
                            
                            if let Some(p) = self.projectiles.iter_mut().find(|p| {
                                p.x == proj.x && p.y == proj.y && p.owner_id == proj.owner_id
                            }) {
                                p.active = false;
                            }
                            
                            break;
                        }
                    }
                    
                    if let Some(idx) = corpse_hit_idx {
                        let corpse = &self.corpses[idx];
                        let gib_x = corpse.player.x;
                        let gib_y = corpse.player.y;
                        
                        self.weapon_hit_effects.push(
                            weapon_hit_effect::WeaponHitEffect::new_blood(proj.x, proj.y),
                        );
                        
                        self.audio_events.push(crate::audio::events::AudioEvent::PlayerGib { x: gib_x });
                        
                        for _ in 0..15 {
                            self.particles.push(particle::Particle::new(
                                gib_x,
                                gib_y,
                                crate::compat_rand::gen_range_f32(-6.0, 6.0),
                                crate::compat_rand::gen_range_f32(-9.0, -3.0),
                                true,
                            ));
                        }
                        
                        self.gibs.extend(gib::spawn_gibs(gib_x, gib_y));
                        
                        self.corpses.remove(idx);
                    }
                }
            }
            
            for proj in &projectiles_to_check {
                if proj.active
                    && matches!(
                        proj.weapon_type,
                        weapon::Weapon::RocketLauncher
                            | weapon::Weapon::GrenadeLauncher
                            | weapon::Weapon::Plasmagun
                            | weapon::Weapon::BFG
                    )
                {
                    let mut corpse_hit_idx = None;
                    
                    for (idx, corpse) in self.corpses.iter().enumerate() {
                        let hitbox_height = constants::PLAYER_HITBOX_HEIGHT_CROUCH;
                        let hitbox_width = constants::PLAYER_HITBOX_WIDTH;
                        let hitbox_center_y = corpse.player.y - hitbox_height / 2.0 + 16.0;
                        
                        if proj.check_collision(
                            corpse.player.x,
                            hitbox_center_y,
                            hitbox_width,
                            hitbox_height,
                        ) {
                            corpse_hit_idx = Some(idx);
                            
                            if let Some(p) = self.projectiles.iter_mut().find(|p| {
                                p.x == proj.x && p.y == proj.y && p.owner_id == proj.owner_id
                            }) {
                                p.active = false;
                                
                                let explosion_radius = p.explosion_radius();
                                let damage = p.damage;
                                let weapon_type = p.weapon_type;
                                let owner_id = p.owner_id;
                                let x = p.x;
                                let y = p.y;
                                
                                println!("[{:.3}] [CORPSE HIT] Explosive projectile hit corpse, adding explosion", 
                                    macroquad::prelude::get_time());
                                exploded_projectiles.push((
                                    x,
                                    y,
                                    weapon_type,
                                    owner_id,
                                    damage,
                                    explosion_radius,
                                    None,
                                ));
                            }
                            
                            break;
                        }
                    }
                }
            }
        }

        let hits = self.pending_hits.drain(..).collect::<Vec<_>>();
        for (idx, damage, hit_x, hit_y, owner_id) in hits {
            let player_x = self.players[idx].x;
            let player_y = self.players[idx].y;
            let player_weapon = self.players[idx].weapon;
            let was_alive = !self.players[idx].dead;
            let (died, gibbed) = self.players[idx].take_damage(damage);
            
            if was_alive {
                for _ in 0..3 {
                    let gib_type = match crate::compat_rand::gen_range_usize(0, 3) {
                        0 => gib::GibType::Intestine,
                        1 => gib::GibType::Brain,
                        _ => gib::GibType::Fist,
                    };
                    self.gibs.push(gib::Gib::new(
                        player_x,
                        player_y,
                        crate::compat_rand::gen_range_f32(-3.0, 3.0),
                        crate::compat_rand::gen_range_f32(-5.0, -2.0),
                        gib_type,
                    ));
                }
            }

            if was_alive && owner_id == 1 {
                self.audio_events
                    .push(crate::audio::events::AudioEvent::PlayerHit { damage });
                let target_id = self.players[idx].id;
                let target_health = self.players[idx].health;
                let target_armor = self.players[idx].armor;
                pending_damage_numbers.push((owner_id as u32, target_id, player_x, player_y, damage, target_health, target_armor));
            }

            if died && was_alive {
                if let Some(item_type) = player_weapon.to_item_type() {
                    let base_x = (crate::compat_rand::gen_f32() - 0.5) * 6.0;
                    let base_y = -8.0 + (crate::compat_rand::gen_f32() - 0.5) * 4.0;
                    let vel_x = self.players[idx].vel_x.clamp(-10.0, 10.0) * 0.3 + base_x;
                    let vel_y = self.players[idx].vel_y.clamp(-10.0, 10.0) * 0.2 + base_y;
                    let pitch = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let yaw = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let roll = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let spin_pitch = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    let spin_yaw = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    let spin_roll = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    self.map.items.push(map::Item {
                        x: player_x,
                        y: player_y - 10.0,
                        item_type,
                        respawn_time: 0,
                        active: true,
                        vel_x,
                        vel_y,
                        dropped: true,
                        yaw,
                        spin_yaw,
                        pitch,
                        roll,
                        spin_pitch,
                        spin_roll,
                    });
                }
            }

            if died && was_alive && !gibbed {
                let corpse_player = self.players[idx].clone();
                self.corpses.push(Corpse {
                    player: corpse_player,
                    lifetime: 10.0,
                });
            }

            if gibbed && died && was_alive {
                self.audio_events
                    .push(crate::audio::events::AudioEvent::PlayerGib { x: player_x });
                for _ in 0..10 {
                    self.particles.push(particle::Particle::new(
                        player_x,
                        self.players[idx].y,
                        rand::gen_range(-5.0, 5.0),
                        rand::gen_range(-8.0, -2.0),
                        true,
                    ));
                }
                self.gibs
                    .extend(gib::spawn_gibs(player_x, self.players[idx].y));
                if was_alive {
                    if owner_id == self.players[idx].id {
                        if let Some(player) = self.players.iter_mut().find(|p| p.id == owner_id) {
                            player.frags -= 1;
                        }
                    } else {
                        if let Some(killer) = self.players.iter_mut().find(|p| p.id == owner_id) {
                            killer.frags += 1;
                        }
                    }

                    let victim_was_airborne = self.players[idx].was_in_air;
                    self.check_and_award(owner_id, self.players[idx].id, victim_was_airborne, weapon::Weapon::MachineGun);

                    let killer_name = self
                        .players
                        .iter()
                        .find(|p| p.id == owner_id)
                        .map(|p| p.name.clone())
                        .unwrap_or("Unknown".to_string());
                    let victim_name = self.players[idx].name.clone();
                    self.messages.push(message::GameMessage::kill_message(
                        &killer_name,
                        &victim_name,
                        owner_id as u8,
                    ));
                }
            } else if died && was_alive {
                if let Some(item_type) = player_weapon.to_item_type() {
                    let base_x = (crate::compat_rand::gen_f32() - 0.5) * 6.0;
                    let base_y = -8.0 + (crate::compat_rand::gen_f32() - 0.5) * 4.0;
                    let vel_x = self.players[idx].vel_x.clamp(-10.0, 10.0) * 0.3 + base_x;
                    let vel_y = self.players[idx].vel_y.clamp(-10.0, 10.0) * 0.2 + base_y;
                    let pitch = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let yaw = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let roll = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let spin_pitch = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    let spin_yaw = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    let spin_roll = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    self.map.items.push(map::Item {
                        x: player_x,
                        y: player_y - 10.0,
                        item_type,
                        respawn_time: 0,
                        active: true,
                        vel_x,
                        vel_y,
                        dropped: true,
                        yaw,
                        spin_yaw,
                        pitch,
                        roll,
                        spin_pitch,
                        spin_roll,
                    });
                }
                if owner_id == self.players[idx].id {
                    if let Some(player) = self.players.iter_mut().find(|p| p.id == owner_id) {
                        player.frags -= 1;
                    }
                } else {
                    if let Some(killer) = self.players.iter_mut().find(|p| p.id == owner_id) {
                        killer.frags += 1;
                    }
                }

                let victim_was_airborne = self.players[idx].was_in_air;
                self.check_and_award(owner_id, self.players[idx].id, victim_was_airborne, weapon::Weapon::MachineGun);

                let killer_name = self
                    .players
                    .iter()
                    .find(|p| p.id == owner_id)
                    .map(|p| p.name.clone())
                    .unwrap_or("Unknown".to_string());
                let victim_name = self.players[idx].name.clone();
                let victim_model = self.players[idx].model.clone();
                self.messages.push(message::GameMessage::kill_message(
                    &killer_name,
                    &victim_name,
                    owner_id as u8,
                ));

                self.audio_events
                    .push(crate::audio::events::AudioEvent::PlayerDeath {
                        x: player_x,
                        model: victim_model.clone(),
                    });
            } else if was_alive {
                self.audio_events
                    .push(crate::audio::events::AudioEvent::PlayerPain {
                        health: self.players[idx].health,
                        x: player_x,
                        model: self.players[idx].model.clone(),
                    });
            }

            for _ in 0..2 {
                self.particles.push(particle::Particle::new(
                    hit_x,
                    hit_y,
                    rand::gen_range(-2.0, 2.0),
                    rand::gen_range(-2.0, 2.0),
                    false,
                ));
            }
            self.weapon_hit_effects
                .push(weapon_hit_effect::WeaponHitEffect::new_blood(hit_x, hit_y));

            self.lights.push(light::LightPulse::new(
                hit_x,
                hit_y,
                80.0,
                Color::from_rgba(255, 220, 140, 140),
                70,
            ));
        }

        for (x, y, weapon, owner_id, damage, radius, direct_hit_player_id) in exploded_projectiles {
            self.audio_events
                .push(crate::audio::events::AudioEvent::Explosion { x });

            let explosion_particles =
                projectile::Projectile::new(x, y, 0.0, owner_id, weapon, 0.0, 0.0)
                    .create_explosion_particles();
            self.particles.extend(explosion_particles);

            self.weapon_hit_effects
                .push(weapon_hit_effect::WeaponHitEffect::new(x, y, weapon));

            self.lights
                .push(light::LightPulse::new_explosion_flash(x, y, radius * 3.0));

            for i in 0..5 {
                let offset_x = (i as f32 - 2.5) * 4.0;
                let offset_y = (i as f32 - 2.5) * 3.0;
                self.smokes.push(smoke::Smoke::new(
                    x + offset_x,
                    y + offset_y,
                    12.0 + i as f32 * 2.0,
                ));
            }

            let mut new_corpses = Vec::new();
            let mut weapons_to_drop = Vec::new();
            
            for player in &mut self.players {
                if Some(player.id) == direct_hit_player_id {
                    continue;
                }
                
                let dx = player.x - x;
                let dy = player.y - y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < radius {
                    let damage_points = (damage as f32 - 0.5 * dist).max(0.0);
                    
                    let mass = 200.0;
                    let g_knockback = 1200.0;
                    let knockback = damage_points.min(200.0);
                    let knockback_scale = g_knockback * knockback / mass;
                    
                    if dist > 0.1 {
                        let dir_x = dx / dist;
                        let dir_y = dy / dist;
                        
                        player.vel_x += dir_x * knockback_scale;
                        player.vel_y += dir_y * knockback_scale;

                        // 20% chance to flip if knocked up significantly
                        if player.vel_y < -4.0 && crate::compat_rand::gen_range_f32(0.0, 1.0) < 0.2 {
                            player.somersault_time = 1.0;
                        }
                    }

                    if !self.is_multiplayer {
                        if player.id != owner_id {
                            let actual_damage = damage_points as i32;
                            let was_alive = !player.dead;
                            
                            let weapon_drop_pos = if was_alive {
                                if let Some(model) = self.model_cache.get(&player.model) {
                                    let flip = player.angle.abs() > std::f32::consts::PI / 2.0;
                                    let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                                    let mut rel_angle = player.angle - base_dir;
                                    while rel_angle > std::f32::consts::PI {
                                        rel_angle -= 2.0 * std::f32::consts::PI;
                                    }
                                    while rel_angle < -std::f32::consts::PI {
                                        rel_angle += 2.0 * std::f32::consts::PI;
                                    }
                                    let pitch = rel_angle;
                                    let weapon_model = self.weapon_model_cache.get(player.weapon);
                                    model.get_barrel_position(
                                        player.x,
                                        player.y,
                                        flip,
                                        pitch,
                                        player.angle,
                                        player.lower_frame,
                                        player.upper_frame,
                                        weapon_model,
                                    )
                                } else {
                                    (player.x, player.y)
                                }
                            } else {
                                (player.x, player.y)
                            };
                            
                            let (died, gibbed) = player.take_damage(actual_damage);
                            
                            if was_alive && actual_damage > 5 {
                                let gib_count = (actual_damage / 10).min(5);
                                for _ in 0..gib_count {
                                    let gib_type = match crate::compat_rand::gen_range_usize(0, 3) {
                                        0 => gib::GibType::Intestine,
                                        1 => gib::GibType::Brain,
                                        _ => gib::GibType::Fist,
                                    };
                                    self.gibs.push(gib::Gib::new(
                                        player.x,
                                        player.y,
                                        crate::compat_rand::gen_range_f32(-4.0, 4.0),
                                        crate::compat_rand::gen_range_f32(-6.0, -2.0),
                                        gib_type,
                                    ));
                                }
                            }

                            if owner_id == 1 && was_alive {
                                self.audio_events
                                    .push(crate::audio::events::AudioEvent::PlayerHit {
                                        damage: actual_damage,
                                    });
                                pending_damage_numbers.push((
                                    owner_id as u32,
                                    player.id,
                                    player.x,
                                    player.y,
                                    actual_damage,
                                    player.health,
                                    player.armor,
                                ));
                            }

                            if died && was_alive {
                                weapons_to_drop.push((player.weapon, weapon_drop_pos.0, weapon_drop_pos.1));
                                if !gibbed {
                                    new_corpses.push(player.clone());
                                } else {
                                    self.gibs.extend(gib::spawn_gibs(player.x, player.y));
                                    self.audio_events
                                        .push(crate::audio::events::AudioEvent::PlayerGib { x: player.x });
                                }
                                kills.push((owner_id, player.id, false, weapon));
                            }
                        } else if owner_id as u16 == player.id {
                            let self_damage = (damage_points * 0.5) as i32;
                            let was_alive = !player.dead;
                            
                            let weapon_drop_pos = if was_alive {
                                if let Some(model) = self.model_cache.get(&player.model) {
                                    let flip = player.angle.abs() > std::f32::consts::PI / 2.0;
                                    let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                                    let mut rel_angle = player.angle - base_dir;
                                    while rel_angle > std::f32::consts::PI {
                                        rel_angle -= 2.0 * std::f32::consts::PI;
                                    }
                                    while rel_angle < -std::f32::consts::PI {
                                        rel_angle += 2.0 * std::f32::consts::PI;
                                    }
                                    let pitch = rel_angle;
                                    let weapon_model = self.weapon_model_cache.get(player.weapon);
                                    model.get_barrel_position(
                                        player.x,
                                        player.y,
                                        flip,
                                        pitch,
                                        player.angle,
                                        player.lower_frame,
                                        player.upper_frame,
                                        weapon_model,
                                    )
                                } else {
                                    (player.x, player.y)
                                }
                            } else {
                                (player.x, player.y)
                            };
                            
                            let (died, gibbed) = player.take_damage(self_damage);
                            
                            if was_alive && self_damage > 5 {
                                let gib_count = (self_damage / 10).min(5);
                                for _ in 0..gib_count {
                                    let gib_type = match crate::compat_rand::gen_range_usize(0, 3) {
                                        0 => gib::GibType::Intestine,
                                        1 => gib::GibType::Brain,
                                        _ => gib::GibType::Fist,
                                    };
                                    self.gibs.push(gib::Gib::new(
                                        player.x,
                                        player.y,
                                        crate::compat_rand::gen_range_f32(-4.0, 4.0),
                                        crate::compat_rand::gen_range_f32(-6.0, -2.0),
                                        gib_type,
                                    ));
                                }
                            }
                            
                            if owner_id == 1 && was_alive {
                                pending_damage_numbers.push((
                                    owner_id as u32,
                                    player.id,
                                    player.x,
                                    player.y,
                                    self_damage,
                                    player.health,
                                    player.armor,
                                ));
                            }
                            
                            if died && was_alive {
                                weapons_to_drop.push((player.weapon, weapon_drop_pos.0, weapon_drop_pos.1));
                                if !gibbed {
                                    new_corpses.push(player.clone());
                                } else {
                                    self.gibs.extend(gib::spawn_gibs(player.x, player.y));
                                    self.audio_events
                                        .push(crate::audio::events::AudioEvent::PlayerGib { x: player.x });
                                }
                                kills.push((owner_id, player.id, false, weapon));
                            }
                        }
                    }
                }
            }
            
            for (weapon, px, py) in weapons_to_drop {
                if let Some(item_type) = weapon.to_item_type() {
                    let player = self.players.iter().find(|p| (p.x - px).abs() < 1.0 && (p.y - py).abs() < 1.0);
                    let (vel_x, vel_y) = if let Some(p) = player {
                        let base_x = (crate::compat_rand::gen_f32() - 0.5) * 6.0;
                        let base_y = -8.0 + (crate::compat_rand::gen_f32() - 0.5) * 4.0;
                        (p.vel_x.clamp(-10.0, 10.0) * 0.3 + base_x, p.vel_y.clamp(-10.0, 10.0) * 0.2 + base_y)
                    } else {
                        ((crate::compat_rand::gen_f32() - 0.5) * 6.0, -8.0 + (crate::compat_rand::gen_f32() - 0.5) * 4.0)
                    };
                    let pitch = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let yaw = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let roll = crate::compat_rand::gen_f32() * std::f32::consts::PI * 2.0;
                    let spin_pitch = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    let spin_yaw = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    let spin_roll = (crate::compat_rand::gen_f32() - 0.5) * 0.3;
                    self.map.items.push(map::Item {
                        x: px,
                        y: py - 10.0,
                        item_type,
                        respawn_time: 0,
                        active: true,
                        vel_x,
                        vel_y,
                        dropped: true,
                        yaw,
                        spin_yaw,
                        pitch,
                        roll,
                        spin_pitch,
                        spin_roll,
                    });
                }
            }
            
            let mut corpses_to_gib = Vec::new();
            for (idx, corpse) in self.corpses.iter_mut().enumerate() {
                let dx = corpse.player.x - x;
                let dy = corpse.player.y - y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < radius {
                    let damage_points = (damage as f32 - 0.5 * dist).max(0.0);
                    
                    if damage_points > 20.0 {
                        corpses_to_gib.push((idx, corpse.player.x, corpse.player.y));
                    }
                    
                    let mass = 200.0;
                    let g_knockback = 1200.0;
                    let knockback = damage_points.min(200.0);
                    let knockback_scale = g_knockback * knockback / mass;
                    
                    if dist > 0.1 {
                        let dir_x = dx / dist;
                        let dir_y = dy / dist;
                        
                        corpse.player.vel_x += dir_x * knockback_scale;
                        corpse.player.vel_y += dir_y * knockback_scale;
                    }
                }
            }
            
            for (idx, gib_x, gib_y) in corpses_to_gib.iter().rev() {
                self.audio_events.push(crate::audio::events::AudioEvent::PlayerGib { x: *gib_x });
                
                for _ in 0..15 {
                    self.particles.push(particle::Particle::new(
                        *gib_x,
                        *gib_y,
                        crate::compat_rand::gen_range_f32(-6.0, 6.0),
                        crate::compat_rand::gen_range_f32(-9.0, -3.0),
                        true,
                    ));
                }
                
                self.gibs.extend(gib::spawn_gibs(*gib_x, *gib_y));
                self.corpses.remove(*idx);
            }
            
            for corpse_player in new_corpses {
                self.corpses.push(Corpse {
                    player: corpse_player,
                    lifetime: 10.0,
                });
            }
        }

        for (pid, target_id, x, y, dmg, target_health, target_armor) in pending_damage_numbers.drain(..) {
            self.add_damage_number(pid, target_id, x, y, dmg, target_health, target_armor);
        }

        let kill_count = kills.len();

        let local_player_id = if self.is_multiplayer {
            self.network_client.as_ref().and_then(|c| c.player_id())
        } else {
            self.players.get(0).map(|p| p.id)
        };

        for &(killer_id, victim_id, was_airborne, weapon) in &kills {
            let old_leader = self.lead_announcements.current_leader;
            
            let scores_before: Vec<_> = self.players.iter().map(|p| (p.id, p.frags)).collect();
            let top_score_before = scores_before.iter().map(|(_, f)| *f).max().unwrap_or(0);
            let local_score_before = scores_before.iter().find(|(id, _)| Some(*id) == local_player_id).map(|(_, f)| *f).unwrap_or(0);
            let was_local_in_lead_group = old_leader == local_player_id || (old_leader.is_none() && local_score_before == top_score_before);
            
            if killer_id == victim_id {
                if let Some(player) = self.players.iter_mut().find(|p| p.id == killer_id) {
                    player.frags -= 1;
                }
            } else {
                if let Some(killer) = self.players.iter_mut().find(|p| p.id == killer_id) {
                    killer.frags += 1;
                }
            }
            
            let scores_after: Vec<_> = self.players.iter().map(|p| (p.id, p.frags)).collect();
            
            println!("Kill: killer={}, victim={}, old_leader={:?}", killer_id, victim_id, old_leader);
            println!("  Scores before: {:?}", scores_before);
            println!("  Scores after: {:?}", scores_after);
            
            self.check_and_award(killer_id, victim_id, was_airborne, weapon);

            if let Some(local_id) = local_player_id {
                if killer_id == local_id || was_local_in_lead_group {
                    if let Some(announcement) = self.lead_announcements.update(&self.players, local_player_id, Some(killer_id), old_leader, was_local_in_lead_group) {
                        println!("  -> Announcement: {}", announcement);
                        self.audio_events.push(crate::audio::events::AudioEvent::LeadChange {
                            announcement: announcement.to_string(),
                        });
                    }
                }
            }
        }

        if let Some(ref mut story) = self.story_mode {
            for _ in 0..kill_count {
                story.on_enemy_killed();
            }
        }

        let spawn_points = self.map.spawn_points.clone();
        for player in &mut self.players {
            if player.dead {
                player.respawn_timer -= dt;
                if player.respawn_timer <= 0.0 {
                    let spawn_idx = if spawn_points.len() > 1 {
                        rand::gen_range(0, spawn_points.len())
                    } else {
                        0
                    };
                    let spawn = &spawn_points[spawn_idx];
                    player.spawn(spawn.x, spawn.y, &self.map);
                    self.audio_events
                        .push(crate::audio::events::AudioEvent::ItemPickup { x: spawn.x });
                }
            }
        }

        if let Some(defrag) = &mut self.defrag_mode {
            if let Some(player) = self.players.first() {
                let events = defrag.update(dt, player.x, player.y);
                for event in events {
                    match event {
                        defrag::DefragEvent::RunStarted => {
                            println!("[Defrag] Run started!");
                        }
                        defrag::DefragEvent::CheckpointReached(idx) => {
                            println!("[Defrag] Checkpoint {} reached!", idx + 1);
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::ItemPickup { x: player.x });
                        }
                        defrag::DefragEvent::RunFinished(time) => {
                            println!("[Defrag] Run finished in {:.3}s", time);
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::TeleportIn { x: player.x });
                        }
                        defrag::DefragEvent::NewRecord(time) => {
                            println!("[Defrag] NEW RECORD: {:.3}s!", time);
                            self.audio_events
                                .push(crate::audio::events::AudioEvent::TeleportIn { x: player.x });
                        }
                    }
                }
            }
        }

        self.awards.retain_mut(|award| {
            award.update(dt);
            !award.is_expired()
        });

        if let Some(announcement) = self.time_announcements.update(self.match_time, self.time_limit) {
            self.audio_events.push(crate::audio::events::AudioEvent::TimeAnnouncement {
                announcement: announcement.to_string(),
            });
        }


        if self.match_time >= self.time_limit && !self.game_results.show {
            self.game_results.trigger(&self.players, self.match_time);
        }
    }

    pub fn render(&mut self, camera_x: f32, camera_y: f32, zoom: f32) {
        let _t_total_start = get_time();
        let _render_total = crate::profiler::scope("render_total");

        {
            let _scope = crate::profiler::scope("render_setup");
            if self.deferred_renderer.is_none() {
                self.deferred_renderer = Some(deferred_renderer::DeferredRenderer::new(&self.map));
            }

            md3_render::clear_light_cache();
        }

        let _t_before_begin = get_time();
        let renderer = self.deferred_renderer.as_mut().unwrap();

        if !self.disable_deferred {
            renderer.begin_scene_with_scale(self.render_scale, zoom);
        }

        #[cfg(target_arch = "wasm32")]
        {
            renderer.apply_lighting(
                &self.map,
                &self.map.lights,
                &[],
                &self.linear_lights,
                camera_x,
                camera_y,
                zoom,
                self.ambient_light,
                self.disable_shadows,
                self.disable_dynamic_lights,
                self.cartoon_shader,
            );
        }

        {
            let _scope = crate::profiler::scope("render_background");
            self.map.render_background(camera_x, camera_y);
        }

        {
            let _scope = crate::profiler::scope("render_shadows");
            let screen_w = screen_width();
            let screen_h = screen_height();
            let shadow_margin = 100.0;

            // Initialize or resize shadow target
            let target_w = screen_w as u32;
            let target_h = screen_h as u32;
            
            let needs_resize = if let Some(target) = &self.shadow_target {
                target.texture.width() != screen_w || target.texture.height() != screen_h
            } else {
                true
            };

            if needs_resize {
                let target = render_target(target_w, target_h);
                target.texture.set_filter(FilterMode::Linear);
                self.shadow_target = Some(target);
            }

            if let Some(target) = &self.shadow_target {
                // Render shadows to target
                let mut camera = Camera2D {
                    render_target: Some(target.clone()),
                    zoom: vec2((2.0 * zoom) / screen_w, (2.0 * zoom) / screen_h),
                    target: vec2(screen_w / 2.0, screen_h / 2.0),
                    offset: vec2(0.0, 0.0),
                    ..Default::default()
                };
                
                // Flip Y for render target
                camera.zoom.y = -camera.zoom.y;
                
                set_camera(&camera);
                clear_background(Color::new(0.0, 0.0, 0.0, 0.0));

                for player in &mut self.players {
                    if player.gibbed { continue; }
                    if self.disable_shadows { continue; }

                    let screen_x = player.x - camera_x;
                    let screen_y = player.y - camera_y;

                    if screen_x < -shadow_margin || screen_x > screen_w + shadow_margin ||
                       screen_y < -shadow_margin || screen_y > screen_h + shadow_margin {
                        continue;
                    }

                    if let Some(model) = self.model_cache.get_or_load(&player.model) {
                        let flip = player.angle.abs() > std::f32::consts::PI / 2.0;
                        let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                        let mut rel_angle = player.angle - base_dir;
                        while rel_angle > std::f32::consts::PI { rel_angle -= 2.0 * std::f32::consts::PI; }
                        while rel_angle < -std::f32::consts::PI { rel_angle += 2.0 * std::f32::consts::PI; }
                        
                        let in_air = player.was_in_air;
                        let model_yaw_offset = if in_air {
                            (player.vel_x / self::constants::MAX_SPEED_AIR).clamp(-1.0, 1.0) * 0.8
                        } else {
                            0.0
                        };

                        let sway = if !in_air && player.vel_x.abs() > 0.5 {
                            (self.time as f32 * 12.0).sin() * 0.06
                        } else {
                            0.0
                        };

                        let mut pitch = rel_angle + sway;
                        if !in_air && player.vel_x.abs() <= 0.1 {
                            pitch += player.idle_yaw;
                        }

                        let aim_angle = player.angle;
                        let weapon_model = self.weapon_model_cache.get(player.weapon);
                        
                        let mut acc_x = 0.0;
                        let mut acc_y = 0.0;
                        let mut acc_w = 0.0;
                        let mut acc_r = 0.0;
                        
                        for lp in &self.lights {
                            let sx = lp.x - camera_x;
                            let sy = lp.y - camera_y;
                            let dx = screen_x - sx;
                            let dy = screen_y - sy;
                            let d = (dx * dx + dy * dy).sqrt();
                            let w = (1.0 - (d / (lp.radius + 1.0))).clamp(0.0, 1.0);
                            if w > 0.0 {
                                acc_x += sx * w;
                                acc_y += sy * w;
                                acc_r += lp.radius * w;
                                acc_w += w;
                            }
                        }

                        if acc_w > 0.0 {
                            let target_sx = acc_x / acc_w;
                            let target_sy = acc_y / acc_w;
                            let target_sr = acc_r / acc_w;
                            let lerp = 0.18;
                            
                            player.shadow_lx = player.shadow_lx + (target_sx - player.shadow_lx) * lerp;
                            player.shadow_ly = player.shadow_ly + (target_sy - player.shadow_ly) * lerp;
                            player.shadow_lr = player.shadow_lr + (target_sr - player.shadow_lr) * lerp;
                            
                            model.render_shadow_with_light(
                                screen_x,
                                screen_y,
                                player.shadow_lx,
                                player.shadow_ly,
                                player.shadow_lr.max(1.0),
                                1.5,
                                flip,
                                pitch,
                                aim_angle,
                                player.lower_frame,
                                player.upper_frame,
                                weapon_model,
                                model_yaw_offset,
                                BLACK,
                                if matches!(player.weapon, crate::game::weapon::Weapon::MachineGun) { player.barrel_spin_angle } else { 0.0 },
                            );
                        }
                    }
                }

                // Restore previous camera
                if !self.disable_deferred {
                    if let Some(target) = &renderer.scene_target {
                        let camera = Camera2D {
                            render_target: Some(target.clone()),
                            zoom: vec2((2.0 * zoom) / screen_w, (2.0 * zoom) / screen_h),
                            target: vec2(screen_w / 2.0, screen_h / 2.0),
                            offset: vec2(0.0, 0.0),
                            ..Default::default()
                        };
                        set_camera(&camera);
                    }
                } else {
                    set_default_camera();
                }
                
                // Draw the shadow target to screen with transparency
                // If deferred rendering is enabled, this draws to the scene_target
                draw_texture_ex(
                    &target.texture,
                    0.0,
                    0.0,
                    Color::from_rgba(0, 0, 0, 166), // Shadow opacity
                    DrawTextureParams {
                        dest_size: Some(vec2(screen_w, screen_h)),
                        flip_y: true, // Texture coordinates are flipped
                        ..Default::default()
                    },
                );
            }
        }

        {
            let _scope = crate::profiler::scope("render_tiles");
            self.map.render_tiles(camera_x, camera_y);
        }

        {
            let _scope = crate::profiler::scope("render_misc");
            for bullet in &self.bullet_holes {
                bullet.render(camera_x, camera_y);
            }

            for debug_ray in &self.debug_rays {
                debug_ray.render(camera_x, camera_y);
            }

            for teleport in &self.teleports {
                teleport.render(camera_x, camera_y);
            }

            for jumppad in &self.map.jumppads {
                jumppad.render(camera_x, camera_y);
            }
        }

        {
            let _scope = crate::profiler::scope("render_items");
            let screen_w = screen_width();
            let screen_h = screen_height();
            let item_margin = 100.0;

            for item in &self.map.items {
                if item.active {
                    let screen_x = item.x - camera_x;
                    let screen_y = item.y - camera_y;

                    if screen_x < -item_margin
                        || screen_x > screen_w + item_margin
                        || screen_y < -item_margin
                        || screen_y > screen_h + item_margin
                    {
                        continue;
                    }

                    if self.use_item_icons {
                        crate::render::draw_item_icon(
                            screen_x,
                            screen_y,
                            &item.item_type,
                            32.0,
                            WHITE,
                        );
                    } else {
                        if let Some(model_type) =
                            item_model::ItemModelType::from_item_type(item.item_type)
                        {
                            let item_color = model_type.item_color();
                            if let Some(model) = self.item_model_cache.get_mut(model_type) {
                                if model.prelit_color.is_none() {
                                    model.precompute_lighting(
                                        item.x,
                                        item.y,
                                        &self.map.lights,
                                        self.ambient_light,
                                    );
                                }
                                if item.dropped {
                                    model.render_with_full_rotation(screen_x, screen_y, 1.33, item_color, item.pitch, item.yaw, item.roll);
                                } else {
                                    model.render(screen_x, screen_y, 1.0, item_color);
                                }
                            } else {
                                draw_circle(screen_x, screen_y, 8.0, item_color);
                            }
                        } else {
                            draw_circle(screen_x, screen_y, 8.0, YELLOW);
                        }
                    }
                }
            }
        }

        {
            let _scope = crate::profiler::scope("render_projectiles");
            let screen_w = screen_width();
            let screen_h = screen_height();
            let proj_margin = 100.0;

            let mut batch = batched_effects::BatchedEffectsRenderer::new();

            for trail in &self.trails {
                let screen_x = trail.x - camera_x;
                let screen_y = trail.y - camera_y;
                
                let alpha = (1.0 - (trail.life as f32 / trail.max_life as f32)) * 0.8;
                let size = trail.size * (1.0 - (trail.life as f32 / trail.max_life as f32) * 0.5);
                
                let mut color = trail.color;
                color.a = alpha;
                
                batch.add_trail(screen_x, screen_y, size, color);
            }

            for smoke in &self.smokes {
                let sx = smoke.x - camera_x;
                let sy = smoke.y - camera_y;
                if sx > -proj_margin
                    && sx < screen_w + proj_margin
                    && sy > -proj_margin
                    && sy < screen_h + proj_margin
                {
                    smoke.render(camera_x, camera_y);
                }
            }

            for projectile in &self.projectiles {
                let sx = projectile.x - camera_x;
                let sy = projectile.y - camera_y;
                if sx > -proj_margin
                    && sx < screen_w + proj_margin
                    && sy > -proj_margin
                    && sy < screen_h + proj_margin
                {
                    if matches!(projectile.weapon_type, weapon::Weapon::Plasmagun) {
                        batch.add_plasma(sx, sy, 6.0, Color::from_rgba(50, 150, 255, 120));
                        batch.add_plasma(sx, sy, 5.0, Color::from_rgba(80, 180, 255, 200));
                        batch.add_plasma(sx, sy, 3.5, Color::from_rgba(150, 220, 255, 255));
                    } else {
                        projectile.render(camera_x, camera_y, &mut self.projectile_model_cache);
                    }
                }
            }

            batch.render();
        }

        let lighting_ctx = {
            let _scope = crate::profiler::scope("compute_lighting");

            let screen_w = screen_width();
            let screen_h = screen_height();
            let cull_margin = 200.0;

            let mut all_lights_for_models: Vec<map::LightSource> = Vec::with_capacity(
                self.map.lights.len()
                    + self.lights.len()
                    + self.muzzle_flashes.len()
                    + self.projectiles.len()
                    + 10,
            );

            for light in &self.map.lights {
                let dx = light.x - (camera_x + screen_w * 0.5);
                let dy = light.y - (camera_y + screen_h * 0.5);
                if dx.abs() < light.radius + screen_w * 0.5 + cull_margin
                    && dy.abs() < light.radius + screen_h * 0.5 + cull_margin
                {
                    all_lights_for_models.push(light.clone());
                }
            }

            let railgun_explosion_lights = self.railgun_effects.get_explosion_lights();
            all_lights_for_models.extend(railgun_explosion_lights);

            let railgun_linear_lights = self.railgun_effects.get_linear_lights();
            self.linear_lights.clear();
            self.linear_lights.extend(railgun_linear_lights);

            for light_pulse in &self.lights {
                let fade = 1.0 - (light_pulse.life as f32 / light_pulse.max_life as f32);
                if fade > 0.01 {
                    let dx = light_pulse.x - (camera_x + screen_w * 0.5);
                    let dy = light_pulse.y - (camera_y + screen_h * 0.5);
                    let effective_radius = light_pulse.radius * fade;
                    if dx.abs() < effective_radius + screen_w * 0.5 + cull_margin
                        && dy.abs() < effective_radius + screen_h * 0.5 + cull_margin
                    {
                        all_lights_for_models.push(map::LightSource {
                            x: light_pulse.x,
                            y: light_pulse.y,
                            radius: effective_radius,
                            r: (light_pulse.color.r * 255.0) as u8,
                            g: (light_pulse.color.g * 255.0) as u8,
                            b: (light_pulse.color.b * 255.0) as u8,
                            intensity: fade,
                            flicker: false,
                        });
                    }
                }
            }

            for muzzle in &self.muzzle_flashes {
                let elapsed = (self.time - muzzle.birth_time) as f32;
                let fade = (1.0 - elapsed / 0.1).max(0.0);
                if fade > 0.01 && !matches!(muzzle.weapon, weapon::Weapon::Railgun) {
                    let (r, g, b) = match muzzle.weapon {
                        weapon::Weapon::RocketLauncher => (255, 180, 80),
                        weapon::Weapon::Plasmagun => (80, 180, 255),
                        weapon::Weapon::BFG => (120, 255, 120),
                        _ => (255, 220, 180),
                    };

                    all_lights_for_models.push(map::LightSource {
                        x: muzzle.x,
                        y: muzzle.y,
                        radius: 150.0 * fade,
                        r,
                        g,
                        b,
                        intensity: fade * 3.0,
                        flicker: false,
                    });
                }
            }

            for projectile in &self.projectiles {
                let (r, g, b, rad) = match projectile.weapon_type {
                    weapon::Weapon::RocketLauncher => (255, 180, 80, 160.0),
                    weapon::Weapon::GrenadeLauncher => (150, 255, 120, 120.0),
                    weapon::Weapon::Plasmagun => (80, 180, 255, 100.0),
                    weapon::Weapon::BFG => (120, 255, 120, 200.0),
                    _ => (0, 0, 0, 0.0),
                };
                if rad > 0.0 {
                    all_lights_for_models.push(map::LightSource {
                        x: projectile.x,
                        y: projectile.y,
                        radius: rad,
                        r,
                        g,
                        b,
                        intensity: 1.5,
                        flicker: false,
                    });
                }
            }

            md3_render::LightingContext {
                lights: all_lights_for_models,
                ambient: self.ambient_light,
                camera_x,
                camera_y,
            }
        };

        {
            let _scope = crate::profiler::scope("render_players");
            let screen_w = screen_width();
            let screen_h = screen_height();
            let render_margin = 150.0;

            for corpse in &self.corpses {
                if corpse.player.gibbed {
                    continue;
                }
                
                let player = &corpse.player;
                let screen_x = player.x - camera_x;
                let screen_y = player.y - camera_y;

                if screen_x < -render_margin
                    || screen_x > screen_w + render_margin
                    || screen_y < -render_margin
                    || screen_y > screen_h + render_margin
                {
                    continue;
                }

                if let Some(model) = self.model_cache.get_mut(&player.model) {
                    let alpha = (corpse.lifetime / 2.0 * 255.0).min(150.0) as u8;
                    let color = Color::from_rgba(255, 255, 255, alpha);

                    let flip = player.angle.abs() > std::f32::consts::PI / 2.0;
                    let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                    let mut rel_angle = player.angle - base_dir;
                    while rel_angle > std::f32::consts::PI {
                        rel_angle -= 2.0 * std::f32::consts::PI;
                    }
                    while rel_angle < -std::f32::consts::PI {
                        rel_angle += 2.0 * std::f32::consts::PI;
                    }

                    model.render_simple(
                        screen_x,
                        screen_y,
                        color,
                        2.0,
                        flip,
                        rel_angle,
                        player.angle,
                        player.lower_frame,
                        player.upper_frame,
                        None,
                        false,
                        Some(&lighting_ctx),
                        0.0,
                        0.0,
                        0.0,
                        false,
                        0.0,
                    );
                }
            }

            for player in &self.players {
                if player.gibbed || player.dead {
                    continue;
                }
                let screen_x = player.x - camera_x;
                let screen_y = player.y - camera_y;

                if screen_x < -render_margin
                    || screen_x > screen_w + render_margin
                    || screen_y < -render_margin
                    || screen_y > screen_h + render_margin
                {
                    continue;
                }

                if let Some(model) = self.model_cache.get_mut(&player.model) {
                    let color = if player.dead {
                        Color::from_rgba(255, 255, 255, 150)
                    } else if player.powerups.quad > 0 {
                        Color::from_rgba(150, 150, 255, 255)
                    } else if player.powerups.invis > 0 {
                        Color::from_rgba(255, 255, 255, 100)
                    } else {
                        WHITE
                    };

                    let flip = if let Some(manual) = player.manual_flip_x {
                        manual
                    } else {
                        player.angle.abs() > std::f32::consts::PI / 2.0
                    };

                    let base_dir = if flip { std::f32::consts::PI } else { 0.0 };
                    let mut rel_angle = player.angle - base_dir;
                    while rel_angle > std::f32::consts::PI {
                        rel_angle -= 2.0 * std::f32::consts::PI;
                    }
                    while rel_angle < -std::f32::consts::PI {
                        rel_angle += 2.0 * std::f32::consts::PI;
                    }

                    let in_air = player.was_in_air;
                    let model_yaw_offset = if in_air {
                        (player.vel_x / self::constants::MAX_SPEED_AIR).clamp(-1.0, 1.0) * 0.8
                    } else {
                        0.0
                    };

                    let sway = if !in_air && player.vel_x.abs() > 0.5 {
                        (self.time as f32 * 12.0).sin() * 0.06
                    } else {
                        0.0
                    };

                    let mut pitch = rel_angle + sway;

                    if !in_air && player.vel_x.abs() <= 0.1 {
                        pitch += player.idle_yaw;
                    }

                    let mut model_roll = if pitch < -0.5 {
                        let intensity = ((pitch + 0.5).abs() / 1.0).min(1.0);
                        -intensity * 0.8
                    } else if pitch > 0.5 {
                        let intensity = ((pitch - 0.5) / 1.0).min(1.0);
                        intensity * 0.8
                    } else {
                        0.0
                    };

                    let mut somersault_angle = 0.0;
                    if player.somersault_time > 0.0 {
                        let t = player.somersault_time;
                        // Cubic easing for sharper, earlier rotation
                        let eased = t * t * t;
                        somersault_angle = eased * std::f32::consts::PI * 2.0;
                    }
                    
                    model_roll += somersault_angle;

                    let aim_angle = player.angle;
                    let weapon_model = self.weapon_model_cache.get(player.weapon);
                    let lower_frame = player.lower_frame;
                    let upper_frame = player.upper_frame;
                    let has_quad_damage = player.powerups.quad > 0;
                    model.render_simple(
                        screen_x,
                        screen_y,
                        color,
                        2.0,
                        flip,
                        pitch,
                        aim_angle,
                        lower_frame,
                        upper_frame,
                        weapon_model,
                        self.debug_md3,
                        Some(&lighting_ctx),
                        model_yaw_offset,
                        model_roll,
                        somersault_angle,
                        has_quad_damage,
                        if matches!(player.weapon, crate::game::weapon::Weapon::MachineGun) { player.barrel_spin_angle } else { 0.0 },
                    );

                    if !player.dead {
                        let base_y = screen_y - 50.0;
                        let name_y = base_y - 10.0;

                        let is_boss = self
                            .story_mode
                            .as_ref()
                            .and_then(|story| {
                                story.get_current_level().and_then(|level| {
                                    level.enemy_spawns.iter().find(|spawn| {
                                        spawn.is_boss
                                            && spawn.model == player.model
                                            && player.health > 300
                                    })
                                })
                            })
                            .is_some();

                        if is_boss {
                            let pulse = ((get_time() * 4.0).sin() * 0.5 + 0.5) as f32;
                            let glow_size = 50.0 + pulse * 15.0;
                            draw_circle(
                                screen_x,
                                screen_y,
                                glow_size,
                                Color::from_rgba(255, 50, 50, 40),
                            );

                            if let Some(story) = &self.story_mode {
                                if let Some(level) = story.get_current_level() {
                                    if let Some(spawn) = level.enemy_spawns.iter().find(|s| {
                                        s.is_boss && s.model == player.model && player.health > 300
                                    }) {
                                        if !spawn.boss_name.is_empty() {
                                            let boss_y = name_y - 15.0;
                                            draw_text(
                                                &spawn.boss_name,
                                                screen_x - 100.0,
                                                boss_y + 1.0,
                                                16.0,
                                                BLACK,
                                            );
                                            draw_text(
                                                &spawn.boss_name,
                                                screen_x - 100.0,
                                                boss_y,
                                                16.0,
                                                Color::from_rgba(255, 50, 50, 255),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                    }
                } else {
                    if !player.dead {
                        player.render(camera_x, camera_y);
                    }
                }
            }
        }

        {
            let _scope = crate::profiler::scope("render_debug_hitboxes");
            if self.debug_hitboxes {
                Self::render_debug_hitboxes(&self.players, &self.projectiles, camera_x, camera_y);
            }
        }

        {
            let _scope = crate::profiler::scope("render_effects");

            if self.disable_particles {
                self.railgun_effects.render(camera_x, camera_y);
            } else {
                let screen_w = screen_width();
                let screen_h = screen_height();
                let effect_margin = 50.0;

                for muzzle in &self.muzzle_flashes {
                    let sx = muzzle.x - camera_x;
                    let sy = muzzle.y - camera_y;
                    if sx > -effect_margin
                        && sx < screen_w + effect_margin
                        && sy > -effect_margin
                        && sy < screen_h + effect_margin
                    {
                        muzzle.render(camera_x, camera_y, &self.muzzle_flash_cache);
                    }
                }

                for particle in &self.particles {
                    let sx = particle.x - camera_x;
                    let sy = particle.y - camera_y;
                    if sx > -effect_margin
                        && sx < screen_w + effect_margin
                        && sy > -effect_margin
                        && sy < screen_h + effect_margin
                    {
                        particle.render(camera_x, camera_y);
                    }
                }

                if !self.gibs.is_empty() {
                    let mut gib_batch = md3_render::MD3Batch::new();
                    let mut rendered_count = 0;
                    const MAX_GIBS_PER_FRAME: usize = 150;
                    
                    for gib in &self.gibs {
                        if rendered_count >= MAX_GIBS_PER_FRAME {
                            break;
                        }
                        
                        let sx = gib.x - camera_x;
                        let sy = gib.y - camera_y;
                        if sx > -effect_margin
                            && sx < screen_w + effect_margin
                            && sy > -effect_margin
                            && sy < screen_h + effect_margin
                        {
                            gib.render_batched(camera_x, camera_y, &self.gib_model_cache, &mut gib_batch);
                            rendered_count += 1;
                        }
                    }
                    gib_batch.flush(None);
                }

                for weapon_hit in &self.weapon_hit_effects {
                    let sx = weapon_hit.x - camera_x;
                    let sy = weapon_hit.y - camera_y;
                    if sx > -effect_margin
                        && sx < screen_w + effect_margin
                        && sy > -effect_margin
                        && sy < screen_h + effect_margin
                    {
                        weapon_hit.render(camera_x, camera_y, &self.weapon_hit_texture_cache);
                    }
                }

                self.railgun_effects.render(camera_x, camera_y);
            }
        }

        {
            let _scope = crate::profiler::scope("render_story_mode");
            if let Some(ref story) = self.story_mode {
                story.render_exit_portal(camera_x, camera_y);
            }
        }

        let _t_before_end = get_time();
        {
            let _scope = crate::profiler::scope("render_end_scene");
            if !self.disable_deferred {
                renderer.end_scene();
            }
        }

        if !self.disable_deferred {
            {
                let _scope = crate::profiler::scope("render_lighting_final");

                let all_lights: Vec<map::LightSource> = {
                    let _scope = crate::profiler::scope("lighting_collect_sources");

                    let estimated_capacity = self.map.lights.len()
                        + self.lights.len() * 2
                        + self.muzzle_flashes.len()
                        + self.projectiles.len()
                        + 10;
                    let mut lights = Vec::with_capacity(estimated_capacity);
                    lights.extend_from_slice(&self.map.lights);

                    let railgun_explosion_lights = self.railgun_effects.get_explosion_lights();
                    lights.extend(railgun_explosion_lights);

                    for light_pulse in &self.lights {
                        let fade = 1.0 - (light_pulse.life as f32 / light_pulse.max_life as f32);
                        if fade > 0.01 {
                            let r = (light_pulse.color.r * 255.0) as u8;
                            let g = (light_pulse.color.g * 255.0) as u8;
                            let b = (light_pulse.color.b * 255.0) as u8;

                            lights.push(map::LightSource {
                                x: light_pulse.x,
                                y: light_pulse.y,
                                radius: light_pulse.radius * fade,
                                r,
                                g,
                                b,
                                intensity: fade * 4.0,
                                flicker: false,
                            });
                        }
                    }

                    for muzzle in &self.muzzle_flashes {
                        let elapsed = (self.time - muzzle.birth_time) as f32;
                        let fade = (1.0 - elapsed / 0.1).max(0.0);

                        if fade > 0.01 && !matches!(muzzle.weapon, weapon::Weapon::Railgun) {
                            let (r, g, b) = match muzzle.weapon {
                                weapon::Weapon::RocketLauncher => (255, 180, 80),
                                weapon::Weapon::Plasmagun => (80, 180, 255),
                                weapon::Weapon::BFG => (120, 255, 120),
                                _ => (255, 220, 180),
                            };

                            lights.push(map::LightSource {
                                x: muzzle.x,
                                y: muzzle.y,
                                radius: 150.0 * fade,
                                r,
                                g,
                                b,
                                intensity: fade * 3.0,
                                flicker: false,
                            });
                        }
                    }

                    for projectile in &self.projectiles {
                        let (r, g, b, rad) = match projectile.weapon_type {
                            weapon::Weapon::RocketLauncher => (255, 180, 80, 160.0),
                            weapon::Weapon::Plasmagun => (80, 180, 255, 100.0),
                            weapon::Weapon::BFG => (120, 255, 120, 200.0),
                            _ => (0, 0, 0, 0.0),
                        };
                        if rad > 0.0 {
                            lights.push(map::LightSource {
                                x: projectile.x,
                                y: projectile.y,
                                radius: rad,
                                r,
                                g,
                                b,
                                intensity: 1.5,
                                flicker: false,
                            });
                        }
                    }

                    lights
                };

                let _t_before_lighting = get_time();
                {
                    let _scope = crate::profiler::scope("lighting_apply");
                    renderer.apply_lighting(
                        &self.map,
                        &self.map.lights,
                        &all_lights,
                        &self.linear_lights,
                        camera_x,
                        camera_y,
                        zoom,
                        self.ambient_light,
                        self.disable_shadows,
                        self.disable_dynamic_lights,
                        self.cartoon_shader,
                    );
                }
            }
        }

        {
            let _scope = crate::profiler::scope("render_damage_numbers");
            for damage_number in &self.damage_numbers {
                damage_number.render(camera_x, camera_y);
            }
        }

        {
            let _scope = crate::profiler::scope("render_awards");
            
            let local_player_id = if self.is_multiplayer {
                self.network_client.as_ref().and_then(|c| c.player_id())
            } else {
                self.players.get(0).map(|p| p.id)
            };
            
            let award_material = award_shader::get_award_shader_material();
            gl_use_material(award_material);
            
            for award in &self.awards {
                let is_local_award = Some(award.player_id) == local_player_id;
                
                if is_local_award {
                    if let Some(texture) = self.award_icon_cache.get(&award.award_type) {
                        let size = 96.0 * award.scale;
                        let screen_w = macroquad::prelude::screen_width();
                        let x = screen_w / 2.0 - size / 2.0;
                        let y = 100.0;
                        let bounce = (award.lifetime * 6.0).sin() * 8.0 * award.scale;
                        
                        award_material.set_uniform("time", award.lifetime);
                        award_material.set_uniform("scale", award.scale);
                        
                        draw_texture_ex(
                            texture,
                            x,
                            y + bounce,
                            Color::new(1.0, 1.0, 1.0, award.scale),
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(size, size)),
                                ..Default::default()
                            },
                        );
                    }
                    continue;
                }
                
                if let Some(player) = self.players.iter().find(|p| p.id == award.player_id) {
                    if player.gibbed || player.dead {
                        continue;
                    }
                    
                    let world_x = player.x;
                    let world_y = player.y - 120.0;
                    
                    let screen_x = world_x - camera_x;
                    let screen_y = world_y - camera_y;
                    
                    if let Some(texture) = self.award_icon_cache.get(&award.award_type) {
                        let size = 48.0 * award.scale;
                        let bounce = (award.lifetime * 6.0).sin() * 4.0 * award.scale;
                        
                        award_material.set_uniform("time", award.lifetime);
                        award_material.set_uniform("scale", award.scale);
                        
                        draw_texture_ex(
                            texture,
                            screen_x - size / 2.0,
                            screen_y - bounce,
                            Color::new(1.0, 1.0, 1.0, award.scale),
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(size, size)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            
            gl_use_default_material();
        }

        if let Some(defrag) = &self.defrag_mode {
            defrag.render(camera_x, camera_y);
        }
        
        self.game_results.draw(self.match_time, &mut self.model_cache, &self.award_icon_cache, &self.weapon_model_cache, camera_x, camera_y);
    }

    pub fn render_messages(&self) {
        for (i, msg) in self.messages.iter().enumerate() {
            msg.render(i as f32 * 22.0);
        }
    }

    pub fn render_defrag_hud(&self) {
        if let Some(defrag) = &self.defrag_mode {
            defrag.render_hud();
        }
    }

    fn render_debug_hitboxes(
        players: &[player::Player],
        projectiles: &[projectile::Projectile],
        camera_x: f32,
        camera_y: f32,
    ) {
        for player in players {
            if player.gibbed {
                continue;
            }

            let screen_x = player.x - camera_x;
            let screen_y = player.y - camera_y;

            let hitbox_height = if player.dead {
                constants::PLAYER_HITBOX_HEIGHT_CROUCH
            } else if player.crouch {
                constants::PLAYER_HITBOX_HEIGHT_CROUCH
            } else {
                constants::PLAYER_HITBOX_HEIGHT
            };
            let hitbox_width = constants::PLAYER_HITBOX_WIDTH;

            let hitbox_x = screen_x - hitbox_width / 2.0;
            let hitbox_y = screen_y - hitbox_height + 16.0;

            let color = if player.dead {
                Color::from_rgba(128, 128, 128, 100)
            } else if player.is_bot {
                Color::from_rgba(255, 100, 100, 150)
            } else {
                Color::from_rgba(100, 255, 100, 150)
            };

            draw_rectangle_lines(hitbox_x, hitbox_y, hitbox_width, hitbox_height, 2.0, color);

            let center_color = Color::from_rgba(color.r as u8, color.g as u8, color.b as u8, 80);
            draw_rectangle(
                hitbox_x,
                hitbox_y,
                hitbox_width,
                hitbox_height,
                center_color,
            );

            draw_circle(screen_x, screen_y, 2.0, Color::from_rgba(255, 255, 0, 200));

            let hitbox_center_y = screen_y - hitbox_height / 2.0;
            draw_circle(
                screen_x,
                hitbox_center_y,
                1.5,
                Color::from_rgba(255, 0, 255, 255),
            );
        }

        for projectile in projectiles {
            if !projectile.active {
                continue;
            }

            let screen_x = projectile.x - camera_x;
            let screen_y = projectile.y - camera_y;

            let proj_size = 4.0;
            let proj_color = Color::from_rgba(255, 0, 255, 180);

            draw_rectangle_lines(
                screen_x - proj_size / 2.0,
                screen_y - proj_size / 2.0,
                proj_size,
                proj_size,
                1.0,
                proj_color,
            );

            draw_circle(
                screen_x,
                screen_y,
                1.0,
                Color::from_rgba(255, 255, 255, 255),
            );
        }
    }
}
