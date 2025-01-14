use std::vec;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span, Text}, widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap}, Frame
};

use crate::ui::app::*;

fn render_main_block<'a>(
    frame: &mut Frame,
    title_text: &'a str,
    instructions_text: Vec<Span<'a>>,
) -> Block<'a> {
    let title = Line::from(title_text.bold());
    let instructions = Line::from(instructions_text);

    let block = Block::bordered()
        .title(title.left_aligned())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    frame.render_widget(block.clone(), frame.area());

    block
}

pub fn ui(frame: &mut Frame, app: &App) {
    match &app.current_screen {
        CurrentScreen::MenuScreen(menu_state) => {
            // Render main block
            let block = render_main_block(
                frame,
                " quadropong ",
                vec![
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );

            // 30|40|30
            let layout = Layout::horizontal(vec![
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(block.inner(frame.area()));

            // 30
            // --
            // 40
            // --
            // 30
            let middle_split = Layout::vertical(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]).split(layout[1]);
            frame.render_widget(Block::bordered(), middle_split[1]);

            //
            let middle_middle_split = Layout::vertical(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]).split(middle_split[1]);

            let menu_text = [Line::from("P L A Y  W I T H  F R I E N D S"), Line::from("T R A I N I N G"), Line::from("S E T T I N G S")];

            let menu_blocks = [
                Block::new(),
                Block::new(),
                Block::new(),
            ];

            // add "> " before menuitem
            // render either with "> <" or without base on menu_state
            for (i, text) in menu_text.iter().enumerate() {
                let display_text = if i == *menu_state as usize {
                    format!(">  {}  <", text)
                } else {
                    text.to_string()
                };
            
                let paragraph = Paragraph::new(Line::from(display_text).centered())
                    .block(menu_blocks[i].clone());
            
                frame.render_widget(paragraph, middle_middle_split[i + 1]);
            }
        },
        CurrentScreen::OnlineCreateScreen => {
            let _block = render_main_block(
                frame,
                " quadropong - Creating online lobby ",
                vec![
                    " Back to menu ".into(),
                    "<ESC> ".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );
        },
        CurrentScreen::TrainingCreateScreen => {
            let _block = render_main_block(
                frame,
                " quadropong - Creating training lobby ",
                vec![
                    " Back to menu ".into(),
                    "<ESC> ".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );
        },
        CurrentScreen::SettingsScreen => {
            let _block = render_main_block(
                frame,
                " quadropong - Settings ",
                vec![
                    " Back to menu ".into(),
                    "<ESC> ".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );
        },
        _ => (),
    }
}
