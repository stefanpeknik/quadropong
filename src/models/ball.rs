use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Ball {
    pub position: Vec2, // Current position (x, y)
    pub velocity: Vec2, // Current velocity vector
    pub radius: f32,
}

#[derive(Serialize, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            position: Vec2 { x: 0.0, y: 0.0 },
            velocity: Vec2 { x: 0.3, y: 0.4 },
            radius: 0.25,
        }
    }
}
