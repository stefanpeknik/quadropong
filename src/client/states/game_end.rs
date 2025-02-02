use axum::async_trait;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use uuid::Uuid;

use crate::{
    client::{config, net::tcp::TcpClient, states::lobby::Lobby},
    common::models::{GameDto, PlayerDto},
};

use super::{
    menu::Menu,
    traits::{HasConfig, Render, State, Update},
    utils::render::render_outer_rectangle,
};

pub struct GameEnd {
    game: GameDto,
    our_player_id: Uuid,
    config: config::Config,
    tcp_client: TcpClient,
    error_message: Option<String>,
}

impl GameEnd {
    pub fn new(game: GameDto, our_player_id: Uuid, config: config::Config) -> Self {
        Self {
            game,
            our_player_id,
            tcp_client: TcpClient::new(&config.api_url),
            config,
            error_message: None,
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
                KeyCode::Esc => {
                    log::info!("Moving from GameEnd to Menu");
                    return Ok(Some(Box::new(Menu::new(0, self.config.clone()))));
                }
                KeyCode::Enter => {
                    log::info!("Player wants to play again");
                    match self
                        .tcp_client
                        .play_again(self.game.id, Some(self.config.player_name.clone()))
                        .await
                    {
                        Ok(player) => {
                            log::info!("Play again request sent");
                            match self.tcp_client.get_game(self.game.id).await {
                                Ok(game) => {
                                    log::info!("Game received");
                                    return Ok(Some(Box::new(Lobby::new(
                                        game.into(),
                                        player.id,
                                        self.config.clone(),
                                    ))));
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to get game after play again request: {}",
                                        e
                                    );
                                    self.error_message =
                                        Some("Failed to get game after play again request".into());
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to send play again request: {}", e);
                            self.error_message = Some("Failed to send play again request".into());
                        }
                    }
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
            vec![
                " Return menu ".into(),
                "<Esc> ".light_blue(),
                "| Play again ".into(),
                "<Enter> ".light_blue(),
            ],
        );

        let inner = outer_rect.inner(Margin {
            horizontal: 5,
            vertical: 5,
        });
        // let inner = render_inner_rectangle(frame, outer_rect);

        // Sort players by score (assuming PlayerDto has a `score` field)
        let mut players: Vec<&PlayerDto> = self.game.players.values().collect();
        players.sort_by(|a, b| b.score.cmp(&a.score)); // Sort in descending order

        // Define podium heights
        let podium_heights = [inner.height / 2, inner.height / 3, inner.height / 4];
        let podium_width = inner.width / 5; // Adjust width to fit all podiums

        // Create a layout for the podiums and the 4th player message
        let [_, podium_area, _, humiliation_area, _] = Layout::vertical(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(60),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Percentage(10),
        ])
        .areas(inner);

        // Create a layout for the podiums with 1st place centered
        let [second_place_area, first_place_area, third_place_area] = Layout::horizontal(vec![
            Constraint::Length(podium_width),
            Constraint::Length(podium_width),
            Constraint::Length(podium_width),
        ])
        .flex(Flex::Center)
        .areas(podium_area);

        // Draw each podium
        for (i, (player, podium_height)) in players.iter().take(3).zip(podium_heights).enumerate() {
            // Determine the podium position based on rank
            let rect = match i {
                0 => first_place_area,
                1 => second_place_area,
                2 => third_place_area,
                _ => unreachable!(),
            };

            let podium_rect = Rect::new(
                rect.x,
                podium_area.y + podium_area.height - podium_height,
                podium_width,
                podium_height,
            );

            // Draw the podium block
            let podium_block = Block::bordered();
            frame.render_widget(podium_block, podium_rect);

            let style = if self.our_player_id == player.id {
                Style::default().fg(self.config.player_color)
            } else {
                Style::default()
            };

            // Draw the player name and crown (if 1st place)
            let name_paragraph = Paragraph::new(if i == 0 {
                // For 1st place, render the crown on top of the name
                vec![
                    Line::from("👑"),
                    Line::from(Span::styled(player.name.clone(), style)),
                ]
            } else {
                vec![
                    Line::from("\n"),
                    Line::from(Span::styled(player.name.clone(), style)),
                ]
            })
            .centered();

            frame.render_widget(
                name_paragraph,
                Rect::new(podium_rect.x, podium_rect.y - 2, podium_rect.width, 2),
            );

            // Draw the podium number (1st, 2nd, 3rd)
            let number_paragraph = Paragraph::new(Line::from(
                match i {
                    0 => "1st",
                    1 => "2nd",
                    2 => "3rd",
                    _ => "",
                }
                .to_string(),
            ));
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

            let humiliation_paragraph =
                Paragraph::new(Line::from(humiliation_text.red())).centered();

            frame.render_widget(humiliation_paragraph, humiliation_area);
        }
    }
}
