use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::{models::player::PlayerPosition, Player};

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerDto {
    pub id: Uuid,
    pub name: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub score: u32,
    pub position: Option<PlayerPosition>,
    pub paddle_position: f32,
    pub paddle_delta: f32,
    pub paddle_width: f32,
    pub is_ready: bool,
}

impl From<Player> for PlayerDto {
    fn from(player: Player) -> Self {
        PlayerDto {
            id: player.id,
            name: player.name,
            joined_at: player.joined_at,
            score: player.score,
            position: player.position,
            paddle_position: player.paddle_position,
            paddle_delta: player.paddle_delta,
            paddle_width: player.paddle_width,
            is_ready: player.is_ready,
        }
    }
}
