use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Direction;

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
    pub ping_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub score: u32,
    pub addr: Option<SocketAddr>,
    pub position: Option<PlayerPosition>,
    pub paddle_position: f32,
    pub paddle_delta: f32,
    pub paddle_width: f32,
    pub is_ready: bool,
    pub is_ai: bool,
}

impl Player {
    pub fn new(name: String, is_ai: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            joined_at: chrono::Utc::now(),
            ping_timestamp: None,
            score: 0,
            addr: None,
            position: None,
            paddle_delta: 0.3,
            paddle_position: 5.0,
            paddle_width: 1.0,
            is_ready: is_ai, // AI players are always ready
            is_ai,
        }
    }

    pub fn increment_score(&mut self) {
        self.score += 1;
    }

    pub fn move_paddle(&mut self, direction: Direction) {
        let delta = match direction {
            Direction::Positive => self.paddle_delta,
            Direction::Negative => -self.paddle_delta,
        };
        self.paddle_position = (self.paddle_position + delta).clamp(
            0.0 + (self.paddle_width / 2.0),
            10.0 - (self.paddle_width / 2.0),
        );
    }
}
