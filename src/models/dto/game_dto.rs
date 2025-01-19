use chrono;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::{Ball, GameState},
    Game,
};

use super::PlayerDto;

#[derive(Serialize, Clone)]
pub struct GameDto {
    pub id: Uuid,
    pub state: GameState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ball: Option<Ball>,
    pub players: HashMap<Uuid, PlayerDto>,
}

impl From<Game> for GameDto {
    fn from(game: Game) -> Self {
        GameDto {
            id: game.id,
            state: game.state,
            created_at: game.created_at,
            started_at: game.started_at,
            ball: game.ball,
            players: game
                .players
                .into_iter()
                .map(|(id, player)| (id, PlayerDto::from(player)))
                .collect(),
        }
    }
}
