use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::{draw_inner_rectangle, draw_outer_rectangle, render_list};

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;

#[derive(Clone)]
pub enum Options {
    Create,
    Join,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::Create, Options::Join]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Create => write!(f, "Create Lobby"),
            Options::Join => write!(f, "Join Lobby"),
        }
    }
}

#[derive(Clone)]
pub struct CreateOrJoinLobby {
    options: Vec<Options>,
    selected: usize,
}

impl CreateOrJoinLobby {
    pub fn new() -> Self {
        Self {
            options: Options::list(),
            selected: 0,
        }
    }
}

impl HasOptions for CreateOrJoinLobby {
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

impl State for CreateOrJoinLobby {}

impl Update for CreateOrJoinLobby {
    fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Up | KeyCode::Char('w') => self.previous(),
                KeyCode::Down | KeyCode::Char('s') => self.next(),
                KeyCode::Enter => {
                    // TODO
                }
                KeyCode::Char('q') => {
                    return Ok(None);
                }
                _ => {}
            };
        }
        Ok(Some(Box::new(self.clone())))
    }
}

impl Render for CreateOrJoinLobby {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = draw_outer_rectangle(
            frame,
            " quadropong ",
            vec![" Quit ".into(), " <Q> ".blue().bold()],
        );

        let inner_rect = draw_inner_rectangle(frame, outer_rect);

        render_list(
            frame,
            &self
                .options
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            self.selected,
            inner_rect,
        );
    }
}
