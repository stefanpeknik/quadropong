use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::client::config;
use crate::client::error::ClientError;
use crate::client::net::tcp::TcpClient;
use crate::client::net::udp::UdpClient;
use crate::common::models::{ClientInput, ClientInputType, GameDto, GameState};
use crate::common::Game;

use super::create_or_join_lobby::CreateOrJoinLobby;
use super::game_board::GameBoard;
use super::traits::{HasConfig, Render, State, Update};
use super::utils::render::{render_disconnect_popup, render_outer_rectangle, render_player_list};

use arboard::Clipboard;
use crossterm::event::KeyCode;
use log::{debug, error, info};
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct Lobby {
    game: Arc<Mutex<GameDto>>,
    game_id: Uuid,
    our_player_id: Uuid,
    cancellation_token: CancellationToken,
    _receive_update_handle: JoinHandle<Result<(), ClientError>>,
    _ping_handle: JoinHandle<()>,
    udp_client: Arc<UdpClient>,
    tcp_client: Arc<TcpClient>,
    config: config::Config,
    disconnected: Arc<AtomicBool>,
}

impl Lobby {
    pub fn new(
        game: Game,
        our_player_id: Uuid,
        config: config::Config,
    ) -> Result<Self, ClientError> {
        let udp_client = Arc::new(UdpClient::new(&config.socket_addr)?);

        let tcp_client = Arc::new(TcpClient::new(&config.api_url));

        let cancellation_token = CancellationToken::new();
        let game_id = game.id;
        let game_dto = Arc::new(Mutex::new(GameDto::from(game)));
        let disconnected = Arc::new(AtomicBool::new(false));

        // Start a task to receive updates
        let game_clone = Arc::clone(&game_dto);
        let udp_client_clone = Arc::clone(&udp_client);
        let cancellation_token_clone = cancellation_token.clone();
        let disconnected_clone = Arc::clone(&disconnected);
        let receive_update_handle = tokio::spawn(async move {
            // send introduction message
            let client_input = ClientInput::new(
                game_id.to_string(),
                our_player_id.to_string(),
                ClientInputType::JoinGame,
            );
            udp_client_clone.send_client_input(client_input).await?;

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
                            Err(e) => error!("Failed to receive updated game: {}", e),
                        }
                    }
                }
            }

            Ok(())
        });

        // Start a task to send ping messages
        let udp_client_clone = Arc::clone(&udp_client);
        let cancellation_token_clone = cancellation_token.clone();
        let ping_handle = tokio::spawn(async move {
            let ping_interval = std::time::Duration::from_secs(1);
            loop {
                tokio::time::sleep(ping_interval).await;
                let client_input = ClientInput::new(
                    game_id.to_string(),
                    our_player_id.to_string(),
                    ClientInputType::Ping,
                );

                tokio::select! {
                    _ = cancellation_token_clone.cancelled() => break,
                    _ = udp_client_clone.send_client_input(client_input) => {
                        debug!("Sent ping message");
                    }
                }
            }
        });

        Ok(Self {
            game: game_dto,
            game_id,
            our_player_id,
            udp_client,
            tcp_client,
            cancellation_token,
            _receive_update_handle: receive_update_handle,
            _ping_handle: ping_handle,
            config,
            disconnected,
        })
    }
}

impl State for Lobby {}

impl HasConfig for Lobby {
    fn config(&self) -> config::Config {
        self.config.clone()
    }
}

#[async_trait::async_trait]
impl Update for Lobby {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, ClientError> {
        // if game is started
        if let Ok(game) = self.game.lock() {
            if game.state == GameState::Active {
                info!("Moving from Lobby to GameBoard as game is started");
                return Ok(Some(Box::new(GameBoard::new(
                    game.clone(),
                    self.our_player_id,
                    Arc::clone(&self.udp_client),
                    self.config.clone(),
                )?)));
            }
        } else {
            error!("Failed to lock game");
        }

        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Tab => {
                    // copy game id to clipboard
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Err(_clipboard_content) =
                            clipboard.set_text(self.game_id.to_string())
                        {
                            error!("Failed to set clipboard content");
                        }
                    } else {
                        error!("Failed to create clipboard");
                    }
                }

                KeyCode::Enter => {
                    // send player ready
                    let client_input = ClientInput::new(
                        self.game_id.to_string(),
                        self.our_player_id.to_string(),
                        ClientInputType::PlayerReady,
                    );
                    self.udp_client.send_client_input(client_input).await?;
                    info!("Toggle player ready");
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    match self.tcp_client.add_bot(self.game_id).await {
                        Err(e) => info!("Add bot failed: {}", e),
                        Ok(_) => info!("Add bot called"),
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    match self.tcp_client.remove_bot(self.game_id).await {
                        Err(e) => info!("Remove bot failed: {}", e),
                        Ok(_) => info!("Remove bot called"),
                    }
                }
                KeyCode::Esc => {
                    info!("Moving from Lobby to CreateOrJoinLobby");
                    return Ok(Some(Box::new(CreateOrJoinLobby::new(self.config.clone())?)));
                }
                _ => {}
            };
        }
        Ok(None)
    }
}

impl Render for Lobby {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = render_outer_rectangle(
            frame,
            " quadropong - Lobby ",
            vec![
                " Leave Game ".into(),
                "<Esc> ".light_blue().bold(),
                "| Ready ".into(),
                "<Enter> ".light_blue().bold(),
                "| Add bot ".into(),
                "<A> ".light_cyan().bold(),
                "| Delete bot ".into(),
                "<D> ".light_cyan().bold(),
            ],
        );
        let inner_rect = outer_rect.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        let layout = Layout::vertical(vec![Constraint::Length(3), Constraint::Fill(1)]);
        let [lobby_id_area, lobby_area] = layout.areas(inner_rect);

        if let Ok(game) = self.game.lock() {
            let mut list = vec![];
            let mut players_info: Vec<_> = game
                .players
                .iter()
                .map(|(p_id, p)| {
                    if *p_id == self.our_player_id {
                        (
                            format!("{} (You)", p.name),
                            p.joined_at,
                            p.is_ready,
                            p.position,
                        )
                    } else {
                        (p.name.clone(), p.joined_at, p.is_ready, p.position)
                    }
                })
                .collect();
            players_info.sort_by(|(_, p1_joined_at, _, _), (_, p2_joined_at, _, _)| {
                p1_joined_at.cmp(p2_joined_at)
            });
            let players: Vec<_> = players_info
                .into_iter()
                .map(|(players, _, is_ready, position)| (players, is_ready, position))
                .collect();
            list.extend(players);

            // render lobby ID
            let lobby_id_block = Block::bordered().title_bottom(
                Line::from(vec![" Copy ".into(), "<TAB> ".green().bold()]).right_aligned(),
            );
            let inner_lobby_id_area = lobby_id_block.inner(lobby_id_area);
            let lobby_id_paragraph = Paragraph::new(format!(" Game ID - {}", game.id));
            frame.render_widget(lobby_id_paragraph, inner_lobby_id_area);
            frame.render_widget(lobby_id_block, lobby_id_area);

            render_player_list(frame, &list, lobby_area);

            if self.disconnected.load(Ordering::Relaxed) {
                render_disconnect_popup(frame, lobby_area);
            }
        } else {
            error!("Failed to lock game");
        }
    }
}

impl Drop for Lobby {
    fn drop(&mut self) {
        // Signal the task to stop
        self.cancellation_token.cancel();
    }
}
