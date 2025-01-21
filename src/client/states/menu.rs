use super::create_or_join_lobby::CreateOrJoinLobby;
use super::quit::Quit;
use super::settings::Settings;
use super::training::Training;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::render::{render_inner_rectangle, render_list, render_outer_rectangle};

use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;

#[derive(Clone)]
pub enum Options {
    Online,
    Training,
    Settings,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::Online, Options::Training, Options::Settings]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Online => write!(f, "P L A Y  W I T H  F R I E N D S"),
            Options::Training => write!(f, "T R A I N I N G"),
            Options::Settings => write!(f, "S E T T I N G S"),
        }
    }
}

#[derive(Clone)]
pub struct Menu {
    options: Vec<Options>,
    selected: usize,
}

impl Menu {
    pub fn new(selected: usize) -> Self {
        Self {
            options: Options::list(),
            selected,
        }
    }
}

impl HasOptions for Menu {
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

impl State for Menu {}

#[async_trait]
impl Update for Menu {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Enter => match self.options[self.selected] {
                    Options::Online => {
                        return Ok(Some(Box::new(CreateOrJoinLobby::new())));
                    }
                    Options::Training => return Ok(Some(Box::new(Training::new()))),
                    Options::Settings => {
                        return Ok(Some(Box::new(Settings::new())));
                    }
                },
                KeyCode::Char('q') => {
                    return Ok(Some(Box::new(Quit::new())));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for Menu {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong ",
            vec![
                " Quit".into(),
                " <Q> ".light_blue().bold(),
                " Up".into(),
                " <\u{2191}> ".light_blue().into(),
                " Down".into(),
                " <\u{2193}> ".light_blue().into(),
            ],
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
