use crate::client::config;

use super::traits::{HasConfig, Render, State, Update};

use axum::async_trait;
use crossterm::event::KeyCode;
use log::info;
use ratatui::Frame;

pub struct Quit {
    config: config::Config,
}

impl Quit {
    pub fn new(config: config::Config) -> Self {
        Self { config }
    }
}

impl State for Quit {}

impl HasConfig for Quit {
    fn config(&self) -> config::Config {
        self.config.clone()
    }
}

#[async_trait]
impl Update for Quit {
    async fn update(
        &mut self,
        _key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        info!("Quitting the game");
        Ok(None)
    }
}

impl Render for Quit {
    fn render(&self, _: &mut Frame) {}
}
