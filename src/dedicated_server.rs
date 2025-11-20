use std::collections::HashMap;
use sas::network::{NetworkConfig, NetMessage, PlayerState, server::NetworkServer, Trajectory, TrajectoryType, get_network_time, get_absolute_time};
use sas::game::map::Map;
use sas::game::bg_pmove::{PmoveState, PmoveCmd, pmove};
use sas::game::projectile::Projectile;
use sas::game::usercmd::UserCmd;

const GIB_HEALTH: i32 = -40;

struct DedicatedServer {
    server: NetworkServer,
    game_state: GameState,
    map_name: String,
    pmove_accumulator: f32,
    last_frame_time: std::time::Instant,
    next_bot_id: u16,
}

struct GameState {
    map: Map,
    players: HashMap<u16, ServerPlayer>,
    projectiles: Vec<Projectile>,
    tick: u32,
    next_projectile_id: u32,
}

struct ServerPlayer {
    name: String,
    pmove_state: PmoveState,
    angle: f32,
    health: i32,
    armor: i32,
    weapon: u8,
    ammo: [u16; 10],
    frags: i32,
    deaths: i32,
    powerup_quad: u16,
    last_cmd: UserCmd,
    pending_commands: Vec<UserCmd>,
    last_executed_time: u32,
    dead: bool,
    gibbed: bool,
    respawn_timer: f32,
    corpse_timer: f32,
    is_bot: bool,
    bot_ai: Option<sas::game::bot_ai::BotAI>,
}

impl DedicatedServer {
    fn new(config: NetworkConfig, map_name: String) -> Self {
        let map = Map::load_from_file(&map_name)
            .unwrap_or_else(|e| {
                eprintln!("Failed to load map '{}': {}", map_name, e);
                eprintln!("Using default map");
                Map::new(&map_name)
            });
        
        println!("Loaded map: {}", map_name);
        println!("Spawn points: {}", map.spawn_points.len());
        
        Self {
            server: NetworkServer::new(config),
            game_state: GameState {
                map,
                players: HashMap::new(),
                projectiles: Vec::new(),
                tick: 0,
                next_projectile_id: 1,
            },
            map_name,
            pmove_accumulator: 0.0,
            last_frame_time: std::time::Instant::now(),
            next_bot_id: 1000,
        }
    }

    fn start(&mut self) -> Result<(), String> {
        self.server.start()?;
        println!("Dedicated server running. Press Ctrl+C to stop.");
        Ok(())
    }

    fn run(&mut self) {
        const FIXED_DT: f32 = 1.0 / 60.0;
        
        loop {
            let now = std::time::Instant::now();
            let dt = now.duration_since(self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;
            
            let (messages, timed_out_clients) = self.server.update();
            
            for client_id in timed_out_clients {
                if let Some(player) = self.game_state.players.get(&client_id) {
                    if !player.is_bot {
                        println!("[{:.3}] [SERVER] Removing timed out player {} ({})", 
                            sas::network::get_network_time(), client_id, player.name);
                        self.remove_player(client_id);
                    }
                }
            }
            
            for (client_id, msg) in messages {
                self.handle_message(client_id, msg);
            }

            self.pmove_accumulator += dt;
            
            while self.pmove_accumulator >= FIXED_DT {
                self.update_bot_ai(FIXED_DT);
                self.simulate_physics(FIXED_DT);
                self.update_projectiles(FIXED_DT);
                self.check_collisions();
                self.check_item_pickups();

                self.game_state.tick += 1;

                if self.game_state.tick % 2 == 0 {
                    self.broadcast_game_state();
                }
                
                self.pmove_accumulator -= FIXED_DT;
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    fn handle_message(&mut self, client_id: u16, msg: NetMessage) {
        self.server.update_delta_message(client_id);
        
        match &msg {
            NetMessage::ConnectRequest { player_name, .. } => {
                println!("[{:.3}] [SERVER] Client {} connecting: {}", 
                    sas::network::get_network_time(), client_id, player_name);
                self.add_player(client_id, player_name.clone());
            }
            NetMessage::Disconnect { reason, .. } => {
                println!("[{:.3}] [SERVER] Client {} disconnected: {}", 
                    sas::network::get_network_time(), client_id, reason);
                self.remove_player(client_id);
            }
            NetMessage::PlayerInput { move_forward, move_right, angle, buttons, server_time, .. } => {
                self.update_player_input(client_id, *move_forward, *move_right, *angle, *buttons, *server_time);
            }
            NetMessage::PlayerInputBatch { commands, .. } => {
                self.update_player_input_batch(client_id, commands.clone());
            }
            NetMessage::PlayerShoot { weapon, origin, direction, .. } => {
                println!("[{:.3}] [SERVER] Player {} shot weapon {} at ({:.1}, {:.1})", 
                    sas::network::get_network_time(), client_id, weapon, origin.0, origin.1);
                self.handle_player_shoot(client_id, *weapon, *origin, *direction);
            }
            NetMessage::Chat { message, .. } => {
                if message.starts_with("addbot") {
                    self.handle_addbot_command(client_id);
                } else {
                    self.handle_chat(client_id, message.clone());
                }
            }
            NetMessage::Heartbeat => {
            }
            _ => {}
        }
    }

    fn add_player(&mut self, client_id: u16, name: String) {
        self.add_player_internal(client_id, name, false);
    }
    
    fn add_player_internal(&mut self, client_id: u16, name: String, is_bot: bool) {
        let spawn_pos = if !self.game_state.map.spawn_points.is_empty() {
            let spawn_idx = (client_id as usize) % self.game_state.map.spawn_points.len();
            let spawn = &self.game_state.map.spawn_points[spawn_idx];
            (spawn.x, spawn.y)
        } else {
            (0.0, 0.0)
        };
        
        let player = ServerPlayer {
            name: name.clone(),
            pmove_state: PmoveState {
                x: spawn_pos.0,
                y: spawn_pos.1,
                vel_x: 0.0,
                vel_y: 0.0,
                was_in_air: false,
            },
            angle: 0.0,
            health: 100,
            armor: 0,
            weapon: 2,
            ammo: [100, 50, 10, 10, 5, 0, 0, 0, 0, 0],
            frags: 0,
            deaths: 0,
            powerup_quad: 0,
            last_cmd: UserCmd::new(),
            pending_commands: Vec::new(),
            last_executed_time: 0,
            dead: false,
            gibbed: false,
            respawn_timer: 0.0,
            corpse_timer: 0.0,
            is_bot,
            bot_ai: if is_bot { Some(sas::game::bot_ai::BotAI::new()) } else { None },
        };

        self.game_state.players.insert(client_id, player);
        
        if !is_bot {
            let map_msg = NetMessage::MapChange {
                map_name: self.map_name.clone(),
            };
            self.server.send_to(client_id, map_msg).ok();
        }
        
        let respawn_msg = NetMessage::PlayerRespawn {
            player_id: client_id,
            position: spawn_pos,
        };
        self.server.broadcast(respawn_msg).ok();
        
        if is_bot {
            println!("Bot {} ({}) joined the game on map {} at ({}, {})", 
                     client_id, name, self.map_name, spawn_pos.0, spawn_pos.1);
        } else {
            println!("Player {} ({}) joined the game on map {} at ({}, {})", 
                     client_id, name, self.map_name, spawn_pos.0, spawn_pos.1);
        }
    }

    fn remove_player(&mut self, client_id: u16) {
        if let Some(server_player) = self.game_state.players.remove(&client_id) {
            println!("Player {} ({}) left the game", client_id, server_player.name);
            
            let msg = NetMessage::Disconnect {
                player_id: client_id,
                reason: "Player disconnected".to_string(),
            };
            self.server.broadcast(msg).ok();
        }
    }

    fn update_player_input(&mut self, client_id: u16, _move_forward: f32, move_right: f32, angle: f32, buttons: u32, server_time: u32) {
        if let Some(server_player) = self.game_state.players.get_mut(&client_id) {
            println!("[{}] [SERVER INPUT] p{} pos=({:.1},{:.1}) move_right={:.2}", 
                get_absolute_time(), client_id, server_player.pmove_state.x, server_player.pmove_state.y, move_right);
            server_player.angle = angle;
            server_player.last_cmd = UserCmd {
                right: move_right,
                buttons: buttons as u8,
                angles: (angle, 0.0),
                server_time,
            };
            
            if server_player.dead && (buttons & 1) != 0 {
                let can_respawn = server_player.respawn_timer <= 5.0;
                if can_respawn {
                    self.trigger_respawn(client_id);
                }
            }
        }
    }
    
    fn update_player_input_batch(&mut self, client_id: u16, commands: Vec<sas::network::PlayerInputCmd>) {
        if let Some(server_player) = self.game_state.players.get_mut(&client_id) {
            for cmd in commands {
                if cmd.server_time > server_player.last_executed_time {
                    server_player.pending_commands.push(UserCmd {
                        right: cmd.move_right,
                        buttons: cmd.buttons as u8,
                        angles: (cmd.angle, 0.0),
                        server_time: cmd.server_time,
                    });
                }
            }
            
            server_player.pending_commands.sort_by_key(|c| c.server_time);
            
            if let Some(last_cmd) = server_player.pending_commands.last() {
                server_player.angle = last_cmd.angles.0;
                server_player.last_cmd = *last_cmd;
                
                if server_player.dead && (last_cmd.buttons & 1) != 0 {
                    let can_respawn = server_player.respawn_timer <= 5.0;
                    if can_respawn {
                        self.trigger_respawn(client_id);
                    }
                }
            }
        }
    }
    
    fn trigger_respawn(&mut self, client_id: u16) {
        if let Some(server_player) = self.game_state.players.get_mut(&client_id) {
            if !server_player.dead {
                return;
            }
            
            server_player.corpse_timer = 2.0;
            self.respawn_player(client_id);
        }
    }

    fn simulate_physics(&mut self, dt: f32) {
        let player_ids: Vec<u16> = self.game_state.players.keys().copied().collect();
        
        for player_id in player_ids {
            if let Some(server_player) = self.game_state.players.get_mut(&player_id) {
                if server_player.corpse_timer > 0.0 {
                    server_player.corpse_timer -= dt;
                }
                
                if server_player.dead {
                    server_player.respawn_timer -= dt;
                    if server_player.respawn_timer <= 0.0 {
                        server_player.corpse_timer = 2.0;
                        self.respawn_player(player_id);
                    }
                    continue;
                }
                
                if server_player.powerup_quad > 0 {
                    server_player.powerup_quad = server_player.powerup_quad.saturating_sub(1);
                }
                
                let commands: Vec<UserCmd> = server_player.pending_commands.drain(..).collect();
                
                if !commands.is_empty() {
                    for cmd in commands {
                        if cmd.server_time <= server_player.last_executed_time {
                            continue;
                        }
                        
                        let pmove_cmd = PmoveCmd {
                            move_right: cmd.right,
                            jump: (cmd.buttons & 2) != 0,
                            crouch: (cmd.buttons & 4) != 0,
                            haste_active: false,
                        };
                        
                        let cmd_dt = if server_player.last_executed_time > 0 {
                            (cmd.server_time - server_player.last_executed_time) as f32 / 1000.0
                        } else {
                            1.0 / 60.0
                        };
                        
                        let cmd_dt = cmd_dt.max(0.001).min(0.1);
                        
                        println!("[{}] [SERVER PMOVE] p{} pos=({:.1},{:.1}) dt={:.4} move_right={:.2} cmd_time={} last_exec={}",
                            get_absolute_time(), player_id, server_player.pmove_state.x, server_player.pmove_state.y, cmd_dt, cmd.right, cmd.server_time, server_player.last_executed_time);
                        
                        let result = pmove(&server_player.pmove_state, &pmove_cmd, cmd_dt, &self.game_state.map);
                        
                        let mut teleported = false;
                        for teleporter in &self.game_state.map.teleporters {
                            if result.new_x >= teleporter.x
                                && result.new_x <= teleporter.x + teleporter.width
                                && result.new_y >= teleporter.y
                                && result.new_y <= teleporter.y + teleporter.height
                            {
                                println!("[{:.3}] [SERVER TELEPORT] p{} from ({:.1},{:.1}) to ({:.1},{:.1})",
                                    sas::network::get_network_time(), player_id,
                                    result.new_x, result.new_y,
                                    teleporter.dest_x, teleporter.dest_y);
                                server_player.pmove_state.x = teleporter.dest_x;
                                server_player.pmove_state.y = teleporter.dest_y;
                                server_player.pmove_state.vel_x = result.new_vel_x;
                                server_player.pmove_state.vel_y = result.new_vel_y;
                                server_player.pmove_state.was_in_air = result.new_was_in_air;
                                teleported = true;
                                break;
                            }
                        }
                        
                        if !teleported {
                            server_player.pmove_state.x = result.new_x;
                            server_player.pmove_state.y = result.new_y;
                            server_player.pmove_state.vel_x = result.new_vel_x;
                            server_player.pmove_state.vel_y = result.new_vel_y;
                            server_player.pmove_state.was_in_air = result.new_was_in_air;
                        }
                        
                        server_player.last_executed_time = cmd.server_time;
                        server_player.angle = cmd.angles.0;
                        server_player.last_cmd = cmd;
                    }
                } else {
                    let cmd = server_player.last_cmd;
                    
                    let pmove_cmd = PmoveCmd {
                        move_right: cmd.right,
                        jump: (cmd.buttons & 2) != 0,
                        crouch: (cmd.buttons & 4) != 0,
                        haste_active: false,
                    };
                    
                    let current_tick = self.game_state.tick;
                    let tick_rate = 60;
                    let current_server_time = ((current_tick as u64) * 1000 / tick_rate) as u32;
                    server_player.last_executed_time = current_server_time;
                    
                    let result = pmove(&server_player.pmove_state, &pmove_cmd, dt, &self.game_state.map);
                    
                    let mut teleported = false;
                    for teleporter in &self.game_state.map.teleporters {
                        if result.new_x >= teleporter.x
                            && result.new_x <= teleporter.x + teleporter.width
                            && result.new_y >= teleporter.y
                            && result.new_y <= teleporter.y + teleporter.height
                        {
                            println!("[{:.3}] [SERVER TELEPORT] p{} from ({:.1},{:.1}) to ({:.1},{:.1})",
                                sas::network::get_network_time(), player_id,
                                result.new_x, result.new_y,
                                teleporter.dest_x, teleporter.dest_y);
                            server_player.pmove_state.x = teleporter.dest_x;
                            server_player.pmove_state.y = teleporter.dest_y;
                            server_player.pmove_state.vel_x = result.new_vel_x;
                            server_player.pmove_state.vel_y = result.new_vel_y;
                            server_player.pmove_state.was_in_air = result.new_was_in_air;
                            teleported = true;
                            break;
                        }
                    }
                    
                    if !teleported {
                        server_player.pmove_state.x = result.new_x;
                        server_player.pmove_state.y = result.new_y;
                        server_player.pmove_state.vel_x = result.new_vel_x;
                        server_player.pmove_state.vel_y = result.new_vel_y;
                        server_player.pmove_state.was_in_air = result.new_was_in_air;
                    }
                }
            }
        }
    }
    

    fn handle_player_shoot(&mut self, client_id: u16, weapon: u8, origin: (f32, f32), direction: f32) {
        let msg = NetMessage::PlayerShoot {
            player_id: client_id,
            weapon,
            origin,
            direction,
        };
        self.server.broadcast(msg).ok();
        
        if weapon == 3 || weapon == 4 || weapon == 5 || weapon == 6 {
            let weapon_enum: sas::game::weapon::Weapon = unsafe { std::mem::transmute(weapon) };
            
            let mut projectile = Projectile::new(
                origin.0,
                origin.1,
                direction,
                client_id,
                weapon_enum,
                0.0,
                0.0,
            );
            projectile.id = self.game_state.next_projectile_id;
            
            self.game_state.next_projectile_id += 1;
            self.game_state.projectiles.push(projectile);
        }
    }

    fn handle_chat(&mut self, client_id: u16, message: String) {
        if let Some(name) = self.server.get_client_name(client_id) {
            println!("[CHAT] {}: {}", name, message);
            
            let chat_msg = NetMessage::Chat {
                player_id: client_id,
                message,
            };
            self.server.broadcast(chat_msg).ok();
        }
    }
    
    fn update_projectiles(&mut self, dt: f32) {
        for proj in &mut self.game_state.projectiles {
            proj.update(dt, &self.game_state.map);
        }
    }
    
    fn check_collisions(&mut self) {
        let projectiles = self.game_state.projectiles.clone();
        let mut explosion_events: Vec<(f32, f32, i32, f32, u16)> = Vec::new();
        
        for proj in &projectiles {
            let explosion_radius = proj.explosion_radius();
            let has_explosion = explosion_radius > 0.0;
            
            if !proj.active && has_explosion {
                println!("[{:.3}] [SERVER] Projectile {} hit wall, explosion at ({:.1}, {:.1})", 
                    sas::network::get_network_time(), proj.id, proj.x, proj.y);
                explosion_events.push((proj.x, proj.y, proj.damage, explosion_radius, proj.owner_id));
                continue;
            }
            
            if !proj.active {
                continue;
            }
            
            for (player_id, player) in &self.game_state.players {
                if *player_id != proj.owner_id && proj.check_hit(player.pmove_state.x, player.pmove_state.y) {
                    if has_explosion {
                        println!("[{:.3}] [SERVER] Projectile {} direct hit on player {}", 
                            sas::network::get_network_time(), proj.id, player_id);
                        
                        explosion_events.push((proj.x, proj.y, proj.damage, explosion_radius, proj.owner_id));
                        
                        if let Some(p) = self.game_state.projectiles.iter_mut()
                            .find(|p| p.id == proj.id) {
                            p.active = false;
                        }
                    } else {
                        println!("[{:.3}] [SERVER] Projectile {} hit player {}", 
                            sas::network::get_network_time(), proj.id, player_id);
                        
                        self.apply_direct_damage(proj.owner_id, *player_id, proj.damage);
                        
                        if let Some(p) = self.game_state.projectiles.iter_mut()
                            .find(|p| p.id == proj.id) {
                            p.active = false;
                        }
                    }
                    break;
                }
            }
        }
        
        for (explosion_x, explosion_y, damage, radius, owner_id) in explosion_events {
            self.apply_explosion_damage(explosion_x, explosion_y, damage, radius, owner_id);
        }
        
        self.game_state.projectiles.retain(|p| p.active);
    }
    
    fn apply_explosion_damage(&mut self, explosion_x: f32, explosion_y: f32, damage: i32, radius: f32, owner_id: u16) {
        let player_ids: Vec<u16> = self.game_state.players.keys().copied().collect();
        
        for player_id in player_ids {
            if let Some(player) = self.game_state.players.get_mut(&player_id) {
                let dx = player.pmove_state.x - explosion_x;
                let dy = player.pmove_state.y - explosion_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < radius {
                    let distance_falloff = (radius - dist) / radius;
                    let damage_points = (damage as f32 * distance_falloff).max(0.0);
                    
                    let mass = 200.0;
                    let g_knockback = 1600.0;
                    let knockback = damage_points.min(200.0);
                    let knockback_scale = g_knockback * knockback / mass;
                    
                    if dist > 0.1 {
                        let dir_x = dx / dist;
                        let dir_y = dy / dist;
                        
                        player.pmove_state.vel_x += dir_x * knockback_scale;
                        player.pmove_state.vel_y += dir_y * knockback_scale;
                    }
                    
                    if player.dead {
                        if !player.gibbed {
                            player.health -= damage_points as i32;
                            if player.health <= GIB_HEALTH {
                                player.gibbed = true;
                                
                                let gib_msg = NetMessage::PlayerGibbed {
                                    player_id,
                                    position: (player.pmove_state.x, player.pmove_state.y),
                                };
                                self.server.broadcast(gib_msg).ok();
                            }
                        }
                    } else if player_id != owner_id {
                        let actual_damage = damage_points as i32;
                        player.health -= actual_damage;
                        
                        let msg = NetMessage::PlayerDamaged {
                            target_id: player_id,
                            attacker_id: owner_id,
                            damage: actual_damage,
                            health_remaining: player.health,
                            knockback_x: player.pmove_state.vel_x,
                            knockback_y: player.pmove_state.vel_y,
                        };
                        self.server.broadcast(msg).ok();
                        
                        if player.health <= 0 {
                            player.deaths += 1;
                            player.dead = true;
                            player.respawn_timer = 7.0;
                            
                            let gibbed = player.health <= GIB_HEALTH;
                            let death_pos = (player.pmove_state.x, player.pmove_state.y);
                            let death_vel = (player.pmove_state.vel_x, player.pmove_state.vel_y);
                            
                            if let Some(attacker) = self.game_state.players.get_mut(&owner_id) {
                                attacker.frags += 1;
                            }
                            
                            println!("[{:.3}] [SERVER] Player {} killed by {} explosion (gibbed: {}) at ({:.1}, {:.1})",
                                get_network_time(), player_id, owner_id, gibbed, death_pos.0, death_pos.1);
                            
                            let death_msg = NetMessage::PlayerDied {
                                player_id,
                                killer_id: owner_id,
                                gibbed,
                                position: death_pos,
                                velocity: death_vel,
                            };
                            self.server.broadcast(death_msg).ok();
                        }
                    } else if owner_id == player_id {
                        let self_damage = (damage_points * 0.5) as i32;
                        player.health -= self_damage;
                        
                        println!("[{:.3}] [SERVER] Rocketjump! Player {} dmg={} vel=({:.2}, {:.2})",
                            sas::network::get_network_time(), 
                            player_id, self_damage, player.pmove_state.vel_x, player.pmove_state.vel_y);
                        
                        let msg = NetMessage::PlayerDamaged {
                            target_id: player_id,
                            attacker_id: owner_id,
                            damage: self_damage,
                            health_remaining: player.health,
                            knockback_x: player.pmove_state.vel_x,
                            knockback_y: player.pmove_state.vel_y,
                        };
                        self.server.broadcast(msg).ok();
                        
                        if player.health <= 0 {
                            player.deaths += 1;
                            player.dead = true;
                            player.respawn_timer = 7.0;
                            
                            let gibbed = player.health <= GIB_HEALTH;
                            let death_pos = (player.pmove_state.x, player.pmove_state.y);
                            let death_vel = (player.pmove_state.vel_x, player.pmove_state.vel_y);
                            
                            println!("[{:.3}] [SERVER] Player {} suicide (gibbed: {}) at ({:.1}, {:.1})",
                                get_network_time(), player_id, gibbed, death_pos.0, death_pos.1);
                            
                            let death_msg = NetMessage::PlayerDied {
                                player_id,
                                killer_id: owner_id,
                                gibbed,
                                position: death_pos,
                                velocity: death_vel,
                            };
                            self.server.broadcast(death_msg).ok();
                        }
                    }
                }
            }
        }
    }
    
    fn apply_direct_damage(&mut self, attacker_id: u16, target_id: u16, damage: i32) {
        if let Some(target) = self.game_state.players.get_mut(&target_id) {
            if target.dead && !target.gibbed {
                let was_gibbed = target.gibbed;
                target.health -= damage;
                if target.health <= GIB_HEALTH && !was_gibbed {
                    target.gibbed = true;
                    
                    let gib_msg = NetMessage::PlayerGibbed {
                        player_id: target_id,
                        position: (target.pmove_state.x, target.pmove_state.y),
                    };
                    self.server.broadcast(gib_msg).ok();
                }
                return;
            }
            
            target.health -= damage;
            
            let msg = NetMessage::PlayerDamaged {
                target_id,
                attacker_id,
                damage,
                health_remaining: target.health,
                knockback_x: target.pmove_state.vel_x,
                knockback_y: target.pmove_state.vel_y,
            };
            self.server.broadcast(msg).ok();
            
            if target.health <= 0 {
                target.deaths += 1;
                target.dead = true;
                target.respawn_timer = 7.0;
                
                let gibbed = target.health <= GIB_HEALTH;
                let death_pos = (target.pmove_state.x, target.pmove_state.y);
                let death_vel = (target.pmove_state.vel_x, target.pmove_state.vel_y);
                
                if let Some(attacker) = self.game_state.players.get_mut(&attacker_id) {
                    attacker.frags += 1;
                }
                
                println!("[{:.3}] [SERVER] Player {} killed by {} (gibbed: {}) at ({:.1}, {:.1})",
                    get_network_time(), target_id, attacker_id, gibbed, death_pos.0, death_pos.1);
                
                let death_msg = NetMessage::PlayerDied {
                    player_id: target_id,
                    killer_id: attacker_id,
                    gibbed,
                    position: death_pos,
                    velocity: death_vel,
                };
                self.server.broadcast(death_msg).ok();
            }
        }
    }
    
    fn respawn_player(&mut self, player_id: u16) {
        if let Some(player) = self.game_state.players.get_mut(&player_id) {
            let spawn_idx = (player_id as usize) % self.game_state.map.spawn_points.len().max(1);
            if let Some(spawn_point) = self.game_state.map.spawn_points.get(spawn_idx) {
                player.pmove_state.x = spawn_point.x;
                player.pmove_state.y = spawn_point.y;
                player.pmove_state.vel_x = 0.0;
                player.pmove_state.vel_y = 0.0;
                player.health = 100;
                player.dead = false;
                player.gibbed = false;
                player.respawn_timer = 0.0;
                player.armor = 0;
                
                let respawn_msg = NetMessage::PlayerRespawn {
                    player_id,
                    position: (spawn_point.x, spawn_point.y),
                };
                self.server.broadcast(respawn_msg).ok();
            }
        }
    }

    fn check_item_pickups(&mut self) {
        let player_ids: Vec<u16> = self.game_state.players.keys().copied().collect();
        
        for player_id in player_ids {
            if let Some(player) = self.game_state.players.get(&player_id) {
                let px = player.pmove_state.x;
                let py = player.pmove_state.y;
                
                for item in &mut self.game_state.map.items {
                    if !item.active {
                        if item.respawn_time > 0 {
                            item.respawn_time -= 1;
                        } else {
                            item.active = true;
                        }
                        continue;
                    }
                    
                    let dx = px - item.x;
                    let dy = py - item.y;
                    if (dx * dx + dy * dy).sqrt() < 24.0 {
                        use sas::game::map::ItemType::*;
                        
                        let mut picked_up = false;
                        
                        if let Some(player) = self.game_state.players.get_mut(&player_id) {
                            match item.item_type {
                                Health25 => {
                                    if player.health < 100 {
                                        player.health = (player.health + 25).min(100);
                                        picked_up = true;
                                        item.respawn_time = 300;
                                    }
                                }
                                Health50 => {
                                    if player.health < 100 {
                                        player.health = (player.health + 50).min(100);
                                        picked_up = true;
                                        item.respawn_time = 300;
                                    }
                                }
                                Health100 => {
                                    if player.health < 200 {
                                        player.health = (player.health + 100).min(200);
                                        picked_up = true;
                                        item.respawn_time = 300;
                                    }
                                }
                                Armor50 => {
                                    if player.armor < 100 {
                                        player.armor = (player.armor + 50).min(100);
                                        picked_up = true;
                                        item.respawn_time = 300;
                                    }
                                }
                                Armor100 => {
                                    if player.armor < 200 {
                                        player.armor = (player.armor + 100).min(200);
                                        picked_up = true;
                                        item.respawn_time = 300;
                                    }
                                }
                                RocketLauncher => {
                                    picked_up = true;
                                    item.respawn_time = 300;
                                }
                                Railgun => {
                                    picked_up = true;
                                    item.respawn_time = 300;
                                }
                                Plasmagun => {
                                    picked_up = true;
                                    item.respawn_time = 300;
                                }
                                Shotgun => {
                                    picked_up = true;
                                    item.respawn_time = 300;
                                }
                                GrenadeLauncher => {
                                    picked_up = true;
                                    item.respawn_time = 300;
                                }
                                BFG => {
                                    picked_up = true;
                                    item.respawn_time = 600;
                                }
                                Quad => {
                                    player.powerup_quad = 1800;
                                    picked_up = true;
                                    item.respawn_time = 7200;
                                    println!("[{:.3}] [SERVER] Player {} picked up QUAD DAMAGE!", 
                                        sas::network::get_network_time(), player_id);
                                }
                                Regen => {
                                    picked_up = true;
                                    item.respawn_time = 7200;
                                }
                                Battle => {
                                    picked_up = true;
                                    item.respawn_time = 7200;
                                }
                                Flight => {
                                    picked_up = true;
                                    item.respawn_time = 3600;
                                }
                                Haste => {
                                    picked_up = true;
                                    item.respawn_time = 7200;
                                }
                                Invis => {
                                    picked_up = true;
                                    item.respawn_time = 7200;
                                }
                            }
                        }
                        
                        if picked_up {
                            item.active = false;
                        }
                    }
                }
            }
        }
    }

    fn handle_addbot_command(&mut self, requesting_client_id: u16) {
        let bot_id = self.next_bot_id;
        self.next_bot_id += 1;
        
        let bot_names = vec!["Sarge", "Doom", "Visor", "Ranger", "Anarki", "Bitterman", "Slash"];
        let bot_name = bot_names[bot_id as usize % bot_names.len()].to_string();
        
        self.add_player_internal(bot_id, bot_name.clone(), true);
        
        println!("[{:.3}] [SERVER] Bot {} added by client {}", 
            sas::network::get_network_time(), bot_name, requesting_client_id);
        
        let chat_msg = NetMessage::Chat {
            player_id: 0,
            message: format!("Bot {} joined the game", bot_name),
        };
        self.server.broadcast(chat_msg).ok();
    }
    
    fn update_bot_ai(&mut self, _dt: f32) {
        let bot_ids: Vec<u16> = self.game_state.players.iter()
            .filter(|(_, p)| p.is_bot && !p.dead)
            .map(|(id, _)| *id)
            .collect();
        
        if bot_ids.is_empty() {
            return;
        }
        
        let players_snapshot: Vec<sas::game::player::Player> = self.game_state.players.iter()
            .map(|(id, sp)| Self::convert_to_player_static(*id, sp))
            .collect();
        
        let projectiles_snapshot: Vec<sas::game::projectile::Projectile> = 
            self.game_state.projectiles.clone();
        
        let game_map = &self.game_state.map;
        let current_tick = self.game_state.tick;
        
        let mut bot_actions: Vec<(u16, u8, (f32, f32), f32)> = Vec::new();
        
        for bot_id in bot_ids {
            if let Some(bot) = self.game_state.players.get_mut(&bot_id) {
                let bot_player = Self::convert_to_player_static(bot_id, bot);
                
                if let Some(ref mut ai) = bot.bot_ai {
                    ai.think(&bot_player, &players_snapshot, game_map, 
                            &projectiles_snapshot, None);
                    
                    let move_right = ai.move_direction;
                    let want_jump = ai.want_jump;
                    let want_shoot = ai.want_shoot;
                    let target_player = ai.target_player;
                    
                    let mut buttons = 0u32;
                    if want_jump {
                        buttons |= 2;
                    }
                    
                    let angle = if let Some(target_id) = target_player {
                        if let Some(target) = players_snapshot.iter().find(|p| p.id == target_id) {
                            let dx = target.x - bot.pmove_state.x;
                            let dy = target.y - bot.pmove_state.y;
                            dy.atan2(dx)
                        } else {
                            bot.angle
                        }
                    } else {
                        bot.angle
                    };
                    
                    bot.angle = angle;
                    bot.last_cmd = UserCmd {
                        right: move_right,
                        buttons: buttons as u8,
                        angles: (angle, 0.0),
                        server_time: current_tick,
                    };
                    
                    if want_shoot {
                        let weapon = bot.weapon;
                        let weapon_idx = weapon as usize;
                        if weapon_idx < bot.ammo.len() && bot.ammo[weapon_idx] > 0 {
                            let origin = (bot.pmove_state.x, bot.pmove_state.y - 24.0);
                            bot_actions.push((bot_id, weapon, origin, angle));
                            
                            if weapon > 0 && weapon_idx < bot.ammo.len() {
                                bot.ammo[weapon_idx] = bot.ammo[weapon_idx].saturating_sub(1);
                            }
                        }
                    }
                }
            }
        }
        
        for (bot_id, weapon, origin, angle) in bot_actions {
            self.handle_player_shoot(bot_id, weapon, origin, angle);
        }
    }
    
    fn convert_to_player_static(id: u16, sp: &ServerPlayer) -> sas::game::player::Player {
        use sas::game::weapon::Weapon;
        
        let weapon: Weapon = unsafe { std::mem::transmute(sp.weapon.min(8)) };
        
        sas::game::player::Player {
            id,
            name: sp.name.clone(),
            model: "sarge".to_string(),
            x: sp.pmove_state.x,
            y: sp.pmove_state.y,
            cx: sp.pmove_state.x,
            cy: sp.pmove_state.y,
            vel_x: sp.pmove_state.vel_x,
            vel_y: sp.pmove_state.vel_y,
            prev_x: sp.pmove_state.x,
            prev_y: sp.pmove_state.y,
            interpolation_time: 0.0,
            should_interpolate: false,
            angle: sp.angle,
            direction: if sp.angle.cos() > 0.0 { 0 } else { 1 },
            health: sp.health,
            armor: sp.armor,
            frags: sp.frags,
            deaths: sp.deaths,
            team: 0,
            dead: sp.dead,
            gibbed: false,
            is_bot: sp.is_bot,
            crouch: (sp.last_cmd.buttons & 4) != 0,
            weapon,
            ammo: [
                sp.ammo[0] as u8, sp.ammo[1] as u8, sp.ammo[2] as u8,
                sp.ammo[3] as u8, sp.ammo[4] as u8, sp.ammo[5] as u8,
                sp.ammo[6] as u8, sp.ammo[7] as u8, sp.ammo[8] as u8,
            ],
            has_weapon: [
                true, true, sp.ammo[2] > 0, sp.ammo[3] > 0,
                sp.ammo[4] > 0, false, sp.ammo[6] > 0,
                sp.ammo[7] > 0, sp.ammo[8] > 0,
            ],
            refire: 0.0,
            weapon_switch_time: 0.0,
            powerups: sas::game::player::PowerUps {
                quad: sp.powerup_quad,
                regen: 0,
                battle: 0,
                flight: 0,
                haste: 0,
                invis: 0,
            },
            animation: sas::game::animation::PlayerAnimation::new(),
            bot_ai: None,
            was_in_air: sp.pmove_state.was_in_air,
            respawn_timer: sp.respawn_timer,
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
        }
    }

    fn broadcast_game_state(&mut self) {
        let player_states: Vec<PlayerState> = self.game_state.players.iter().map(|(id, server_player)| {
            let on_ground = !server_player.pmove_state.was_in_air;
            let is_crouching = (server_player.last_cmd.buttons & 4) != 0;
            let is_attacking = false;
            
            PlayerState {
                player_id: *id,
                position: (server_player.pmove_state.x, server_player.pmove_state.y),
                velocity: (server_player.pmove_state.vel_x, server_player.pmove_state.vel_y),
                angle: server_player.angle,
                health: server_player.health,
                armor: server_player.armor,
                weapon: server_player.weapon,
                command_time: server_player.last_executed_time,
                ammo: server_player.ammo,
                frags: server_player.frags,
                deaths: server_player.deaths,
                powerup_quad: server_player.powerup_quad,
                on_ground,
                is_crouching,
                is_attacking,
                is_dead: server_player.dead,
            }
        }).collect();

        let projectile_states: Vec<sas::network::ProjectileState> = self.game_state.projectiles.iter()
            .filter(|p| p.active)
            .map(|proj| {
                let tr_type = match proj.weapon_type {
                    sas::game::weapon::Weapon::RocketLauncher => TrajectoryType::Linear,
                    sas::game::weapon::Weapon::Plasmagun => TrajectoryType::Linear,
                    sas::game::weapon::Weapon::BFG => TrajectoryType::Linear,
                    sas::game::weapon::Weapon::GrenadeLauncher => TrajectoryType::Gravity,
                    _ => TrajectoryType::Linear,
                };
                
                let trajectory = Trajectory {
                    tr_type,
                    tr_time: (get_network_time() * 1000.0) as u32,
                    tr_base_x: proj.x,
                    tr_base_y: proj.y,
                    tr_delta_x: proj.vel_x,
                    tr_delta_y: proj.vel_y,
                };
                
                sas::network::ProjectileState {
                    id: proj.id,
                    trajectory,
                    weapon_type: proj.weapon_type as u8,
                    owner_id: proj.owner_id,
                    spawn_time: (get_network_time() * 1000.0) as u32,
                }
            })
            .collect();

        static mut LAST_PROJ_PRINT: f64 = 0.0;
        unsafe {
            if get_network_time() - LAST_PROJ_PRINT > 2.0 {
                println!("[SERVER] Broadcasting {} projectiles (total active: {})", 
                    projectile_states.len(), self.game_state.projectiles.iter().filter(|p| p.active).count());
                LAST_PROJ_PRINT = get_network_time();
            }
        }

        self.server.broadcast_game_state(self.game_state.tick, player_states, projectile_states).ok();
    }
}

fn main() {
    let mut config = NetworkConfig::default();
    let mut map_name = "0-arena".to_string();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if let Ok(port) = args[1].parse::<u16>() {
            config.server_port = port;
        }
    }
    
    if args.len() > 2 {
        if let Ok(max_players) = args[2].parse::<u8>() {
            config.max_players = max_players;
        }
    }
    
    if args.len() > 3 {
        map_name = args[3].clone();
    }

    println!("=================================");
    println!("  NFK Dedicated Server");
    println!("=================================");
    println!("Port: {}", config.server_port);
    println!("Max players: {}", config.max_players);
    println!("Map: {}", map_name);
    println!("=================================");

    let mut server = DedicatedServer::new(config, map_name);
    
    if let Err(e) = server.start() {
        eprintln!("Failed to start server: {}", e);
        return;
    }

    server.run();
}
