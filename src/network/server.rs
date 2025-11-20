use super::{NetMessage, NetworkConfig, PlayerState, PACKET_BACKUP};
use super::protocol::{NetChan, NetAddr, UdpNetworking, serialize_message, deserialize_message, MAX_PACKETLEN};
use super::snapshot_delta::SnapshotDelta;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::io;

pub struct NetworkServer {
    config: NetworkConfig,
    running: bool,
    clients: HashMap<u16, ClientInfo>,
    next_client_id: u16,
    networking: UdpNetworking,
    recv_buffer: Vec<u8>,
    current_tick: u32,
    last_tick_time: f64,
    delta_generator: SnapshotDelta,
    use_delta_compression: bool,
}

#[derive(Clone, Debug)]
pub struct ClientSnapshot {
    pub tick: u32,
    pub message_num: u32,
    pub players: Vec<PlayerState>,
    pub projectiles: Vec<super::ProjectileState>,
    pub sent_time: f64,
}

impl Default for ClientSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            message_num: 0,
            players: Vec::new(),
            projectiles: Vec::new(),
            sent_time: 0.0,
        }
    }
}

struct ClientInfo {
    net_chan: NetChan,
    player_name: String,
    last_heartbeat: f64,
    snapshot_history: [Option<ClientSnapshot>; PACKET_BACKUP],
    delta_message: u32,
}

impl NetworkServer {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            running: false,
            clients: HashMap::new(),
            next_client_id: 1,
            networking: UdpNetworking::new(),
            recv_buffer: vec![0u8; MAX_PACKETLEN],
            current_tick: 0,
            last_tick_time: 0.0,
            delta_generator: SnapshotDelta::new(),
            use_delta_compression: false,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let bind_addr = format!("{}:{}", self.config.server_address, self.config.server_port);
        self.networking.bind(&bind_addr)
            .map_err(|e| format!("Failed to bind to {}: {}", bind_addr, e))?;
        
        self.running = true;
        self.last_tick_time = super::get_network_time();
        
        println!("[{:.3}] Server started on {}", super::get_network_time(), bind_addr);
        Ok(())
    }

    pub fn stop(&mut self) {
        let client_ids: Vec<u16> = self.clients.keys().copied().collect();
        
        for client_id in client_ids {
            let msg = NetMessage::Disconnect {
                player_id: client_id,
                reason: "Server shutting down".to_string(),
            };
            self.send_to(client_id, msg).ok();
        }
        
        self.running = false;
        self.clients.clear();
        println!("[{:.3}] Server stopped", super::get_network_time());
    }

    pub fn update(&mut self) -> (Vec<(u16, NetMessage)>, Vec<u16>) {
        let mut messages = Vec::new();
        
        loop {
            let result = self.networking.recv_from(&mut self.recv_buffer);
            match result {
                Ok((size, addr)) => {
                    let data = self.recv_buffer[..size].to_vec();
                    if let Some(msgs) = self.process_packet(&data, addr) {
                        messages.extend(msgs);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    eprintln!("[{:.3}] Server recv error: {}", super::get_network_time(), e);
                    break;
                }
            }
        }

        let timed_out_clients = self.check_timeouts();
        self.update_tick();

        (messages, timed_out_clients)
    }

    fn process_packet(&mut self, data: &[u8], addr: SocketAddr) -> Option<Vec<(u16, NetMessage)>> {
        let client_id = self.find_client_by_addr(&addr);

        if let Some(id) = client_id {
            if let Some(client) = self.clients.get_mut(&id) {
                if let Some(payload) = client.net_chan.process_packet(data) {
                    if let Ok(msg) = deserialize_message(&payload) {
                        client.last_heartbeat = super::get_network_time();
                        return Some(vec![(id, msg)]);
                    }
                }
            }
        } else {
            if let Ok(msg) = deserialize_message(&data[6..]) {
                if let NetMessage::ConnectRequest { player_name, protocol_version } = msg {
                    return Some(self.handle_connect_request(player_name, protocol_version, addr));
                }
            }
        }

        None
    }

    fn handle_connect_request(&mut self, player_name: String, protocol_version: u32, addr: SocketAddr) -> Vec<(u16, NetMessage)> {
        if protocol_version != self.config.protocol_version {
            let response = NetMessage::ConnectResponse {
                player_id: 0,
                accepted: false,
                reason: "Protocol version mismatch".to_string(),
            };
            if let Ok(data) = serialize_message(&response) {
                self.networking.send_to(&data, &addr).ok();
            }
            return Vec::new();
        }

        if self.clients.len() >= self.config.max_players as usize {
            let response = NetMessage::ConnectResponse {
                player_id: 0,
                accepted: false,
                reason: "Server full".to_string(),
            };
            if let Ok(data) = serialize_message(&response) {
                self.networking.send_to(&data, &addr).ok();
            }
            return Vec::new();
        }

        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let qport = (addr.port() & 0xFFFF) as u16;
        let challenge = super::get_network_time() as i32;
        let net_chan = NetChan::new(NetAddr::from_socket_addr(addr), qport, challenge);

        let client_info = ClientInfo {
            net_chan,
            player_name: player_name.clone(),
            last_heartbeat: super::get_network_time(),
            snapshot_history: [None, None, None, None, None, None, None, None,
                              None, None, None, None, None, None, None, None,
                              None, None, None, None, None, None, None, None,
                              None, None, None, None, None, None, None, None],
            delta_message: 0,
        };

        self.clients.insert(client_id, client_info);

        let response = NetMessage::ConnectResponse {
            player_id: client_id,
            accepted: true,
            reason: "Welcome".to_string(),
        };

        self.send_to(client_id, response).ok();

        println!("[{:.3}] Client {} connected: {}", super::get_network_time(), client_id, player_name);

        vec![(client_id, NetMessage::ConnectRequest { player_name, protocol_version })]
    }

    fn find_client_by_addr(&self, addr: &SocketAddr) -> Option<u16> {
        for (id, client) in self.clients.iter() {
            if client.net_chan.remote_address.addr == *addr {
                return Some(*id);
            }
        }
        None
    }

    fn check_timeouts(&mut self) -> Vec<u16> {
        let current_time = super::get_network_time();
        let timeout = 30.0;
        
        let mut disconnected = Vec::new();
        
        for (id, client) in self.clients.iter() {
            if current_time - client.last_heartbeat > timeout {
                disconnected.push(*id);
            }
        }

        for id in &disconnected {
            println!("[{:.3}] Client {} timed out", super::get_network_time(), id);
            self.clients.remove(id);
        }
        
        disconnected
    }

    fn update_tick(&mut self) {
        let current_time = super::get_network_time();
        let tick_interval = 1.0 / self.config.tick_rate as f64;
        
        if current_time - self.last_tick_time >= tick_interval {
            self.current_tick += 1;
            self.last_tick_time = current_time;
        }
    }

    pub fn broadcast(&mut self, msg: NetMessage) -> Result<(), String> {
        let client_ids: Vec<u16> = self.clients.keys().copied().collect();
        
        for client_id in client_ids {
            self.send_to(client_id, msg.clone())?;
        }
        
        Ok(())
    }

    pub fn send_to(&mut self, client_id: u16, msg: NetMessage) -> Result<(), String> {
        if let Some(client) = self.clients.get_mut(&client_id) {
            if let Some(socket) = self.networking.socket() {
                let data = serialize_message(&msg)?;
                client.net_chan.transmit(socket, &data)
                    .map_err(|e| format!("Failed to send to client {}: {}", client_id, e))?;
                Ok(())
            } else {
                Err("Socket not initialized".to_string())
            }
        } else {
            Err(format!("Client {} not found", client_id))
        }
    }

    pub fn broadcast_game_state(&mut self, tick: u32, players: Vec<PlayerState>, projectiles: Vec<super::ProjectileState>) -> Result<(), String> {
        let snapshot = ClientSnapshot {
            tick,
            message_num: 0,
            players: players.clone(),
            projectiles: projectiles.clone(),
            sent_time: super::get_network_time(),
        };
        
        let client_ids: Vec<u16> = self.clients.keys().copied().collect();
        
        for client_id in client_ids {
            self.send_snapshot_to_client(client_id, &snapshot)?;
        }
        
        Ok(())
    }
    
    fn send_snapshot_to_client(&mut self, client_id: u16, snapshot: &ClientSnapshot) -> Result<(), String> {
        let client = self.clients.get(&client_id).ok_or("Client not found")?;
        
        let outgoing_seq = client.net_chan.outgoing_sequence;
        let delta_message_seq = client.delta_message;
        
        let baseline = if delta_message_seq > 0 && self.use_delta_compression {
            let index = (delta_message_seq % PACKET_BACKUP as u32) as usize;
            client.snapshot_history[index].clone()
        } else {
            None
        };
        
        let mut snapshot_with_seq = snapshot.clone();
        snapshot_with_seq.message_num = outgoing_seq;
        
        let msg = if let Some(ref base) = baseline {
            self.create_delta_message_from_baseline(base, &snapshot_with_seq, client_id)
        } else {
            NetMessage::GameStateSnapshot {
                tick: snapshot_with_seq.tick,
                players: snapshot_with_seq.players.clone(),
                projectiles: snapshot_with_seq.projectiles.clone(),
            }
        };
        
        if let Some(client) = self.clients.get_mut(&client_id) {
            let index = (outgoing_seq % PACKET_BACKUP as u32) as usize;
            client.snapshot_history[index] = Some(snapshot_with_seq);
        }
        
        self.send_to(client_id, msg)
    }
    
    fn create_delta_message_from_baseline(&self, base: &ClientSnapshot, current: &ClientSnapshot, client_id: u16) -> NetMessage {
            let mut player_deltas = Vec::new();
            for current_player in &current.players {
                if let Some(base_player) = base.players.iter().find(|p| p.player_id == current_player.player_id) {
                    let delta = self.delta_generator.compare_players(base_player, current_player);
                    player_deltas.push(delta);
                } else {
                    let delta = self.delta_generator.compare_players(
                        self.delta_generator.get_dummy_player(),
                        current_player
                    );
                    player_deltas.push(delta);
                }
            }
            
            let mut new_projectiles = Vec::new();
            let mut removed_projectiles = Vec::new();
            let mut projectile_deltas = Vec::new();
            
            for base_proj in &base.projectiles {
                if !current.projectiles.iter().any(|p| p.id == base_proj.id) {
                    removed_projectiles.push(base_proj.id);
                }
            }
            
            for current_proj in &current.projectiles {
                if let Some(base_proj) = base.projectiles.iter().find(|p| p.id == current_proj.id) {
                    let delta = self.delta_generator.compare_projectiles(base_proj, current_proj);
                    if delta.count_changed_fields() > 0 {
                        projectile_deltas.push(delta);
                    }
                } else {
                    new_projectiles.push(current_proj.clone());
                }
            }
            
        
        NetMessage::GameStateDelta {
            tick: current.tick,
            base_message_num: base.message_num,
            player_deltas,
            projectile_deltas,
            new_projectiles,
            removed_projectiles,
        }
    }
    
    pub fn update_delta_message(&mut self, client_id: u16) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.delta_message = client.net_chan.incoming_sequence;
        }
    }

    pub fn disconnect_client(&mut self, client_id: u16, reason: String) {
        let msg = NetMessage::Disconnect {
            player_id: client_id,
            reason: reason.clone(),
        };
        self.send_to(client_id, msg).ok();
        self.clients.remove(&client_id);
        println!("[{:.3}] Client {} disconnected: {}", super::get_network_time(), client_id, reason);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn get_client_name(&self, client_id: u16) -> Option<String> {
        self.clients.get(&client_id).map(|c| c.player_name.clone())
    }

    pub fn current_tick(&self) -> u32 {
        self.current_tick
    }
}

