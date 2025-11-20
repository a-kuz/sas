use super::{NetMessage, NetworkConfig, PlayerState, ProjectileState};
use super::protocol::{NetChan, NetAddr, UdpNetworking, serialize_message, deserialize_message, MAX_PACKETLEN};
use super::prediction::{CommandBuffer, UserCommand};
use super::interpolation::{SnapshotBuffer, InterpolatedPlayer, InterpolatedProjectile};
use super::client_prediction::{ClientPrediction, PredictedPlayerState};
use super::net_stats::NetStats;
use std::net::SocketAddr;
use std::io;

pub struct NetworkClient {
    config: NetworkConfig,
    connected: bool,
    player_id: Option<u16>,
    net_chan: Option<NetChan>,
    networking: UdpNetworking,
    recv_buffer: Vec<u8>,
    last_heartbeat_sent: f64,
    server_addr: Option<SocketAddr>,
    last_snapshot: Option<GameSnapshot>,
    command_buffer: CommandBuffer,
    server_time_delta: i32,
    last_snapshot_time: f64,
    snapshot_buffer: SnapshotBuffer,
    client_prediction: ClientPrediction,
    net_stats: NetStats,
    last_processed_tick: u32,
    received_snapshots: std::collections::HashMap<u32, GameSnapshot>,
    last_server_time_ms: u32,
    last_packet_sent_time: f64,
    max_packets_per_sec: u32,
    extrapolated_snapshot: bool,
    ping_samples: [u32; 16],
    ping_index: usize,
}

#[derive(Clone, Debug)]
pub struct GameSnapshot {
    pub tick: u32,
    pub players: Vec<PlayerState>,
    pub projectiles: Vec<ProjectileState>,
}

impl NetworkClient {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            connected: false,
            player_id: None,
            net_chan: None,
            networking: UdpNetworking::new(),
            recv_buffer: vec![0u8; MAX_PACKETLEN],
            last_heartbeat_sent: 0.0,
            server_addr: None,
            last_snapshot: None,
            command_buffer: CommandBuffer::new(),
            server_time_delta: 0,
            last_snapshot_time: 0.0,
            snapshot_buffer: SnapshotBuffer::new(),
            client_prediction: ClientPrediction::new(),
            net_stats: NetStats::new(),
            last_processed_tick: 0,
            received_snapshots: std::collections::HashMap::new(),
            last_server_time_ms: 0,
            last_packet_sent_time: 0.0,
            max_packets_per_sec: 60,
            extrapolated_snapshot: false,
            ping_samples: [0; 16],
            ping_index: 0,
        }
    }

    pub fn connect(&mut self, player_name: String, server_address: &str) -> Result<(), String> {
        self.networking.bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to bind client socket: {}", e))?;

        let server_addr: SocketAddr = server_address.parse()
            .map_err(|e| format!("Invalid server address: {}", e))?;
        
        self.server_addr = Some(server_addr);

        let connect_msg = NetMessage::ConnectRequest {
            player_name,
            protocol_version: self.config.protocol_version,
        };

        let data = serialize_message(&connect_msg)?;
        
        let mut send_buf = Vec::with_capacity(MAX_PACKETLEN);
        send_buf.extend_from_slice(&[0, 0, 0, 1]);
        send_buf.extend_from_slice(&[0, 0]);
        send_buf.extend_from_slice(&data);

        self.networking.send_to(&send_buf, &server_addr)
            .map_err(|e| format!("Failed to send connect request: {}", e))?;

        println!("[{}] Connecting to {}...", super::get_absolute_time(), server_address);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let (Some(player_id), Some(ref mut net_chan)) = (self.player_id, &mut self.net_chan) {
            let msg = NetMessage::Disconnect {
                player_id,
                reason: "Client disconnecting".to_string(),
            };
            if let Some(socket) = self.networking.socket() {
                if let Ok(data) = serialize_message(&msg) {
                    net_chan.transmit(socket, &data).ok();
                }
            }
        }

        self.connected = false;
        self.player_id = None;
        self.net_chan = None;
        self.server_addr = None;
        println!("[{}] Disconnected from server", super::get_absolute_time());
    }

    pub fn update(&mut self) -> Vec<NetMessage> {
        let mut messages = Vec::new();

        loop {
            let result = self.networking.recv_from(&mut self.recv_buffer);
            match result {
                Ok((size, addr)) => {
                    self.net_stats.record_incoming(size);
                    let data = self.recv_buffer[..size].to_vec();
                    if let Some(msgs) = self.process_packet(&data, addr) {
                        messages.extend(msgs);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    eprintln!("[{}] Client recv error: {}", super::get_absolute_time(), e);
                    break;
                }
            }
        }

        self.send_heartbeat();

        messages
    }

    fn process_packet(&mut self, data: &[u8], addr: SocketAddr) -> Option<Vec<NetMessage>> {
        if let Some(server_addr) = self.server_addr {
            if addr != server_addr {
                return None;
            }
        }

        if self.net_chan.is_none() {
            if let Ok(msg) = deserialize_message(&data[6..]) {
                if let NetMessage::ConnectResponse { player_id, accepted, reason } = msg {
                    if accepted {
                        let qport = (addr.port() & 0xFFFF) as u16;
                        let challenge = super::get_network_time() as i32;
                        self.net_chan = Some(NetChan::new(NetAddr::from_socket_addr(addr), qport, challenge));
                        self.player_id = Some(player_id);
                        self.connected = true;
                        println!("[{}] Connected to server as player {}", super::get_absolute_time(), player_id);
                    } else {
                        println!("[{}] Connection rejected: {}", super::get_absolute_time(), reason);
                    }
                    return Some(vec![NetMessage::ConnectResponse { player_id, accepted, reason }]);
                }
            }
            return None;
        }

        if let Some(ref mut net_chan) = self.net_chan {
            if let Some(payload) = net_chan.process_packet(data) {
                if let Ok(msg) = deserialize_message(&payload) {
                    match &msg {
                        NetMessage::GameStateSnapshot { tick, players, projectiles } => {
                            self.update_server_time(*tick);
                            self.net_stats.record_snapshot(*tick);
                            
                            let alive_players: Vec<_> = players.iter().filter(|p| !p.is_dead).cloned().collect();
                            let ts = (*tick as f64) / self.config.tick_rate.max(1) as f64;
                            self.snapshot_buffer.add_snapshot(*tick, ts, alive_players, projectiles.clone());
                            
                            let snapshot = GameSnapshot {
                                tick: *tick,
                                players: players.clone(),
                                projectiles: projectiles.clone(),
                            };
                            
                            let msg_num = self.net_chan.as_ref().map(|c| c.incoming_sequence).unwrap_or(0);
                            self.received_snapshots.insert(msg_num, snapshot.clone());
                            self.last_snapshot = Some(snapshot);
                        }
                        NetMessage::GameStateDelta { tick, base_message_num, player_deltas, projectile_deltas, new_projectiles, removed_projectiles } => {
                            // println!("[{}] *** [CLIENT] DELTA tick={} base={} with {} player_deltas ***", 
                            //     super::get_absolute_time(), tick, base_message_num, player_deltas.len());
                            
                            self.update_server_time(*tick);
                            self.net_stats.record_snapshot(*tick);
                            
                            let base_snapshot = self.received_snapshots.get(base_message_num);
                            
                            if let Some(base) = base_snapshot {
                                // println!("[{}] ***   Base had {} players, applying {} deltas ***", 
                                //     super::get_absolute_time(), base.players.len(), player_deltas.len());
                                
                                let reconstructed = self.reconstruct_delta_from_baseline(
                                    *tick,
                                    base,
                                    player_deltas,
                                    projectile_deltas,
                                    new_projectiles,
                                    removed_projectiles,
                                );
                                
                                if let Some((players, projectiles)) = reconstructed {
                                    let alive_players: Vec<_> = players.iter().filter(|p| !p.is_dead).cloned().collect();
                                    let ts = (*tick as f64) / self.config.tick_rate.max(1) as f64;
                                    self.snapshot_buffer.add_snapshot(*tick, ts, alive_players, projectiles.clone());
                                    
                                    let snapshot = GameSnapshot {
                                        tick: *tick,
                                        players,
                                        projectiles,
                                    };
                                    
                                    let msg_num = self.net_chan.as_ref().map(|c| c.incoming_sequence).unwrap_or(0);
                                    self.received_snapshots.insert(msg_num, snapshot.clone());
                                    self.last_snapshot = Some(snapshot);
                                }
                            } else {
                                println!("[{}] *** [CLIENT] No baseline for message {}, waiting for full snapshot ***", 
                                    super::get_absolute_time(), base_message_num);
                            }
                        }
                        _ => {}
                    }
                    return Some(vec![msg]);
                }
            }
        }

        None
    }

    fn send_heartbeat(&mut self) {
        let current_time = super::get_network_time();
        if current_time - self.last_heartbeat_sent > 5.0 {
            if self.connected {
                let msg = NetMessage::Heartbeat;
                self.send_message(msg).ok();
                self.last_heartbeat_sent = current_time;
            }
        }
    }

    pub fn send_message(&mut self, msg: NetMessage) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        if let Some(ref mut net_chan) = self.net_chan {
            if let Some(socket) = self.networking.socket() {
                let data = serialize_message(&msg)?;
                net_chan.transmit(socket, &data)
                    .map_err(|e| format!("Failed to send message: {}", e))?;
        Ok(())
            } else {
                Err("Socket not initialized".to_string())
            }
        } else {
            Err("Network channel not established".to_string())
        }
    }

    pub fn create_command(&mut self, move_forward: f32, move_right: f32, angle: f32, buttons: u32) {
        if self.player_id.is_some() {
            let server_time_now = self.get_server_time();
            
            let cmd = UserCommand {
                server_time: server_time_now,
                sequence: 0,
                move_forward,
                move_right,
                buttons,
                angles: angle,
            };
            
            self.command_buffer.add_command(cmd);
        }
    }
    
    pub fn flush_commands(&mut self) -> Result<(), String> {
        let now = super::get_network_time();
        let min_interval = 1.0 / self.max_packets_per_sec as f64;
        
        if now - self.last_packet_sent_time < min_interval {
            return Ok(());
        }
        
        if let Some(player_id) = self.player_id {
            let commands_to_send: Vec<_> = self.command_buffer.get_commands_since(0)
                .into_iter()
                .rev()
                .take(4)
                .rev()
                .map(|c| super::PlayerInputCmd {
                    move_forward: c.move_forward,
                    move_right: c.move_right,
                    angle: c.angles,
                    buttons: c.buttons,
                    server_time: c.server_time,
                })
                .collect();
            
            if !commands_to_send.is_empty() {
                static mut LAST_SEND_PRINT: f64 = 0.0;
                unsafe {
                    if super::get_network_time() - LAST_SEND_PRINT > 2.0 {
                        println!("[INPUT] Sending {} commands, latest right={:.1} buttons={}", 
                            commands_to_send.len(), 
                            commands_to_send.last().map(|c| c.move_right).unwrap_or(0.0),
                            commands_to_send.last().map(|c| c.buttons).unwrap_or(0));
                        LAST_SEND_PRINT = super::get_network_time();
                    }
                }
                
                let msg = NetMessage::PlayerInputBatch {
                    player_id,
                    commands: commands_to_send,
                };
                self.last_packet_sent_time = now;
                self.send_message(msg)
            } else {
                Ok(())
            }
        } else {
            Err("Not connected".to_string())
        }
    }
    
    pub fn send_input(&mut self, move_forward: f32, move_right: f32, angle: f32, buttons: u32) -> Result<(), String> {
        self.create_command(move_forward, move_right, angle, buttons);
        self.flush_commands()
    }
    
    pub fn get_command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }
    
    pub fn get_server_time(&mut self) -> u32 {
        let now = super::get_network_time();
        let realtime_ms = (now * 1000.0) as i32;
        let server_time_ms = realtime_ms.saturating_add(self.server_time_delta);
        let mut st = if server_time_ms < 0 { 0 } else { server_time_ms as u32 };
        if st < self.last_server_time_ms { st = self.last_server_time_ms; }
        self.last_server_time_ms = st;
        st
    }
    
    pub fn get_player_id(&self) -> Option<u16> {
        self.player_id
    }
    
    pub fn update_server_time(&mut self, snapshot_tick: u32) {
        let current_time = super::get_network_time();
        let realtime_ms = (current_time * 1000.0) as i32;
        let tick_rate = self.config.tick_rate.max(1) as i32;
        let snapshot_ms = ((snapshot_tick as i64) * 1000 / tick_rate as i64) as i32;
        let new_delta = snapshot_ms - realtime_ms;

        if self.last_snapshot_time <= 0.0 {
            self.server_time_delta = new_delta;
            self.last_snapshot_time = current_time;
            self.extrapolated_snapshot = false;
            return;
        }

        let ping_ms = ((current_time - self.last_snapshot_time) * 1000.0) as u32;
        self.ping_samples[self.ping_index] = ping_ms;
        self.ping_index = (self.ping_index + 1) % 16;

        let delta_delta = (new_delta - self.server_time_delta).abs();

        if delta_delta > 500 {
            self.server_time_delta = new_delta;
        } else if delta_delta > 100 {
            self.server_time_delta = (self.server_time_delta + new_delta) / 2;
        }

        self.last_snapshot_time = current_time;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn player_id(&self) -> Option<u16> {
        self.player_id
    }

    pub fn last_snapshot(&self) -> Option<&GameSnapshot> {
        self.last_snapshot.as_ref()
    }
    
    pub fn get_new_snapshot(&mut self) -> Option<GameSnapshot> {
        if let Some(ref snap) = self.last_snapshot {
            if snap.tick > self.last_processed_tick {
                self.last_processed_tick = snap.tick;
                return Some(snap.clone());
            }
        }
        None
    }
    
    pub fn interpolate_player(&self, player_id: u16, interpolation_time: f64) -> Option<InterpolatedPlayer> {
        self.snapshot_buffer.interpolate_player(player_id, interpolation_time)
    }
    
    pub fn interpolate_projectile(&self, projectile_id: u32, interpolation_time: f64) -> Option<InterpolatedProjectile> {
        self.snapshot_buffer.interpolate_projectile(projectile_id, interpolation_time)
    }
    
    pub fn get_interpolation_time(&mut self) -> f64 {
        let now = super::get_network_time();
        let realtime_ms = (now * 1000.0) as i32;
        let server_time_ms = realtime_ms.saturating_add(self.server_time_delta);
        let st = if server_time_ms < 0 { 0 } else { server_time_ms as u32 };
        
        if let Some(ref snap) = self.last_snapshot {
            let tick_rate = self.config.tick_rate.max(1) as i32;
            let snap_server_time = ((snap.tick as i64) * 1000 / tick_rate as i64) as i32;
            
            if (realtime_ms + self.server_time_delta) - snap_server_time >= -5 {
                self.extrapolated_snapshot = true;
            }
        }
        
        let auto_nudge = crate::cvar::get_cvar_float("cl_autoNudge");
        
        let interp_delay_ms = if auto_nudge > 0.0 {
            let avg_ping = self.get_average_ping();
            (avg_ping as f32 * auto_nudge).max(30.0) as u32
        } else {
            let manual_nudge = crate::cvar::get_cvar_integer("cl_timeNudge");
            if manual_nudge > 0 {
                manual_nudge as u32
            } else {
                0
            }
        };
        
        let interp_time = st.saturating_sub(interp_delay_ms);
        
        interp_time as f64 / 1000.0
    }
    
    fn get_average_ping(&self) -> u32 {
        let valid_samples: Vec<u32> = self.ping_samples.iter()
            .copied()
            .filter(|&p| p > 0 && p < 999)
            .collect();
        
        if valid_samples.is_empty() {
            return 50;
        }
        
        let mut sorted = valid_samples.clone();
        sorted.sort_unstable();
        
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2
        } else {
            sorted[mid]
        }
    }
    
    pub fn predict_local_player(
        &mut self,
        map: &crate::game::map::Map,
    ) -> Option<PredictedPlayerState> {
        let snapshot = self.last_snapshot.as_ref()?;
        let local_player_id = self.player_id?;

        let base_state_owned = snapshot.players.iter()
            .find(|p| p.player_id == local_player_id)?
            .clone();
        
        let commands = self.command_buffer.get_commands_since(0);
        let recent_cmds: Vec<_> = commands.into_iter()
            .rev()
            .take(10)
            .rev()
            .collect();
        
        if recent_cmds.is_empty() {
            return None;
        }
        
        let current_ms = self.get_server_time();
        let predicted = self.client_prediction.predict_player_movement(
            &base_state_owned,
            &recent_cmds,
            map,
            current_ms,
        );
        
        static mut LAST_PRED_PRINT: f64 = 0.0;
        unsafe {
            if super::get_network_time() - LAST_PRED_PRINT > 2.0 {
                println!("[PREDICT] base_cmd={} current={} cmds={} base_pos=({:.1},{:.1}) pred_pos=({:.1},{:.1})",
                    base_state_owned.command_time, current_ms, recent_cmds.len(),
                    base_state_owned.position.0, base_state_owned.position.1,
                    predicted.x, predicted.y);
                LAST_PRED_PRINT = super::get_network_time();
            }
        }
        
        if let Some(err) = self.client_prediction.check_prediction_error(&predicted, &base_state_owned) {
            static mut LAST_ERR_PRINT: f64 = 0.0;
            unsafe {
                if super::get_network_time() - LAST_ERR_PRINT > 2.0 {
                    println!("[PREDICT] ERROR: {:.1}px", err.magnitude);
                    LAST_ERR_PRINT = super::get_network_time();
                }
            }
        }
        
        Some(predicted)
    }
    
    pub fn get_prediction(&self) -> &ClientPrediction {
        &self.client_prediction
    }
    
    pub fn get_prediction_mut(&mut self) -> &mut ClientPrediction {
        &mut self.client_prediction
    }
    
    pub fn get_stats(&self) -> &NetStats {
        &self.net_stats
    }
    
    pub fn get_stats_mut(&mut self) -> &mut NetStats {
        &mut self.net_stats
    }
    
    fn reconstruct_delta_from_baseline(
        &self,
        _tick: u32,
        base_snapshot: &GameSnapshot,
        player_deltas: &[super::PlayerStateDelta],
        _projectile_deltas: &[super::ProjectileStateDelta],
        new_projectiles: &[ProjectileState],
        removed_projectiles: &[u32],
    ) -> Option<(Vec<PlayerState>, Vec<ProjectileState>)> {
        
        let mut players = base_snapshot.players.clone();
        
        for delta in player_deltas {
            if let Some(player) = players.iter_mut().find(|p| p.player_id == delta.player_id) {
                if let Some(cmd_time) = delta.command_time { player.command_time = cmd_time; }
                if let Some(pos) = delta.position { player.position = pos; }
                if let Some(vel) = delta.velocity { player.velocity = vel; }
                if let Some(angle) = delta.angle { player.angle = angle; }
                if let Some(health) = delta.health { player.health = health; }
                if let Some(armor) = delta.armor { player.armor = armor; }
                if let Some(weapon) = delta.weapon { player.weapon = weapon; }
                if let Some(frags) = delta.frags { player.frags = frags; }
                if let Some(deaths) = delta.deaths { player.deaths = deaths; }
                if let Some(quad) = delta.powerup_quad { player.powerup_quad = quad; }
                if let Some(on_ground) = delta.on_ground { player.on_ground = on_ground; }
                if let Some(crouching) = delta.is_crouching { player.is_crouching = crouching; }
                if let Some(attacking) = delta.is_attacking { player.is_attacking = attacking; }
                if let Some(dead) = delta.is_dead { player.is_dead = dead; }
            } else {
                let new_player = PlayerState {
                    player_id: delta.player_id,
                    position: delta.position.unwrap_or((0.0, 0.0)),
                    velocity: delta.velocity.unwrap_or((0.0, 0.0)),
                    angle: delta.angle.unwrap_or(0.0),
                    health: delta.health.unwrap_or(100),
                    armor: delta.armor.unwrap_or(0),
                    weapon: delta.weapon.unwrap_or(0),
                    command_time: delta.command_time.unwrap_or(0),
                    ammo: delta.ammo.unwrap_or([0; 10]),
                    frags: delta.frags.unwrap_or(0),
                    deaths: delta.deaths.unwrap_or(0),
                    powerup_quad: delta.powerup_quad.unwrap_or(0),
                    on_ground: delta.on_ground.unwrap_or(true),
                    is_crouching: delta.is_crouching.unwrap_or(false),
                    is_attacking: delta.is_attacking.unwrap_or(false),
                    is_dead: delta.is_dead.unwrap_or(false),
                };
                // println!("[{}] *** [CLIENT] Delta contains NEW PLAYER {} - adding! ***", 
                //     super::get_absolute_time(), delta.player_id);
                players.push(new_player);
            }
        }
        
        let mut projectiles = base_snapshot.projectiles.clone();
        let before_count = projectiles.len();
        projectiles.retain(|p| !removed_projectiles.contains(&p.id));
        let after_count = projectiles.len();
        projectiles.extend(new_projectiles.iter().cloned());
        
        if !removed_projectiles.is_empty() {
            println!("[CLIENT] Removed {} projectiles: {:?}, before={} after={}", 
                removed_projectiles.len(), removed_projectiles, before_count, after_count);
        }
        
        Some((players, projectiles))
    }
}





