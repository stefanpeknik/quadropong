use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::ball::Ball;
use super::player::Player;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameState {
    WaitingForPlayers,
    Active,
    Paused,
    Finished,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    pub id: Uuid,
    pub players: HashMap<Uuid, Player>,
    pub state: GameState,
    pub max_players: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ball: Option<Ball>,
}
