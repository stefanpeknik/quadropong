use chrono;
use serde::Serialize;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

use crate::GameError;

use super::ball::Ball;
use super::player::PlayerPosition;
use super::Player;

const MAX_ANGLE: f32 = PI / 4.0; // Maximum reflection angle (45 degrees in radians)
const BALL_SPEED: f32 = 0.15; // Constant ball speed
const PADDLE_PADDING: f32 = 0.5; // Padding around paddle to prevent collisions
const SAFE_ZONE_MARGIN: f32 = 1.5; // Multiplier for padding to define safe zone
const GAME_SIZE: f32 = 10.0; // Since it's a square

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum GameState {
    WaitingForPlayers,
    Active,
    Paused,
    Finished,
}

#[derive(Serialize, Clone)]
pub struct Game {
    pub id: Uuid,
    pub players: HashMap<Uuid, Player>,
    pub state: GameState,
    pub max_players: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ball: Option<Ball>,
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Game ID: {}", self.id)?;
        writeln!(f, "State: {:?}", self.state)?;
        writeln!(f, "Players ({}):", self.players.len())?;
        for (_, player) in &self.players {
            writeln!(
                f,
                "  - {} (ID: {}), Score: {}",
                player.name, player.id, player.score
            )?;
        }
        Ok(())
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            players: HashMap::new(),
            state: GameState::WaitingForPlayers,
            max_players: 4,
            created_at: chrono::Utc::now(),
            started_at: None,
            ball: Some(Ball::new()),
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        if self.players.len() >= self.max_players {
            return Err(GameError::GameFull);
        }
        self.players.insert(player.id, player);
        Ok(())
    }

    pub fn assign_position(&self) -> Option<PlayerPosition> {
        let existing_positions: Vec<PlayerPosition> = self
            .players
            .values()
            .filter_map(|player| player.position)
            .collect();

        let all_positions = [
            PlayerPosition::Top,
            PlayerPosition::Bottom,
            PlayerPosition::Left,
            PlayerPosition::Right,
        ];

        all_positions
            .iter()
            .find(|&&pos| !existing_positions.contains(&pos))
            .copied()
    }

    pub fn remove_player(&mut self, id: Uuid) {
        self.players.remove(&id);
    }

    pub fn set_game_state(&mut self, state: GameState) {
        self.state = state;
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    pub fn get_player(&self, id: &Uuid) -> Option<&Player> {
        self.players.get(id)
    }

    pub fn get_player_mut(&mut self, id: &Uuid) -> Option<&mut Player> {
        self.players.get_mut(id)
    }

    pub fn start_game(&mut self) -> Result<(), GameError> {
        if self.state != GameState::WaitingForPlayers {
            return Err(GameError::InvalidStateTransition);
        }

        if self.started_at.is_some() {
            return Err(GameError::InvalidStateTransition);
        }

        let joined_players = self
            .players
            .values() // Iterate over the players
            .filter(|player| player.addr.is_some()) // Filter players with `addr` assigned
            .count();

        if self.players.len() < self.max_players || joined_players < self.max_players {
            return Err(GameError::InvalidStateTransition);
        }

        self.started_at = Some(chrono::Utc::now());
        self.state = GameState::Active;
        Ok(())
    }

    pub fn pause_game(&mut self) -> Result<(), GameError> {
        if self.state != GameState::Active {
            return Err(GameError::InvalidStateTransition);
        }

        self.state = GameState::Paused;
        Ok(())
    }

    pub fn update_ball_position(&mut self) {
        if let Some(ball) = &mut self.ball {
            ball.position.x += ball.velocity.x;
            ball.position.y += ball.velocity.y;

            if ball.position.x - ball.radius < 0.0 {
                ball.position.x = 0.0 + ball.radius;
                ball.velocity.x *= -1.0;
            } else if ball.position.x + ball.radius > 10.0 {
                ball.position.x = 10.0 - ball.radius;
                ball.velocity.x *= -1.0;
            }

            if ball.position.y - ball.radius < 0.0 {
                ball.position.y = 0.0 + ball.radius;
                ball.velocity.y *= -1.0;
            } else if ball.position.y + ball.radius > 10.0 {
                ball.position.y = 10.0 - ball.radius;
                ball.velocity.y *= -1.0;
            }
        }

        self.check_collision();
    }

    pub fn is_ball_in_safe_zone(ball: &Ball, paddle_padding: f32) -> bool {
        let safe_distance = paddle_padding * SAFE_ZONE_MARGIN;

        ball.position.x > safe_distance
            && ball.position.x < (GAME_SIZE - safe_distance)
            && ball.position.y > safe_distance
            && ball.position.y < (GAME_SIZE - safe_distance)
    }

    pub fn check_collision(&mut self) {
        if let Some(ball) = &mut self.ball {
            // check if we need to check collision
            if Game::is_ball_in_safe_zone(ball, PADDLE_PADDING) {
                return;
            }
            for player in self.players.values_mut() {
                match player.position {
                    Some(PlayerPosition::Top) => {
                        let paddle_start = player.paddle_position - player.paddle_width / 2.0;
                        let paddle_end = player.paddle_position + player.paddle_width / 2.0;
                        let paddle_y = PADDLE_PADDING;

                        let next_ball_y = ball.position.y + ball.velocity.y;

                        // Check if the ball will collide with the paddle
                        if next_ball_y < paddle_y
                            && (ball.position.x + ball.radius) >= paddle_start
                            && (ball.position.x - ball.radius) <= paddle_end
                        {
                            let hit_offset = ((ball.position.x - player.paddle_position)
                                / (player.paddle_width / 2.0))
                                .clamp(-1.0, 1.0);

                            let angle = (3.0 * PI / 2.0) + hit_offset * MAX_ANGLE;

                            // Update the ball's velocity based on the reflection angle
                            ball.velocity.x = BALL_SPEED * angle.cos();
                            ball.velocity.y = -BALL_SPEED * angle.sin();

                            ball.position.y = paddle_y + ball.radius;

                            // Increment the player's score
                            player.score += 1;
                        }
                    }
                    Some(PlayerPosition::Bottom) => {
                        let paddle_start = player.paddle_position - player.paddle_width / 2.0;
                        let paddle_end = player.paddle_position + player.paddle_width / 2.0;
                        let paddle_y = GAME_SIZE - PADDLE_PADDING;

                        let next_ball_y = ball.position.y + ball.velocity.y;

                        // Check if the ball will collide with the paddle
                        if next_ball_y > paddle_y
                            && (ball.position.x + ball.radius) >= paddle_start
                            && (ball.position.x - ball.radius) <= paddle_end
                        {
                            let hit_offset = -((ball.position.x - player.paddle_position)
                                / (player.paddle_width / 2.0))
                                .clamp(-1.0, 1.0);

                            let angle = (PI / 2.0) + hit_offset * MAX_ANGLE;

                            ball.velocity.x = BALL_SPEED * angle.cos();
                            ball.velocity.y = -BALL_SPEED * angle.sin();

                            ball.position.y = paddle_y - ball.radius;

                            // Increment the player's score
                            player.score += 1;
                        }
                    }
                    _ => {} // Handle other paddle positions (Left, Right, Bottom) if needed
                }
            }
        }
    }
}
