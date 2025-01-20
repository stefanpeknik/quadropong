use super::traits::{Render, State, Update};

use crossterm::event::KeyCode;
use ratatui::Frame;

#[derive(Clone)]
pub struct Quit {}

impl Quit {
    pub fn new() -> Self {
        Self {}
    }
}

impl State for Quit {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

#[async_trait::async_trait]
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
