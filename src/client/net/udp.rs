use std::net::ToSocketAddrs;

use crate::common::models::{ClientInput, GameDto};

use super::error::UdpError;

#[derive(Debug)]
pub struct UdpClient {
    server_addr: std::net::SocketAddr,
    socket: tokio::net::UdpSocket,
}

impl UdpClient {
    pub fn new(server_addr: &str) -> Result<Self, UdpError> {
        let parts: Vec<&str> = server_addr.split(':').collect();
        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid address format",
            )
            .into());
        }

        let port = parts[1].parse::<u16>().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid port number")
        })?;

        // Continue with DNS resolution
        let server_addr = (parts[0], port).to_socket_addrs()?.next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "DNS resolution failed")
        })?;

        // Then create the socket
        let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            server_addr,
            socket: tokio::net::UdpSocket::from_std(socket)?,
        })
    }

    pub async fn send_client_input(&self, client_input: ClientInput) -> Result<(), UdpError> {
        let serialized = rmp_serde::to_vec(&client_input)?;
        self.socket.send_to(&serialized, &self.server_addr).await?;
        Ok(())
    }

    pub async fn recv_updated_game(&self) -> Result<GameDto, UdpError> {
        let mut buf = [0; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        if addr != self.server_addr {
            return Err(UdpError::InvalidSource);
        }
        let game: GameDto = rmp_serde::from_slice(&buf[..len])?;
        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::models::{
        BallDto, ClientInput, ClientInputType, Direction, GameDto, GameState, PlayerDto,
        PlayerPosition, Vec2,
    };
    use std::{collections::HashMap, net::SocketAddr};
    use tokio::net::UdpSocket;
    use uuid::Uuid;

    // Helper function to create test client and server
    async fn setup() -> (UdpClient, UdpSocket, SocketAddr) {
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();
        let client = UdpClient::new(&server_addr.to_string()).unwrap();
        (client, server_socket, server_addr)
    }

    #[tokio::test]
    async fn test_send_all_input_types() {
        let (client, server_socket, _) = setup().await;

        let test_cases = vec![
            ClientInputType::JoinGame,
            ClientInputType::PauseGame,
            ClientInputType::ResumeGame,
            ClientInputType::PlayerReady,
            ClientInputType::MovePaddle(Direction::Positive),
            ClientInputType::MovePaddle(Direction::Negative),
            ClientInputType::Disconnect,
            ClientInputType::Ping,
        ];

        for action in test_cases {
            let input = ClientInput {
                game_id: Uuid::new_v4().to_string(),
                player_id: Uuid::new_v4().to_string(),
                action: action.clone(),
            };

            client.send_client_input(input.clone()).await.unwrap();

            let mut buf = [0; 1024];
            let (len, _) = server_socket.recv_from(&mut buf).await.unwrap();
            let received: ClientInput = rmp_serde::from_slice(&buf[..len]).unwrap();

            assert_eq!(received.game_id, input.game_id);
            assert_eq!(received.player_id, input.player_id);
            assert_eq!(received.action, input.action);
        }
    }

    #[tokio::test]
    async fn test_receive_empty_game_state() {
        let (client, server_socket, _server_addr) = setup().await;

        // Get client address
        client
            .send_client_input(ClientInput {
                game_id: Uuid::new_v4().to_string(),
                player_id: Uuid::new_v4().to_string(),
                action: ClientInputType::Ping,
            })
            .await
            .unwrap();
        let (_, client_addr) = server_socket.recv_from(&mut [0; 1024]).await.unwrap();

        // Send empty game state
        let game_dto = GameDto {
            id: Uuid::new_v4(),
            state: GameState::WaitingForPlayers,
            players: HashMap::new(),
            ball: None,
            created_at: chrono::Utc::now(),
            started_at: None,
        };

        server_socket
            .send_to(&rmp_serde::to_vec(&game_dto).unwrap(), client_addr)
            .await
            .unwrap();

        let received = client.recv_updated_game().await.unwrap();
        assert!(received.players.is_empty());
        assert!(received.ball.is_none());
        assert_eq!(received.state, GameState::WaitingForPlayers);
    }

    #[tokio::test]
    async fn test_multiple_players_reception() {
        let (client, server_socket, _server_addr) = setup().await;
        let (_, client_addr) = get_client_addr(&client, &server_socket).await;

        let mut players = HashMap::new();
        let player_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        for (i, id) in player_ids.iter().enumerate() {
            players.insert(
                *id,
                PlayerDto {
                    id: *id,
                    name: format!("Player {}", i + 1),
                    joined_at: chrono::Utc::now(),
                    score: (i * 10) as u32,
                    position: match i {
                        0 => Some(PlayerPosition::Top),
                        1 => Some(PlayerPosition::Bottom),
                        _ => None,
                    },
                    paddle_position: 0.5,
                    paddle_delta: 0.0,
                    paddle_width: 0.2,
                    is_ready: i == 0,
                },
            );
        }

        let game_dto = GameDto {
            id: Uuid::new_v4(),
            state: GameState::Active,
            players,
            ball: Some(BallDto {
                position: Vec2 { x: 0.5, y: 0.5 },
                velocity: Vec2 { x: 0.1, y: -0.1 },
                radius: 0.05,
            }),
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
        };

        server_socket
            .send_to(&rmp_serde::to_vec(&game_dto).unwrap(), client_addr)
            .await
            .unwrap();

        let received = client.recv_updated_game().await.unwrap();
        assert_eq!(received.players.len(), 2);

        for id in &player_ids {
            let player = received.players.get(id).unwrap();
            assert!(player.name.starts_with("Player"));
            assert!(player.score <= 10);
        }
    }

    #[tokio::test]
    async fn test_invalid_packet_source() {
        let (client, _, _) = setup().await;
        let rogue_server = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send packet from rogue server
        let bad_game = GameDto {
            id: Uuid::new_v4(),
            state: GameState::Finished,
            players: HashMap::new(),
            ball: None,
            created_at: chrono::Utc::now(),
            started_at: None,
        };

        rogue_server
            .send_to(
                &rmp_serde::to_vec(&bad_game).unwrap(),
                client.socket.local_addr().unwrap(),
            )
            .await
            .unwrap();

        match client.recv_updated_game().await {
            Err(UdpError::InvalidSource) => (),
            other => panic!("Expected InvalidSource error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_malformed_packet_handling() {
        let (client, server_socket, _server_addr) = setup().await;
        let (_, client_addr) = get_client_addr(&client, &server_socket).await;

        // Send invalid data
        server_socket
            .send_to(b"invalid_messagepack_data", client_addr)
            .await
            .unwrap();

        match client.recv_updated_game().await {
            Err(UdpError::MsgPackDeserialization(_)) => (),
            other => panic!("Expected deserialization error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_client_creation_errors() {
        // Test invalid address format
        match UdpClient::new("invalid_address:1234") {
            Err(UdpError::Io(_)) => (), // We only care that it's an Io error
            other => panic!("Expected UdpError::Io, got {:?}", other),
        }

        // Test valid address but invalid port
        match UdpClient::new("127.0.0.1:99999") {
            Err(UdpError::Io(_)) => (), // We only care that it's an Io error
            other => panic!("Expected UdpError::Io, got {:?}", other),
        }
    }

    // Helper to get client address
    async fn get_client_addr(client: &UdpClient, server_socket: &UdpSocket) -> (usize, SocketAddr) {
        client
            .send_client_input(ClientInput {
                game_id: Uuid::new_v4().to_string(),
                player_id: Uuid::new_v4().to_string(),
                action: ClientInputType::Ping,
            })
            .await
            .unwrap();

        let mut buf = [0; 1024];
        server_socket.recv_from(&mut buf).await.unwrap()
    }
}
