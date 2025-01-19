/// Captures the user typing input and the cursor position in the input
#[derive(Clone)]
pub struct Input {
    pub input: String,
    pub char_index: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            char_index: 0,
        }
    }

    pub fn move_left(&mut self) {
        let cursor_left = self.char_index.saturating_sub(1);
        self.char_index = cursor_left;
    }

    pub fn move_right(&mut self) {
        if self.char_index < self.input.len() {
            self.char_index = self.char_index.saturating_add(1);
        }
    }

    fn move_right_multiple(&mut self, num: usize) {
        if self.char_index < self.input.len() {
            self.char_index = self.char_index.saturating_add(num);
        }
    }

    pub fn insert_char(&mut self, new_char: char) {
        self.input.insert(self.char_index, new_char);
        self.move_right();
    }

    pub fn insert_clipboard(&mut self, new_string: String) {
        self.input.insert_str(self.char_index, &new_string);
        self.move_right_multiple(new_string.chars().count());
    }

    pub fn delete_char(&mut self) {
        if self.char_index != 0 {
            self.char_index -= 1;
            self.input.remove(self.char_index);
        }
    }
}
