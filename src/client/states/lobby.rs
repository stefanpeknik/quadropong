use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::client::net::udp::UdpClient;
use crate::common::models::{ClientInput, ClientInputType, GameDto};
use crate::common::Game;

use super::create_or_join_lobby::CreateOrJoinLobby;
use super::game_board::GameBoard;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::render::{render_player_list, render_outer_rectangle};

use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use uuid::Uuid;

pub enum Options {
    TODO,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::TODO]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::TODO => write!(f, "TODO"),
        }
    }
}

pub struct Lobby {
    options: Vec<Options>,
    selected: usize,
    game: Arc<Mutex<GameDto>>,
    our_player_id: Uuid,
    receive_updates: Arc<AtomicBool>,
    receive_update_handle: tokio::task::JoinHandle<()>,
    udp_client: Arc<UdpClient>,
}

impl Lobby {
    pub fn new(game: Game, our_player_id: Uuid) -> Self {
        let udp_client = Arc::new(UdpClient::new().expect("Failed to create UDP client")); // TODO: Handle this error

        // send introduction message
        let client_input = ClientInput::new(
            game.id.to_string(),
            our_player_id.to_string(),
            ClientInputType::JoinGame,
        );
        udp_client
            .send_client_input(client_input)
            .expect("Failed to send introduction message"); // TODO: Handle this error

        let receive_updates = Arc::new(AtomicBool::new(true));
        let game_dto = Arc::new(Mutex::new(GameDto::from(game)));

        // Start a task to receive updates
        let udp_client_clone = Arc::clone(&udp_client);
        let receive_updates_clone = Arc::clone(&receive_updates);
        let game_clone = Arc::clone(&game_dto);
        let receive_update_handle = tokio::spawn(async move {
            while receive_updates_clone.load(Ordering::Relaxed) {
                match udp_client_clone.recv_updated_game() {
                    Ok(updated_game) => {
                        match game_clone.lock() {
                            Ok(mut current_game) => {
                                *current_game = updated_game;
                            }
                            Err(_) => {
                                // TODO: Most likely ignore this error?
                            }
                        }
                    }
                    Err(_) => {
                        // TODO: Most likely ignore this error?
                    }
                }
            }
        });

        Self {
            options: Options::list(),
            selected: 0,
            game: game_dto,
            our_player_id,
            udp_client,
            receive_updates,
            receive_update_handle,
        }
    }
}

impl HasOptions for Lobby {
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

impl State for Lobby {}

#[async_trait::async_trait]
impl Update for Lobby {
    async fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            match key_code {
                KeyCode::Tab => {
                    // copy game id to clipboard
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Ok(game) = self.game.lock() {
                            if let Ok(_clipboard_content) = clipboard.set_text(game.id.to_string())
                            {
                                // TODO
                            }
                        }
                    }
                }
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Enter => match self.options[self.selected] {
                    // TODO: Implement this
                    _ => {
                        // TODO: for now send into game
                        return Ok(Some(Box::new(GameBoard::new(
                            self.game
                                .lock()
                                .expect("Failed to lock game") // TODO: Handle this error
                                .to_owned(),
                            self.our_player_id,
                            Arc::clone(&self.udp_client),
                        ))));
                    }
                },
                KeyCode::Esc => {
                    return Ok(Some(Box::new(CreateOrJoinLobby::new())));
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
                "| Start Game ".into(),
                "<Space> ".light_blue().bold(),
            ],
        );
        let inner_rect = outer_rect.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        let layout = Layout::vertical(vec![Constraint::Length(3), Constraint::Percentage(90)]);
        let [lobby_id_area, _] = layout.areas(inner_rect);

        if let Ok(game) = self.game.lock() {
            let mut list = vec![];
            let mut players_info: Vec<_> = game
                .players
                .iter()
                .map(|(p_id, p)|
                    if *p_id == self.our_player_id {
                        (format!("You ({}): {}", p.name, p_id), p.joined_at)
                    } else {
                        (format!("{}: {}", p.name, p_id), p.joined_at)
                    })
                .collect();
            // TODO add joined_at to playerDto and sort by it
            players_info.sort_by(|(_, p1_joined_at), (_, p2_joined_at)| p1_joined_at.cmp(p2_joined_at) );
            let players: Vec<_> = players_info.into_iter().map(|(players, _)| players).collect();
            list.extend(players);

            // render lobby ID
            let lobby_id_block = Block::bordered().title_bottom(
                Line::from(vec![" Copy ".into(), "<TAB> ".green().bold()]).right_aligned(),
            );
            let inner_lobby_id_area = lobby_id_block.inner(lobby_id_area);
            let lobby_id_paragraph = Paragraph::new(format!(" Game ID - {}", game.id.to_string()));
            frame.render_widget(lobby_id_paragraph, inner_lobby_id_area);
            frame.render_widget(lobby_id_block, lobby_id_area);

            render_player_list(
                frame,
                &list,
                self.selected,
                outer_rect.inner(Margin::new(5, 5)),
            );
        }
    }
}

impl Drop for Lobby {
    fn drop(&mut self) {
        // Signal the task to stop
        self.receive_updates.store(false, Ordering::Relaxed);

        self.receive_update_handle.abort();
    }
}
