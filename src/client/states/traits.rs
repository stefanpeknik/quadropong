use async_trait::async_trait;
use std::any::Any;

use crossterm::event::KeyCode;
use ratatui::Frame;

use crate::client::settings;

pub trait Render {
    fn render(&self, frame: &mut Frame);
}

#[async_trait]
pub trait Update {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: State> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait HasSettings {
    fn settings(&self) -> settings::Settings;
}

pub trait State: Render + Update + Send + AsAny + HasSettings + 'static {}
