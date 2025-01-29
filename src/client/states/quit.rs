use crate::client::settings;

use super::traits::{HasSettings, Render, State, Update};

use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct Quit {
    settings: settings::Settings,
}

impl Quit {
    pub fn new(settings: settings::Settings) -> Self {
        Self { settings }
    }
}

impl State for Quit {}

impl HasSettings for Quit {
    fn settings(&self) -> settings::Settings {
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
