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

    pub fn from(string: String) -> Self {
        Self {
            input: string.clone(),
            char_index: string.len(),
        }
    }

    pub fn get_text(self) -> String {
        self.input
    }

    fn remove_whitespace(s: &mut String) {
        s.retain(|c| !c.is_whitespace());
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

    pub fn insert_clipboard(&mut self, mut new_string: String) {
        Self::remove_whitespace(&mut new_string);
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
