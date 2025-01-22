use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::Frame;
use uuid::Uuid;

use crate::common::models::GameDto;

use super::{
    menu::Menu,
    traits::{Render, State, Update},
    utils::render::{render_inner_rectangle, render_list, render_outer_rectangle},
};

pub struct GameEnd {
    game: GameDto,
    our_player_id: Uuid,
}

impl GameEnd {
    pub fn new(game: GameDto, our_player_id: Uuid) -> Self {
        Self {
            game,
            our_player_id,
        }
    }
}

impl State for GameEnd {}

#[async_trait]
impl Update for GameEnd {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Enter => {
                    return Ok(Some(Box::new(Menu::new(0))));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for GameEnd {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong - Game End ",
            vec![" Press Enter to return to the main menu ".into()],
        );

        let inner = render_inner_rectangle(frame, outer_rect);

        render_list(
            frame,
            &[
                self.game.id.to_string(),
                format!(
                    "Your Score: {}",
                    self.game.players[&self.our_player_id].score
                ),
                format!("Game Over - Press <Enter> to return back to main Menu..."),
            ],
            2,
            inner,
        );
    }
}
