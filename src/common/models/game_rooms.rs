use std::collections::HashMap;
use uuid::Uuid;

use super::Game;

pub struct GameRooms {
    pub lobbies: HashMap<Uuid, Game>,
}

impl GameRooms {
    pub fn new() -> Self {
        Self {
            lobbies: HashMap::new(),
        }
    }

    pub fn create_game(&mut self) -> Uuid {
        let game = Game::new();
        let game_id = game.id;
        self.lobbies.insert(game_id, game);

        game_id
    }

    pub fn find_lobby_mut(&mut self, id: Uuid) -> Option<&mut Game> {
        self.lobbies.get_mut(&id)
    }

    pub fn find_lobby(&mut self, id: Uuid) -> Option<&Game> {
        self.lobbies.get(&id)
    }
}
