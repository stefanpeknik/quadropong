use std::rc::Rc;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Position, Rect},
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

/// Draws the outer rectangle, renders it, and returns its Rect
pub fn draw_outer_rectangle(
    frame: &mut Frame,
    title_text: &str,
    instructions_text: Vec<Span>,
) -> Rect {
    // Create the outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title(Line::from(title_text.bold()).left_aligned())
        .title_bottom(Line::from(instructions_text).centered());

    // Render the outer block
    frame.render_widget(outer_block, frame.area());

    // Return the outer rectangle (entire frame area)
    frame.area()
}

/// Draws the inner rectangle, renders it, and returns its Rect
pub fn draw_inner_rectangle(frame: &mut Frame, outer_rect: Rect) -> Rect {
    // Calculate the inner rectangle with 40% spacing on each side
    let inner_rect = centered_rect(outer_rect, 60, 60);

    // Create the inner block
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    // Render the inner block
    frame.render_widget(inner_block, inner_rect);

    // Return the inner rectangle
    inner_rect
}

/// Helper function to calculate evenly distributed rectangles within a given rectangle
pub fn evenly_distanced_rects(rect: Rect, num_rects: usize) -> Rc<[Rect]> {
    let vertical = Layout::vertical(
        std::iter::repeat(Constraint::Percentage(100 / num_rects as u16)).take(num_rects),
    )
    .split(rect);

    vertical
}

/// Renders a list of strings evenly distributed and centered within a rectangle.
/// The selected item is highlighted.
pub fn render_list(frame: &mut Frame, items: &[String], selected_index: usize, rect: Rect) {
    let vertical_text_spaces = evenly_distanced_rects(rect, items.len());

    for (i, (text, area)) in items
        .iter()
        .zip(vertical_text_spaces.iter()) // skip the first area as that only creates space from the top
        .enumerate()
    {
        let text = if i == selected_index {
            Line::from(format!("> {} <", text)).bold()
        } else {
            Line::from(text.as_str())
        };

        render_text_in_center_of_rect(frame, Paragraph::new(text), *area);
    }
}

pub fn render_text_in_center_of_rect(frame: &mut Frame, text: Paragraph, rect: Rect) {
    frame.render_widget(text.centered(), evenly_distanced_rects(rect, 2)[1]);
}

/// Helper function to calculate a centered rectangle with given horizontal and vertical percentages
fn centered_rect(area: Rect, horizontal_percent: u16, vertical_percent: u16) -> Rect {
    let horizontal =
        Layout::horizontal([Constraint::Percentage(horizontal_percent)]).flex(Flex::Center);
    let vertical = Layout::vertical([Constraint::Percentage(vertical_percent)]).flex(Flex::Center);
    let [rect] = vertical.areas(area);
    let [rect] = horizontal.areas(rect);

    rect
}

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
