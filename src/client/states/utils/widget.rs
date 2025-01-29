use arboard::Clipboard;
use crossterm::event::KeyCode;

use super::input::Input;
use super::slider::Slider;

pub enum Widget {
    Slider(Slider),
    Input(Input),
}

pub fn get_widget_text(widget: &Widget) -> String {
    match widget {
        Widget::Input(input) => input.input.clone(),
        Widget::Slider(slider) => slider.clone().get_text(),
    }
}

pub trait WidgetTrait {
    fn handle_key_event(&mut self, key: KeyCode);
}

impl WidgetTrait for Slider {
    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left => self.previous(),
            KeyCode::Right => self.next(),
            _ => (),
        }
    }
}

impl WidgetTrait for Input {
    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Tab => {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(clipboard_content) = clipboard.get_text() {
                        self.insert_clipboard(clipboard_content);
                    }
                }
            }
            _ => (),
        }
    }
}
