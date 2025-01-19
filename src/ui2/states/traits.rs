use crossterm::event::KeyCode;
use ratatui::Frame;

pub trait HasOptions {
    fn next(&mut self);

    fn previous(&mut self);
}

pub trait Render {
    fn render(&self, frame: &mut Frame);
}

pub trait Update {
    fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error>;
}

pub trait ListEnum {
    fn list() -> Vec<Self>
    where
        Self: std::marker::Sized;
}

pub trait State: Render + Update + Send {
    // Add a method to clone the trait object
    fn clone_box(&self) -> Box<dyn State>;
}

// Implement `Clone` for `Box<dyn State>`
impl Clone for Box<dyn State> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
