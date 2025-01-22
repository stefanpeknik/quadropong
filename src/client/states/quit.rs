use super::traits::{Render, State, Update};

use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct Quit {}

impl Quit {
    pub fn new() -> Self {
        Self {}
    }
}

impl State for Quit {}

#[async_trait]
impl Update for Quit {
    async fn update(
        &mut self,
        _: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        Ok(None)
    }
}

impl Render for Quit {
    fn render(&self, _: &mut Frame) {}
}
