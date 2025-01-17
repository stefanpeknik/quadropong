use std::{error::Error, vec};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect, Flex},
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

            let horizontal = Layout::horizontal([Constraint::Percentage(40)]).flex(Flex::Center);
            let vertical = Layout::vertical([Constraint::Percentage(40)]).flex(Flex::Center);
            let [area] = vertical.areas(block.inner(frame.area()));
            let [area] = horizontal.areas(area);


            frame.render_widget(Block::bordered(), area);
            let middle_middle_split = Layout::vertical(vec![
                // one more than needed for looks
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

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

            let vertical = Layout::vertical([Constraint::Percentage(40)]).flex(Flex::Center);
            let horizontal = Layout::horizontal([Constraint::Percentage(40)]).flex(Flex::Center);
            let [area] = vertical.areas(block.inner(frame.area()));
            let [middle_area] = horizontal.areas(area);

            
            // render border around menu items
            frame.render_widget(Block::bordered(), middle_area);
            let middle_split = Layout::vertical(vec![
                // one more than needed for looks
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(middle_area);

            match online_state {
                OnlineOptions::EnterCode(input) => {
                    // load input window
                    let title = Line::from(" Enter code ");
                    let block = Block::bordered()
                        .title(title.left_aligned());

                    frame.render_widget(block.clone(), middle_area);

                    let input_paragraph = Paragraph::new(input.input.clone()).block(Block::bordered().title("Input"));

                    frame.render_widget(input_paragraph, middle_split[1]);
                },
                OnlineOptions::Create => {
                    // for each menu_text render its paragraph
                    for (i, text) in menu_text.iter().enumerate() {
                        let display_text = if i == 0 {
                            format!(">  {}  <", text)
                        } else {
                            text.to_string()
                        };

                        let paragraph = Paragraph::new(Line::from(display_text).centered())
                            .block(menu_blocks[i].clone());

                        // render menu items
                        frame.render_widget(paragraph, middle_split[i + 1]);
                    }
                }
                OnlineOptions::Join => {
                    for (i, text) in menu_text.iter().enumerate() {
                        let display_text = if i == 1 {
                            format!(">  {}  <", text)
                        } else {
                            text.to_string()
                        };

                        let paragraph = Paragraph::new(Line::from(display_text).centered())
                            .block(menu_blocks[i].clone());

                        // render menu items
                        frame.render_widget(paragraph, middle_split[i + 1]);
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
        CurrentScreen::OnlineLobbyScreen => {
            let lobby_code = "123456";
            let title_text = format!(" quadropong - Online lobby {} ", lobby_code);
            let block = render_main_block(
                frame,
                title_text.as_str(),
                vec![
                    " Back ".into(),
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
                    " Back ".into(),
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
                    " Back ".into(),
                    "<ESC> ".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ],
            );
        }
    }
}
