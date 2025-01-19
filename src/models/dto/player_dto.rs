use serde::Serialize;

use crate::{models::player::PlayerPosition, Player};

#[derive(Serialize, Clone)]
pub struct PlayerDto {
    pub name: String,
    pub score: u32,
    pub position: Option<PlayerPosition>,
    pub paddle_position: f32,
    pub paddle_delta: f32,
    pub paddle_width: f32,
}

impl From<Player> for PlayerDto {
    fn from(player: Player) -> Self {
        PlayerDto {
            name: player.name,
            score: player.score,
            position: player.position,
            paddle_position: player.paddle_position,
            paddle_delta: player.paddle_delta,
            paddle_width: player.paddle_width,
        }
    }
}
