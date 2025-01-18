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
    let inner_rect = centered_rect(outer_rect, 40, 40);

    // Create the inner block
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    // Render the inner block
    frame.render_widget(inner_block, inner_rect);

    // Return the inner rectangle
    inner_rect
}

/// Renders a list of strings evenly distributed and centered within a rectangle.
/// The selected item is highlighted.
pub fn render_list(frame: &mut Frame, items: &[String], selected_index: usize, rect: Rect) {
    let vertical_text_spaces = Layout::vertical(
        std::iter::repeat(Constraint::Percentage(100 / (items.len() + 1) as u16))
            .take(items.len() + 1),
    )
    .split(rect);

    for (i, (text, area)) in items
        .iter()
        .zip(vertical_text_spaces.iter().skip(1)) // skip the first area as that only creates space from the top
        .enumerate()
    {
        let text = if i == selected_index {
            Line::from(format!("> {} <", text)).bold()
        } else {
            Line::from(text.as_str())
        };

        let paragraph = Paragraph::new(text).alignment(Alignment::Center);

        frame.render_widget(paragraph, *area);
    }
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
