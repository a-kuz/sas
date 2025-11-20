use std::collections::VecDeque;

const PING_HISTORY_SIZE: usize = 32;
const RATE_CALC_WINDOW: f64 = 1.0;

#[derive(Clone, Debug)]
pub struct NetStats {
    pub ping: u32,
    pub packet_loss: f32,
    pub incoming_rate: u32,
    pub outgoing_rate: u32,
    pub snapshot_rate: u32,
    pub prediction_errors: u32,
    pub extrapolations: u32,
    pub interpolation_buffer_ms: f32,
    
    ping_samples: VecDeque<(f64, u32)>,
    incoming_bytes: VecDeque<(f64, usize)>,
    outgoing_bytes: VecDeque<(f64, usize)>,
    snapshots_received: VecDeque<(f64, u32)>,
    packets_sent: u32,
    packets_received: u32,
    packets_lost: u32,
    last_update_time: f64,
}

impl NetStats {
    pub fn new() -> Self {
        Self {
            ping: 0,
            packet_loss: 0.0,
            incoming_rate: 0,
            outgoing_rate: 0,
            snapshot_rate: 0,
            prediction_errors: 0,
            extrapolations: 0,
            interpolation_buffer_ms: 0.0,
            ping_samples: VecDeque::with_capacity(PING_HISTORY_SIZE),
            incoming_bytes: VecDeque::new(),
            outgoing_bytes: VecDeque::new(),
            snapshots_received: VecDeque::new(),
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            last_update_time: 0.0,
        }
    }
    
    pub fn record_ping(&mut self, ping_ms: u32) {
        let current_time = super::get_network_time();
        
        self.ping_samples.push_back((current_time, ping_ms));
        
        while self.ping_samples.len() > PING_HISTORY_SIZE {
            self.ping_samples.pop_front();
        }
        
        if !self.ping_samples.is_empty() {
            let sum: u32 = self.ping_samples.iter().map(|(_, p)| p).sum();
            self.ping = sum / self.ping_samples.len() as u32;
        }
    }
    
    pub fn record_incoming(&mut self, bytes: usize) {
        let current_time = super::get_network_time();
        self.incoming_bytes.push_back((current_time, bytes));
        self.packets_received += 1;
        
        self.incoming_bytes.retain(|(t, _)| current_time - t < RATE_CALC_WINDOW);
        
        let total_bytes: usize = self.incoming_bytes.iter().map(|(_, b)| b).sum();
        let time_span = if self.incoming_bytes.len() > 1 {
            self.incoming_bytes.back().unwrap().0 - self.incoming_bytes.front().unwrap().0
        } else {
            1.0
        };
        
        if time_span > 0.0 {
            self.incoming_rate = (total_bytes as f64 / time_span) as u32;
        }
    }
    
    pub fn record_outgoing(&mut self, bytes: usize) {
        let current_time = super::get_network_time();
        self.outgoing_bytes.push_back((current_time, bytes));
        self.packets_sent += 1;
        
        self.outgoing_bytes.retain(|(t, _)| current_time - t < RATE_CALC_WINDOW);
        
        let total_bytes: usize = self.outgoing_bytes.iter().map(|(_, b)| b).sum();
        let time_span = if self.outgoing_bytes.len() > 1 {
            self.outgoing_bytes.back().unwrap().0 - self.outgoing_bytes.front().unwrap().0
        } else {
            1.0
        };
        
        if time_span > 0.0 {
            self.outgoing_rate = (total_bytes as f64 / time_span) as u32;
        }
    }
    
    pub fn record_snapshot(&mut self, tick: u32) {
        let current_time = super::get_network_time();
        self.snapshots_received.push_back((current_time, tick));
        
        self.snapshots_received.retain(|(t, _)| current_time - t < RATE_CALC_WINDOW);
        
        if self.snapshots_received.len() > 1 {
            let time_span = self.snapshots_received.back().unwrap().0 
                - self.snapshots_received.front().unwrap().0;
            if time_span > 0.0 {
                self.snapshot_rate = ((self.snapshots_received.len() - 1) as f64 / time_span) as u32;
            }
        }
    }
    
    pub fn record_packet_loss(&mut self, lost: u32) {
        self.packets_lost += lost;
        
        let total_packets = self.packets_sent + self.packets_received;
        if total_packets > 0 {
            self.packet_loss = (self.packets_lost as f32 / total_packets as f32) * 100.0;
        }
    }
    
    pub fn record_prediction_error(&mut self) {
        self.prediction_errors += 1;
    }
    
    pub fn record_extrapolation(&mut self) {
        self.extrapolations += 1;
    }
    
    pub fn update(&mut self) {
        let current_time = super::get_network_time();
        
        if current_time - self.last_update_time > 1.0 {
            self.last_update_time = current_time;
        }
    }
    
    pub fn get_summary(&self) -> String {
        format!(
            "Ping: {}ms | Loss: {:.1}% | In: {} B/s | Out: {} B/s | Snaps: {}/s | Errors: {} | Extrap: {}",
            self.ping,
            self.packet_loss,
            self.incoming_rate,
            self.outgoing_rate,
            self.snapshot_rate,
            self.prediction_errors,
            self.extrapolations,
        )
    }
    
    pub fn get_ping_graph_data(&self) -> Vec<u32> {
        self.ping_samples.iter().map(|(_, p)| *p).collect()
    }
    
    pub fn reset_frame_counters(&mut self) {
        self.prediction_errors = 0;
        self.extrapolations = 0;
    }
}

impl Default for NetStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ping_averaging() {
        let mut stats = NetStats::new();
        
        stats.record_ping(50);
        stats.record_ping(60);
        stats.record_ping(70);
        
        assert_eq!(stats.ping, 60);
    }
    
    #[test]
    fn test_rate_calculation() {
        let mut stats = NetStats::new();
        
        stats.record_incoming(1000);
        std::thread::sleep(std::time::Duration::from_millis(100));
        stats.record_incoming(1000);
        
        assert!(stats.incoming_rate > 0);
    }
    
    #[test]
    fn test_packet_loss() {
        let mut stats = NetStats::new();
        
        stats.packets_sent = 100;
        stats.packets_received = 90;
        stats.record_packet_loss(10);
        
        assert!(stats.packet_loss > 5.0 && stats.packet_loss < 15.0);
    }
}











