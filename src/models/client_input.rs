use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum ClientInputType {
    JoinGame,
    LeaveGame,
    PauseGame,
    ResumeGame,
}

#[derive(Deserialize, Debug)]
pub struct ClientInput {
    pub game_id: String,
    pub player_id: String,
    pub action: ClientInputType,
}
