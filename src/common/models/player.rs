use std::net::SocketAddr;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Ball, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl std::fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position = match self {
            PlayerPosition::Top => "Top",
            PlayerPosition::Bottom => "Bottom",
            PlayerPosition::Left => "Left",
            PlayerPosition::Right => "Right",
        };
        write!(f, "{}", position)
    }
}

#[derive(Serialize, Clone, Deserialize, PartialEq, Debug)]
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
        let mut delta = match direction {
            Direction::Positive => self.paddle_delta,
            Direction::Negative => -self.paddle_delta,
        };

        // artificially slow down the paddle movement for AI players
        if self.is_ai {
            delta *= 0.2;
        }

        self.paddle_position = (self.paddle_position + delta).clamp(
            0.0 + (self.paddle_width / 2.0),
            10.0 - (self.paddle_width / 2.0),
        );
    }

    pub fn move_towards(&mut self, position: f32) {
        let mut target_position = position;

        if (position - self.paddle_position).abs() < self.paddle_width / 2.0 {
            let offset = rand::random::<f32>() * (self.paddle_width / 2.0);
            let mut rng = rand::rng();
            let sign = if rng.random_bool(0.5) { 1.0 } else { -1.0 };

            target_position = position + (offset * sign);
            target_position = target_position.clamp(
                self.paddle_position - self.paddle_width / 2.0,
                self.paddle_position + self.paddle_width / 2.0,
            );
        }

        if self.paddle_position > target_position {
            self.move_paddle(Direction::Negative);
        } else {
            self.move_paddle(Direction::Positive);
        }
    }

    pub fn calculate_ball_position(&self, ball: Ball, rec_step: i8) -> Option<f32> {
        if rec_step > 2 {
            return None;
        }

        let side_intersection: Option<f32> = match self.position {
            Some(PlayerPosition::Top) => {
                if ball.velocity.y >= 0.0 {
                    None
                } else {
                    let time = (0.0 - ball.position.y) / ball.velocity.y;
                    let x = ball.position.x + ball.velocity.x * time;
                    if time >= 0.0 && (0.0..=10.0).contains(&x) {
                        Some(x)
                    } else {
                        None
                    }
                }
            }
            Some(PlayerPosition::Bottom) => {
                if ball.velocity.y < 0.0 {
                    None
                } else {
                    let time = (10.0 - ball.position.y) / ball.velocity.y;
                    let x = ball.position.x + ball.velocity.x * time;
                    if time >= 0.0 && (0.0..=10.0).contains(&x) {
                        Some(x)
                    } else {
                        let time_to_wall = if ball.velocity.x < 0.0 {
                            (0.0 + ball.radius - ball.position.x) / ball.velocity.x
                        } else {
                            (10.0 - ball.radius - ball.position.x) / ball.velocity.x
                        };

                        let mut new_ball = ball.clone();
                        new_ball.position.x = if ball.velocity.x < 0.0 {
                            ball.radius
                        } else {
                            10.0 - ball.radius
                        };
                        new_ball.position.y = ball.position.y + time_to_wall * ball.velocity.y;
                        new_ball.velocity.x = -ball.velocity.x;

                        self.calculate_ball_position(new_ball, rec_step + 1)
                    }
                }
            }
            Some(PlayerPosition::Left) => {
                if ball.velocity.x >= 0.0 {
                    None
                } else {
                    let time = (0.0 - ball.position.x) / ball.velocity.x;
                    let y = ball.position.y + ball.velocity.y * time;
                    if time >= 0.0 && (0.0..=10.0).contains(&y) {
                        Some(y)
                    } else {
                        None
                    }
                }
            }
            Some(PlayerPosition::Right) => {
                if ball.velocity.x < 0.0 {
                    None
                } else {
                    let time = (10.0 - ball.position.x) / ball.velocity.x;
                    let y = ball.position.y + ball.velocity.y * time;
                    if time >= 0.0 && (0.0..=10.0).contains(&y) {
                        Some(y)
                    } else {
                        None
                    }
                }
            }
            None => None,
        };
        side_intersection
    }

    pub fn ai(&mut self, ball: Ball) {
        let side_intersection: Option<f32> = self.calculate_ball_position(ball, 1);

        match side_intersection {
            Some(x) => {
                self.move_towards(x);
            }
            None => {
                self.move_towards(5.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_paddle() {
        let mut player = Player::new("Test".to_string(), false);
        player.paddle_position = 5.0;
        player.paddle_delta = 0.5;
        player.paddle_width = 1.0;

        player.move_paddle(Direction::Positive);
        assert_eq!(player.paddle_position, 5.5);

        player.move_paddle(Direction::Negative);
        assert_eq!(player.paddle_position, 5.0);

        player.paddle_position = 0.5;
        player.move_paddle(Direction::Negative);
        assert_eq!(player.paddle_position, 0.5);

        player.paddle_position = 9.5;
        player.move_paddle(Direction::Positive);
        assert_eq!(player.paddle_position, 9.5);
    }
}
