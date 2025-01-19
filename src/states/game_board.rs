use crate::game_models::game::Game;
use crate::net::tcp::get_game;

use super::menu::Menu;
use super::traits::{Render, State, Update};
use super::utils::render::{draw_inner_rectangle, draw_outer_rectangle, render_list};

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;
use uuid::Uuid;

#[derive(Clone)]
pub struct GameBoard {
    game: Game,
    our_player_id: Uuid,
}

impl GameBoard {
    pub fn new(game: Game, our_player_id: Uuid) -> Self {
        Self {
            game,
            our_player_id,
        }
    }
}

impl State for GameBoard {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

#[async_trait::async_trait]
impl Update for GameBoard {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        match get_game(self.game.id).await {
            Ok(game) => {
                self.game = game;
            }
            Err(e) => {
                // TODO: Handle this error
            }
        }
        if let Some(key_code) = key_code {
            match key_code {
                // TODO: Handle key presses
                _ => {}
            };
        }
        Ok(Some(Box::new(self.clone())))
    }
}

impl Render for GameBoard {
    fn render(&self, frame: &mut Frame) {}
}
