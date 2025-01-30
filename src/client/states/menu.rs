use super::create_or_join_lobby::CreateOrJoinLobby;
use super::quit::Quit;
use super::settings::Settings;
use super::training::Training;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::render::{
    into_title, render_inner_rectangle, render_list, render_outer_rectangle,
};
use crate::client::config;

use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::Frame;

pub enum Options {
    Online,
    Training,
    Settings,
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Online => write!(f, " {} ", into_title("play with friends")),
            Options::Training => write!(f, " {} ", into_title("training")),
            Options::Settings => write!(f, " {} ", into_title("settings")),
        }
    }
}

pub struct Menu {
    options: Vec<Options>,
    selected: usize,
    config: config::Config,
}

impl Menu {
    pub fn new(selected: usize, config: config::Config) -> Self {
        Self {
            options: vec![Options::Online, Options::Training, Options::Settings],
            selected,
            config,
        }
    }

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

impl HasConfig for Menu {
    fn config(&self) -> config::Config {
        self.config.clone()
    }
}

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
                        return Ok(Some(Box::new(CreateOrJoinLobby::new(self.config.clone()))));
                    }
                    Options::Training => {
                        return Ok(Some(Box::new(Training::new(self.config.clone()))))
                    }
                    Options::Settings => {
                        return Ok(Some(Box::new(Settings::new(self.config.clone()))));
                    }
                },
                KeyCode::Char('q') => {
                    return Ok(Some(Box::new(Quit::new(self.config.clone()))));
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
                "| Up".into(),
                " <\u{2191}> ".light_blue().into(),
                "| Down".into(),
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
