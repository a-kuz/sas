use sas::network::{NetworkConfig, NetworkServer, NetMessage, PlayerState};
use sas::game::map::Map;
use sas::game::bg_pmove::{PmoveState, PmoveCmd, pmove};
use sas::game::usercmd::UserCmd;
use std::collections::HashMap;
use std::process::Command;

pub fn kill_process_on_port(port: u16) {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("lsof")
            .args(&["-ti", &format!(":{}", port)])
            .output();
        
        if let Ok(output) = output {
            if !output.stdout.is_empty() {
                let pids = String::from_utf8_lossy(&output.stdout);
                for pid in pids.lines() {
                    if let Ok(pid_num) = pid.trim().parse::<u32>() {
                        println!("[CLEANUP] Killing process {} on port {}", pid_num, port);
                        Command::new("kill")
                            .args(&["-9", &pid_num.to_string()])
                            .output()
                            .ok();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("fuser")
            .args(&["-k", "-TERM", &format!("{}/tcp", port)])
            .output();
        
        if output.is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netstat")
            .args(&["-ano"])
            .output();
        
        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(pid) = parts.last() {
                        println!("[CLEANUP] Killing process {} on port {}", pid, port);
                        Command::new("taskkill")
                            .args(&["/PID", pid, "/F"])
                            .output()
                            .ok();
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub struct TestServer {
    pub server: NetworkServer,
    pub players: HashMap<u16, ServerPlayer>,
    pub map: Map,
    pub tick: u32,
}

pub struct ServerPlayer {
    pub pmove_state: PmoveState,
    pub angle: f32,
    pub health: i32,
    pub armor: i32,
    pub weapon: u8,
    pub ammo: [u16; 10],
    pub frags: i32,
    pub deaths: i32,
    pub powerup_quad: u16,
    pub last_cmd: UserCmd,
    pub last_executed_time: u32,
}

impl TestServer {
    pub fn new(port: u16) -> Self {
        kill_process_on_port(port);
        
        let mut config = NetworkConfig::default();
        config.server_port = port;
        
        let map = Map::load_from_file("0-arena")
            .unwrap_or_else(|_| {
                let mut map = Map::new("test_map");
                map.spawn_points.push(sas::game::map::SpawnPoint { x: 100.0, y: 100.0, team: 0 });
                map.spawn_points.push(sas::game::map::SpawnPoint { x: 200.0, y: 200.0, team: 0 });
                map
            });

        Self {
            server: NetworkServer::new(config),
            players: HashMap::new(),
            map,
            tick: 0,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.server.start()
    }

    pub fn update(&mut self) {
        let (messages, _timeouts) = self.server.update();
        
        for (client_id, msg) in messages {
            self.handle_message(client_id, msg);
        }

        self.tick += 1;

        if self.tick % 2 == 0 {
            self.broadcast_state();
        }
    }

    fn handle_message(&mut self, client_id: u16, msg: NetMessage) {
        self.server.update_delta_message(client_id);
        
        match msg {
            NetMessage::ConnectRequest { player_name, .. } => {
                println!("[SERVER] Client {} connecting: {}", client_id, player_name);
                self.add_player(client_id);
            }
            NetMessage::Disconnect { .. } => {
                println!("[SERVER] Client {} disconnected", client_id);
                self.players.remove(&client_id);
            }
            NetMessage::PlayerInput { move_forward, move_right, angle, buttons, server_time, .. } => {
                self.update_player_input(client_id, move_forward, move_right, angle, buttons, server_time);
            }
            NetMessage::PlayerInputBatch { commands, .. } => {
                if let Some(last_cmd) = commands.last() {
                    self.update_player_input(client_id, 0.0, last_cmd.move_right, last_cmd.angle, last_cmd.buttons, last_cmd.server_time);
                }
            }
            _ => {}
        }
    }

    fn add_player(&mut self, client_id: u16) {
        let spawn_idx = (client_id as usize) % self.map.spawn_points.len().max(1);
        let spawn = &self.map.spawn_points[spawn_idx];
        
        let player = ServerPlayer {
            pmove_state: PmoveState {
                x: spawn.x,
                y: spawn.y,
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
            last_executed_time: 0,
        };

        self.players.insert(client_id, player);

        let map_msg = NetMessage::MapChange {
            map_name: "test_map".to_string(),
        };
        self.server.send_to(client_id, map_msg).ok();

        let respawn_msg = NetMessage::PlayerRespawn {
            player_id: client_id,
            position: (spawn.x, spawn.y),
        };
        self.server.broadcast(respawn_msg).ok();
    }

    fn update_player_input(&mut self, client_id: u16, _move_forward: f32, move_right: f32, angle: f32, buttons: u32, server_time: u32) {
        if let Some(player) = self.players.get_mut(&client_id) {
            player.angle = angle;
            player.last_cmd = UserCmd {
                right: move_right,
                buttons: buttons as u8,
                angles: (angle, 0.0),
                server_time,
            };
        }
    }

    pub fn simulate_physics(&mut self, dt: f32) {
        let current_tick = self.tick;
        let tick_rate = 60;
        let current_server_time = ((current_tick as u64) * 1000 / tick_rate) as u32;
        
        for (_, player) in self.players.iter_mut() {
            player.last_executed_time = current_server_time;
            
            let cmd = player.last_cmd;
            
            let pmove_cmd = PmoveCmd {
                move_right: cmd.right,
                jump: (cmd.buttons & 2) != 0,
                crouch: (cmd.buttons & 4) != 0,
                haste_active: false,
            };
            
            let result = pmove(&player.pmove_state, &pmove_cmd, dt, &self.map);
            
            player.pmove_state.x = result.new_x;
            player.pmove_state.y = result.new_y;
            player.pmove_state.vel_x = result.new_vel_x;
            player.pmove_state.vel_y = result.new_vel_y;
            player.pmove_state.was_in_air = result.new_was_in_air;
        }
    }

    fn broadcast_state(&mut self) {
        let player_states: Vec<PlayerState> = self.players.iter().map(|(id, p)| {
            PlayerState {
                player_id: *id,
                position: (p.pmove_state.x, p.pmove_state.y),
                velocity: (p.pmove_state.vel_x, p.pmove_state.vel_y),
                angle: p.angle,
                health: p.health,
                armor: p.armor,
                weapon: p.weapon,
                command_time: p.last_executed_time,
                ammo: p.ammo,
                frags: p.frags,
                deaths: p.deaths,
                powerup_quad: p.powerup_quad,
                on_ground: !p.pmove_state.was_in_air,
                is_crouching: (p.last_cmd.buttons & 4) != 0,
                is_attacking: false,
                is_dead: false,
            }
        }).collect();

        self.server.broadcast_game_state(self.tick, player_states, vec![]).ok();
    }

    pub fn get_player_position(&self, client_id: u16) -> Option<(f32, f32)> {
        self.players.get(&client_id).map(|p| (p.pmove_state.x, p.pmove_state.y))
    }
}

