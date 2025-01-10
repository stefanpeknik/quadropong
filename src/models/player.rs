use std::net::SocketAddr;

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PlayerPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Serialize, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub score: u32,
    pub addr: Option<SocketAddr>,
    pub position: Option<PlayerPosition>,
    pub paddle_position: f32, // Paddle's position along its axis (-10.0 to 10.0)
    pub paddle_delta: f32,
    pub paddle_width: f32,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name,
            score: 0,
            addr: None,
            position: None,
            paddle_delta: 0.35,
            paddle_position: 0.0,
            paddle_width: 2.0,
        }
    }

    pub fn increment_score(&mut self) {
        self.score += 1;
    }
}
