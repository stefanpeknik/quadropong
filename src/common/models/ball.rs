use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct Ball {
    pub position: Vec2, // Current position (x, y)
    pub velocity: Vec2, // Current velocity vector
    pub radius: f32,
    pub last_touched_by: Option<Uuid>,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            position: Vec2 { x: 5.0, y: 5.0 },
            velocity: Vec2 { x: 0.075, y: 0.1 },
            radius: 0.125,
            last_touched_by: None,
        }
    }

    pub fn reset(&mut self) {
        self.last_touched_by = None;
        self.position = Vec2 { x: 5.0, y: 5.0 };

        self.velocity = Vec2 { x: 0.0, y: 0.125 };
    }
}
