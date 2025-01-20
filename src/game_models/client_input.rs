use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientInputType {
    JoinGame,
    LeaveGame,
    PauseGame,
    ResumeGame,
    StartGame,
    MovePaddle(Direction),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Positive,
    Negative,
}

#[derive(Serialize, Deserialize, Debug)]
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
