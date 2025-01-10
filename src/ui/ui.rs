use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    symbols::border,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::*;

pub fn ui(frame: &mut Frame, app: &App) {
    let title = Line::from(" quadropong ".bold());

    let instructions = Line::from(vec![
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    frame.render_widget(block, frame.area());
}
