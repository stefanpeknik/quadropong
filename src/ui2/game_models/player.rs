use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum PlayerPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub score: u32,
    pub addr: Option<SocketAddr>,
    pub position: Option<PlayerPosition>,
    pub paddle_position: f32,
    pub paddle_delta: f32,
    pub paddle_width: f32,
}
