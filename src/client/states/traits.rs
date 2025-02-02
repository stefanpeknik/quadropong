use async_trait::async_trait;
use std::any::Any;

use crossterm::event::KeyCode;
use ratatui::Frame;

use crate::client::{config, error::ClientError};

pub trait Render {
    fn render(&self, frame: &mut Frame);
}

#[async_trait]
pub trait Update {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, ClientError>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: State> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait HasConfig {
    fn config(&self) -> config::Config;
}

pub trait State: Render + Update + Send + AsAny + HasConfig + 'static {}
