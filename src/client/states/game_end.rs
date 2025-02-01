use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;

use crate::{
    client::config,
    common::models::{GameDto, PlayerDto},
};

use super::{
    menu::Menu,
    traits::{HasConfig, Render, State, Update},
    utils::render::{render_inner_rectangle, render_outer_rectangle},
};

pub struct GameEnd {
    game: GameDto,
    our_player_id: Uuid,
    config: config::Config,
}

impl GameEnd {
    pub fn new(game: GameDto, our_player_id: Uuid, config: config::Config) -> Self {
        Self {
            game,
            our_player_id,
            config,
        }
    }
}

impl State for GameEnd {}

impl HasConfig for GameEnd {
    fn config(&self) -> config::Config {
        self.config.clone()
    }
}

#[async_trait]
impl Update for GameEnd {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Enter => {
                    log::info!("Moving from GameEnd to Menu");
                    return Ok(Some(Box::new(Menu::new(0, self.config.clone()))));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for GameEnd {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong - Game End ",
            vec![" Press Enter to return to the main menu ".into()],
        );

        let inner = render_inner_rectangle(frame, outer_rect);

        // Sort players by score (assuming PlayerDto has a `score` field)
        let mut players: Vec<&PlayerDto> = self.game.players.values().collect();
        players.sort_by(|a, b| b.score.cmp(&a.score)); // Sort in descending order

        // Define podium heights
        let podium_heights = [inner.height / 2, inner.height / 3, inner.height / 4];
        let podium_width = inner.width / 5; // Adjust width to fit all podiums

        // Create a layout for the podiums and the 4th player message
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),    // Podiums
                    Constraint::Length(1), // Space for the 4th player message
                ]
                .as_ref(),
            )
            .split(inner);

        // Create a layout for the podiums with 1st place centered
        let podium_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(25),       // Left spacing
                    Constraint::Length(podium_width), // 2nd place
                    Constraint::Length(podium_width), // 1st place
                    Constraint::Length(podium_width), // 3rd place
                    Constraint::Percentage(25),       // Right spacing
                ]
                .as_ref(),
            )
            .split(main_layout[0]); // Use the top part of the main layout

        // Draw each podium
        for (i, player) in players.iter().take(3).enumerate() {
            // Determine the podium position based on rank
            let rect = match i {
                0 => podium_layout[2], // 1st place (center)
                1 => podium_layout[1], // 2nd place (left)
                2 => podium_layout[3], // 3rd place (right)
                _ => unreachable!(),
            };

            let podium_rect = Rect::new(
                rect.x,
                main_layout[0].y + main_layout[0].height - podium_heights[i],
                podium_width,
                podium_heights[i],
            );

            // Draw the podium block
            let podium_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Gray));
            frame.render_widget(podium_block, podium_rect);

            // Draw the player name and crown (if 1st place)
            let name_paragraph = Paragraph::new(if i == 0 {
                // For 1st place, render the crown on top of the name
                vec![
                    Line::from(Span::styled("👑", Style::default().fg(Color::Yellow))),
                    Line::from(Span::styled(
                        player.name.clone(),
                        Style::default().fg(Color::White),
                    )),
                ]
            } else {
                // For 2nd and 3rd, just render the name
                vec![Line::from(Span::styled(
                    player.name.clone(),
                    Style::default().fg(Color::White),
                ))]
            })
            .alignment(ratatui::layout::Alignment::Center); // Center the text

            frame.render_widget(
                name_paragraph,
                Rect::new(
                    podium_rect.x,
                    podium_rect.y - 2, // Adjust for the crown and name
                    podium_rect.width,
                    2, // Height for crown and name
                ),
            );

            // Draw the podium number (1st, 2nd, 3rd)
            let number_paragraph = Paragraph::new(Line::from(vec![Span::styled(
                match i {
                    0 => "1st",
                    1 => "2nd",
                    2 => "3rd",
                    _ => "",
                }
                .to_string(),
                Style::default().fg(Color::Yellow),
            )]));
            frame.render_widget(
                number_paragraph,
                Rect::new(
                    podium_rect.x + podium_rect.width / 2 - 1,
                    podium_rect.y + podium_rect.height / 2,
                    3,
                    1,
                ),
            );
        }

        // If there is a 4th player, display them below the standings
        if players.len() > 3 {
            let fourth_player = &players[3];
            let humiliation_text = format!(
                " {} came in 4th... better luck next time! ",
                fourth_player.name
            );

            let humiliation_paragraph = Paragraph::new(Line::from(vec![Span::styled(
                humiliation_text,
                Style::default().fg(Color::Red),
            )]))
            .alignment(ratatui::layout::Alignment::Center); // Center the text

            frame.render_widget(
                humiliation_paragraph,
                main_layout[1], // Use the bottom part of the main layout
            );
        }
    }
}
