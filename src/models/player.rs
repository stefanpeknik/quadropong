use std::net::SocketAddr;

use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub score: u32,
    pub addr: Option<SocketAddr>,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name,
            score: 0,
            addr: None,
        }
    }

    pub fn increment_score(&mut self) {
        self.score += 1;
    }
}
