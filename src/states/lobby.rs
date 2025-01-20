use crate::game_models::game::Game;
use crate::net::tcp::get_game;

use super::menu::Menu;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::render::{draw_inner_rectangle, draw_outer_rectangle, render_list};

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;
use uuid::Uuid;

#[derive(Clone)]
pub enum Options {
    TODO,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::TODO]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::TODO => write!(f, "TODO"),
        }
    }
}

#[derive(Clone)]
pub struct Lobby {
    options: Vec<Options>,
    selected: usize,
    game: Game,
    our_player_id: Uuid,
}

impl Lobby {
    pub fn new(game: Game, our_player_id: Uuid) -> Self {
        Self {
            options: Options::list(),
            selected: 0,
            game,
            our_player_id,
        }
    }
}

impl HasOptions for Lobby {
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

impl State for Lobby {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

#[async_trait::async_trait]
impl Update for Lobby {
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
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Enter => match self.options[self.selected] {
                    // TODO: Implement this
                    _ => {}
                },
                KeyCode::Esc => {
                    return Ok(Some(Box::new(Menu::new(0))));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for Lobby {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = draw_outer_rectangle(
            frame,
            " quadropong ",
            vec![" Back ".into(), " <Esc> ".blue().bold()],
        );

        let inner_rect = draw_inner_rectangle(frame, outer_rect);

        let mut list = vec![self.game.id.to_string()];
        let mut players: Vec<_> = self
            .game
            .players
            .iter()
            .map(|(p_id, p)| format!("{}: {}", p_id, p.name))
            .collect();
        players.sort();
        list.extend(players);
        list.push(format!("You: {}", self.our_player_id));

        render_list(frame, &list, self.selected, inner_rect);
    }
}
