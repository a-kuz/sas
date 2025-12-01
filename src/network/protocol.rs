use super::NetMessage;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::net::{SocketAddr, UdpSocket};

pub const MAX_PACKETLEN: usize = 1400;
pub const FRAGMENT_SIZE: usize = MAX_PACKETLEN - 100;
pub const PACKET_HEADER: usize = 10;
pub const FRAGMENT_BIT: u32 = 1u32 << 31;

#[derive(Clone, Debug)]
pub struct NetAddr {
    pub addr: SocketAddr,
}

impl NetAddr {
    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

pub struct NetChan {
    pub remote_address: NetAddr,
    pub qport: u16,
    pub incoming_sequence: u32,
    pub outgoing_sequence: u32,
    pub challenge: i32,
    pub unsent_fragments: bool,
    pub unsent_buffer: Vec<u8>,
    pub unsent_length: usize,
    pub unsent_fragment_start: usize,
    pub last_sent_time: f64,
    pub last_sent_size: usize,
    pub dropped: u32,
    fragment_sequence: u32,
    fragment_length: usize,
    fragment_buffer: Vec<u8>,
}

impl NetChan {
    pub fn new(remote_address: NetAddr, qport: u16, challenge: i32) -> Self {
        Self {
            remote_address,
            qport,
            incoming_sequence: 0,
            outgoing_sequence: 1,
            challenge,
            unsent_fragments: false,
            unsent_buffer: vec![0u8; MAX_PACKETLEN * 4],
            unsent_length: 0,
            unsent_fragment_start: 0,
            last_sent_time: 0.0,
            last_sent_size: 0,
            dropped: 0,
            fragment_sequence: 0,
            fragment_length: 0,
            fragment_buffer: vec![0u8; MAX_PACKETLEN * 4],
        }
    }

    pub fn transmit(&mut self, socket: &UdpSocket, data: &[u8]) -> io::Result<()> {
        if data.len() >= FRAGMENT_SIZE {
            self.unsent_fragments = true;
            self.unsent_length = data.len();
            self.unsent_buffer[..data.len()].copy_from_slice(data);
            self.unsent_fragment_start = 0;
            return self.transmit_next_fragment(socket);
        }

        let mut send_buf = Vec::with_capacity(MAX_PACKETLEN);
        send_buf.write_u32::<LittleEndian>(self.outgoing_sequence)?;
        send_buf.write_u16::<LittleEndian>(self.qport)?;
        send_buf.extend_from_slice(data);

        socket.send_to(&send_buf, self.remote_address.addr)?;

        self.last_sent_time = super::get_network_time();
        self.last_sent_size = send_buf.len();
        self.outgoing_sequence += 1;

        Ok(())
    }

    pub fn transmit_next_fragment(&mut self, socket: &UdpSocket) -> io::Result<()> {
        let mut send_buf = Vec::with_capacity(MAX_PACKETLEN);

        let outgoing_sequence = self.outgoing_sequence | FRAGMENT_BIT;
        send_buf.write_u32::<LittleEndian>(outgoing_sequence)?;
        send_buf.write_u16::<LittleEndian>(self.qport)?;

        let fragment_length = if self.unsent_fragment_start + FRAGMENT_SIZE > self.unsent_length {
            self.unsent_length - self.unsent_fragment_start
        } else {
            FRAGMENT_SIZE
        };

        send_buf.write_u16::<LittleEndian>(self.unsent_fragment_start as u16)?;
        send_buf.write_u16::<LittleEndian>(fragment_length as u16)?;
        send_buf.extend_from_slice(
            &self.unsent_buffer
                [self.unsent_fragment_start..self.unsent_fragment_start + fragment_length],
        );

        socket.send_to(&send_buf, self.remote_address.addr)?;

        self.last_sent_time = super::get_network_time();
        self.last_sent_size = send_buf.len();

        self.unsent_fragment_start += fragment_length;

        if self.unsent_fragment_start == self.unsent_length && fragment_length != FRAGMENT_SIZE {
            self.outgoing_sequence += 1;
            self.unsent_fragments = false;
        }

        Ok(())
    }

    pub fn process_packet(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 6 {
            return None;
        }

        let mut cursor = io::Cursor::new(data);
        let mut sequence = cursor.read_u32::<LittleEndian>().ok()?;
        let _qport = cursor.read_u16::<LittleEndian>().ok()?;

        let is_fragment = sequence & FRAGMENT_BIT != 0;
        if is_fragment {
            sequence &= !FRAGMENT_BIT;
        }

        if sequence <= self.incoming_sequence {
            eprintln!(
                "[{:.3}] Out of order packet {} at {}",
                super::get_network_time(),
                sequence,
                self.incoming_sequence
            );
            return None;
        }

        self.dropped = sequence - (self.incoming_sequence + 1);
        if self.dropped > 0 {
            eprintln!(
                "[{:.3}] Dropped {} packets at {}",
                super::get_network_time(),
                self.dropped,
                sequence
            );
        }

        if is_fragment {
            let fragment_start = cursor.read_u16::<LittleEndian>().ok()? as usize;
            let fragment_length = cursor.read_u16::<LittleEndian>().ok()? as usize;

            if sequence != self.fragment_sequence {
                self.fragment_sequence = sequence;
                self.fragment_length = 0;
            }

            if fragment_start != self.fragment_length {
                eprintln!(
                    "[{:.3}] Missed fragment at {}, expected {}",
                    super::get_network_time(),
                    fragment_start,
                    self.fragment_length
                );
                return None;
            }

            let position = cursor.position() as usize;
            if position + fragment_length > data.len()
                || self.fragment_length + fragment_length > self.fragment_buffer.len()
            {
                return None;
            }

            let fragment_data = &data[position..position + fragment_length];
            self.fragment_buffer[self.fragment_length..self.fragment_length + fragment_length]
                .copy_from_slice(fragment_data);
            self.fragment_length += fragment_length;

            if fragment_length == FRAGMENT_SIZE {
                return None;
            }

            let complete_data = self.fragment_buffer[..self.fragment_length].to_vec();
            self.fragment_length = 0;
            self.incoming_sequence = sequence;

            return Some(complete_data);
        }

        self.incoming_sequence = sequence;
        Some(data[6..].to_vec())
    }
}

pub struct UdpNetworking {
    socket: Option<UdpSocket>,
}

impl UdpNetworking {
    pub fn new() -> Self {
        Self { socket: None }
    }

    pub fn bind(&mut self, addr: &str) -> io::Result<()> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        self.socket = Some(socket);
        Ok(())
    }

    pub fn send_to(&self, data: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        if let Some(ref socket) = self.socket {
            socket.send_to(data, addr)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Socket not bound",
            ))
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        if let Some(ref socket) = self.socket {
            socket.recv_from(buf)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Socket not bound",
            ))
        }
    }

    pub fn socket(&self) -> Option<&UdpSocket> {
        self.socket.as_ref()
    }
}

pub fn serialize_message(msg: &NetMessage) -> Result<Vec<u8>, String> {
    bincode::serialize(msg).map_err(|e| e.to_string())
}

pub fn deserialize_message(data: &[u8]) -> Result<NetMessage, String> {
    bincode::deserialize(data).map_err(|e| e.to_string())
}
