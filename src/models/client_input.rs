use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientInputType {
    JoinGame,
    LeaveGame,
    PauseGame,
    ResumeGame,
    StartGame,
    MovePaddle(Direction),
}

#[derive(Deserialize, Debug)]
pub enum Direction {
    Positive,
    Negative,
}

#[derive(Deserialize, Debug)]
pub struct ClientInput {
    pub game_id: String,
    pub player_id: String,
    pub action: ClientInputType,
}

pub struct ClientInputWithAddr {
    pub addr: SocketAddr,
    pub input: ClientInput,
}
