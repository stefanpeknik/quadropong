use std::{error::Error, vec};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
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

fn get_rect_from_percentage(
    direction: Direction,
    percentages: Vec<u16>,
    area: Rect,
    rect_num: usize,
) -> Option<Rect> {
    let layout = Layout::new(direction, Constraint::from_percentages(percentages)).split(area);

    if rect_num < layout.len() {
        Some(layout[rect_num])
    } else {
        None
    }
}

pub fn ui(frame: &mut Frame, app: &App) {
    match &app.current_screen {
        CurrentScreen::MenuScreen(menu_state) => {
            // Render main block
            let block = render_main_block(
                frame,
                " quadropong ",
                vec![" Quit ".into(), "<Q> ".blue().bold()],
            );

            let menu_text = [
                Line::from("P L A Y  W I T H  F R I E N D S"),
                Line::from("T R A I N I N G"),
                Line::from("S E T T I N G S"),
            ];

            let menu_blocks = [Block::new(), Block::new(), Block::new()];

            // get middle of 30 | *40* | 30 horizontaly
            if let Some(horiz_middle) = get_rect_from_percentage(
                Direction::Horizontal,
                vec![30, 40, 30],
                block.inner(frame.area()),
                1,
            ) {
                // get middle of 30 | *40* | 30 verticaly
                if let Some(vert_middle) =
                    get_rect_from_percentage(Direction::Vertical, vec![30, 40, 30], horiz_middle, 1)
                {
                    // render border around menu items
                    frame.render_widget(Block::bordered(), vert_middle);
                    let middle_middle_split = Layout::vertical(vec![
                        // one more than needed for looks
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ])
                    .split(vert_middle);

                    // for each menu_text render its paragraph
                    for (i, text) in menu_text.iter().enumerate() {
                        let display_text = if i == *menu_state as usize {
                            format!(">  {}  <", text)
                        } else {
                            text.to_string()
                        };

                        let paragraph = Paragraph::new(Line::from(display_text).centered())
                            .block(menu_blocks[i].clone());

                        // render menu items
                        frame.render_widget(paragraph, middle_middle_split[i + 1]);
                    }
                }
            }
        }
        CurrentScreen::OnlineScreen(online_state) => {
            let block = render_main_block(
                frame,
                " quadropong - Online lobby ",
                vec![
                    " Back to menu ".into(),
                    "<ESC> ".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );

            let menu_text = [
                Line::from("C R E A T E  N E W  G A M E"),
                Line::from("J O I N  G A M E"),
            ];

            let menu_blocks = [Block::new(), Block::new()];

            // get middle of 30 | *40* | 30 horizontaly
            if let Some(horiz_middle) = get_rect_from_percentage(
                Direction::Horizontal,
                vec![30, 40, 30],
                block.inner(frame.area()),
                1,
            ) {
                // get middle of 30 | *40* | 30 verticaly
                if let Some(vert_middle) =
                    get_rect_from_percentage(Direction::Vertical, vec![30, 40, 30], horiz_middle, 1)
                {
                    // render border around menu items
                    frame.render_widget(Block::bordered(), vert_middle);
                    let middle_middle_split = Layout::vertical(vec![
                        // one more than needed for looks
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ])
                    .split(vert_middle);

                    // for each menu_text render its paragraph
                    for (i, text) in menu_text.iter().enumerate() {
                        let display_text = if i == *online_state as usize {
                            format!(">  {}  <", text)
                        } else {
                            text.to_string()
                        };

                        let paragraph = Paragraph::new(Line::from(display_text).centered())
                            .block(menu_blocks[i].clone());

                        // render menu items
                        frame.render_widget(paragraph, middle_middle_split[i + 1]);
                    }
                }
            }
        }
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
        }
        CurrentScreen::OnlineJoinScreen => {
            let _block = render_main_block(
                frame,
                " quadropong - Join online lobby ",
                vec![
                    " Back to menu ".into(),
                    "<ESC> ".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );
        }
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
        }
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
        }
    }
}
