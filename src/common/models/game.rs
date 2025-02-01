use chrono::{self, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::PI;
use uuid::Uuid;

use crate::common::game_error::GameError;

use super::ball::Ball;
use super::dto::GameDto;
use super::player::PlayerPosition;
use super::Player;

const MAX_ANGLE: f32 = PI / 3.0; // Maximum reflection angle (60 degrees in radians)
const BALL_SPEED: f32 = 0.15; // Constant ball speed
const PADDLE_PADDING: f32 = 0.25; // Padding around paddle to prevent collisions
const SAFE_ZONE_MARGIN: f32 = 1.0; // Multiplier for padding to define safe zone
const GAME_SIZE: f32 = 10.0;
const MAX_PLAYERS: usize = 4;
const PING_TIMEOUT: u64 = 2000;
const MAX_SCORE: u32 = 10;
const GOAL_TIMEOUT: u64 = 750;

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub enum GameState {
    WaitingForPlayers,
    Active,
    Paused,
    Finished,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub players: HashMap<Uuid, Player>,
    pub state: GameState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ball: Option<Ball>,
    pub last_goal_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            players: HashMap::new(),
            state: GameState::WaitingForPlayers,
            created_at: chrono::Utc::now(),
            started_at: None,
            ball: Some(Ball::new()),
            last_goal_at: None,
        }
    }

    pub fn to_network_bytes(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let dto = GameDto::from(self.clone());
        rmp_serde::to_vec(&dto)
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        if self.is_full() {
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
            PlayerPosition::Right,
            PlayerPosition::Left,
        ];

        all_positions
            .iter()
            .find(|&&pos| !existing_positions.contains(&pos))
            .copied()
    }

    pub fn remove_player(&mut self, id: Uuid) {
        self.players.remove(&id);
        if self.players.values().filter(|player| !player.is_ai).count() < 1 {
            self.set_game_state(GameState::Finished);
        }
    }

    pub fn set_game_state(&mut self, state: GameState) {
        self.state = state;
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= MAX_PLAYERS
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

        if self.players.values().count() < 2 {
            return Err(GameError::InvalidStateTransition);
        }

        if self.players.values().any(|player| !player.is_ready) {
            return Err(GameError::PlayersNotReady);
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

    pub fn get_player_by_side(&self, side: PlayerPosition) -> Option<&Player> {
        self.players
            .values()
            .find(|player| player.position == Some(side))
    }

    pub fn goal_action(&mut self, goal_pos: PlayerPosition) {
        if self.state != GameState::Active {
            return;
        }

        let mut last_touched: Option<Uuid> = None;

        if let Some(ref mut ball) = self.ball {
            last_touched = ball.last_touched_by;
            self.last_goal_at = Some(Utc::now());
            ball.reset(
                self.players
                    .values()
                    .map(|p| p.position.unwrap_or(PlayerPosition::Top))
                    .collect(),
            );
        }

        if let Some(id) = last_touched {
            let player = self.get_player_mut(&id);
            if let Some(player) = player {
                if player.position != Some(goal_pos) {
                    player.increment_score();
                    info!("game {}: player {} scored", self.id, id);
                }
            }
        }
    }

    pub fn check_players_health(&mut self) {
        let current_time = Utc::now();

        let players_to_remove: Vec<_> = self
            .players
            .values()
            .filter_map(|player| {
                player.ping_timestamp.and_then(|timestamp| {
                    let elapsed = current_time.signed_duration_since(timestamp);
                    (elapsed.num_milliseconds() as u64 > PING_TIMEOUT).then_some(player.id)
                })
            })
            .collect();

        for player_id in players_to_remove {
            info!("game {}: player {} timed out", self.id, player_id);
            self.remove_player(player_id);
        }
    }

    pub fn game_tick(&mut self) {
        if self.state == GameState::Finished {
            return;
        }

        self.check_players_health();

        if self.state != GameState::Active {
            return;
        }

        // create an artificial pause after the goal was scored
        if let Some(last_goal_at) = self.last_goal_at {
            let elapsed_since_goal = Utc::now().signed_duration_since(last_goal_at);
            if (elapsed_since_goal.num_milliseconds() as u64) < GOAL_TIMEOUT {
                return;
            }
        }

        if let Some(ball) = &mut self.ball {
            ball.update_position();

            self.players.values_mut().for_each(|player| {
                if player.is_ai {
                    player.ai(ball.clone());
                }
            });

            const ALL_POSITIONS: &[PlayerPosition] = &[
                PlayerPosition::Top,
                PlayerPosition::Bottom,
                PlayerPosition::Right,
                PlayerPosition::Left,
            ];

            for empty_pos in ALL_POSITIONS.iter().filter(|pos| {
                self.players
                    .values()
                    .all(|player| player.position != Some(**pos))
            }) {
                ball.calculate_wall_reflection(*empty_pos);
            }

            if let Some(goal_pos) = ball.clone().is_goal() {
                self.goal_action(goal_pos);

                if self
                    .players
                    .values()
                    .into_iter()
                    .any(|p| p.score >= MAX_SCORE)
                {
                    self.set_game_state(GameState::Finished);
                    info!("game {}: finished", self.id);
                    return;
                }
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

                            ball.last_touched_by = Some(player.id);
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

                            ball.last_touched_by = Some(player.id);
                        }
                    }
                    Some(PlayerPosition::Left) => {
                        let paddle_start = player.paddle_position - player.paddle_width / 2.0;
                        let paddle_end = player.paddle_position + player.paddle_width / 2.0;
                        let paddle_x = PADDLE_PADDING;

                        let next_ball_x = ball.position.x + ball.velocity.x;

                        // Check if the ball will collide with the paddle
                        if next_ball_x < paddle_x
                            && (ball.position.y + ball.radius) >= paddle_start
                            && (ball.position.y - ball.radius) <= paddle_end
                        {
                            let hit_offset = -((ball.position.y - player.paddle_position)
                                / (player.paddle_width / 2.0))
                                .clamp(-1.0, 1.0);

                            let angle = (PI) + hit_offset * MAX_ANGLE;

                            ball.velocity.x = -BALL_SPEED * angle.cos();
                            ball.velocity.y = BALL_SPEED * angle.sin();

                            ball.position.x = paddle_x + ball.radius;

                            ball.last_touched_by = Some(player.id);
                        }
                    }
                    Some(PlayerPosition::Right) => {
                        let paddle_start = player.paddle_position - player.paddle_width / 2.0;
                        let paddle_end = player.paddle_position + player.paddle_width / 2.0;
                        let paddle_x = GAME_SIZE - PADDLE_PADDING;

                        let next_ball_x = ball.position.x + ball.velocity.x;

                        // Check if the ball will collide with the paddle
                        if next_ball_x > paddle_x
                            && (ball.position.y + ball.radius) >= paddle_start
                            && (ball.position.y - ball.radius) <= paddle_end
                        {
                            let hit_offset = ((ball.position.y - player.paddle_position)
                                / (player.paddle_width / 2.0))
                                .clamp(-1.0, 1.0);

                            let angle = (2.0 * PI) + hit_offset * MAX_ANGLE;

                            ball.velocity.x = -BALL_SPEED * angle.cos();
                            ball.velocity.y = BALL_SPEED * angle.sin();

                            ball.position.x = paddle_x - ball.radius;

                            ball.last_touched_by = Some(player.id);
                        }
                    }
                    None => {}
                }
            }
        }
    }
}
