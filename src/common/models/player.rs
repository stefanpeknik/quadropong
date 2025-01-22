use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub score: u32,
    pub addr: Option<SocketAddr>,
    pub position: Option<PlayerPosition>,
    pub paddle_position: f32,
    pub paddle_delta: f32,
    pub paddle_width: f32,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            joined_at: chrono::Utc::now(),
            score: 0,
            addr: None,
            position: None,
            paddle_delta: 0.15,
            paddle_position: 5.0,
            paddle_width: 1.0,
        }
    }

    pub fn increment_score(&mut self) {
        self.score += 1;
    }
}
