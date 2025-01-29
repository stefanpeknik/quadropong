use std::net::UdpSocket;
use std::time::Duration;

use crate::common::models::{ClientInput, GameDto};

use super::error::UdpError;

pub struct UdpClient {
    server_addr: String,
    socket: UdpSocket,
}

impl UdpClient {
    pub fn new(server_addr: &str) -> Result<Self, UdpError> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self {
            server_addr: server_addr.to_string(),
            socket,
        })
    }

    pub fn send_client_input(&self, client_input: ClientInput) -> Result<(), UdpError> {
        let serialized = rmp_serde::to_vec(&client_input)?;
        self.socket.send_to(&serialized, &self.server_addr)?;
        Ok(())
    }

    pub fn recv_updated_game(&self) -> Result<GameDto, UdpError> {
        let mut buf = [0; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf)?;
        if addr.to_string() != self.server_addr {
            return Err(UdpError::InvalidSource);
        }
        let game: GameDto = rmp_serde::from_slice(&buf[..len])?;
        Ok(game)
    }
}
