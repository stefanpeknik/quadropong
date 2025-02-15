use serde::{Deserialize, Serialize};

use crate::common::models::{ball::Vec2, Ball};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BallDto {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
}

impl From<Ball> for BallDto {
    fn from(ball: Ball) -> Self {
        BallDto {
            position: ball.position,
            velocity: ball.velocity,
            radius: ball.radius,
        }
    }
}
