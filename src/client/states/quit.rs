use crate::client::config;

use super::traits::{HasSettings, Render, State, Update};

use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct Quit {
    settings: config::Config,
}

impl Quit {
    pub fn new(settings: config::Config) -> Self {
        Self { settings }
    }
}

impl State for Quit {}

impl HasSettings for Quit {
    fn settings(&self) -> config::Config {
        self.settings.clone()
    }
}

#[async_trait]
impl Update for Quit {
    async fn update(
        &mut self,
        _key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        Ok(None)
    }
}

impl Render for Quit {
    fn render(&self, _: &mut Frame) {}
}
