#[derive(Clone)]
pub struct Input {
    pub input: String,
    pub char_index: usize,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let input = Input::new();
        assert_eq!(input.input, String::new());
        assert_eq!(input.char_index, 0);
    }

    #[test]
    fn test_from() {
        let input = Input::from(String::from("hello"));
        assert_eq!(input.input, "hello");
        assert_eq!(input.char_index, 5);
    }

    #[test]
    fn test_get_text() {
        let input = Input::from(String::from("hello"));
        assert_eq!(input.get_text(), "hello");
    }

    #[test]
    fn test_move_left() {
        let mut input = Input::from(String::from("hello"));
        input.move_left();
        assert_eq!(input.char_index, 4);
        input.move_left();
        assert_eq!(input.char_index, 3);
        input.move_left();
        assert_eq!(input.char_index, 2);
        input.move_left();
        assert_eq!(input.char_index, 1);
        input.move_left();
        assert_eq!(input.char_index, 0);
        input.move_left(); // Should not go below 0
        assert_eq!(input.char_index, 0);
    }

    #[test]
    fn test_move_right() {
        let mut input = Input::from(String::from("hello"));
        input.move_right();
        assert_eq!(input.char_index, 5); // Already at the end, should not move
        input.char_index = 0;
        input.move_right();
        assert_eq!(input.char_index, 1);
        input.move_right();
        assert_eq!(input.char_index, 2);
        input.move_right();
        assert_eq!(input.char_index, 3);
        input.move_right();
        assert_eq!(input.char_index, 4);
        input.move_right();
        assert_eq!(input.char_index, 5);
        input.move_right(); // Should not go beyond the length
        assert_eq!(input.char_index, 5);
    }

    #[test]
    fn test_insert_char() {
        let mut input = Input::from(String::from("hello"));
        input.char_index = 2;
        input.insert_char('x');
        assert_eq!(input.input, "hexllo");
        assert_eq!(input.char_index, 3);
    }

    #[test]
    fn test_insert_clipboard() {
        let mut input = Input::from(String::from("hello"));
        input.char_index = 2;
        input.insert_clipboard(String::from(" world "));
        assert_eq!(input.input, "heworldllo");
        assert_eq!(input.char_index, 7);
    }

    #[test]
    fn test_delete_char() {
        let mut input = Input::from(String::from("hello"));
        input.char_index = 3;
        input.delete_char();
        assert_eq!(input.input, "helo");
        assert_eq!(input.char_index, 2);
        input.delete_char();
        assert_eq!(input.input, "hlo");
        assert_eq!(input.char_index, 1);
        input.delete_char();
        assert_eq!(input.input, "lo");
        assert_eq!(input.char_index, 0);
        input.delete_char(); // Should not delete if char_index is 0
        assert_eq!(input.input, "lo");
        assert_eq!(input.char_index, 0);
    }

    #[test]
    fn test_remove_whitespace() {
        let mut input = Input::from(String::from("hello world"));
        Input::remove_whitespace(&mut input.input);
        assert_eq!(input.input, "helloworld");
    }
}
