use crate::client::config;
use crate::client::error::ClientError;
use crate::client::net::udp::UdpClient;
use crate::client::states::menu::Menu;
use crate::common::models::{ClientInput, ClientInputType, Direction, GameDto, GameState};
use crate::common::PlayerPosition;

use super::game_end::GameEnd;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::render::{render_disconnect_popup, render_game};

use crossterm::event::KeyCode;
use log::{debug, error, info};
use ratatui::Frame;
use std::sync::atomic::{AtomicBool, Ordering};
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
    disconnected: Arc<AtomicBool>,
}

impl GameBoard {
    pub fn new(
        game: GameDto,
        our_player_id: Uuid,
        udp_client: Arc<UdpClient>,
        config: config::Config,
    ) -> Result<Self, ClientError> {
        // if for some reason the player position is not set, default to left
        let our_player_position = game
            .players
            .get(&our_player_id)
            .map(|player| player.position.unwrap_or(PlayerPosition::Left))
            .unwrap_or(PlayerPosition::Left);
        let game = Arc::new(Mutex::new(game));
        let cancellation_token = CancellationToken::new();
        let disconnected = Arc::new(AtomicBool::new(false));

        let game_clone = Arc::clone(&game);
        let udp_client_clone = Arc::clone(&udp_client);
        let cancellation_token_clone = cancellation_token.clone();
        let disconnected_clone = Arc::clone(&disconnected);
        let receive_update_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Exit loop on cancellation
                    _ = cancellation_token_clone.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                        disconnected_clone.store(true, Ordering::Relaxed);
                    }
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

        Ok(Self {
            game,
            our_player_id,
            our_player_position,
            cancellation_token,
            _receive_update_handle: receive_update_handle,
            _ping_handle: ping_handle,
            udp_client,
            config,
            disconnected,
        })
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
            None
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
    ) -> Result<Option<Box<dyn State>>, ClientError> {
        if let Ok(game) = self.game.lock() {
            if game.state == GameState::Finished {
                info!("Game finished");
                info!("Moving from GameBoard to GameEnd");
                return Ok(Some(Box::new(GameEnd::new(
                    game.clone(),
                    self.our_player_id,
                    self.config.clone(),
                )?)));
            }
        } else {
            error!("Failed to lock game");
        }
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Esc => {
                    if self.disconnected.load(Ordering::Relaxed) {
                        info!("Moving from Lobby to CreateOrJoinLobby due to disconnection");
                    } else {
                        info!("Moving from GameBoard to Menu due to user leaving");
                    }
                    return Ok(Some(Box::new(Menu::new(0, self.config.clone())?)));
                }
                _ => match self.our_player_position {
                    PlayerPosition::Left | PlayerPosition::Right => match key_code {
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                            if let Some(input) = self.create_move_input(Direction::Negative) {
                                self.udp_client
                                    .send_client_input(input)
                                    .await
                                    .unwrap_or_else(|e| error!("Failed to send move input: {}", e));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
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
                        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                            if let Some(input) = self.create_move_input(Direction::Positive) {
                                self.udp_client
                                    .send_client_input(input)
                                    .await
                                    .unwrap_or_else(|e| error!("Failed to send move input: {}", e));
                            }
                        }
                        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
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
            render_game(
                &game,
                self.our_player_id,
                self.config.player_color,
                self.config.other_players_color,
                frame,
            );
        } else {
            error!("Failed to lock game");
        }
        if self.disconnected.load(Ordering::Relaxed) {
            render_disconnect_popup(frame, frame.area());
        }
    }
}

impl Drop for GameBoard {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
