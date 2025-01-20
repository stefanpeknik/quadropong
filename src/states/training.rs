use super::menu::Menu;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::render::{render_inner_rectangle, render_outer_rectangle, render_list};

use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;

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
pub struct Training {
    options: Vec<Options>,
    selected: usize,
}

impl Training {
    pub fn new() -> Self {
        Self {
            options: Options::list(),
            selected: 0,
        }
    }
}

impl HasOptions for Training {
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

impl State for Training {}

#[async_trait]
impl Update for Training {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Enter => match self.options[self.selected] {
                    // TODO: Implement this
                    _ => {}
                },
                KeyCode::Esc => {
                    return Ok(Some(Box::new(Menu::new(1))));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for Training {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong ",
            vec![" Back ".into(), " <Esc> ".blue().bold()],
        );

        let inner_rect = render_inner_rectangle(frame, outer_rect);

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
