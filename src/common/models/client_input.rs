use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, PartialEq, Clone)]
pub enum ClientInputType {
    JoinGame,
    PauseGame,
    ResumeGame,
    PlayerReady,
    MovePaddle(Direction),
    Disconnect,
    Ping,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Direction {
    Positive,
    Negative,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInput {
    pub game_id: String,
    pub player_id: String,
    pub action: ClientInputType,
}

impl ClientInput {
    pub fn new(game_id: String, player_id: String, action: ClientInputType) -> Self {
        Self {
            game_id,
            player_id,
            action,
        }
    }
}

pub struct ClientInputWithAddr {
    pub addr: SocketAddr,
    pub input: ClientInput,
}

#[derive(Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub username: Option<String>,
}
