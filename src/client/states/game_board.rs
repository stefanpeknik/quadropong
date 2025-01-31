use crate::client::config;
use crate::client::net::udp::UdpClient;
use crate::common::models::{ClientInput, ClientInputType, Direction, GameDto, GameState};
use crate::common::PlayerPosition;

use super::game_end::GameEnd;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::render::{calculate_game_area, render_ball, render_player};

use crossterm::event::KeyCode;
use log::{debug, error, info};
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use uuid::Uuid;

pub struct GameBoard {
    game: Arc<Mutex<GameDto>>,
    our_player_id: Uuid,
    our_player_position: PlayerPosition,
    cancellation_token: CancellationToken,
    _receive_update_handle: JoinHandle<()>,
    _ping_handle: JoinHandle<()>,
    udp_client: Arc<UdpClient>,
    config: config::Config,
}

impl GameBoard {
    pub fn new(
        game: GameDto,
        our_player_id: Uuid,
        udp_client: Arc<UdpClient>,
        config: config::Config,
    ) -> Self {
        let our_player_position = game
            .players
            .get(&our_player_id)
            .map(|player| player.position.unwrap_or(PlayerPosition::Left)) // TODO: Handle this error better
            .unwrap_or(PlayerPosition::Left); // TODO: Handle this error better
        let game = Arc::new(Mutex::new(game));
        let cancellation_token = CancellationToken::new();

        let game_clone = Arc::clone(&game);
        let udp_client_clone = Arc::clone(&udp_client);
        let cancellation_token_clone = cancellation_token.clone();
        let receive_update_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Exit loop on cancellation
                    _ = cancellation_token_clone.cancelled() => break,
                    // Process incoming game updates
                    result = udp_client_clone.recv_updated_game() => {
                        match result {
                            Ok(updated_game) => {
                                if let Ok(mut current_game) = game_clone.lock() {
                                    *current_game = updated_game;
                                } else {
                                    error!("Failed to lock game");
                                }
                            }
                            Err(e) => {
                                error!("Failed to receive updated game: {}", e);
                            }
                        }
                    }
                }
            }
        });

        let udp_client_clone = Arc::clone(&udp_client);
        let cancellation_token_clone = cancellation_token.clone();
        let game_clone = Arc::clone(&game);
        let ping_handle = tokio::spawn(async move {
            let ping_interval = std::time::Duration::from_secs(1);
            loop {
                tokio::time::sleep(ping_interval).await;
                let client_input = if let Ok(g) = game_clone.lock() {
                    ClientInput::new(
                        g.id.to_string(),
                        our_player_id.to_string(),
                        ClientInputType::Ping,
                    )
                } else {
                    error!("Failed to lock game");
                    continue;
                };

                tokio::select! {
                    _ = cancellation_token_clone.cancelled() => break,
                    _ = udp_client_clone.send_client_input(client_input) => {
                        debug!("Ping sent");
                    }
                }
            }
        });

        Self {
            game,
            our_player_id,
            our_player_position,
            cancellation_token,
            _receive_update_handle: receive_update_handle,
            _ping_handle: ping_handle,
            udp_client,
            config,
        }
    }

    fn create_move_input(&self, direction: Direction) -> Option<ClientInput> {
        if let Ok(game) = self.game.lock() {
            Some(ClientInput::new(
                game.id.to_string(),
                self.our_player_id.to_string(),
                ClientInputType::MovePaddle(direction),
            ))
        } else {
            error!("Failed to lock game");
            return None;
        }
    }
}

impl State for GameBoard {}

impl HasConfig for GameBoard {
    fn config(&self) -> config::Config {
        self.config.clone()
    }
}

#[async_trait::async_trait]
impl Update for GameBoard {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Ok(game) = self.game.lock() {
            if game.state == GameState::Finished {
                info!("Game finished");
                info!("Moving from GameBoard to GameEnd");
                return Ok(Some(Box::new(GameEnd::new(
                    game.clone(),
                    self.our_player_id,
                    self.config.clone(),
                ))));
            }
        } else {
            error!("Failed to lock game");
        }
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Esc => {
                    // TODO: just a placeholder for now
                    if let Ok(game) = self.game.lock() {
                        return Ok(Some(Box::new(GameEnd::new(
                            game.clone(),
                            self.our_player_id,
                            self.config.clone(),
                        ))));
                    } else {
                        error!("Failed to lock game at Esc");
                    }
                }
                _ => match self.our_player_position {
                    PlayerPosition::Left | PlayerPosition::Right => match key_code {
                        KeyCode::Up | KeyCode::Char('w') => {
                            if let Some(input) = self.create_move_input(Direction::Negative) {
                                self.udp_client
                                    .send_client_input(input)
                                    .await
                                    .unwrap_or_else(|e| error!("Failed to send move input: {}", e));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s') => {
                            if let Some(input) = self.create_move_input(Direction::Positive) {
                                self.udp_client
                                    .send_client_input(input)
                                    .await
                                    .unwrap_or_else(|e| error!("Failed to send move input: {}", e));
                            }
                        }
                        _ => {}
                    },
                    PlayerPosition::Top | PlayerPosition::Bottom => match key_code {
                        KeyCode::Right | KeyCode::Char('d') => {
                            if let Some(input) = self.create_move_input(Direction::Positive) {
                                self.udp_client
                                    .send_client_input(input)
                                    .await
                                    .unwrap_or_else(|e| error!("Failed to send move input: {}", e));
                            }
                        }
                        KeyCode::Left | KeyCode::Char('a') => {
                            if let Some(input) = self.create_move_input(Direction::Negative) {
                                self.udp_client
                                    .send_client_input(input)
                                    .await
                                    .unwrap_or_else(|e| error!("Failed to send move input: {}", e));
                            }
                        }
                        _ => {}
                    },
                },
            };
        }
        Ok(None)
    }
}

impl Render for GameBoard {
    fn render(&self, frame: &mut Frame) {
        if let Ok(game) = self.game.lock() {
            // Calculate the game area and scaling factors once
            let (game_area_bounding_box, game_area, scale_x, scale_y) = calculate_game_area(&frame);

            // Render the game area border
            frame.render_widget(Block::bordered(), game_area_bounding_box);

            // Render players scores
            for player in game.players.values() {
                let desc = format!(" {} {} ", player.name, player.score);
                let desc_len = desc.len() as u16;

                match player.position {
                    Some(PlayerPosition::Top) => {
                        // Position at top-center of the game area
                        let x = game_area_bounding_box.x + game_area_bounding_box.width / 2
                            - desc_len / 2;
                        let y = game_area_bounding_box.y;
                        frame.render_widget(
                            Paragraph::new(desc).alignment(Alignment::Center),
                            Rect::new(x, y, desc_len, 1),
                        );
                    }
                    Some(PlayerPosition::Bottom) => {
                        // Position at bottom-center of the game area
                        let x = game_area_bounding_box.x + game_area_bounding_box.width / 2
                            - desc_len / 2;
                        let y = game_area_bounding_box.y + game_area_bounding_box.height - 1;
                        frame.render_widget(
                            Paragraph::new(desc).alignment(Alignment::Center),
                            Rect::new(x, y, desc_len, 1),
                        );
                    }
                    Some(PlayerPosition::Left) => {
                        // Vertical text on the left side
                        let x = game_area_bounding_box.x;
                        let y = game_area_bounding_box.y + game_area_bounding_box.height / 2
                            - desc_len / 2;
                        frame.render_widget(
                            Paragraph::new(
                                desc.chars()
                                    .map(|c| Line::from(c.to_string()))
                                    .collect::<Vec<Line>>(),
                            ),
                            Rect::new(x, y, 1, desc_len),
                        );
                    }
                    Some(PlayerPosition::Right) => {
                        // Vertical text on the right side
                        let x = game_area_bounding_box.x + game_area_bounding_box.width - 1;
                        let y = game_area_bounding_box.y + game_area_bounding_box.height / 2
                            - desc_len / 2;
                        frame.render_widget(
                            Paragraph::new(
                                desc.chars()
                                    .map(|c| Line::from(c.to_string()))
                                    .collect::<Vec<Line>>(),
                            ),
                            Rect::new(x, y, 1, desc_len),
                        );
                    }
                    None => {}
                }
            }

            // Render players
            for player in game.players.values() {
                let player_color = if player.id == self.our_player_id {
                    self.config.player_color
                } else {
                    self.config.other_players_color
                };
                render_player(player, player_color, frame, &game_area, scale_x, scale_y);
            }

            // Render the ball
            if let Some(ball) = &game.ball {
                render_ball(ball, frame, &game_area, scale_x, scale_y);
            }
        } else {
            error!("Failed to lock game");
        }
    }
}

impl Drop for GameBoard {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
