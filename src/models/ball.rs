use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ball {
    pub position: Vec2, // Current position (x, y)
    pub velocity: Vec2, // Current velocity vector
    pub radius: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
