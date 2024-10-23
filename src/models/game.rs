use chrono;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

use crate::GameError;

use super::Player;

#[derive(Debug, Serialize, Clone)]
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
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        if self.players.len() >= self.max_players {
            return Err(GameError::GameFull);
        }
        self.players.insert(player.id, player);
        Ok(())
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
}
