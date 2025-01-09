use chrono;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

use crate::GameError;

use super::ball::Ball;
use super::player::PlayerPosition;
use super::Player;

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
        println!("Updating ball position");
        if let Some(ball) = &mut self.ball {
            ball.position.x += ball.velocity.x;
            ball.position.y += ball.velocity.y;

            if ball.position.x - ball.radius < -10.0 {
                ball.position.x = -10.0 + ball.radius;
                ball.velocity.x *= -1.0;
            } else if ball.position.x + ball.radius > 10.0 {
                // Ball hits the right wall
                ball.position.x = 10.0 - ball.radius;
                ball.velocity.x *= -1.0;
            }

            // Check for collisions with the top and bottom walls
            if ball.position.y - ball.radius < -10.0 {
                // Ball hits the top wall
                ball.position.y = -10.0 + ball.radius; // Move ball back inside the boundary
                ball.velocity.y *= -1.0; // Reverse vertical velocity
            } else if ball.position.y + ball.radius > 10.0 {
                // Ball hits the bottom wall
                ball.position.y = 10.0 - ball.radius; // Move ball back inside the boundary
                ball.velocity.y *= -1.0; // Reverse vertical velocity
            }
        }
    }
}
