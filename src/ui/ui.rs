use std::vec;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span, Text}, widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap}, Frame
};

use crate::ui::app::*;

pub fn ui(frame: &mut Frame, app: &App) {
    match app.current_screen {
        CurrentScreen::Menu => {
            let title = Line::from(" quadropong ".bold());

            let instructions = Line::from(vec![
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ]);

            let block = Block::bordered()
                .title(title.left_aligned())
                .title_bottom(instructions.centered())
                .border_set(border::THICK);

            frame.render_widget(block.clone(), frame.area());

            let layout = Layout::horizontal(vec![
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(block.clone().inner(frame.area()));

            // LEFT SETTING
            // let middle_split = Layout::vertical(vec![
            //     Constraint::Length(2),
            //     Constraint::Length(2),
            //     Constraint::Length(2),
            //     Constraint::Length(2),
            // ]).split(layout[0]);
            // frame.render_widget(Block::bordered(), layout[0]);

            // CENTER SETTING
            let middle_middle_split = Layout::vertical(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]).split(layout[1]);

            let middle_split = Layout::vertical(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]).split(middle_middle_split[1]);
            frame.render_widget(Block::bordered(), middle_middle_split[1]);

            let menu_text = [Line::from("P L A Y  W I T H  F R I E N D S"), Line::from("T R A I N I N G"), Line::from("S E T T I N G S")];

            let online_block = Block::new();
            let offline_block = Block::new();
            let settings_block = Block::new();

            let online_paragraph = Paragraph::new(
                menu_text[0].clone().centered()
            ).block(online_block.clone());
            let offline_paragraph = Paragraph::new(
                menu_text[1].clone().centered()
            ).block(offline_block.clone());
            let settings_paragraph = Paragraph::new(
                menu_text[2].clone().centered()
            ).block(settings_block.clone());

            frame.render_widget(online_paragraph, middle_split[1]);
            frame.render_widget(offline_paragraph, middle_split[2]);
            frame.render_widget(settings_paragraph, middle_split[3]);
        },
        CurrentScreen::OfflineCreate => {

        },
        CurrentScreen::OnlineCreate => {

        },
        CurrentScreen::Settings => {

        },
        _ => (),
    }
}
