use tokio::net::UdpSocket;

use crate::game_models::{client_input::ClientInput, game::Game};

#[derive(Debug, thiserror::Error)]
pub enum UdpError {}

const SERVER_ADDR: &str = "127.0.0.1:34254";

struct UdpClient {
    socket: UdpSocket,
}

// impl UdpClient {
//     async fn new() -> Result<Self, UdpError> {
//         let socket = UdpSocket::bind(SERVER_ADDR).await?;
//         Ok(Self { socket })
//     }

//     async fn send_client_input(&self, client_input: ClientInput) -> Result<(), UdpError> {
//         let serialized = serde_json::to_string(&client_input)?;
//         self.socket.send(serialized.as_bytes()).await?;
//         Ok(())
//     }

//     async fn recv_updated_game(&self) -> Result<Game, UdpError> {
//         let mut buf = [0; 1024];
//         let (len, _) = self.socket.recv_from(&mut buf).await?;
//         let response_text = std::str::from_utf8(&buf[..len])?;
//         let game: Game = serde_json::from_str(&response_text)?;
//         Ok(game)
//     }
// }
