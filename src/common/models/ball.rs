use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PlayerPosition;

const GAME_SIZE: f32 = 10.0;

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
            velocity: Vec2 { x: 0.0, y: 0.125 },
            radius: 0.125,
            last_touched_by: None,
        }
    }

    pub fn reset(&mut self, player_positions: Vec<PlayerPosition>) {
        self.last_touched_by = None;
        self.position = Vec2 { x: 5.0, y: 5.0 };

        let initial_speed = 0.125;

        self.velocity = match player_positions.choose(&mut rand::rng()) {
            Some(PlayerPosition::Top) => Vec2 {
                x: 0.0,
                y: initial_speed,
            },
            Some(PlayerPosition::Bottom) => Vec2 {
                x: 0.0,
                y: -initial_speed,
            },
            Some(PlayerPosition::Left) => Vec2 {
                x: initial_speed,
                y: 0.0,
            },
            Some(PlayerPosition::Right) => Vec2 {
                x: -initial_speed,
                y: 0.0,
            },
            None => Vec2 {
                x: 0.0,
                y: initial_speed,
            },
        };
    }

    pub fn update_position(&mut self) {
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
    }

    pub fn is_goal(self) -> Option<PlayerPosition> {
        if self.position.x - self.radius < 0.0 {
            Some(PlayerPosition::Left)
        } else if self.position.x + self.radius > 10.0 {
            Some(PlayerPosition::Right)
        } else if self.position.y - self.radius < 0.0 {
            Some(PlayerPosition::Top)
        } else if self.position.y + self.radius > 10.0 {
            Some(PlayerPosition::Bottom)
        } else {
            None
        }
    }

    pub fn calculate_wall_reflection(&mut self, pos: PlayerPosition) {
        match pos {
            PlayerPosition::Top => {
                if self.position.y - self.radius < 0.0 {
                    self.position.y = 0.0 + self.radius;
                    self.velocity.y *= -1.0;
                }
            }
            PlayerPosition::Bottom => {
                if self.position.y + self.radius > GAME_SIZE {
                    self.position.y = 10.0 - self.radius;
                    self.velocity.y *= -1.0;
                }
            }
            PlayerPosition::Left => {
                if self.position.x - self.radius < 0.0 {
                    self.position.x = 0.0 + self.radius;
                    self.velocity.x *= -1.0;
                }
            }
            PlayerPosition::Right => {
                if self.position.x + self.radius > GAME_SIZE {
                    self.position.x = 10.0 - self.radius;
                    self.velocity.x *= -1.0;
                }
            }
        }
    }
}
